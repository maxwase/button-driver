#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(all(feature = "embassy", not(feature = "std")), no_std)]

use core::time::Duration;

pub use config::{ButtonConfig, Mode};
pub use instant::InstantProvider;
pub use pin_wrapper::PinWrapper;

/// Button configuration.
pub mod config;
/// Different current global time sources.
pub mod instant;
/// Wrappers for different APIs.
mod pin_wrapper;

#[cfg(all(test, feature = "std"))]
mod tests;

/// Generic button abstraction.
///
/// The crate is designed to provide a finished ([`released`](ButtonConfig#structfield.release)) state by the accessor methods.
/// However, it is also possible to get the `raw` state using the corresponding methods.
#[derive(Clone, Debug)]
pub struct Button<P, I, D = Duration> {
    /// An inner pin.
    pub pin: P,
    state: State<I>,
    clicks: usize,
    held: Option<D>,
    config: ButtonConfig<D>,
}

/// Represents current button state.
///
///
/// State machine diagram:
///```ignore
/// Down => Pressed | Released
/// Pressed => Held => Up
/// Up => Released | Down
/// Held => Released
/// Released => Down
/// Unknown => Down | Released
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State<I> {
    /// The button has been just pressed, so it is in *down* position.
    Down(I),
    /// Debounced press.
    Pressed(I),
    /// The button has been just released, so it is in *up* position.
    Up(I),
    /// The button is being held.
    Held(I),
    /// Fully released state, idle.
    Released,
    /// Initial state.
    Unknown,
}

impl<I: PartialEq> State<I> {
    /// Returns [true] if the state is [Down](State::Down).
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down(_))
    }

    /// Returns [true] if the state is [Pressed](State::Pressed).
    pub fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed(_))
    }

    /// Returns [true] if the state is [Up](State::Up).
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Up(_))
    }

    /// Returns [true] if the state is [Held](State::Held).
    pub fn is_held(&self) -> bool {
        matches!(self, Self::Held(_))
    }

    /// Returns [true] if the state is [Released](State::Released).
    pub fn is_released(&self) -> bool {
        *self == Self::Released
    }

    /// Returns [true] if the state is [Unknown](State::Unknown).
    pub fn is_unknown(&self) -> bool {
        *self == Self::Unknown
    }
}

impl<P, I, D> Button<P, I, D>
where
    P: PinWrapper,
    I: InstantProvider<D> + PartialEq,
    D: Clone + Ord,
{
    /// Creates a new [Button].
    pub const fn new(pin: P, config: ButtonConfig<D>) -> Self {
        Self {
            pin,
            config,
            state: State::Unknown,
            clicks: 0,
            held: None,
        }
    }

    /// Returns number of clicks that happened before last release.
    /// Returns 0 if clicks are still being counted or a new streak has started.
    pub fn clicks(&self) -> usize {
        if self.state == State::Released {
            self.clicks
        } else {
            0
        }
    }

    /// Resets clicks amount and held time after release.
    ///
    /// Example:
    ///
    /// In this example, reset method makes "Clicked!" print once per click.
    /// ```ignore
    /// let mut button = Button::new(pin, ButtonConfig::default());
    ///
    /// loop {
    ///     button.tick();
    ///     
    ///     if button.is_clicked() {
    ///         println!("Clicked!");
    ///     }
    ///
    ///     button.reset();
    /// }
    /// ```
    pub fn reset(&mut self) {
        if self.state == State::Released {
            self.clicks = 0;
            self.held = None;
        }
    }

    /// Returns [true] if the button was pressed once before release.
    pub fn is_clicked(&self) -> bool {
        self.clicks() == 1
    }

    /// Returns [true] if the button was pressed twice before release.
    pub fn is_double_clicked(&self) -> bool {
        self.clicks() == 2
    }

    /// Returns [true] if the button was pressed three times before release.
    pub fn is_triple_clicked(&self) -> bool {
        self.clicks() == 3
    }

    /// Returns holing duration before last release.
    /// Returns [None] if the button is still being held or was not held at all.
    pub fn held_time(&self) -> Option<D> {
        self.held.clone()
    }

    /// Returns current holding duration.
    /// Returns [None] if the button is not being held.
    pub fn current_holding_time(&self) -> Option<D> {
        if let State::Held(dur) = &self.state {
            Some(dur.elapsed())
        } else {
            None
        }
    }

    /// Returns current button state.
    pub fn raw_state(&self) -> &State<I> {
        &self.state
    }

    /// Returns current amount of clicks, ignoring release timeout.
    pub fn raw_clicks(&self) -> usize {
        self.clicks
    }

    /// Updates button state.
    /// Call as frequently as you can, ideally in a loop in separate thread or interrupt.
    pub fn tick(&mut self) {
        match &self.state {
            State::Unknown if self.is_pin_pressed() => {
                self.clicks = 1;
                self.state = State::Down(I::now());
            }
            State::Unknown if self.is_pin_released() => self.state = State::Released,

            State::Down(elapsed) => {
                if self.is_pin_pressed() {
                    if elapsed.elapsed() >= self.config.debounce {
                        self.state = State::Pressed(elapsed.clone());
                    } else {
                        // debounce
                    }
                } else {
                    self.state = State::Released;
                }
            }
            State::Pressed(elapsed) => {
                if self.is_pin_pressed() {
                    if elapsed.elapsed() >= self.config.hold {
                        self.clicks = 0;
                        self.state = State::Held(elapsed.clone());
                    } else {
                        // holding
                    }
                } else {
                    self.state = State::Up(I::now())
                }
            }
            State::Up(elapsed) => {
                if elapsed.elapsed() < self.config.release {
                    if self.is_pin_pressed() {
                        self.clicks += 1;
                        self.state = State::Down(I::now());
                    } else {
                        // waiting for the release timeout
                    }
                } else {
                    self.state = State::Released;
                }
            }

            State::Released if self.is_pin_pressed() => {
                self.clicks = 1;
                self.held = None;
                self.state = State::Down(I::now());
            }
            State::Held(elapsed) if self.is_pin_released() => {
                self.held = Some(elapsed.elapsed());
                self.state = State::Released;
            }
            _ => {}
        }
    }

    /// Reads current pin status, returns [true] if the button pin is released without debouncing.
    fn is_pin_released(&self) -> bool {
        self.pin.is_high() == self.config.mode.is_pullup()
    }

    /// Reads current pin status, returns [true] if the button pin is pressed without debouncing.
    fn is_pin_pressed(&self) -> bool {
        !self.is_pin_released()
    }
}
