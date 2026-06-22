//! Flash-backed network config (firmware side).
//!
//! The Nucleo-F429ZI has no EEPROM, so we persist `f429zi_logic::config::Config`
//! in the LAST Flash sector. On the 2 MB part that is sector 23 = 0x081E_0000 ..
//! 0x081F_FFFF, 128 KB (RM0090 Table 6, p.77). We only use the first ~80 bytes of
//! it; erasing a whole 128 KB sector to save a tiny record is wasteful but simple
//! and rare (only on /save).
//!
//! Flash programming sequence (RM0090 §3.9): unlock with FLASH_KEYR (KEY1=
//! 0x45670123, KEY2=0xCDEF89AB — §3.9.3, p.100); to change bytes you must ERASE
//! the sector first (NOR flash only clears 1→0 per program, so a rewrite needs an
//! erase). Set SER + SNB=23 + STRT in FLASH_CR (§3.9.7) to erase, poll BSY, then
//! program with PG. embassy-stm32 wraps this in a `Flash` driver — prefer that
//! over raw PAC unless you want to learn the registers directly.
//!
//! Validation is delegated to the logic crate (magic check → defaults on garbage),
//! so a blank/corrupt sector can never lock you out.

use f429zi_logic::config::Config;

/// Address of the config sector. RM0090 Table 6, p.77 (sector 23 base).
const CONFIG_ADDR: u32 = 0x081E_0000;
/// Sector 23 length (RM0090 Table 6, p.77).
const CONFIG_SECTOR_LEN: u32 = 128 * 1024;

/// Load config from Flash at boot. Returns defaults on blank/corrupt sector.
pub fn load() -> Config {
    let _ = (CONFIG_ADDR, CONFIG_SECTOR_LEN);
    todo!(
        "read Config::LEN bytes from CONFIG_ADDR (it's memory-mapped, just a slice \
         at 0x081E_0000), Config::from_bytes(...).unwrap_or_default()"
    )
}

/// Erase the config sector and write `cfg`. Async because the Flash driver is.
/// Call this from the web /save handler, then reboot via fault::safe_reboot().
pub async fn save(cfg: &Config) -> Result<(), ()> {
    let _ = cfg;
    todo!(
        "use embassy_stm32 Flash driver: erase sector @ CONFIG_ADDR, \
         write cfg.to_bytes(); map errors to Err(())"
    )
}
