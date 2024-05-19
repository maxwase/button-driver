use core::time::Duration;

/// This trait defines the interface between the driver and platform APIs typically provided by operating system or timer.
pub trait Platform {
    /// Returns the current time as a monotonic duration since the start of the program
    fn duration_since_init(&self) -> Duration;
}


cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub(crate) use std_platform::DefaultPlatform;
    } else if #[cfg(feature = "embassy")] {
        pub(crate) use embassy_platform::DefaultPlatform;
    } else {
        compile_error!("unsupport error")
    }
}

#[cfg(feature = "std")]
mod std_platform {
    use core::time::Duration;
    use std::time::Instant;

    use super::Platform;
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
}

#[cfg(feature = "embassy")]
mod embassy_platform {
    use core::time::Duration;
    use embassy_time::Instant;

    use super::Platform;
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
            return (Instant::now() - self.start).into();
        }
    }
}
