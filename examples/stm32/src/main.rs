//! An example of how to define an external `Instant` for the `Button` using an interrupt. Typically, you would
//! prefer to use a `DWT`; `stm32f1xx_hal` even has one, however this example aims to be more general.
//!
//! Required features: `embedded_hal_old` or `embedded_hal` with some modifications.
#![no_std]
#![no_main]

use core::{cell::RefCell, ops::Sub, time::Duration};
use cortex_m::interrupt::Mutex;

use button_driver::{Button, ButtonConfig, InstantProvider, Mode};
use panic_halt as _;

use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    pac::{self, interrupt, Interrupt, TIM2},
    prelude::*,
    timer::{CounterMs, Event},
};

/// This setting affects how fast a button can track a state change.
// Maximum resolution supported by the timer.
const TIMER_PERIOD: Duration = Duration::from_millis(2);
/// How much time has passed since the interrupt start?
static mut GLOBAL_TIMER_COUNTER: Duration = Duration::ZERO;
/// A way to move a timer into an interrupt handler.
static GLOBAL_TIM: Mutex<RefCell<Option<CounterMs<TIM2>>>> = Mutex::new(RefCell::new(None));

/// Retrieve the current time.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Instant {
    counter: Duration,
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        self.counter - rhs.counter
    }
}

impl InstantProvider<Duration> for Instant {
    fn now() -> Self {
        Instant {
            counter: cortex_m::interrupt::free(|_| unsafe { GLOBAL_TIMER_COUNTER }),
        }
    }
}

#[interrupt]
fn TIM2() {
    static mut TIM: Option<CounterMs<TIM2>> = None;

    let tim = TIM.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| {
            // Move timer here, leaving a None in its place
            GLOBAL_TIM.borrow(cs).replace(None).unwrap()
        })
    });

    cortex_m::interrupt::free(|_| unsafe { GLOBAL_TIMER_COUNTER += TIMER_PERIOD });
    let _ = tim.wait();
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    rtt_init_print!();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split();

    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer
        .start((TIMER_PERIOD.as_millis() as u32).millis())
        .unwrap();
    // Generate an interrupt when the timer expires
    timer.listen(Event::Update);

    cortex_m::interrupt::free(|cs| *GLOBAL_TIM.borrow(cs).borrow_mut() = Some(timer));

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    let mut led_pin = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let button_pin = gpioc.pc15.into_pull_down_input(&mut gpioc.crh);

    let config = ButtonConfig {
        mode: Mode::PullDown,
        ..Default::default()
    };
    let mut button = Button::<_, Instant>::new(button_pin, config);

    loop {
        button.tick();

        if button.is_clicked() {
            led_pin.set_low();
            rprintln!("Click");
        } else if button.is_double_clicked() {
            led_pin.set_high();
            rprintln!("Double click");
        } else if button.is_triple_clicked() {
            rprintln!("Triple click");
        } else if let Some(dur) = button.current_holding_time() {
            rprintln!("Held for {:?}", dur);
        } else if let Some(dur) = button.held_time() {
            rprintln!("Total holding time {:?}", dur);
        }

        button.reset();
    }
}
