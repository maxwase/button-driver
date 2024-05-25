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

        assert_eq!(button.clicks(), 1)
    }

    // double click
    {
        button.press_button();
        button.release_button();

        button.press_button();
        button.release_button();

        sleep(CONFIG.release);
        button.tick();

        assert_eq!(button.clicks(), 2)
    }
}

#[test]
fn test_thread() {
    let pin = MockPin::default();

    let button = Arc::new(Mutex::new(Button::<_, Instant>::new(pin.clone(), CONFIG)));

    let button1 = button.clone();
    spawn(move || loop {
        button1.lock().tick();
    });

    sleep(Duration::from_millis(50));

    // single click
    {
        pin.click();

        sleep(CONFIG.release);

        let button = button.lock();
        assert_eq!(button.clicks(), 1)
    }

    // double click
    {
        pin.click();
        pin.click();

        sleep(CONFIG.release);

        let button = button.lock();
        assert_eq!(button.clicks(), 2);
    }

    // two single clicks
    {
        pin.click();
        sleep(CONFIG.release);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 1);
        drop(btn);

        pin.click();
        sleep(CONFIG.release);

        let button = button.lock();
        assert_eq!(button.clicks(), 1);
    }

    // holding
    {
        pin.press();
        sleep(Duration::from_millis(1));
        assert_eq!(button.lock().raw_clicks(), 1);
        sleep(CONFIG.hold);
        let btn = button.lock();
        assert_eq!(btn.clicks(), 0);
        assert_eq!(btn.raw_clicks(), 0);
        drop(btn);

        pin.release();
        sleep(CONFIG.release);

        let button = button.lock();
        assert_eq!(button.clicks(), 0);
        assert_eq!(button.state, State::Released);
        assert!(button.held_time().unwrap() > CONFIG.hold);
    }
}
