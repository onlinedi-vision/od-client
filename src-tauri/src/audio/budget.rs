use super::catalog::AudioError;
use std::time::Duration;

// Control-plane checks only; never called by the real-time sample callback.
pub(super) trait Budget {
    fn check(&self) -> Result<Duration, AudioError>;
}

impl Budget for super::catalog::RefreshPermit {
    fn check(&self) -> Result<Duration, AudioError> {
        self.check()
    }
}
