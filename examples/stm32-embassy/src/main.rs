//! An example of how to use embassy's `Instant`. You might also want to separate it into another task.
//! Required features: `embassy`, `embedded_hal_old`
#![no_std]
#![no_main]

use button_driver::{Button, ButtonConfig, Mode};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::Instant;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    let button_pin = Input::new(p.PC15, Pull::Down);

    let config = ButtonConfig {
        mode: Mode::PullDown,
        ..Default::default()
    };
    let mut button = Button::<_, Instant, embassy_time::Duration>::new(button_pin, config);

    loop {
        button.tick();

        if button.is_clicked() {
            info!("Click");
            led.set_low();
        } else if button.is_double_clicked() {
            info!("Double click");
            led.set_high();
        } else if button.is_triple_clicked() {
            info!("Triple click");
        } else if let Some(dur) = button.current_holding_time() {
            info!("Held for {:?}", dur);
        } else if let Some(dur) = button.held_time() {
            info!("Total holding time {:?}", dur);
        }

        button.reset();
    }
}
