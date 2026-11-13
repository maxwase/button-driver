/// An abstraction over different switching APIs.
pub trait PinWrapper {
    /// Is source on?
    fn is_high(&mut self) -> bool;

    /// Is source off?
    fn is_low(&mut self) -> bool {
        !self.is_high()
    }

    /// Is source on? Returns [None] if the source cannot be read.
    ///
    /// [Button::tick](crate::Button::tick) skips the update when this returns [None],
    /// so a failing pin is never interpreted as a press or a release.
    fn try_is_high(&mut self) -> Option<bool> {
        Some(self.is_high())
    }
}

#[cfg(all(feature = "embedded_hal_old", feature = "embedded_hal"))]
compile_error!(
    "features `embedded_hal_old` and `embedded_hal` (or `esp`) are mutually exclusive: enable only one"
);

#[cfg(feature = "embedded_hal_old")]
impl<P> PinWrapper for P
where
    Self: embedded_hal_old::digital::v2::InputPin,
{
    fn is_high(&mut self) -> bool {
        embedded_hal_old::digital::v2::InputPin::is_high(self).unwrap_or_default()
    }

    fn try_is_high(&mut self) -> Option<bool> {
        embedded_hal_old::digital::v2::InputPin::is_high(self).ok()
    }
}

#[cfg(feature = "embedded_hal")]
impl<P> PinWrapper for P
where
    Self: embedded_hal::digital::InputPin,
{
    fn is_high(&mut self) -> bool {
        embedded_hal::digital::InputPin::is_high(self).unwrap_or_default()
    }

    fn try_is_high(&mut self) -> Option<bool> {
        embedded_hal::digital::InputPin::is_high(self).ok()
    }
}

#[cfg(all(test, feature = "std"))]
pub(crate) mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread::sleep,
        time::{Duration, Instant},
    };

    use crate::{Button, ButtonConfig, Mode, PinWrapper, State};

    pub const CONFIG: ButtonConfig = ButtonConfig {
        hold: Duration::from_millis(500),
        debounce: Duration::from_micros(700),
        release: Duration::from_millis(30),
        mode: Mode::PullDown,
    };

    #[derive(Debug, Default, Clone)]
    pub struct MockPin(Arc<AtomicBool>);

    impl PinWrapper for MockPin {
        fn is_high(&mut self) -> bool {
            self.0.load(Ordering::SeqCst)
        }
    }

    impl Button<MockPin, Instant> {
        /// Simulate pin state changes corresponding to one full button click with debounce.
        pub fn press_button(&mut self) {
            self.pin.press();
            self.tick();
            assert!(matches!(self.state, State::Down(_)));

            sleep(CONFIG.debounce);
            self.tick();
        }

        /// Simulate pin state changes corresponding to one full button release with debounce.
        pub fn release_button(&mut self) {
            self.pin.release();
            self.tick();

            sleep(CONFIG.debounce);
            self.tick();
        }

        /// Simulate pin state changes corresponding to one full button hold with debounce.
        pub fn hold_button(&mut self) {
            self.press_button();
            sleep(CONFIG.hold);
            self.tick();
            self.release_button();
        }
    }

    impl MockPin {
        /// Press the pin with debounce.
        pub fn press(&self) {
            self.0.store(true, Ordering::SeqCst);
            sleep(CONFIG.debounce);
        }

        /// Release the pin with debounce.
        pub fn release(&self) {
            self.0.store(false, Ordering::SeqCst);
            sleep(CONFIG.debounce);
        }

        /// Simulate pin state changes corresponding to one full button click with debounce.
        pub fn click(&self) {
            self.press();
            self.release();
        }

        /// Simulate pin state changes corresponding to one full button hold with debounce.
        pub fn hold(&self) {
            self.press();
            sleep(CONFIG.hold);
            self.release();
        }
    }
}
