use core::{ops::Sub, time::Duration};

/// An abstraction for retrieving the current time.
///
/// The underlying counter shall be monotonic in order for the crate to
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

#[cfg(feature = "wasm")]
pub mod wasm {
    //! A `performance.now()`-based implementation of `InstantProvider`.
    //! This can be useful for `wasm32-*` targets.

    use core::{ops::Sub, time::Duration};

    use js_sys::{Function, Reflect};
    use wasm_bindgen::{JsCast, JsValue};

    use crate::InstantProvider;

    /// A monotonic timestamp that implements `InstantProvider`.
    ///
    /// Uses `performance.now()` (monotonic, sub-millisecond resolution) when available
    /// and falls back to `Date.now()` (wall clock, millisecond resolution) otherwise.
    /// The value is stored in milliseconds.
    #[derive(Clone, PartialEq)]
    pub struct Instant(f64);

    impl Sub for Instant {
        type Output = Duration;

        fn sub(self, rhs: Self) -> Self::Output {
            // Clamp to zero in case the fallback clock goes backwards.
            Duration::from_secs_f64(((self.0 - rhs.0) / 1000.0).max(0.0))
        }
    }

    fn performance_now() -> Option<f64> {
        let performance =
            Reflect::get(&js_sys::global(), &JsValue::from_str("performance")).ok()?;
        let now = Reflect::get(&performance, &JsValue::from_str("now"))
            .ok()?
            .dyn_into::<Function>()
            .ok()?;
        now.call0(&performance).ok()?.as_f64()
    }

    impl InstantProvider<Duration> for Instant {
        fn now() -> Self {
            Instant(performance_now().unwrap_or_else(js_sys::Date::now))
        }
    }
}
