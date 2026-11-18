//! Bounded local playout. Unique producer/consumer, atomic slots: no unsafe
//! queue, callback locks, allocations, retries, or retained recording.
use super::catalog::AudioError;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

const MAX_BYTES: usize = 65536;
const MAX_AGE_NS: i64 = 40_000_000;
const GAIN: f32 = 0.1;
const PEAK: f32 = 0.25;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub capacity_frames: u32,
    pub queued_frames: u32,
    pub high_water_frames: u32,
    pub dropped_frames: u32,
    pub expired_frames: u32,
    pub underruns: u32,
    pub overflows: u32,
    pub timing_misses: u32,
    pub output_buffer_frames: u32,
}

pub(super) struct Stats {
    clock: Instant,
    capacity: AtomicU32,
    queued: AtomicU32,
    high_water: AtomicU32,
    dropped: AtomicU32,
    expired: AtomicU32,
    underruns: AtomicU32,
    overflows: AtomicU32,
    timing_misses: AtomicU32,
    output_buffer_frames: AtomicU32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            clock: Instant::now(),
            capacity: 0.into(),
            queued: 0.into(),
            high_water: 0.into(),
            dropped: 0.into(),
            expired: 0.into(),
            underruns: 0.into(),
            overflows: 0.into(),
            timing_misses: 0.into(),
            output_buffer_frames: 0.into(),
        }
    }
}

impl Stats {
    pub fn now(&self) -> i64 {
        self.clock.elapsed().as_nanos().min(i64::MAX as u128) as i64
    }
    pub fn captured_at(&self, age: Duration) -> i64 {
        self.now()
            .saturating_sub(age.as_nanos().min(i64::MAX as u128) as i64)
    }
    pub fn timing_miss(&self) {
        self.timing_misses.fetch_add(1, Ordering::Relaxed);
    }
    pub fn output_buffer(&self, frames: usize) {
        self.output_buffer_frames
            .store(frames.min(u32::MAX as usize) as u32, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> Diagnostics {
        let load = |value: &AtomicU32| value.load(Ordering::Relaxed);
        Diagnostics {
            capacity_frames: load(&self.capacity),
            queued_frames: load(&self.queued),
            high_water_frames: load(&self.high_water),
            dropped_frames: load(&self.dropped),
            expired_frames: load(&self.expired),
            underruns: load(&self.underruns),
            overflows: load(&self.overflows),
            timing_misses: load(&self.timing_misses),
            output_buffer_frames: load(&self.output_buffer_frames),
        }
    }
}

struct Frame {
    left: AtomicU32,
    right: AtomicU32,
    captured: AtomicI64,
}
struct Shared {
    slots: Box<[Frame]>,
    written: AtomicU64,
    read: AtomicU64,
    overflow: AtomicBool,
    stats: Arc<Stats>,
    rate: u32,
    channels: u16,
    target: usize,
}

#[derive(Default)]
struct Cursor {
    count: u64,
    position: usize,
}
impl Cursor {
    fn advance(&mut self, frames: usize, capacity: usize) {
        self.count = self.count.wrapping_add(frames as u64);
        self.position = (self.position + frames) % capacity;
    }
}

pub(super) struct Producer {
    shared: Arc<Shared>,
    cursor: Cursor,
}
pub(super) struct Consumer {
    shared: Arc<Shared>,
    cursor: Cursor,
}

pub(super) fn queue(
    rate: u32,
    channels: u16,
    stats: Arc<Stats>,
) -> Result<(Producer, Consumer), AudioError> {
    if rate == 0 || !(1..=2).contains(&channels) {
        return Err(incompatible());
    }
    // Include per-frame capture timestamps/atomics in the 64 KiB bound. At
    // 48 kHz: 1,920 frames / 30,720 bytes total; PCM itself is 7,680/15,360 bytes.
    let capacity = (u64::from(rate) * 40 / 1000)
        .min((MAX_BYTES / std::mem::size_of::<Frame>()) as u64) as usize;
    if capacity == 0 {
        return Err(incompatible());
    }
    let slots = (0..capacity)
        .map(|_| Frame {
            left: 0.into(),
            right: 0.into(),
            captured: 0.into(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    stats.capacity.store(capacity as u32, Ordering::Relaxed);
    let shared = Arc::new(Shared {
        slots,
        written: 0.into(),
        read: 0.into(),
        overflow: false.into(),
        stats,
        rate,
        channels,
        target: (rate as usize / 100).min(capacity).max(1),
    });
    Ok((
        Producer {
            shared: Arc::clone(&shared),
            cursor: Cursor::default(),
        },
        Consumer {
            shared,
            cursor: Cursor::default(),
        },
    ))
}

pub(super) fn incompatible() -> AudioError {
    AudioError::new("incompatible_audio", "Input and output need compatible sample rates and mono/stereo layouts. Choose compatible devices or system audio profiles, then start again.")
}

fn sample(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}
pub(super) fn output_sample(value: f32) -> f32 {
    (sample(value) * GAIN).clamp(-PEAK, PEAK)
}

impl Producer {
    pub fn push(
        &mut self,
        mut samples: impl ExactSizeIterator<Item = f32>,
        input_channels: u16,
        captured: i64,
    ) {
        if !(1..=2).contains(&input_channels) {
            return;
        }
        let shared = &self.shared;
        let frames = samples.len() / usize::from(input_channels);
        let queued = self
            .cursor
            .count
            .wrapping_sub(shared.read.load(Ordering::Acquire)) as usize;
        let accepted = frames.min(shared.slots.len().saturating_sub(queued));
        for index in 0..accepted {
            let left = sample(samples.next().unwrap_or(0.0));
            let right = if input_channels == 2 {
                sample(samples.next().unwrap_or(0.0))
            } else {
                left
            };
            let (left, right) = if shared.channels == 1 {
                ((left + right) * 0.5, 0.0)
            } else {
                (left, right)
            };
            let slot = &shared.slots[self.cursor.position];
            slot.left
                .store(output_sample(left).to_bits(), Ordering::Relaxed);
            slot.right
                .store(output_sample(right).to_bits(), Ordering::Relaxed);
            slot.captured.store(
                captured.saturating_add((index as i64 * 1_000_000_000) / i64::from(shared.rate)),
                Ordering::Relaxed,
            );
            self.cursor.advance(1, shared.slots.len());
        }
        shared.written.store(self.cursor.count, Ordering::Release);
        let depth = (queued + accepted).min(shared.slots.len()) as u32;
        shared.stats.queued.store(depth, Ordering::Relaxed);
        // Only the producer writes high_water, so a CAS/retry is unnecessary.
        if depth > shared.stats.high_water.load(Ordering::Relaxed) {
            shared.stats.high_water.store(depth, Ordering::Relaxed);
        }
        if accepted < frames {
            shared.stats.dropped.fetch_add(
                (frames - accepted).min(u32::MAX as usize) as u32,
                Ordering::Relaxed,
            );
            shared.stats.overflows.fetch_add(1, Ordering::Relaxed);
            shared.overflow.store(true, Ordering::Release);
        }
    }
}

impl Consumer {
    pub fn render(&mut self, frames: usize, now: i64, mut write: impl FnMut(usize, [f32; 2])) {
        let shared = &self.shared;
        let overflow = shared.overflow.swap(false, Ordering::AcqRel);
        let written = shared.written.load(Ordering::Acquire);
        let mut queued = (written.wrapping_sub(self.cursor.count) as usize).min(shared.slots.len());
        if overflow && queued > shared.target {
            let skip = queued - shared.target;
            self.cursor.advance(skip, shared.slots.len());
            queued -= skip;
            shared
                .stats
                .dropped
                .fetch_add(skip as u32, Ordering::Relaxed);
        }
        let mut missing = false;
        let mut inspected = 0;
        for index in 0..frames {
            let mut frame = [0.0; 2];
            let mut present = false;
            while queued > 0 && inspected < shared.slots.len() {
                let slot = &shared.slots[self.cursor.position];
                let captured = slot.captured.load(Ordering::Relaxed);
                self.cursor.advance(1, shared.slots.len());
                queued -= 1;
                inspected += 1;
                if now.saturating_sub(captured) > MAX_AGE_NS
                    || captured > now.saturating_add(1_000_000)
                {
                    shared.stats.expired.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                frame = [
                    f32::from_bits(slot.left.load(Ordering::Relaxed)),
                    f32::from_bits(slot.right.load(Ordering::Relaxed)),
                ];
                present = true;
                break;
            }
            missing |= !present;
            write(index, frame);
        }
        shared.read.store(self.cursor.count, Ordering::Release);
        shared.stats.queued.store(queued as u32, Ordering::Relaxed);
        if missing {
            shared.stats.underruns.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        self.shared.read.store(
            self.shared.written.load(Ordering::Acquire),
            Ordering::Release,
        );
        self.shared.stats.queued.store(0, Ordering::Relaxed);
    }
}

pub(super) enum Signal {
    Tone { rate: u32, frame: u32 },
    Loopback(Consumer),
}
impl Signal {
    pub fn render(
        &mut self,
        frames: usize,
        now: i64,
        enabled: bool,
        mut write: impl FnMut(usize, [f32; 2]),
    ) {
        if !enabled {
            for index in 0..frames {
                write(index, [0.0; 2]);
            }
            return;
        }
        match self {
            Self::Loopback(consumer) => consumer.render(frames, now, write),
            Self::Tone { rate, frame } => {
                for index in 0..frames {
                    let value = if *frame < *rate {
                        let value = (std::f64::consts::TAU * 440.0 * f64::from(*frame)
                            / f64::from(*rate))
                        .sin() as f32;
                        *frame += 1;
                        output_sample(value)
                    } else {
                        0.0
                    };
                    write(index, [value; 2]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
