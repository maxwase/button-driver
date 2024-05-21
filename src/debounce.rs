use crate::{Button, InstantProvider, State};

/// An abstraction over debounce strategies.
pub trait DebounceStrategy<P, I, D>
where
    Self: Sized,
{
    /// The method returns [true] if current button press is considered debounced.
    fn is_debounced(&self, button: &Button<P, I, D, Self>) -> bool;
}

/// No software debounce. Can be useful if the pin is debounced using hardware.  
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoDebounce;

impl<P, I, D> DebounceStrategy<P, I, D> for NoDebounce {
    fn is_debounced(&self, _: &Button<P, I, D, Self>) -> bool {
        true
    }
}

/// A classic "wait to ensure it's a click".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeBased<D> {
    debounce: D,
}

impl<D> TimeBased<D> {
    /// Create a new [TimeBased] debounce.
    pub const fn new(debounce: D) -> Self {
        Self { debounce }
    }

    /// Get the debounce duration.
    pub const fn debounce(&self) -> &D {
        &self.debounce
    }
}

impl<P, I: InstantProvider<D>, D: Ord> DebounceStrategy<P, I, D> for TimeBased<D> {
    fn is_debounced(&self, button: &Button<P, I, D, Self>) -> bool {
        matches!(&button.state, State::Down(elapsed) if elapsed.elapsed() >= self.debounce)
    }
}

/// Skip some clicks.
// TODO: Won't work as `click` is only incremented after debounce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClickBased {
    clicks: usize,
}

impl ClickBased {
    /// Create a new [ClickBased] debounce.
    pub const fn new(clicks: usize) -> Self {
        Self { clicks }
    }

    /// Get the debounce click amount.
    pub const fn clicks(&self) -> usize {
        self.clicks
    }
}

impl<P, I, D> DebounceStrategy<P, I, D> for ClickBased {
    fn is_debounced(&self, button: &Button<P, I, D, Self>) -> bool {
        button.clicks > self.clicks
    }
}
