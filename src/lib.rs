#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

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
///
/// A click is counted when a debounced press is released before the hold time;
/// a hold is counted when the press lasts for the hold time. A new streak starts
/// (and the counters are cleared) on the first debounced press after the release timeout.
/// Finished results are only visible while the button is in the [Released](State::Released) state,
/// so poll them before the next press.
#[derive(Clone, Debug)]
pub struct Button<P, I, D = Duration> {
    /// An inner pin.
    pub pin: P,
    state: State<I>,
    clicks: usize,
    held: Option<D>,
    holds: usize,
    config: ButtonConfig<D>,
    /// Start of the last debounced release within the current streak.
    /// [None] when the button is idle, used to tell a new streak from a continued one.
    release_at: Option<I>,
    /// Start of a release that is still being debounced.
    release_pending: Option<I>,
}

/// Represents current button state.
///
///
/// State machine diagram:
///```ignore
/// Down => Pressed | Released | Up (bounce, back to the previous state)
/// Pressed => Held | Up
/// Held => Up
/// Up => Released | Down
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
            holds: 0,
            held: None,
            release_at: None,
            release_pending: None,
        }
    }

    /// Returns the number of clicks that happened before the last release.
    /// Returns 0 if clicks are still being counted or a new streak has started.
    pub fn clicks(&self) -> usize {
        if self.state == State::Released {
            self.clicks
        } else {
            0
        }
    }

    /// Returns the number of holds (how many times the button was held) that happened before the last release.
    /// Returns 0 if clicks or holds are still being counted or a new streak has started.
    pub fn holds(&self) -> usize {
        if self.state == State::Released {
            self.holds
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
            self.holds = 0;
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

    /// Returns holding duration of the last hold in the streak, measured from the press to the release.
    /// Returns [None] if the button is still being held, not released or was not held at all.
    pub fn held_time(&self) -> Option<D> {
        if self.state == State::Released {
            self.held.clone()
        } else {
            None
        }
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
    pub const fn raw_state(&self) -> &State<I> {
        &self.state
    }

    /// Returns current amount of clicks, ignoring release timeout.
    pub const fn raw_clicks(&self) -> usize {
        self.clicks
    }

    /// Returns current amount of holds (how many times the button was held), ignoring release timeout.
    pub const fn raw_holds(&self) -> usize {
        self.holds
    }

    /// Updates button state.
    /// Call as frequently as you can, ideally in a loop in separate thread or a **periodic** timer interrupt,
    /// with a period well below [`debounce`](ButtonConfig#structfield.debounce).
    /// Do not call it only from a pin-change (edge) interrupt: debounce, hold and release
    /// timeouts advance only when `tick` runs, so the button would get stuck between edges.
    ///
    /// Both press and release edges are debounced: the pin must stay at the new level
    /// for at least [`debounce`](ButtonConfig#structfield.debounce) before the transition is accepted.
    /// If the pin cannot be read (see [`PinWrapper::try_is_high`]), the tick is skipped.
    pub fn tick(&mut self) {
        let Some(pressed) = self.read_pin_pressed() else {
            return;
        };

        match self.state.clone() {
            State::Unknown | State::Released => {
                self.state = if pressed {
                    State::Down(I::now())
                } else {
                    State::Released
                };
            }

            State::Down(start) => {
                if pressed {
                    if start.elapsed() >= self.config.debounce {
                        if self.release_at.is_none() {
                            // First debounced press after an idle period: start a new streak.
                            self.clicks = 0;
                            self.holds = 0;
                            self.held = None;
                        }
                        self.release_pending = None;
                        self.state = State::Pressed(start);
                    } else {
                        // debounce
                    }
                } else {
                    // Bounce: return to the state we came from without counting anything.
                    self.state = match &self.release_at {
                        Some(released) => State::Up(released.clone()),
                        None => State::Released,
                    };
                }
            }

            State::Pressed(start) => {
                if pressed {
                    self.release_pending = None;
                    if start.elapsed() >= self.config.hold {
                        self.holds += 1;
                        self.state = State::Held(start);
                    } else {
                        // holding
                    }
                } else if let Some(released) = self.debounced_release() {
                    self.clicks += 1;
                    self.release_at = Some(released.clone());
                    self.state = State::Up(released);
                }
            }

            State::Held(start) => {
                if pressed {
                    self.release_pending = None;
                } else if let Some(released) = self.debounced_release() {
                    self.held = Some(released.clone() - start);
                    self.release_at = Some(released.clone());
                    self.state = State::Up(released);
                }
            }

            State::Up(released) => {
                if released.elapsed() >= self.config.release {
                    self.release_at = None;
                    self.state = State::Released;
                } else if pressed {
                    self.state = State::Down(I::now());
                } else {
                    // waiting for the release timeout
                }
            }
        }
    }

    /// Tracks a pending release while the button is pressed or held.
    ///
    /// Returns the instant the release started once the pin has stayed released
    /// for at least the debounce time.
    fn debounced_release(&mut self) -> Option<I> {
        let since = self.release_pending.get_or_insert_with(I::now);
        if since.elapsed() >= self.config.debounce {
            self.release_pending.take()
        } else {
            None
        }
    }

    /// Reads current pin status without debouncing.
    /// Returns [None] if the pin cannot be read, otherwise [true] if the button is pressed.
    fn read_pin_pressed(&mut self) -> Option<bool> {
        let is_high = self.pin.try_is_high()?;
        Some(is_high != self.config.mode.is_pullup())
    }
}
