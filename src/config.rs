use super::Duration;

/// Various [Button] parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonConfig {
    /// How much time the button should be pressed to in order to count it as a press.
    pub debounce: Duration,
    /// How much time the button should not be holed to be released.
    pub release: Duration,
    /// How much time the button should be pressed to be held.
    pub hold: Duration,
    /// Button direction.
    pub mode: Mode,
}

impl ButtonConfig {
    /// Returns new [ButtonConfig].
    ///
    /// As a general rule, `debounce` time is less then `release` time and `hold` time is larger them both.
    pub fn new(debounce: Duration, release: Duration, hold: Duration, mode: Mode) -> Self {
        Self {
            debounce,
            release,
            hold,
            mode,
        }
    }
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            debounce: Duration::from_micros(900),
            release: Duration::from_millis(150),
            hold: Duration::from_millis(500),
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
