//! MAX31865 RTD-to-Digital converter driver (PT100, 2-wire).
//!
//! Bus: SPI Mode 1 (CPOL=0, CPHA=1), <= 5 MHz (MAX31865 datasheet). The chip's
//! registers are read MSB-first; a write sets the MSB of the address (0x80 | reg).
//!
//! The temperature MATH lives in the host-tested `f429zi_logic::rtd` module; this
//! file only does the SPI I/O and hands raw values to that math. That's the
//! "Making Embedded Systems" split: untestable I/O here, testable math there.
//!
//! ⚠ Pin conflict: today this uses SPI1 with MOSI=PA7, which is ALSO RMII_CRS_DV
//! (UM1974 Table 11, p.29). If Ethernet is enabled, move the sensor to a non-RMII
//! SPI (e.g. SPI4 PE2/PE5/PE6). See src/board.rs "PA7 CONFLICT".
//!
//! The shared latest reading is published for the web/SSE layer to display.

use core::sync::atomic::{AtomicI32, Ordering};

use defmt::{info, warn};
use embassy_stm32::{
    gpio::Output,
    mode::Async,
    spi::{Spi, mode::Master},
};
use embassy_time::{Duration, Timer};
use f429zi_logic::rtd;

// ── MAX31865 register addresses (datasheet, Table "Register Memory Map") ──────
const REG_CONFIG: u8 = 0x00;
const REG_RTD_MSB: u8 = 0x01; // RTD MSB; RTD LSB is 0x02, bit0 = fault flag
const REG_FAULT: u8 = 0x07;

// ── Config register bits (MAX31865 datasheet) ────────────────────────────────
const CONFIG_VBIAS: u8 = 0x80; // bit7: VBIAS on
const CONFIG_AUTO: u8 = 0x40; // bit6: conversion mode = auto
const CONFIG_1SHOT: u8 = 0x20; // bit5: 1-shot
const CONFIG_3WIRE: u8 = 0x10; // bit4: 1=3-wire, 0=2/4-wire  (our PT100 is 2-wire → 0)
const CONFIG_FAULT_CLR: u8 = 0x02; // bit1: fault status clear

/// Latest temperature in milli-°C (so we can store an f32 reading in an atomic
/// without locks). Web/SSE read this. i32::MIN means "no valid reading yet".
pub static TEMP_MILLI_C: AtomicI32 = AtomicI32::new(i32::MIN);

/// Store a reading (°C) for the web layer.
pub fn publish_temp(celsius: f32) {
    TEMP_MILLI_C.store((celsius * 1000.0) as i32, Ordering::Relaxed);
}

/// Read the latest temperature (°C), or None if none yet.
pub fn latest_temp() -> Option<f32> {
    match TEMP_MILLI_C.load(Ordering::Relaxed) {
        i32::MIN => None,
        m => Some(m as f32 / 1000.0),
    }
}

/// MAX31865 driver over an Embassy async SPI master + a software CS pin.
///
/// `Spi<'d, Async, Master>` — three generics: lifetime, peri-mode (DMA = Async),
/// communication-mode (Master). (This was the bug in the old max31865.rs: it only
/// passed two generics.)
pub struct Max31865<'d> {
    spi: Spi<'d, Async, Master>,
    cs: Output<'d>,
}

impl<'d> Max31865<'d> {
    pub fn new(spi: Spi<'d, Async, Master>, cs: Output<'d>) -> Self {
        Self { spi, cs }
    }

    /// Configure for 2-wire PT100, VBIAS on, auto-conversion, faults cleared.
    /// (CONFIG_1SHOT / CONFIG_3WIRE are unused: we run continuous-auto, 2-wire.)
    pub async fn init(&mut self) {
        let _reserved = (CONFIG_1SHOT, CONFIG_3WIRE);
        let cfg = CONFIG_VBIAS | CONFIG_AUTO | CONFIG_FAULT_CLR; // 2-wire: CONFIG_3WIRE=0
        self.write_reg(REG_CONFIG, cfg).await;
        info!("MAX31865 init: config=0x{:02x}", cfg);
    }

    /// One temperature reading. Reads the 16-bit RTD register, strips the fault
    /// bit, converts via the host-tested rtd math, and publishes the result.
    pub async fn read_temperature(&mut self) {
        // RTD result is MSB then LSB; LSB bit0 is the fault flag (datasheet).
        let mut rtd = [0u8; 2];
        self.read_regs(REG_RTD_MSB, &mut rtd).await;
        let combined = ((rtd[0] as u16) << 8) | rtd[1] as u16;

        if combined & 0x0001 != 0 {
            // Fault bit set: read the fault register, log it, then clear faults.
            let mut fault = [0u8; 1];
            self.read_regs(REG_FAULT, &mut fault).await;
            warn!("MAX31865 fault: 0x{:02x}", fault[0]);
            // Re-write config with the fault-clear bit to reset the fault status.
            let cfg = CONFIG_VBIAS | CONFIG_AUTO | CONFIG_FAULT_CLR;
            self.write_reg(REG_CONFIG, cfg).await;
            return;
        }

        let raw15 = combined >> 1; // drop the fault bit → 15-bit ratio
        let r = rtd::raw_to_resistance(raw15);
        let t = rtd::resistance_to_celsius(r);
        publish_temp(t);
        info!("RTD raw={} R={=f32} t={=f32}°C", raw15, r, t);
    }

    /// Write one register: CS low, send (0x80 | reg) then value, CS high.
    async fn write_reg(&mut self, reg: u8, val: u8) {
        self.cs.set_low();
        let _ = self.spi.write(&[0x80 | reg, val]).await;
        self.cs.set_high();
    }

    /// Read `buf.len()` bytes starting at `reg`: CS low, send the (read) address,
    /// clock the data out into `buf`, CS high. For a read the address MSB is 0, so
    /// `reg` is sent as-is. MOSI state during the data phase is ignored by the chip.
    async fn read_regs(&mut self, reg: u8, buf: &mut [u8]) {
        self.cs.set_low();
        let _ = self.spi.write(&[reg]).await; // address phase (read: MSB=0)
        let _ = self.spi.read(buf).await; // data phase
        self.cs.set_high();
    }
}

/// Sensor task: init once, then read every 500 ms and publish for the web layer.
#[embassy_executor::task]
pub async fn sensor_task(spi: Spi<'static, Async, Master>, cs: Output<'static>) {
    let mut sensor = Max31865::new(spi, cs);
    sensor.init().await;
    loop {
        sensor.read_temperature().await;
        Timer::after(Duration::from_millis(500)).await;
    }
}
