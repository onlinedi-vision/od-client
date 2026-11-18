//! One admitted test job; no permanent threads. Stop never waits
//! for an OS build/drop call. Admission remains occupied until native cleanup.
use super::playout::{Diagnostics, Stats};
use super::{budget::Budget, catalog::AudioError, meter::Meter};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard, Weak,
};
use std::time::{Duration, Instant};

pub(super) const TEST_LIMIT: Duration = Duration::from_secs(30);
const START_LIMIT: Duration = Duration::from_secs(5);
const LEVEL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Input,
    Output,
    Loopback,
}

pub struct Start {
    pub revision: u32,
    pub input_id: String,
    pub output_id: Option<String>,
    pub mode: Mode,
}

pub(super) struct Selection {
    pub mode: Mode,
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: &'static str,
    pub buffer_frames: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSnapshot {
    pub mode: Mode,
    pub phase: &'static str,
    pub level: u8,
    pub generation: u32,
    pub cleanup_pending: bool,
    pub format: Option<CaptureFormat>,
    pub error: Option<AudioError>,
    pub ended: Option<&'static str>,
    pub overruns: u32,
    pub callback_frames: Option<u32>,
    pub output_format: Option<CaptureFormat>,
    pub playout: Diagnostics,
}

impl Default for CaptureSnapshot {
    fn default() -> Self {
        Self {
            mode: Mode::Input,
            phase: "idle",
            level: 0,
            generation: 0,
            cleanup_pending: false,
            format: None,
            error: None,
            ended: None,
            overruns: 0,
            callback_frames: None,
            output_format: None,
            playout: Diagnostics::default(),
        }
    }
}

#[derive(Default)]
struct State {
    owner: u32,
    intent: u32,
    closed: bool,
    next_run: u32,
    active: Option<Arc<Run>>,
    snapshot: CaptureSnapshot,
    sampled: Option<Instant>,
}

#[derive(Default)]
pub(super) struct Controller(Arc<Mutex<State>>);

fn lock(state: &Mutex<State>) -> Result<MutexGuard<'_, State>, AudioError> {
    state.lock().map_err(|_| {
        AudioError::new(
            "capture_failed",
            "Audio tests are unavailable. Restart the app.",
        )
    })
}

fn cancelled() -> AudioError {
    AudioError::new("capture_stopped", "The audio test was stopped.")
}

pub(super) fn unavailable() -> AudioError {
    AudioError::new(
        "device_unavailable",
        "The selected audio device is unavailable. Refresh devices and start again.",
    )
}

pub(super) fn unsupported() -> AudioError {
    AudioError::new(
        "unsupported_config",
        "The selected audio configuration is not supported. Try another device or system audio profile.",
    )
}

fn cancel(state: &mut State) {
    if let Some(run) = &state.active {
        run.cancelled.store(true, Ordering::Release);
        run.meter.silence();
        state.snapshot.phase = "stopping";
        state.snapshot.cleanup_pending = true;
    } else {
        state.snapshot.phase = "idle";
        state.snapshot.cleanup_pending = false;
    }
    state.snapshot.level = 0;
    state.snapshot.callback_frames = None;
    state.snapshot.error = None;
    state.snapshot.ended = Some("stopped");
}

fn accept(state: &mut State, owner: u32, intent: u32) -> Result<(), AudioError> {
    if owner == 0 || intent == 0 || owner < state.owner {
        return Err(cancelled());
    }
    if owner > state.owner {
        cancel(state);
        state.owner = owner;
        state.intent = 0;
        state.closed = false;
    }
    if intent < state.intent || state.closed {
        return Err(cancelled());
    }
    state.intent = intent;
    Ok(())
}

impl Controller {
    #[cfg(test)]
    pub fn start(
        &self,
        owner: u32,
        intent: u32,
        target: Option<String>,
        spawn: impl FnOnce(Job),
    ) -> Result<CaptureSnapshot, AudioError> {
        self.start_test(
            owner,
            intent,
            Selection {
                mode: Mode::Input,
                input: target,
                output: None,
            },
            spawn,
        )
    }

    pub fn start_test(
        &self,
        owner: u32,
        intent: u32,
        selection: Selection,
        spawn: impl FnOnce(Job),
    ) -> Result<CaptureSnapshot, AudioError> {
        let mut state = lock(&self.0)?;
        let repeated = state.owner == owner && state.intent == intent;
        accept(&mut state, owner, intent)?;
        if repeated {
            return Ok(state.snapshot.clone());
        }
        if let Some(run) = &state.active {
            if !run.cancelled.load(Ordering::Acquire) {
                if run.mode != selection.mode {
                    return Err(AudioError::new(
                        "test_busy",
                        "Stop the active audio test before starting another mode.",
                    ));
                }
                return Ok(state.snapshot.clone());
            }
            return Err(AudioError {
                cleanup_pending: true,
                ..AudioError::new(
                    "capture_busy",
                    "The previous audio test is still stopping. Wait for cleanup.",
                )
            });
        }
        state.next_run = state.next_run.checked_add(1).ok_or_else(|| {
            AudioError::new(
                "generation_exhausted",
                "Restart the app to reset audio settings.",
            )
        })?;
        let run = Arc::new(Run {
            mode: selection.mode,
            stats: Arc::new(Stats::default()),
            activated: 0.into(),
            io_enabled: true.into(),
            generation: state.next_run,
            created: Instant::now(),
            cancelled: AtomicBool::new(false),
            running: AtomicBool::new(false),
            meter: Meter::default(),
            state: Arc::downgrade(&self.0),
        });
        state.snapshot = CaptureSnapshot {
            mode: selection.mode,
            phase: "starting",
            generation: run.generation,
            ..CaptureSnapshot::default()
        };
        state.active = Some(Arc::clone(&run));
        state.sampled = None;
        let result = state.snapshot.clone();
        // Drop the control lock before scheduling. No OS work under this mutex.
        drop(state);
        spawn(Job {
            run,
            state: Arc::clone(&self.0),
            selection,
            completed: false,
        });
        Ok(result)
    }

    pub fn stop(
        &self,
        owner: u32,
        intent: u32,
        close: bool,
    ) -> Result<CaptureSnapshot, AudioError> {
        let mut state = lock(&self.0)?;
        // Closing the same panel twice is harmless; it never reopens its owner.
        if state.owner == owner && state.closed {
            return Ok(state.snapshot.clone());
        }
        accept(&mut state, owner, intent)?;
        cancel(&mut state);
        state.closed = close;
        Ok(state.snapshot.clone())
    }

    pub fn cancel(&self) -> Result<(), AudioError> {
        cancel(&mut *lock(&self.0)?);
        Ok(())
    }

    pub fn busy(&self) -> Result<bool, AudioError> {
        Ok(lock(&self.0)?.active.is_some())
    }

    pub fn reset(&self) -> Result<(), AudioError> {
        let mut state = lock(&self.0)?;
        cancel(&mut state);
        state.owner = 0;
        state.intent = 0;
        state.closed = false;
        Ok(())
    }

    pub fn snapshot(&self, owner: u32) -> Result<CaptureSnapshot, AudioError> {
        let mut state = lock(&self.0)?;
        if owner < state.owner {
            return Err(cancelled());
        }
        if let Some(run) = state.active.clone() {
            if state.snapshot.phase == "running"
                && state
                    .sampled
                    .is_none_or(|time| time.elapsed() >= LEVEL_INTERVAL)
            {
                state.snapshot.level = run.meter.level();
                state.snapshot.overruns = run.meter.overruns();
                state.snapshot.playout = run.stats.snapshot();
                state.snapshot.callback_frames = state
                    .snapshot
                    .format
                    .as_ref()
                    .map(|format| run.meter.last_samples() / u32::from(format.channels));
                state.sampled = Some(Instant::now());
            }
            // Logical deadline also protects UI when an OS call stalls.
            if let Err(error) = run.check() {
                if error.code != "capture_stopped" {
                    cancel(&mut state);
                    if error.code == "test_finished" {
                        state.snapshot.ended = Some("duration_limit");
                    } else {
                        state.snapshot.error = Some(error);
                    }
                }
            }
        }
        Ok(state.snapshot.clone())
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let _ = self.cancel();
    }
}

pub(super) struct Run {
    pub mode: Mode,
    pub stats: Arc<Stats>,
    activated: std::sync::atomic::AtomicU64,
    io_enabled: AtomicBool,
    generation: u32,
    created: Instant,
    cancelled: AtomicBool,
    running: AtomicBool,
    pub meter: Meter,
    state: Weak<Mutex<State>>,
}

impl Budget for Run {
    fn check(&self) -> Result<Duration, AudioError> {
        // Keep the fault's cause even though enabled() already gates callbacks.
        if self.cancelled.load(Ordering::Acquire) || !self.io_enabled.load(Ordering::Acquire) {
            return Err(cancelled());
        }
        match self.meter.fault() {
            0 => {}
            2 => {
                return Err(AudioError::new(
                    "permission_denied",
                    "Audio device access was denied. Check your system audio permissions.",
                ))
            }
            3 => return Err(unavailable()),
            4 => return Err(AudioError::new(
                "capture_timing_unavailable",
                "The input capture clock is unavailable. Choose another audio profile or device.",
            )),
            _ => {
                return Err(AudioError::new(
                    "capture_failed",
                    "The audio stream failed. Stop, check the device and try again.",
                ))
            }
        }
        let running = self.running.load(Ordering::Acquire);
        let limit = if !running {
            START_LIMIT
        } else if self.mode == Mode::Output {
            Duration::from_secs(1)
        } else {
            TEST_LIMIT
        };
        let elapsed = if running && self.mode == Mode::Output {
            self.created
                .elapsed()
                .saturating_sub(Duration::from_nanos(self.activated.load(Ordering::Acquire)))
        } else {
            self.created.elapsed()
        };
        limit.checked_sub(elapsed).filter(|time| !time.is_zero()).ok_or_else(|| {
            if running { AudioError::new("test_finished", "The audio test reached its duration limit.") }
            else { AudioError::new("capture_timeout", "The audio test took too long to start. Wait for cleanup before trying again.") }
        })
    }
}

impl Budget for Arc<Run> {
    fn check(&self) -> Result<Duration, AudioError> {
        self.as_ref().check()
    }
}

impl Run {
    pub fn enabled(&self) -> bool {
        !self.cancelled.load(Ordering::Acquire)
            && self.io_enabled.load(Ordering::Acquire)
            && self.meter.fault() == 0
    }

    // Gate callbacks before any native stream destructor can block. This is
    // separate from user cancellation so backend failures keep their cause.
    pub fn quiet(&self) {
        self.io_enabled.store(false, Ordering::Release);
        self.meter.silence();
    }

    #[cfg(any(target_os = "windows", target_os = "macos", test))]
    pub fn retry_setup(&self) -> Result<(), AudioError> {
        if self.cancelled.load(Ordering::Acquire) || self.running.load(Ordering::Acquire) {
            return Err(cancelled());
        }
        self.io_enabled.store(true, Ordering::Release);
        if let Err(error) = self.check() {
            self.quiet();
            return Err(error);
        }
        Ok(())
    }

    pub fn started(&self, format: CaptureFormat) -> Result<(), AudioError> {
        self.started_io(format, None)
    }

    pub fn started_playback(
        &self,
        input: Option<CaptureFormat>,
        output: CaptureFormat,
    ) -> Result<(), AudioError> {
        self.started_io(input.unwrap_or_else(|| output.clone()), Some(output))
    }

    fn started_io(
        &self,
        format: CaptureFormat,
        output: Option<CaptureFormat>,
    ) -> Result<(), AudioError> {
        if format.sample_rate == 0 || !(1..=2).contains(&format.channels) {
            return Err(unsupported());
        }
        self.check()?;
        let state = self.state.upgrade().ok_or_else(cancelled)?;
        let mut state = lock(&state)?;
        if self.cancelled.load(Ordering::Acquire)
            || !state
                .active
                .as_ref()
                .is_some_and(|run| run.generation == self.generation)
        {
            return Err(cancelled());
        }
        self.activated.store(
            self.created.elapsed().as_nanos().min(u64::MAX as u128) as u64,
            Ordering::Release,
        );
        self.running.store(true, Ordering::Release);
        state.snapshot.phase = "running";
        state.snapshot.format = Some(format);
        state.snapshot.output_format = output;
        Ok(())
    }
}

pub(super) struct Job {
    pub run: Arc<Run>,
    state: Arc<Mutex<State>>,
    selection: Selection,
    completed: bool,
}

impl Job {
    #[cfg(test)]
    pub fn execute(self, record: impl FnOnce(&Arc<Run>, Option<&str>) -> Result<(), AudioError>) {
        self.execute_test(|run, selection| record(run, selection.input.as_deref()));
    }

    pub fn execute_test(
        mut self,
        record: impl FnOnce(&Arc<Run>, &Selection) -> Result<(), AudioError>,
    ) {
        let result = self
            .run
            .check()
            .and_then(|_| record(&self.run, &self.selection));
        // Native record has returned and all its stream/context handles dropped.
        self.finish(result.err());
    }

    fn finish(&mut self, error: Option<AudioError>) {
        if let Ok(mut state) = self.state.lock() {
            if state
                .active
                .as_ref()
                .is_some_and(|run| run.generation == self.run.generation)
            {
                let stopped = self.run.cancelled.load(Ordering::Acquire);
                state.active = None;
                state.snapshot.playout = self.run.stats.snapshot();
                state.snapshot.level = 0;
                state.snapshot.callback_frames = None;
                state.snapshot.cleanup_pending = false;
                state.snapshot.error = if stopped {
                    state.snapshot.error.clone()
                } else {
                    error
                        .clone()
                        .filter(|e| !matches!(e.code, "capture_stopped" | "test_finished"))
                };
                state.snapshot.phase = if state.snapshot.error.is_some() {
                    "failed"
                } else {
                    "idle"
                };
                state.snapshot.ended = Some(
                    if state.snapshot.ended == Some("duration_limit")
                        || error.is_some_and(|e| e.code == "test_finished")
                    {
                        "duration_limit"
                    } else {
                        "stopped"
                    },
                );
            }
        }
        self.run.cancelled.store(true, Ordering::Release);
        self.run.meter.silence();
        self.completed = true;
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if !self.completed {
            self.finish(Some(AudioError::new(
                "capture_failed",
                "The audio test could not complete. Try again.",
            )));
        }
    }
}

#[cfg(test)]
mod tests;
