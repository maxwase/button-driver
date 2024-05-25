Button driver
=============
[![crates](https://img.shields.io/crates/v/button-driver?style=for-the-badge)](https://crates.io/crates/button-driver/)
[![doc](https://img.shields.io/docsrs/button-driver?style=for-the-badge)](https://docs.rs/button-driver/latest/)

This crate is a button driver for embedded Rust projects.
It offers various usage scenarios, supports ESP, `embedded_hal`, `embassy` and `no_std` targets.

This crate aims to be as flexible as possible to support various HALs and use-cases.

## Examples

For more examples consider looking into the [examples](https://github.com/maxwase/button-driver/tree/master/examples) folder.

For **ESP32C3** with std:

Required features: `std`, `esp`
```rust
use button_driver::{Button, ButtonConfig};
use esp_idf_hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pin = PinDriver::input(peripherals.pins.gpio9)?;

    let mut button = Button::new(pin, ButtonConfig::default());

    loop {
        button.tick();

        if button.is_clicked() {
            info!("Click");
        } else if button.is_double_clicked() {
            info!("Double click");
        } else if button.is_triple_clicked() {
            info!("Triple click");
        } else if let Some(dur) = button.current_holding_time() {
            info!("Held for {dur:?}");
        } else if let Some(dur) = button.held_time() {
            info!("Total holding time {time:?}");
        }

        button.reset();
    }
}
```

## TODO
1. `embassy` `async` [support](https://github.com/maxwase/button-driver/issues/1)
2. `no_std` ESP32 [support](https://github.com/maxwase/button-driver/issues/2)

## Algorithm
High level state-machine diagram

<img src="https://github.com/maxwase/button-driver/assets/23321756/e066694a-2379-4805-82e5-18e4a3a557de" width=150% height=150%>