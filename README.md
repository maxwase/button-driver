Button driver
=============
[![crates](https://img.shields.io/crates/v/button-driver?style=for-the-badge)](https://crates.io/crates/button-driver/)
[![doc](https://img.shields.io/docsrs/button-driver?style=for-the-badge)](https://docs.rs/button-driver/latest/)

This crate is a button driver for embedded Rust projects.
It offers various usage scenarios, supports ESP, `embedded_hal`, `embassy` and `no_std` targets.

This crate aims to be as flexible as possible to support various HALs and use-cases.

## Examples

For more examples consider looking into the [examples](https://github.com/maxwase/button-driver/tree/master/examples) folder.
You can easily flash them using `cargo run` command!

For **ESP32C3** with std:

Required features: `std`, `embedded_hal`
```rust
use std::time::Instant;

use button_driver::{Button, ButtonConfig};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio9)?;

    let mut button = Button::<_, Instant>::new(pin, ButtonConfig::default());

    loop {
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
    }
}
```

## TODO
1. `async` [support](https://github.com/maxwase/button-driver/issues/1)
2. Debounce strategies [support](https://github.com/maxwase/button-driver/issues/12)

## Algorithm
High-level state machine diagram

![button-driver-state-machine](https://github.com/maxwase/button-driver/assets/23321756/fd19165a-6107-4a7d-8050-9897afd523c6)