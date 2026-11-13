//! An example of how to define an external `Instant` for the `Button` using a timer interrupt.
//! Typically, you would prefer a free-running hardware counter such as `DWT::CYCCNT`
//! (`stm32f1xx_hal` even has a wrapper for it), however this example aims to be more general.
//!
//! The timer period defines the clock resolution and must be well below the
//! button debounce time, otherwise debouncing becomes unreliable.
//!
//! Required features: `embedded_hal_old` or `embedded_hal` with some modifications.
#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    ops::Sub,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use cortex_m::interrupt::Mutex;

use button_driver::{Button, ButtonConfig, InstantProvider, Mode};
use panic_halt as _;

use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    pac::{self, interrupt, Interrupt, TIM2},
    prelude::*,
    rcc,
    timer::{CounterUs, Event},
};

/// Clock resolution. Must be well below `ButtonConfig::debounce` (900 µs by default).
const TIMER_PERIOD_US: u32 = 100;
/// Number of timer periods since start. Wraps after ~119 hours, see [Instant::sub].
static TICKS: AtomicU32 = AtomicU32::new(0);
/// A way to move a timer into an interrupt handler.
static GLOBAL_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));

/// Retrieve the current time.
#[derive(Clone, Debug, PartialEq)]
struct Instant {
    ticks: u32,
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, rhs: Instant) -> Self::Output {
        // `wrapping_sub` keeps the difference correct across a counter overflow.
        let ticks = self.ticks.wrapping_sub(rhs.ticks);
        Duration::from_micros(u64::from(TIMER_PERIOD_US)).saturating_mul(ticks)
    }
}

impl InstantProvider<Duration> for Instant {
    fn now() -> Self {
        Instant {
            ticks: TICKS.load(Ordering::Relaxed),
        }
    }
}

#[interrupt]
fn TIM2() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    cortex_m::interrupt::free(|cs| {
        if let Some(tim) = GLOBAL_TIM.borrow(cs).borrow_mut().as_mut() {
            tim.clear_interrupt(Event::Update);
        }
    });
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    rtt_init_print!();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.freeze(rcc::Config::default(), &mut flash.acr);

    let mut gpioc = dp.GPIOC.split(&mut rcc);

    let mut timer = dp.TIM2.counter_us(&mut rcc);
    timer.start(TIMER_PERIOD_US.micros()).unwrap();
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
    let mut hold_reported = false;

    loop {
        button.tick();

        if let Some(dur) = button.held_time() {
            rprintln!(
                "Total holding time {:?} ({} holds, {} clicks)",
                dur,
                button.holds(),
                button.clicks()
            );
        } else if button.is_clicked() {
            led_pin.set_low();
            rprintln!("Click");
        } else if button.is_double_clicked() {
            led_pin.set_high();
            rprintln!("Double click");
        } else if button.is_triple_clicked() {
            rprintln!("Triple click");
        }

        // Report the hold once instead of on every loop iteration.
        match button.current_holding_time() {
            Some(dur) if !hold_reported => {
                hold_reported = true;
                rprintln!("Held for {:?}", dur);
            }
            Some(_) => {}
            None => hold_reported = false,
        }

        button.reset();
    }
}
