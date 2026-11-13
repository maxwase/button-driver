//! A WebAssembly example demonstrating button driver usage in the browser.
//! This example uses the built-in [button_driver::instant::wasm::Instant] type which wraps `performance.now()`.
//!
//! To run this example:
//! 1. Install trunk: `cargo install trunk`
//! 2. Run: `trunk serve`
//! 3. Open your browser at `http://localhost:8080`
//!
//! Required features: `wasm`.

use std::{cell::Cell, rc::Rc};

use button_driver::{instant::wasm::Instant, Button, ButtonConfig, Mode, PinWrapper};
use log::info;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{EventTarget, PointerEvent};

/// Polling period in milliseconds. Browser timers are coarse, so the default
/// debounce effectively becomes one polling period.
const POLL_PERIOD_MS: i32 = 10;

#[derive(Clone, Default)]
struct ButtonInput(Rc<Cell<bool>>);

impl PinWrapper for ButtonInput {
    fn is_high(&mut self) -> bool {
        self.0.get()
    }
}

/// Sets the pin to `level` whenever `event` fires on `target`.
fn bind(target: &EventTarget, event: &str, pin: &ButtonInput, level: bool) {
    let pin = pin.clone();
    let handler = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
        // Only the primary (left) button acts as the physical button.
        if event.button() == 0 || !level {
            pin.0.set(level);
        }
    });
    target
        .add_event_listener_with_callback(event, handler.as_ref().unchecked_ref())
        .unwrap();
    handler.forget();
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let clickable_area = document.query_selector(".clickable-area").unwrap().unwrap();

    let pin_state = ButtonInput::default();

    // Pointer events cover mouse, touch and pen input.
    bind(&clickable_area, "pointerdown", &pin_state, true);
    // Listen for the release on the whole window: the pointer may leave the
    // button before it is released, which would otherwise leave it "held" forever.
    bind(&window, "pointerup", &pin_state, false);
    bind(&window, "pointercancel", &pin_state, false);

    let mut button = Button::<_, Instant>::new(
        pin_state,
        ButtonConfig {
            mode: Mode::PullDown,
            ..Default::default()
        },
    );
    let mut hold_reported = false;

    let callback = Closure::<dyn FnMut()>::new(move || {
        button.tick();

        if let Some(dur) = button.held_time() {
            info!("Total holding time {:?}", dur);
            if button.is_clicked() {
                info!("Clicked + held");
            } else if button.is_double_clicked() {
                info!("Double clicked + held");
            } else if button.holds() == 2 && button.clicks() > 0 {
                info!("Held twice with {} clicks", button.clicks());
            } else if button.holds() == 2 {
                info!("Held twice");
            }
        } else if button.is_clicked() {
            info!("Click");
        } else if button.is_double_clicked() {
            info!("Double click");
        } else if button.is_triple_clicked() {
            info!("Triple click");
        }

        // Report the hold once instead of on every poll.
        match button.current_holding_time() {
            Some(dur) if !hold_reported => {
                hold_reported = true;
                info!("Held for {:?}", dur);
            }
            Some(_) => {}
            None => hold_reported = false,
        }

        button.reset();
    });

    window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            POLL_PERIOD_MS,
        )
        .unwrap();
    callback.forget();
}
