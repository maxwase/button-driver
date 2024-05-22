use core::time::Duration;

/// This trait defines the interface between the driver and platform APIs typically provided by operating system or timer.
pub trait Platform {
    /// Returns the current time as a monotonic duration since the start of the program
    fn duration_since_init(&self) -> Duration;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std_platform::DefaultPlatform;
    } else if #[cfg(feature = "embassy")] {
        pub use embassy_platform::DefaultPlatform;
    } else {
        pub use unsupported_platform::DefaultPlatform;
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

mod unsupported_platform {
    use core::time::Duration;

    use super::Platform;
    pub struct DefaultPlatform {}

    impl Default for DefaultPlatform {
        fn default() -> Self {
            unimplemented!("unsupported platform, you can use Button::new_with_platform() to implement custom platform")
        }
    }

    impl Platform for DefaultPlatform {
        fn duration_since_init(&self) -> Duration {
            unimplemented!("unsupported platform, you can use Button::new_with_platform() to implement custom platform")
        }
    }
}