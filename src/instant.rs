use core::{ops::Sub, time::Duration};

/// An abstraction for retrieving the current time.
///
/// The underlying counter shell be monotonic in order for the crate to
/// operate correctly.
pub trait InstantProvider<D = Duration>
where
    // `Clone` is less strict then `Copy` and usually implemented using it.
    Self: Sub<Self, Output = D> + Clone,
{
    /// Returns an instant corresponding to "now".
    fn now() -> Self;

    /// Returns the amount of time elapsed since this instant.
    fn elapsed(&self) -> D {
        Self::now() - self.clone()
    }
}

#[cfg(feature = "std")]
impl InstantProvider<std::time::Duration> for std::time::Instant {
    fn now() -> Self {
        std::time::Instant::now()
    }
}

#[cfg(feature = "embassy")]
impl InstantProvider<embassy_time::Duration> for embassy_time::Instant {
    fn now() -> Self {
        embassy_time::Instant::now()
    }
}
