use std::{
    sync::Arc,
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use parking_lot::Mutex;

use super::pin_wrapper::tests::*;
use super::*;

#[test]
fn test_sequential() {
    let pin = MockPin::default();

    let mut button = Button::new(pin, CONFIG);
    button.tick();
    assert_eq!(button.state, State::Released);

    // single click
    {
        button.press_button();
        assert!(matches!(button.state, State::Pressed(_)));

        button.release_button();
        assert!(matches!(button.state, State::Up(_)));

        sleep(CONFIG.release);
        button.tick();
        assert!(matches!(button.state, State::Released));

        assert_eq!(button.clicks(), 1);
        button.reset();
    }

    // double click
    {
        button.press_button();
        button.release_button();

        button.press_button();
        button.release_button();

        sleep(CONFIG.release);
        button.tick();

        assert_eq!(button.clicks(), 2);
        button.reset();
    }

    // double hold with clicks
    {
        button.press_button();
        button.release_button();

        button.press_button();
        button.release_button();

        button.press_button();
        sleep(CONFIG.hold);
        button.tick();
        button.release_button();
        button.tick();

        button.hold_button();

        sleep(CONFIG.release);
        button.tick();

        assert_eq!(button.clicks(), 2);
        assert_eq!(button.holds(), 2);
        button.reset();
    }
}

/// Start a ticking thread.
fn prepare_button(pin: &MockPin) -> Arc<Mutex<Button<MockPin, Instant>>> {
    let button = Arc::new(Mutex::new(Button::<_, Instant>::new(pin.clone(), CONFIG)));

    let button1 = button.clone();
    spawn(move || loop {
        button1.lock().tick();
    });

    sleep(Duration::from_millis(50));
    button
}

#[test]
fn test_thread_clicks() {
    let pin = MockPin::default();
    let button = prepare_button(&pin);

    // single click
    {
        pin.click();

        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 1);
        button.reset()
    }

    // double click
    {
        pin.click();
        pin.click();

        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 2);
        button.reset()
    }

    // two single clicks
    {
        pin.click();
        sleep(CONFIG.release);
        let mut btn = button.lock();
        assert_eq!(btn.clicks(), 1);
        btn.reset();
        drop(btn);

        pin.click();
        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 1);
        button.reset()
    }
}

#[test]
fn test_thread_holds() {
    let pin = MockPin::default();
    let button = prepare_button(&pin);

    // holding
    {
        pin.press();
        assert_eq!(button.lock().raw_clicks(), 0);
        sleep(CONFIG.hold);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 0);
        assert_eq!(btn.holds(), 0);
        assert_eq!(btn.raw_clicks(), 0);
        assert_eq!(btn.raw_holds(), 1);
        drop(btn);

        pin.release();
        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 0);
        assert_eq!(button.holds(), 1);
        assert_eq!(button.state, State::Released);
        assert!(button.held_time().unwrap() > CONFIG.hold);
        button.reset()
    }

    // holds
    {
        pin.press();
        assert_eq!(button.lock().raw_clicks(), 0);
        sleep(CONFIG.hold);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 0);
        assert_eq!(btn.raw_clicks(), 0);
        drop(btn);
        pin.release();

        pin.hold();
        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 0);
        assert_eq!(button.holds(), 2);
        assert_eq!(button.state, State::Released);
        assert!(button.held_time().unwrap() > CONFIG.hold);
        button.reset()
    }
}

#[test]
fn test_thread_clicks_holds() {
    let pin = MockPin::default();
    let button = prepare_button(&pin);

    // clicks + holding
    {
        pin.click();
        pin.click();
        pin.click();

        assert_eq!(button.lock().raw_clicks(), 3);

        pin.press();
        assert_eq!(button.lock().raw_clicks(), 3);
        sleep(CONFIG.hold);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 0);
        assert_eq!(btn.raw_clicks(), 3);
        drop(btn);

        pin.release();
        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 3);
        assert_eq!(button.state, State::Released);
        assert!(button.held_time().unwrap() > CONFIG.hold);
        button.reset()
    }

    // clicks + holds
    {
        pin.click();
        pin.click();
        pin.click();

        assert_eq!(button.lock().raw_clicks(), 3);

        pin.press();
        assert_eq!(button.lock().raw_clicks(), 3);
        sleep(CONFIG.hold);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 0);
        assert_eq!(btn.holds(), 0);
        assert_eq!(btn.raw_clicks(), 3);
        assert_eq!(btn.raw_holds(), 1);
        drop(btn);
        pin.release();

        pin.hold();
        pin.hold();

        sleep(CONFIG.release);

        let mut button = button.lock();
        assert_eq!(button.clicks(), 3);
        assert_eq!(button.holds(), 3);
        assert_eq!(button.state, State::Released);
        assert!(button.held_time().unwrap() > CONFIG.hold);
        button.reset()
    }
}

/// Deterministic tests driven by a simulated clock instead of `sleep`.
mod simulated {
    use std::{cell::Cell, ops::Sub, rc::Rc, time::Duration};

    use crate::{tests::CONFIG, Button, InstantProvider, PinWrapper, State};

    thread_local! {
        static NOW: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    }

    /// Simulation step between two ticks.
    const STEP: Duration = Duration::from_micros(100);
    /// Shorter than `CONFIG.debounce`.
    const GLITCH: Duration = Duration::from_micros(300);
    /// Longer than `CONFIG.debounce`, shorter than `CONFIG.release`.
    const SHORT: Duration = Duration::from_millis(10);
    /// Longer than `CONFIG.release`.
    const IDLE: Duration = Duration::from_millis(50);

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct SimInstant(Duration);

    impl Sub for SimInstant {
        type Output = Duration;

        fn sub(self, rhs: Self) -> Duration {
            self.0.saturating_sub(rhs.0)
        }
    }

    impl InstantProvider for SimInstant {
        fn now() -> Self {
            SimInstant(NOW.with(Cell::get))
        }
    }

    #[derive(Clone, Default)]
    struct SimPin(Rc<Cell<Option<bool>>>);

    impl PinWrapper for SimPin {
        fn is_high(&mut self) -> bool {
            self.try_is_high().unwrap_or_default()
        }

        fn try_is_high(&mut self) -> Option<bool> {
            self.0.get()
        }
    }

    struct Sim {
        button: Button<SimPin, SimInstant>,
        pin: Rc<Cell<Option<bool>>>,
    }

    impl Sim {
        fn new() -> Self {
            let pin = SimPin(Rc::new(Cell::new(Some(false))));
            let level = pin.0.clone();
            let mut button = Button::new(pin, CONFIG);
            button.tick();
            assert_eq!(button.state, State::Released);
            Self { button, pin: level }
        }

        /// Keeps the pin at `level` for `duration`, ticking every [STEP].
        fn drive(&mut self, level: Option<bool>, duration: Duration) {
            self.pin.set(level);
            let mut elapsed = Duration::ZERO;
            while elapsed < duration {
                self.button.tick();
                NOW.with(|now| now.set(now.get() + STEP));
                elapsed += STEP;
            }
            self.button.tick();
        }

        fn press(&mut self, duration: Duration) {
            // `CONFIG` is `PullDown`: high means pressed.
            self.drive(Some(true), duration);
        }

        fn release(&mut self, duration: Duration) {
            self.drive(Some(false), duration);
        }
    }

    #[test]
    fn glitch_from_released_is_not_a_click() {
        let mut sim = Sim::new();
        sim.press(GLITCH);
        sim.release(IDLE);

        assert_eq!(sim.button.state, State::Released);
        assert_eq!(sim.button.raw_clicks(), 0);
        assert_eq!(sim.button.clicks(), 0);
    }

    #[test]
    fn glitch_in_up_keeps_streak() {
        let mut sim = Sim::new();
        sim.press(SHORT);
        sim.release(SHORT);
        assert!(sim.button.state.is_up());

        sim.press(GLITCH);
        sim.release(Duration::from_micros(200));
        assert!(sim.button.state.is_up());
        assert_eq!(sim.button.raw_clicks(), 1);

        sim.release(IDLE);
        assert_eq!(sim.button.clicks(), 1);
    }

    #[test]
    fn release_bounce_in_pressed_is_single_click() {
        let mut sim = Sim::new();
        sim.press(SHORT);
        sim.release(GLITCH);
        sim.press(SHORT);
        sim.release(IDLE);

        assert_eq!(sim.button.clicks(), 1);
        assert_eq!(sim.button.holds(), 0);
    }

    #[test]
    fn release_bounce_in_held_is_single_hold() {
        let mut sim = Sim::new();
        sim.press(Duration::from_millis(300));
        sim.press(Duration::from_millis(300));
        assert!(sim.button.state.is_held());
        sim.release(GLITCH);
        sim.press(Duration::from_millis(100));
        sim.release(IDLE);

        assert_eq!(sim.button.clicks(), 0);
        assert_eq!(sim.button.holds(), 1);
        let held = sim.button.held_time().unwrap();
        assert!(held >= Duration::from_millis(700), "{held:?}");
        assert!(held < Duration::from_millis(701), "{held:?}");
    }

    #[test]
    fn new_streak_resets_counters() {
        let mut sim = Sim::new();
        sim.press(SHORT);
        sim.release(SHORT);
        sim.press(Duration::from_millis(600));
        sim.release(IDLE);
        assert_eq!(sim.button.clicks(), 1);
        assert_eq!(sim.button.holds(), 1);
        assert!(sim.button.held_time().is_some());

        // No `reset()` call between streaks.
        sim.press(SHORT);
        sim.release(IDLE);
        assert_eq!(sim.button.clicks(), 1);
        assert_eq!(sim.button.holds(), 0);
        assert_eq!(sim.button.held_time(), None);
    }

    #[test]
    fn hold_then_click() {
        let mut sim = Sim::new();
        sim.press(Duration::from_millis(600));
        sim.release(SHORT);
        sim.press(SHORT);
        sim.release(IDLE);

        assert_eq!(sim.button.clicks(), 1);
        assert_eq!(sim.button.holds(), 1);
    }

    #[test]
    fn unreadable_pin_is_ignored() {
        let mut sim = Sim::new();
        sim.press(SHORT);
        sim.drive(None, SHORT);
        assert!(sim.button.state.is_pressed());
        sim.release(IDLE);

        assert_eq!(sim.button.clicks(), 1);
    }
}
