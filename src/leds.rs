use core::cell::Cell;

use embassy_stm32::gpio::Output;
use embassy_sync::{
    blocking_mutex::{Mutex, raw::CriticalSectionRawMutex},
    signal::Signal,
};
use embassy_time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum LedState {
    AllOff,
    SlowBlink,
    FastBlink,
    Chase, // running lights: LD1→LD2→LD3 in sequence
    AllOn,
}

impl LedState {
    pub fn next(self) -> Self {
        match self {
            LedState::AllOff => LedState::SlowBlink,
            LedState::SlowBlink => LedState::FastBlink,
            LedState::FastBlink => LedState::Chase,
            LedState::Chase => LedState::AllOn,
            LedState::AllOn => LedState::AllOff,
        }
    }

    /// Half-period for blink states (ms). 0 = static (AllOff/AllOn). Chase has its
    /// own per-step timing in `led_task`, so it returns its step interval here too.
    pub fn period_ms(self) -> u64 {
        match self {
            LedState::AllOff => 0,
            LedState::SlowBlink => 500,
            LedState::FastBlink => 100,
            LedState::Chase => 150, // per-step dwell
            LedState::AllOn => 0,
        }
    }
}

pub static STATE: Mutex<CriticalSectionRawMutex, Cell<LedState>> =
    Mutex::new(Cell::new(LedState::AllOff));

pub static CHANGED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn led_task(
    mut led1: Output<'static>,
    mut led2: Output<'static>,
    mut led3: Output<'static>,
) {
    // Which LED is lit in the Chase animation; persists across loop iterations.
    let mut chase_idx: usize = 0;

    loop {
        let state = STATE.lock(|s| s.get());
        let period = state.period_ms();

        // Running lights: exactly one LED on, advancing each step. Stays
        // responsive to CHANGED so a button press switches state immediately.
        if state == LedState::Chase {
            led1.set_low();
            led2.set_low();
            led3.set_low();
            match chase_idx {
                0 => led1.set_high(),
                1 => led2.set_high(),
                _ => led3.set_high(),
            }
            chase_idx = (chase_idx + 1) % 3;
            // If CHANGED fires during the dwell, re-read state next iteration.
            let _ = embassy_time::with_timeout(Duration::from_millis(period), CHANGED.wait()).await;
            continue;
        }

        if period == 0 {
            match state {
                LedState::AllOn => {
                    led1.set_high();
                    led2.set_high();
                    led3.set_high();
                }
                _ => {
                    led1.set_low();
                    led2.set_low();
                    led3.set_low();
                }
            }
            CHANGED.wait().await;
            continue;
        }

        led1.set_high();
        led2.set_high();
        led3.set_high();
        if embassy_time::with_timeout(Duration::from_millis(period), CHANGED.wait())
            .await
            .is_ok()
        {
            continue;
        }

        led1.set_low();
        led2.set_low();
        led3.set_low();
        let _ = embassy_time::with_timeout(Duration::from_millis(period), CHANGED.wait()).await;
    }
}
