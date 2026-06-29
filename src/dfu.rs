//! Jump into the STM32 built-in ROM DFU bootloader from running firmware.
//!
//! ─ Why ────────────────────────────────────────────────────────────────────────
//! For a fielded product with a soldered USB port but NO ST-Link and NO BOOT0
//! button, we still want to re-flash over USB. The STM32F4 has a USB DFU loader
//! burned into system memory (ROM) — the same one BOOT0=1 selects. Instead of a
//! jumper we make the *application* hand control back to that ROM loader on
//! command, so the board re-enumerates as "STM32 BOOTLOADER" (USB 0483:df11) and
//! `dfu-util` / ./dfu.sh can program it. No hardware access needed in the field.
//!
//! ─ How the jump works ─────────────────────────────────────────────────────────
//! The trick is the same as any "jump to another image":
//!   1. The ROM loader lives at SYSTEM_MEMORY (0x1FFF_0000 on STM32F42x/43x —
//!      RM0090 Rev 22 §3.4 "Boot configuration", Table 3; the large-Flash F42x/43x
//!      map the system bootloader at 0x1FFF_0000, NOT 0x1FFF_0000-vs-0x1FFFEC00
//!      differences of the smaller parts).
//!   2. Word[0] at that base = the initial Main Stack Pointer for the loader.
//!      Word[1] = the loader's reset/entry address (its vector table base + 4).
//!   3. We set MSP to word[0] and branch to word[1]. From then on we ARE the ROM
//!      loader.
//!
//! ─ The catch: you must jump EARLY and CLEAN ────────────────────────────────────
//! The ROM loader expects a near-reset state (default clocks, peripherals off,
//! interrupts masked). If we jump from deep inside a running Embassy app — PLL at
//! 168 MHz, DMA live, SysTick firing — the loader can misbehave (USB won't
//! enumerate, or it hangs). The robust, well-trodden approach is therefore a
//! TWO-STEP reboot, not an in-place jump:
//!
//!   reboot_to_dfu():  set a RAM magic word, then fault::safe_reboot().
//!   check_and_enter_dfu():  called as the FIRST thing in main(), BEFORE
//!       embassy_stm32::init(). If the magic is set, clear it and do the jump now
//!       — while the chip is still in its just-reset state.
//!
//! The magic word lives in CCM RAM (0x1000_0000): CCM is NOT initialised by
//! cortex-m-rt's .data/.bss setup the way we use it (NOLOAD, see memory.x) and is
//! untouched by a soft reset, so the value survives sys_reset() into the next
//! boot. Same survives-soft-reset property fault.rs relies on for its marker.
//! (Power-on clears CCM, which is fine: a cold boot should never enter DFU.)
//! ─────────────────────────────────────────────────────────────────────────────

use cortex_m::asm;

/// Base of the system-memory ROM bootloader on STM32F42x/43x.
/// RM0090 Rev 22 §3.4 "Boot configuration", Table 3 (system memory @ 0x1FFF_0000).
const SYSTEM_MEMORY_BASE: u32 = 0x1FFF_0000;

/// Magic value meaning "the previous boot asked us to enter DFU". Arbitrary but
/// unlikely to appear by chance in uninitialised RAM.
const DFU_MAGIC: u32 = 0xB00D_DF11; // "boot dfu", 0xdf11 = DFU USB PID

/// One word in CCM RAM that survives a soft reset (CCM is NOLOAD — memory.x).
/// Placing it in `.ccmram` + `#[used]` keeps it at a fixed, reset-stable address
/// that startup code never zeroes. We only ever touch it through volatile ops.
#[unsafe(link_section = ".ccmram")]
#[used]
static mut DFU_REQUEST: u32 = 0;

/// Request a switch into the ROM DFU bootloader on the next boot, then reboot.
///
/// Call this from anywhere in the app (button hold, a web/USB command, etc.).
/// It sets the RAM magic and performs a *safe* reboot; the real jump happens at
/// the very start of the next boot in [`check_and_enter_dfu`].
pub fn reboot_to_dfu() -> ! {
    // SAFETY: single-core, this is the only writer; a torn read can't happen on a
    // 32-bit aligned word. Volatile so the compiler can't drop it before reboot.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DFU_REQUEST), DFU_MAGIC);
    }
    asm::dsb();
    // Plain sys_reset() (SYSRESETREQ — PM0214 §4.4.5) is enough here: we want to
    // land in check_and_enter_dfu() at the next boot, and that does the real jump.
    //
    // NOTE: once Ethernet is wired up, prefer crate::fault::safe_reboot() instead
    // (it pulses ETH reset first so a live ETH DMA can't fire an IRQ mid-reboot —
    // see fault.rs). Today ETH is stubbed, so a bare reset is correct and avoids
    // depending on safe_reboot()'s (still TODO) body.
    cortex_m::peripheral::SCB::sys_reset();
}

/// MUST be the first call in `main()`, BEFORE `embassy_stm32::init()`.
///
/// If the previous boot called [`reboot_to_dfu`], this clears the flag and jumps
/// into the ROM DFU loader while the chip is still near its reset state. Otherwise
/// it returns immediately and normal boot continues.
///
/// # Safety / ordering
/// This reads the magic, then (on a hit) reprograms MSP and branches into ROM. It
/// never returns on a hit. Do not enable interrupts or init peripherals before
/// calling it.
pub fn check_and_enter_dfu() {
    // SAFETY: see DFU_REQUEST; volatile single-word access.
    let requested = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(DFU_REQUEST)) };
    if requested != DFU_MAGIC {
        return;
    }

    // Clear the flag FIRST so a failed/aborted DFU attempt can't trap us in a
    // reboot→DFU loop: next reset boots the app normally.
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(DFU_REQUEST), 0);
    }
    asm::dsb();

    enter_system_bootloader();
}

/// Low-level jump to the ROM bootloader at `SYSTEM_MEMORY_BASE`. Never returns.
///
/// `cortex_m::asm::bootload(base)` does the whole handover for us: it reads the
/// initial MSP from `base[0]` and the reset vector from `base[1]`, switches to the
/// main stack, loads MSP, and branches to the reset vector. So we pass it the
/// system-memory BASE, not a stack-pointer value.
fn enter_system_bootloader() -> ! {
    // Mask all interrupts during the handover; the ROM loader sets up its own.
    cortex_m::interrupt::disable();

    unsafe {
        // Point the vector table at system memory before the jump. bootload()
        // itself does not touch VTOR; the ROM loader normally reprograms it, but
        // setting it here keeps the window between branch and loader-init sane.
        // VTOR = SCB+0x08 (PM0214 §4.4.4). Architectural on Cortex-M4.
        let vtor = 0xE000_ED08 as *mut u32;
        core::ptr::write_volatile(vtor, SYSTEM_MEMORY_BASE);
        asm::dsb();
        asm::isb();

        // Read MSP[0]/reset[1] from system memory, set MSP, branch. Never returns.
        asm::bootload(SYSTEM_MEMORY_BASE as *const u32);
    }
}
