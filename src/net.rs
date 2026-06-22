//! embassy-net runner task for the on-board Ethernet (LAN8742A RMII PHY).
//!
//! `main()` builds the `Ethernet` device + `embassy_net::Stack` and hands the
//! `Runner` here. This task does nothing but drive the MAC/DMA forever — it must
//! be a concrete (non-generic) type so the embassy task macro accepts it.
//!
//! Ethernet pin wiring (UM1974 Table 11, p.29) and the LAN8742A PHY are set up in
//! main.rs; the PHY auto-negotiation handling is inside embassy's `GenericPhy`.
//!
//! Hardware bring-up reminders (set in main.rs, needed with NO debugger):
//!   - DEMCR.TRCENA (ETH DMA needs CoreSight trace enabled).
//!   - RCC_AHB1LPENR ETH LP bits (ETH RX clock must stay alive in WFI sleep) —
//!     RM0090 §7.3.15, p.251.

use embassy_net::Runner;
use embassy_stm32::{
    eth::{Ethernet, GenericPhy, Sma},
    peripherals::{ETH, ETH_SMA},
};

/// Network runner — concrete type so the embassy task macro is happy.
/// `GenericPhy<Sma<'static, ETH_SMA>>`: the generic PHY driver talking to the
/// LAN8742A over the Station Management (MDIO/MDC) interface. Same shape as the
/// JZF407 reference, adapted to F429's ETH/ETH_SMA peripherals.
#[embassy_executor::task]
pub async fn net_task(
    mut runner: Runner<'static, Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>>,
) -> ! {
    runner.run().await
}
