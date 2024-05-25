#[cfg(feature = "esp")]
use esp_idf_hal::gpio::{Input, InputOutput, Pin, PinDriver};

/// An abstraction over different switching APIs.
pub trait PinWrapper {
    /// Is source on?
    fn is_high(&mut self) -> bool;

    /// Is source off?
    fn is_low(&mut self) -> bool {
        !self.is_high()
    }
}

#[cfg(feature = "esp")]
impl<'d, P: Pin> PinWrapper for PinDriver<'d, P, Input> {
    fn is_high(&mut self) -> bool {
        self.is_high()
    }
}

#[cfg(feature = "esp")]
impl<'d, P: Pin> PinWrapper for PinDriver<'d, P, InputOutput> {
    fn is_high(&mut self) -> bool {
        self.is_high()
    }
}

#[cfg(feature = "embedded_hal_old")]
impl<P> PinWrapper for P
where
    Self: embedded_hal_old::digital::v2::InputPin,
{
    fn is_high(&mut self) -> bool {
        embedded_hal_old::digital::v2::InputPin::is_high(self).unwrap_or_default()
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
        pub fn press_button(&mut self) {
            self.pin.press();
            self.tick();
            assert!(matches!(self.state, State::Down(_)));

            sleep(CONFIG.debounce);
            self.tick();
        }

        pub fn release_button(&mut self) {
            self.pin.release();
            self.tick();
        }
    }

    impl MockPin {
        pub fn press(&self) {
            self.0.store(true, Ordering::SeqCst);
            sleep(CONFIG.debounce);
        }

        pub fn release(&self) {
            self.0.store(false, Ordering::SeqCst);
            sleep(CONFIG.debounce);
        }

        pub fn click(&self) {
            self.press();
            self.release();
        }
    }
}
