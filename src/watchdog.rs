//! Independent watchdog (IWDG) — resets the MCU if the firmware wedges.
//!
//! IWDG runs off the ~32 kHz LSI (RM0090 §21 Independent watchdog). We set a
//! ~20 s timeout and "pet" it every ~1.5 s from this task. If any task deadlocks
//! and the executor stops scheduling this one, the IWDG fires and the next boot
//! reports `iwdg_timeout` (see fault.rs). This is the last-resort liveness net.
//!
//! Embassy exposes the IWDG as a driver; configure the timeout and call .pet()/
//! .unleash() per the embassy-stm32 API.

use embassy_stm32::peripherals::IWDG;

/// ~20 s timeout, petted every 1.5 s. (LSI ~32 kHz, RM0090 §21.)
/// TODO: let mut wd = Iwdg::new(iwdg, 20_000_000 us); wd.unleash();
/// loop: wd.pet(); embassy_time::Timer::after_millis(1500).await;
#[embassy_executor::task]
pub async fn watchdog_task(iwdg: IWDG) {
    let _ = iwdg;
    todo!("create IWDG with ~20 s timeout, unleash, pet every 1.5 s")
}
