use std::{cell::RefCell, ops::Sub, rc::Rc, time::Duration};

use button_driver::{Button, ButtonConfig, InstantProvider, Mode, PinWrapper};
use log::info;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{js_sys::Date, MouseEvent};

#[derive(PartialEq, Clone)]
struct WasmInstant(u64);

impl WasmInstant {
    fn now() -> WasmInstant {
        WasmInstant(Date::now() as u64)
    }
}

impl Sub for WasmInstant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let delta_ms = self.0 - rhs.0;
        return Duration::from_millis(delta_ms);
    }
}

impl InstantProvider<Duration> for WasmInstant {
    fn now() -> Self {
        WasmInstant::now()
    }
}

#[derive(Clone, Default)]
struct MockButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MockButtonPin {
    fn is_high(&mut self) -> bool {
        *self.0.borrow()
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let document = web_sys::window().unwrap().document().unwrap();
    let clickable_area = document.query_selector(".clickable-area").unwrap().unwrap();
    let pin_state = MockButtonPin::default();
    {
        let pin_state_ref = pin_state.clone();
        let mouse_down_handler = Closure::wrap(Box::new(move |_: MouseEvent| {
            *pin_state_ref.0.borrow_mut() = true;
        }) as Box<dyn FnMut(_)>);
        clickable_area
            .add_event_listener_with_callback(
                "mousedown",
                mouse_down_handler.as_ref().unchecked_ref(),
            )
            .unwrap();
        mouse_down_handler.forget();
    }
    {
        let pin_state_ref = pin_state.clone();
        let mouse_up_handler = Closure::wrap(Box::new(move |_: MouseEvent| {
            *pin_state_ref.0.borrow_mut() = false;
        }) as Box<dyn FnMut(_)>);

        clickable_area
            .add_event_listener_with_callback("mouseup", mouse_up_handler.as_ref().unchecked_ref())
            .unwrap();
        mouse_up_handler.forget();
    }
    {
        let mut button = Button::<_, WasmInstant>::new(
            pin_state,
            ButtonConfig {
                mode: Mode::PullDown,
                ..Default::default()
            },
        );
        let callback = Closure::wrap(Box::new(move || {
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
            } else {
                if button.is_clicked() {
                    info!("Click");
                } else if button.is_double_clicked() {
                    info!("Double click");
                } else if button.is_triple_clicked() {
                    info!("Triple click");
                } else if let Some(dur) = button.current_holding_time() {
                    info!("Held for {:?}", dur);
                }
            }

            button.reset();
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                20,
            )
            .unwrap();
        callback.forget();
    }
}
