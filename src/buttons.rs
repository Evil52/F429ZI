//! B1 USER button → cycle the LED state machine, OR (long hold) enter USB DFU.
//! Pin = PC13 (UM1974 §7.6, p.25, SB173 ON default). On Nucleo-144 the USER
//! button press drives the line HIGH (board pulls to GND otherwise), so we use
//! Pull::None + rising edge. 50 ms debounce on each edge.
//!
//! Two gestures on the one button:
//!   • short press  → advance the LED state machine (unchanged behaviour).
//!   • hold ≥ 3 s   → dfu::reboot_to_dfu(): reboot into the ROM USB-DFU loader so
//!                    the board can be re-flashed over USB with ./dfu.sh (no
//!                    ST-Link, no BOOT0 jumper). In a real product you'd trigger
//!                    reboot_to_dfu() from your own UI/host command instead; the
//!                    button is just the bench gesture for the Nucleo.

use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_stm32::{exti::ExtiInput, mode::Async};
use embassy_time::{Duration, Timer};

use crate::dfu;
use crate::leds::{CHANGED, STATE};

/// How long USER must stay held to trigger a DFU reboot.
const DFU_HOLD: Duration = Duration::from_secs(3);

#[embassy_executor::task]
pub async fn button_task(mut button: ExtiInput<'static, Async>) {
    loop {
        // PC13 USER button: press -> HIGH — UM1974 §7.6, p.25.
        button.wait_for_rising_edge().await;
        Timer::after(Duration::from_millis(50)).await; // debounce the press
        if !button.is_high() {
            continue; // bounce / spurious edge, ignore
        }

        // Race "still held for DFU_HOLD" against "released". If the hold timer wins
        // while the line is still high, it's a long press → enter DFU.
        match select(Timer::after(DFU_HOLD), button.wait_for_falling_edge()).await {
            Either::First(()) => {
                // Held long enough. Confirm it's genuinely still down, then reboot
                // into the ROM DFU bootloader. reboot_to_dfu() does not return.
                if button.is_high() {
                    info!("USER held {}s -> reboot into USB DFU", DFU_HOLD.as_secs());
                    dfu::reboot_to_dfu();
                }
                // (Released right at the boundary — fall through to wait again.)
            }
            Either::Second(()) => {
                // Short press: released before the hold threshold → cycle LEDs.
                Timer::after(Duration::from_millis(50)).await; // debounce release
                let new_state = STATE.lock(|s| {
                    let next = s.get().next();
                    s.set(next);
                    next
                });
                info!("Button -> {}", new_state);
                CHANGED.signal(());
            }
        }
    }
}
