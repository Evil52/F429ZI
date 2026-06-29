#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)] // embassy task macro on nightly

// ── Module tree ──────────────────────────────────────────────────────────────
mod board; // pin map + datasheet citations (documentation module)
mod buttons; // B1 USER → LED state machine
mod clock; // 168 MHz RCC config + firmware-size helpers
mod config; // Flash-backed network config (load/save)
mod dfu; // jump into ROM USB-DFU bootloader (field re-flash, no ST-Link)
mod fault; // reset-reason + safe_reboot
mod leds; // 3-LED state machine task
mod net; // embassy-net runner (Ethernet)
mod ota; // Web-OTA A/B staging
mod sensor; // MAX31865 PT100 driver + task
mod sse; // Server-Sent Events live stream
mod watchdog; // IWDG liveness
mod web; // HTTP dashboard server

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    dma::InterruptHandler as DmaHandler,
    exti::InterruptHandler as ExtiHandler,
    gpio::{Level, Output, Pull, Speed},
    interrupt::typelevel::EXTI15_10,
    peripherals::{DMA2_CH2, DMA2_CH3},
    spi::{Config as SpiConfig, Mode, Phase, Polarity, Spi},
    time::Hertz,
};
use embassy_time::{Duration, Timer};
use panic_probe as _;

// ── Interrupt bindings ───────────────────────────────────────────────────────
// EXTI15_10 covers PC13 (the USER button) — RM0090 §12 EXTI line mux.
// SPI1 DMA: SPI1_TX = DMA2_CH3, SPI1_RX = DMA2_CH2 (RM0090 §10 DMA request map).
// The DMA InterruptHandler is parameterized on the CHANNEL peripheral, and bound
// to that channel's STREAM interrupt (DMA2_CH2→DMA2_STREAM2, CH3→DMA2_STREAM3).
// When Ethernet is enabled, add:  ETH => eth::InterruptHandler;
bind_interrupts!(struct Irqs {
    EXTI15_10    => ExtiHandler<EXTI15_10>;
    DMA2_STREAM2 => DmaHandler<DMA2_CH2>;
    DMA2_STREAM3 => DmaHandler<DMA2_CH3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // ── FIRST THING: ROM USB-DFU entry check ─────────────────────────────────
    // If the previous boot called dfu::reboot_to_dfu() (e.g. a long USER-button
    // hold), this jumps into the chip's ROM DFU loader NOW — before any clock or
    // peripheral init — so the board re-enumerates as "STM32 BOOTLOADER" and
    // ./dfu.sh can re-flash it over USB. Returns immediately on a normal boot.
    // MUST stay before embassy_stm32::init(). See src/dfu.rs.
    dfu::check_and_enter_dfu();

    // 168 MHz from 8 MHz ST-LINK MCO HSE — see clock.rs (UM1974 §7.8.1, p.26).
    let p = embassy_stm32::init(clock::make_config());

    // Small delay so RTT attaches before the first logs (probe-rs needs a moment).
    Timer::after(Duration::from_millis(10)).await;

    info!("--- Nucleo F429ZI Embassy (commercial build) ---");
    info!("Flash: {} bytes", clock::flash_size());
    info!("RAM:   {} bytes", clock::static_ram_size());

    // ── Bring-up fixes (needed with NO debugger attached) ────────────────────
    // These do nothing visible with probe-rs holding the debug domain alive, but
    // are REQUIRED once the board runs standalone. Enable them when you wire
    // Ethernet — they touch ETH/CoreSight. Kept here as TODO so you understand
    // where they go. (From JZF407 reference bring-up notes.)
    //
    // 1) DEMCR.TRCENA: ETH DMA needs CoreSight trace enabled. DEMCR @ 0xE000_EDFC
    //    bit24 (architectural Cortex-M reg; not in PM0214).
    //    unsafe { let r = 0xE000_EDFC as *mut u32; r.write_volatile(r.read_volatile() | (1<<24)); }
    //
    // 2) RCC_AHB1LPENR ETH LP bits keep ETH RX clock alive in WFI sleep —
    //    RM0090 §7.3.15, p.251.
    //    RCC.ahb1lpenr().modify(|w| { w.set_ethmaclpen(true); w.set_ethmacrxlpen(true); w.set_ethmactxlpen(true); });

    // ── Reset reason (RM0090 §7.3.21 RCC_CSR, p.261) ─────────────────────────
    // let reset_reason = fault::read_and_clear();
    // info!("Reset: {}", reset_reason.as_str());

    // ── LEDs (active-HIGH, UM1974 §7.5, p.25): LD1=PB0, LD2=PB7, LD3=PB14 ─────
    let led1 = Output::new(p.PB0, Level::Low, Speed::Low);
    let led2 = Output::new(p.PB7, Level::Low, Speed::Low);
    let led3 = Output::new(p.PB14, Level::Low, Speed::Low);

    // ── Button (PC13, UM1974 §7.6, p.25; press → HIGH so Pull::None) ──────────
    let button = embassy_stm32::exti::ExtiInput::new(p.PC13, p.EXTI13, Pull::None, Irqs);

    // ── SPI1 → MAX31865 (SPI Mode 1, <=5 MHz). SCK=PA5, MISO=PA6, MOSI=PA7 ────
    // ⚠ PA7 also = RMII_CRS_DV (UM1974 Table 11, p.29). When you turn Ethernet
    //   on, move the sensor to SPI4 (PE2/PE5/PE6). See board.rs "PA7 CONFLICT".
    let mut spi_config = SpiConfig::default();
    spi_config.mode = Mode {
        polarity: Polarity::IdleLow,            // CPOL=0  } MAX31865
        phase: Phase::CaptureOnSecondTransition, // CPHA=1  } SPI Mode 1
    };
    spi_config.frequency = Hertz(1_000_000); // 1 MHz — conservative start (max 5 MHz)
    let spi = Spi::new(
        p.SPI1, p.PA5,  // SCK
        p.PA7,  // MOSI (= MAX31865 SDI)
        p.PA6,  // MISO (= MAX31865 SDO)
        p.DMA2_CH3, // SPI1_TX DMA  (RM0090 §10 request map)
        p.DMA2_CH2, // SPI1_RX DMA
        Irqs, spi_config,
    );
    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh); // software CS, idle high

    // ── Ethernet + web + OTA (wire up when ready) ────────────────────────────
    // Requires: JP6 + JP7 ON (UM1974 §7.11, p.29), the bring-up fixes above, and
    // moving the sensor off PA7. Sketch (see net.rs / web.rs):
    //   let mac = ...uid-derived...;
    //   let eth = eth::Ethernet::new(queue, p.ETH, Irqs,
    //       p.PA1/*REF_CLK*/, p.PA7/*CRS_DV*/, p.PC4/*RXD0*/, p.PC5/*RXD1*/,
    //       p.PG13/*TXD0*/, p.PB13/*TXD1*/, p.PG11/*TX_EN*/,
    //       GenericPhy::new(0), mac);     // LAN8742A PHY addr = 0
    //   let (stack, runner) = embassy_net::new(eth, net_config, resources, seed);
    //   Timer::after(Duration::from_millis(3000)).await; // PHY autoneg
    //   spawner.spawn(net::net_task(runner).unwrap());
    //   let cfg = config::load();
    //   spawner.spawn(web::web_task(stack, cfg.clone(), reset_reason).unwrap());
    //   spawner.spawn(web::web_task_b(stack, cfg, reset_reason).unwrap());

    // ── Spawn the tasks that work today ──────────────────────────────────────
    // A #[embassy_executor::task] fn returns Result<SpawnToken, SpawnError>; the
    // .unwrap() is on that TOKEN. Spawner::spawn(token) then consumes it. (So the
    // unwrap goes INSIDE: spawn(task(args).unwrap()), not on spawn(...).)
    spawner.spawn(buttons::button_task(button).unwrap());
    spawner.spawn(leds::led_task(led1, led2, led3).unwrap());
    spawner.spawn(sensor::sensor_task(spi, cs).unwrap());
    // spawner.spawn(watchdog::watchdog_task(p.IWDG).unwrap());

    info!("Running.");
}
