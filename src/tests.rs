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
        assert_eq!(button.lock().raw_clicks(), 1);
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
        assert_eq!(button.lock().raw_clicks(), 1);
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
        assert_eq!(button.lock().raw_clicks(), 4);
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
        assert_eq!(button.lock().raw_clicks(), 4);
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
