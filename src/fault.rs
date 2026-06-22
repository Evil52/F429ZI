//! Reset-reason reporting + safe reboot.
//!
//! After every boot we want to know WHY the MCU reset (power-on, watchdog,
//! software, pin). Two sources, combined:
//!   1. RCC_CSR hardware reset flags — RM0090 §7.3.21 RCC_CSR, p.261.
//!   2. A software "marker" word kept in CCM RAM (survives a soft reset, cleared
//!      on power-on) for reasons the hardware can't tell us apart (e.g. a remote
//!      reboot vs a plain sys_reset). Marker address = `_fault_marker` (memory.x,
//!      top of CCM at 0x1000_0000+64K-16).
//!
//! safe_reboot(): SCB::sys_reset() (AIRCR.SYSRESETREQ — PM0214 §4.4.5, p.228)
//! does NOT reset peripherals. The ETH DMA keeps running across the reset and can
//! fire an interrupt before cortex-m-rt installs handlers → fault. So before
//! resetting we disable interrupts and pulse the ETH reset in RCC_AHB1RSTR, dsb,
//! then sys_reset. ALWAYS reboot via this, never bare sys_reset().

use defmt::Format;

/// Why the last reset happened.
#[derive(Clone, Copy, PartialEq, Eq, Format)]
pub enum ResetReason {
    PowerOn,
    NrstPin,
    Software,
    IwdgTimeout,
    BrownOut,
    RemoteReboot,
    Unknown,
}

impl ResetReason {
    pub fn as_str(self) -> &'static str {
        match self {
            ResetReason::PowerOn => "power_on",
            ResetReason::NrstPin => "nrst_pin",
            ResetReason::Software => "software",
            ResetReason::IwdgTimeout => "iwdg_timeout",
            ResetReason::BrownOut => "brown_out",
            ResetReason::RemoteReboot => "remote_reboot",
            ResetReason::Unknown => "unknown",
        }
    }
}

/// Read the RCC_CSR reset flags + CCM marker, decide the reason, then CLEAR both
/// (write RCC_CSR.RMVF to clear flags — RM0090 §7.3.21, p.261). Call once at boot.
pub fn read_and_clear() -> ResetReason {
    todo!(
        "read RCC.csr() flags (PORRSTF/PINRSTF/SFTRSTF/IWDGRSTF/BORRSTF), \
         read the CCM marker word, map to ResetReason, set RCC.csr().rmvf, clear marker"
    )
}

/// Set the marker word so the NEXT boot reports `reason` (used before a remote reboot).
pub fn set_marker(reason: ResetReason) {
    todo!("write a tag for `reason` to the *_fault_marker word in CCM (volatile)")
}

/// Reset the MCU safely (reset ETH first so its DMA can't fire mid-reboot).
pub fn safe_reboot() -> ! {
    todo!(
        "cortex_m::interrupt::disable(); pulse RCC.ahb1rstr().ethmacrst; dsb(); \
         cortex_m::peripheral::SCB::sys_reset()"
    )
}
