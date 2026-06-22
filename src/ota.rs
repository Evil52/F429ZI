//! Web-OTA firmware update over Ethernet (A/B dual-bank).
//!
//! ─ Big picture ───────────────────────────────────────────────────────────────
//! The 2 MB Flash is split into two 1 MB banks (RM0090 Table 6, p.77):
//!   Slot A = Bank 1 @ 0x0800_0000   (running app today)
//!   Slot B = Bank 2 @ 0x0810_0000   (staging area for the new image)
//! The web handler POST /firmware streams a new `.bin` into the INACTIVE slot,
//! verifies it (length + CRC/marker), marks it "boot next", then reboots. On the
//! next boot the chosen slot runs. If the new image fails to confirm itself
//! within a timeout, the bootloader rolls back to the previous slot.
//!
//! ─ Two ways to do the actual swap (decide before writing the bodies) ─────────
//! 1. Tiny custom bootloader at 0x0800_0000: it reads a "which slot" flag, then
//!    relocates VTOR and jumps to the chosen slot. The APP is linked at a slot
//!    offset (NOT at 0x0800_0000). Most portable, most code.
//! 2. STM32 built-in BANK SWAP (FLASH_OPTCR BFB2 bit) on dual-bank F4: the
//!    bootloader-from-bank feature lets the chip boot Bank 2 instead of Bank 1 by
//!    flipping an option bit, no custom loader code. F429 supports dual-bank boot
//!    (RM0090 §3 / option bytes). Less code, but option-byte programming is
//!    fiddly and a bad write can brick boot. Read RM0090 §3 carefully first.
//!
//! Recommendation for "commercial but understandable": option 1 (custom loader)
//! — explicit, debuggable, and the rollback logic is yours. Build that as a
//! SEPARATE bootloader crate later; for now this module handles the *staging*
//! (receive + write + verify + set-boot-flag) which is identical for both.
//!
//! ─ Flash programming ─────────────────────────────────────────────────────────
//! Same as config.rs: unlock FLASH_KEYR (RM0090 §3.9.3, p.100), erase the target
//! bank's sectors (SER+SNB+STRT in FLASH_CR, §3.9.7), then program. Use the
//! embassy-stm32 `Flash` driver. NEVER erase the bank you're currently running
//! from.
//! ─────────────────────────────────────────────────────────────────────────────

/// Base address of each OTA slot. RM0090 Table 6, p.77.
pub const SLOT_A_ADDR: u32 = 0x0800_0000; // Bank 1
pub const SLOT_B_ADDR: u32 = 0x0810_0000; // Bank 2

/// A streaming writer that the web /firmware handler feeds chunks into. It erases
/// the inactive slot lazily (on first byte) and programs as data arrives, so we
/// never need to buffer a whole image in RAM.
pub struct OtaWriter {
    // target_addr: u32,  // inactive slot base
    // offset: u32,       // bytes written so far
    // flash: embassy_stm32::flash::Flash<'static, ...>,
}

impl OtaWriter {
    /// Open a writer for the INACTIVE slot (the one we're not running from).
    pub fn new_for_inactive_slot() -> Self {
        todo!("pick inactive slot (opposite of running bank), erase its sectors, hold Flash driver")
    }

    /// Write the next chunk of the incoming image at the current offset.
    pub async fn write_chunk(&mut self, data: &[u8]) -> Result<(), ()> {
        let _ = data;
        todo!("program `data` at target_addr+offset; advance offset; map errors")
    }

    /// Finish: verify the staged image (length + CRC), set the "boot this slot"
    /// flag, and return Ok so the caller can reboot via fault::safe_reboot().
    pub async fn finish(self, expected_len: u32, expected_crc: u32) -> Result<(), ()> {
        let _ = (expected_len, expected_crc);
        todo!("verify length+CRC of staged slot; write boot-select flag; Ok(())")
    }
}

/// Which slot the running firmware was started from (read VTOR or a const baked
/// in by the linker). The OTA target is the OTHER one.
pub fn running_slot_addr() -> u32 {
    todo!("read SCB VTOR base, round to bank → SLOT_A_ADDR or SLOT_B_ADDR")
}
