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
    AllOn,
}

impl LedState {
    pub fn next(self) -> Self {
        match self {
            LedState::AllOff => LedState::SlowBlink,
            LedState::SlowBlink => LedState::FastBlink,
            LedState::FastBlink => LedState::AllOn,
            LedState::AllOn => LedState::AllOff,
        }
    }

    pub fn period_ms(self) -> u64 {
        match self {
            LedState::AllOff => 0,
            LedState::SlowBlink => 500,
            LedState::FastBlink => 100,
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
    loop {
        let state = STATE.lock(|s| s.get());
        let period = state.period_ms();

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
