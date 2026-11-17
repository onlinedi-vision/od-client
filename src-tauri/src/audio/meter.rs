//! Latest RMS only. No PCM storage, allocations, locks, IPC, or callback logging.
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

#[derive(Default)]
pub(super) struct Meter {
    rms: AtomicU32,
    fault: AtomicU8,
    overruns: AtomicU32,
    last_samples: AtomicU32,
}

impl Meter {
    pub fn samples(&self, samples: impl ExactSizeIterator<Item = f32>) {
        let count = samples.len();
        if count == 0 {
            return;
        }
        self.last_samples
            .store(count.min(u32::MAX as usize) as u32, Ordering::Relaxed);
        let mut squares = 0.0f64;
        for sample in samples {
            // Reject non-finite data instead of propagating NaNs to UI.
            if !sample.is_finite() {
                self.fail(1);
                return;
            }
            squares += f64::from(sample.clamp(-1.0, 1.0)).powi(2);
        }
        self.rms.store(
            ((squares / count as f64).sqrt() as f32).to_bits(),
            Ordering::Relaxed,
        );
    }

    pub fn silence(&self) {
        self.rms.store(0, Ordering::Relaxed);
    }

    pub fn fail(&self, code: u8) {
        // First fatal error only; bounded single CAS, no retry loop.
        let _ = self
            .fault
            .compare_exchange(0, code, Ordering::Relaxed, Ordering::Relaxed);
    }

    pub fn fault(&self) -> u8 {
        self.fault.load(Ordering::Relaxed)
    }

    pub fn overrun(&self) {
        self.overruns.fetch_add(1, Ordering::Relaxed);
    }
    pub fn overruns(&self) -> u32 {
        self.overruns.load(Ordering::Relaxed)
    }

    pub fn last_samples(&self) -> u32 {
        self.last_samples.load(Ordering::Relaxed)
    }

    pub fn level(&self) -> u8 {
        let rms = f32::from_bits(self.rms.swap(0, Ordering::Relaxed));
        // Display -60..0 dBFS as 0..100. No gain/AGC is applied to audio.
        if rms <= 0.001 || !rms.is_finite() {
            0
        } else {
            ((20.0 * rms.log10() + 60.0) * (100.0 / 60.0))
                .clamp(0.0, 100.0)
                .round() as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rms_is_bounded_finite_and_latest_only_without_retaining_samples() {
        let meter = Meter::default();
        meter.samples([0.01; 4].into_iter());
        assert_eq!(meter.level(), 33); // -40 dBFS
        assert_eq!(meter.level(), 0); // no stale level without new samples
        meter.samples([0.1, -0.1].into_iter());
        meter.samples([1.0, -1.0].into_iter());
        assert_eq!(meter.level(), 100);
        meter.samples([0.0; 4].into_iter());
        assert_eq!(meter.level(), 0);
        meter.samples([f32::NAN].into_iter());
        meter.fail(2);
        assert_eq!(meter.fault(), 1);
        assert_eq!(meter.level(), 0);
    }
}
