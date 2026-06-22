//! B1 USER button → cycle the LED state machine.
//! Pin = PC13 (UM1974 §7.6, p.25, SB173 ON default). On Nucleo-144 the USER
//! button press drives the line HIGH (board pulls to GND otherwise), so we use
//! Pull::None + rising edge. 50 ms debounce on each edge.

use defmt::info;
use embassy_stm32::{exti::ExtiInput, mode::Async};
use embassy_time::{Duration, Timer};

use crate::leds::{CHANGED, STATE};

#[embassy_executor::task]
pub async fn button_task(mut button: ExtiInput<'static, Async>) {
    loop {
        // PC13 USER button: press -> HIGH — UM1974 §7.6, p.25.
        button.wait_for_rising_edge().await;
        Timer::after(Duration::from_millis(50)).await;
        if button.is_high() {
            let new_state = STATE.lock(|s| {
                let next = s.get().next();
                s.set(next);
                next
            });
            info!("Button -> {}", new_state);
            CHANGED.signal(());
            button.wait_for_falling_edge().await;
            Timer::after(Duration::from_millis(50)).await;
        }
    }
}
