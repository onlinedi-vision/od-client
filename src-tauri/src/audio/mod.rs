mod budget;
mod capture;
mod catalog;
#[cfg(not(target_os = "linux"))]
mod cpal_devices;
#[cfg(any(target_os = "windows", target_os = "macos", test))]
mod formats;
mod meter;
mod playout;
#[cfg(target_os = "linux")]
mod pulse_devices;

#[cfg(not(target_os = "linux"))]
use cpal_devices as platform;
#[cfg(target_os = "linux")]
use pulse_devices as platform;

pub use capture::{CaptureSnapshot, Mode, Start};
pub use catalog::{AudioError, Inventory};
use std::sync::{Arc, Mutex, MutexGuard};

struct Control {
    devices: catalog::DeviceCatalog,
    capture: capture::Controller,
    shutdown: bool,
}

#[derive(Clone)]
pub struct AudioService(Arc<Mutex<Control>>);

impl AudioService {
    fn lock(&self) -> Result<MutexGuard<'_, Control>, AudioError> {
        let state = self.0.lock().map_err(|_| {
            AudioError::new(
                "internal_error",
                "Audio settings are unavailable. Restart the app.",
            )
        })?;
        if state.shutdown {
            return Err(AudioError::new(
                "session_expired",
                "Audio is shutting down.",
            ));
        }
        Ok(state)
    }

    pub fn open_session(&self) -> Result<String, AudioError> {
        let state = self.lock()?;
        state.capture.reset()?;
        state.devices.open_session()
    }

    pub fn close_session(&self, id: &str) -> Result<(), AudioError> {
        let state = self.lock()?;
        if state.devices.validate_session(id).is_ok() {
            state.capture.cancel()?;
        }
        state.devices.close_session(id)
    }

    pub fn begin_refresh(&self, session: &str) -> Result<catalog::RefreshPermit, AudioError> {
        let state = self.lock()?;
        state.devices.validate_session(session)?;
        state.capture.cancel()?;
        if state.capture.busy()? {
            return Err(AudioError {
                cleanup_pending: true,
                ..AudioError::new(
                    "capture_busy",
                    "Wait for the audio test to stop before refreshing devices.",
                )
            });
        }
        state.devices.begin_refresh(session)
    }

    pub fn select(
        &self,
        session: &str,
        revision: u32,
        direction: &str,
        id: &str,
    ) -> Result<Inventory, AudioError> {
        let state = self.lock()?;
        let result = state.devices.select(session, revision, direction, id)?;
        state.capture.cancel()?;
        Ok(result)
    }

    pub fn start_capture(
        &self,
        session: &str,
        owner: u32,
        intent: u32,
        request: Start,
    ) -> Result<CaptureSnapshot, AudioError> {
        let state = self.lock()?;
        let input = if request.mode != Mode::Output {
            state
                .devices
                .input_target(session, request.revision, &request.input_id)?
        } else {
            None
        };
        let output = if request.mode != Mode::Input {
            state.devices.output_target(
                session,
                request.revision,
                request
                    .output_id
                    .as_deref()
                    .ok_or_else(|| AudioError::new("invalid_device", "Choose an output device."))?,
            )?
        } else {
            None
        };
        state.capture.start_test(
            owner,
            intent,
            capture::Selection {
                mode: request.mode,
                input,
                output,
            },
            |job| {
                // One admitted blocking job owns at most two streams for <=30 s. It
                // checks cancellation before OS access and after a delayed build.
                tauri::async_runtime::spawn_blocking(move || job.execute_test(platform::test));
            },
        )
    }

    pub fn stop_capture(
        &self,
        session: &str,
        owner: u32,
        intent: u32,
        close: bool,
    ) -> Result<CaptureSnapshot, AudioError> {
        let state = self.lock()?;
        state.devices.validate_session(session)?;
        state.capture.stop(owner, intent, close)
    }

    pub fn capture_state(&self, session: &str, owner: u32) -> Result<CaptureSnapshot, AudioError> {
        let state = self.lock()?;
        state.devices.validate_session(session)?;
        state.capture.snapshot(owner)
    }

    pub fn shutdown(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.shutdown = true;
            let _ = state.capture.cancel();
            let _ = state.devices.shutdown();
        }
    }
}

pub fn service() -> AudioService {
    AudioService(Arc::new(Mutex::new(Control {
        devices: catalog::DeviceCatalog::new(platform::BACKEND),
        capture: capture::Controller::default(),
        shutdown: false,
    })))
}

pub async fn refresh(service: &AudioService, session: &str) -> Result<Inventory, AudioError> {
    let permit = service.begin_refresh(session)?;
    // The runtime reuses its blocking pool. One global permit prevents parallel OS
    // scans, including after UI timeout/logout. No application thread or queue per Refresh.
    tauri::async_runtime::spawn_blocking(move || permit.run(platform::enumerate))
        .await
        .map_err(|_| AudioError::new("inventory_failed", "The device scan failed. Try Refresh."))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (AudioService, String) {
        let service = service();
        let session = service.open_session().unwrap();
        service
            .begin_refresh(&session)
            .unwrap()
            .finish(Ok(catalog::RawInventory {
                inputs: vec![catalog::RawDevice {
                    native_id: "test-mic".into(),
                    label: "Microphone".into(),
                }],
                default_input: Some("Microphone".into()),
                ..catalog::RawInventory::default()
            }))
            .unwrap();
        (service, session)
    }

    fn queued_job(service: &AudioService) -> capture::Job {
        queued_test(service, Mode::Input)
    }

    fn queued_test(service: &AudioService, mode: Mode) -> capture::Job {
        let mut job = None;
        service
            .lock()
            .unwrap()
            .capture
            .start_test(
                1,
                1,
                capture::Selection {
                    mode,
                    input: None,
                    output: None,
                },
                |work| job = Some(work),
            )
            .unwrap();
        job.unwrap()
    }

    #[test]
    fn every_local_lifecycle_action_gates_all_test_modes_until_cleanup() {
        for mode in [Mode::Input, Mode::Output, Mode::Loopback] {
            for action in [
                "close",
                "remount",
                "refresh",
                "select_input",
                "select_output",
                "exit",
            ] {
                let (service, session) = fixture();
                let inventory = service
                    .lock()
                    .unwrap()
                    .devices
                    .begin_refresh(&session)
                    .unwrap()
                    .finish(Ok(catalog::RawInventory {
                        inputs: vec![catalog::RawDevice {
                            native_id: "mic".into(),
                            label: "Mic".into(),
                        }],
                        outputs: vec![catalog::RawDevice {
                            native_id: "headphones".into(),
                            label: "Headphones".into(),
                        }],
                        default_input: Some("Mic".into()),
                        default_output: Some("Headphones".into()),
                    }))
                    .unwrap();
                let work = queued_test(&service, mode);
                let format = capture::CaptureFormat {
                    sample_rate: 48000,
                    channels: 1,
                    sample_format: "f32",
                    buffer_frames: Some(480),
                };
                if mode == Mode::Input {
                    work.run.started(format.clone()).unwrap();
                } else {
                    work.run
                        .started_playback(
                            (mode == Mode::Loopback).then_some(format.clone()),
                            format.clone(),
                        )
                        .unwrap();
                }
                match action {
                    "close" => {
                        service.close_session(&session).unwrap();
                        service.close_session(&session).unwrap();
                    }
                    "remount" => {
                        service.open_session().unwrap();
                    }
                    "refresh" => {
                        assert_eq!(
                            service.begin_refresh(&session).err().unwrap().code,
                            "capture_busy"
                        );
                    }
                    "select_input" => {
                        service
                            .select(
                                &session,
                                inventory.revision,
                                "input",
                                &inventory.inputs[1].id,
                            )
                            .unwrap();
                    }
                    "select_output" => {
                        service
                            .select(
                                &session,
                                inventory.revision,
                                "output",
                                &inventory.outputs[1].id,
                            )
                            .unwrap();
                    }
                    "exit" => {
                        service.shutdown();
                        service.shutdown();
                    }
                    _ => unreachable!(),
                }
                assert!(!work.run.enabled(), "{action} left {mode:?} enabled");
                assert!(work.run.started(format).is_err());
                assert!(service.0.lock().unwrap().capture.busy().unwrap());
                work.execute_test(|_, _| panic!("cancelled work restarted native audio"));
                assert!(!service.0.lock().unwrap().capture.busy().unwrap());
                if action == "refresh" {
                    assert!(service.begin_refresh(&session).is_ok());
                }
                if action == "exit" {
                    assert!(service.open_session().is_err());
                }
            }
        }
    }

    #[test]
    fn logout_cancels_capture_and_an_old_close_cannot_stop_the_new_session() {
        let (service, session) = fixture();
        let old = queued_job(&service);
        service.close_session(&session).unwrap();
        old.execute(|_, _| panic!("logout left queued capture enabled"));
        let next = service.open_session().unwrap();
        let new = queued_job(&service);
        service.close_session(&session).unwrap();
        assert!(new.run.enabled());
        assert!(service.capture_state(&next, 1).is_ok());
        drop(new);
    }

    #[test]
    fn refresh_stops_capture_and_waits_for_its_native_cleanup_before_scan_admission() {
        let (service, session) = fixture();
        let job = queued_job(&service);
        assert_eq!(
            service.begin_refresh(&session).err().unwrap().code,
            "capture_busy"
        );
        assert!(!job.run.enabled());
        job.execute(|_, _| panic!("refresh did not stop queued capture"));
        assert!(service.begin_refresh(&session).is_ok());
    }

    #[test]
    fn shutdown_cancels_both_queued_native_work_types_and_rejects_new_commands() {
        let (service, session) = fixture();
        let scan = service.begin_refresh(&session).unwrap();
        let job = queued_job(&service); // private injection; neither adapter is run
        service.shutdown();
        job.execute(|_, _| panic!("shutdown left capture enabled"));
        assert!(scan.run(|_| panic!("shutdown left scan enabled")).is_err());
        assert!(service.open_session().is_err());
    }

    #[test]
    #[ignore = "Explicit OS metadata probe; never captures or plays audio"]
    fn native_device_metadata() -> Result<(), AudioError> {
        let service = service();
        let session = service.open_session()?;
        let inventory = tauri::async_runtime::block_on(refresh(&service, &session))?;
        println!(
            "{} metadata: {} explicit inputs, {} explicit outputs; defaults reported: input={}, output={}",
            inventory.backend,
            inventory.inputs.iter().filter(|d| !d.is_default).count(),
            inventory.outputs.iter().filter(|d| !d.is_default).count(),
            inventory.inputs.iter().any(|d| d.is_default && d.available),
            inventory.outputs.iter().any(|d| d.is_default && d.available),
        );
        if let Some(input) = inventory
            .inputs
            .iter()
            .find(|d| !d.is_default && d.available)
        {
            let selected = service.select(&session, inventory.revision, "input", &input.id)?;
            assert_eq!(selected.input_id, input.id);
            assert_eq!(selected.output_id, "output-default");
        }
        service.close_session(&session)?;
        assert!(service.begin_refresh(&session).is_err());
        Ok(())
    }
}
