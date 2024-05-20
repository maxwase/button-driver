use core::time::Duration;

/// Default debounce time for a button.
pub const DEFAULT_DEBOUNCE: Duration = Duration::from_micros(900);
/// Default release time for a button.
pub const DEFAULT_RELEASE: Duration = Duration::from_millis(150);
/// Default hold time for a button.
pub const DEFAULT_HOLD: Duration = Duration::from_millis(500);

/// Various [Button] parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonConfig<D> {
    /// How much time the button should be pressed to in order to count it as a press.
    pub debounce: D,
    /// How much time the button should not be holed to be released.
    pub release: D,
    /// How much time the button should be pressed to be held.
    pub hold: D,
    /// Button direction.
    pub mode: Mode,
}

impl<D> ButtonConfig<D> {
    /// Returns new [ButtonConfig].
    ///
    /// As a general rule, `debounce` time is less then `release` time and `hold` time is larger them both.
    pub fn new(debounce: D, release: D, hold: D, mode: Mode) -> Self {
        Self {
            debounce,
            release,
            hold,
            mode,
        }
    }
}

#[cfg(feature = "std")]
impl Default for ButtonConfig<Duration> {
    fn default() -> Self {
        Self {
            debounce: DEFAULT_DEBOUNCE,
            release: DEFAULT_RELEASE,
            hold: DEFAULT_HOLD,
            mode: Mode::default(),
        }
    }
}

#[cfg(feature = "embassy")]
impl Default for ButtonConfig<embassy_time::Duration> {
    fn default() -> Self {
        use embassy_time::Duration;
        // `as` is safe here because these contacts won't exceed `u64` limit
        Self {
            debounce: Duration::from_micros(DEFAULT_DEBOUNCE.as_micros() as u64),
            release: Duration::from_millis(DEFAULT_RELEASE.as_millis() as u64),
            hold: Duration::from_millis(DEFAULT_HOLD.as_millis() as u64),
            mode: Mode::default(),
        }
    }
}

/// Button direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Active 0.
    #[default]
    PullUp,
    /// Active 1.
    PullDown,
}

impl Mode {
    /// Is button activated by logic zero?
    pub const fn is_pullup(&self) -> bool {
        matches!(self, Mode::PullUp)
    }

    /// Is button activated by logic one?
    pub const fn is_pulldown(&self) -> bool {
        !self.is_pullup()
    }
}
