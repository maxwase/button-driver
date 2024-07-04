use core::time::Duration;

/// This trait defines the interface between the driver and platform APIs typically provided by operating system or timer.
pub trait Platform {
    /// Returns the current time as a monotonic duration since the start of the program
    fn duration_since_init(&self) -> Duration;
}

use std::time::Instant;

pub struct DefaultPlatform {
    start: Instant,
}

impl Default for DefaultPlatform {
    fn default() -> Self {
        return Self {
            start: Instant::now(),
        };
    }
}

impl Platform for DefaultPlatform {
    fn duration_since_init(&self) -> Duration {
        return Instant::now() - self.start;
    }
}
