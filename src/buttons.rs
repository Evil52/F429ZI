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

        // Race: DFU_HOLD timer vs button release.
        // We use with_timeout so we don't miss a falling edge that happened during
        // the press-debounce window — if the button is already low when
        // wait_for_falling_edge() is called it would block forever.
        use embassy_time::with_timeout;
        if button.is_low() {
            // Already released during debounce — treat as short press.
        } else if with_timeout(DFU_HOLD, button.wait_for_falling_edge())
            .await
            .is_err()
        {
            // Timeout fired while still held → long press → DFU.
            if button.is_high() {
                info!("USER held {}s -> reboot into USB DFU", DFU_HOLD.as_secs());
                dfu::reboot_to_dfu();
            }
            // Released right at boundary — fall through to short-press handling.
            button.wait_for_falling_edge().await;
        }

        // Short press (or boundary release): cycle LEDs.
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
