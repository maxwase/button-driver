//! An example of how to use embassy's `Instant`.
//!
//! The button is polled by a `Ticker`, so the loop yields to the executor between
//! ticks instead of busy-looping. You might also want to move it into a separate task.
//!
//! Required features: `embassy`, `embedded_hal_old`
#![no_std]
#![no_main]

use button_driver::{Button, ButtonConfig, Mode};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Duration, Instant, Ticker};
use {defmt_rtt as _, panic_probe as _};

/// Polling period. Must be well below `ButtonConfig::debounce` (900 µs by default).
const POLL_PERIOD: Duration = Duration::from_micros(250);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut led = Output::new(p.PC13, Level::High, Speed::Low);
    let button_pin = Input::new(p.PC15, Pull::Down);

    let config = ButtonConfig {
        mode: Mode::PullDown,
        ..Default::default()
    };
    let mut button = Button::<_, Instant, Duration>::new(button_pin, config);
    let mut ticker = Ticker::every(POLL_PERIOD);
    let mut hold_reported = false;

    loop {
        button.tick();

        if let Some(dur) = button.held_time() {
            info!(
                "Total holding time {} ms ({} holds, {} clicks)",
                dur.as_millis(),
                button.holds(),
                button.clicks()
            );
        } else if button.is_clicked() {
            info!("Click");
            led.set_low();
        } else if button.is_double_clicked() {
            info!("Double click");
            led.set_high();
        } else if button.is_triple_clicked() {
            info!("Triple click");
        }

        // Report the hold once instead of on every poll.
        match button.current_holding_time() {
            Some(dur) if !hold_reported => {
                hold_reported = true;
                info!("Held for {} ms", dur.as_millis());
            }
            Some(_) => {}
            None => hold_reported = false,
        }

        button.reset();
        ticker.next().await;
    }
}
