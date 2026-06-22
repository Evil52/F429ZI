/* ───────────────────────────────────────────────────────────────────────────
   STM32F429ZIT6 memory map for the OTA-capable firmware.

   Flash sector map: RM0090 Rev 22, Table 6 "Flash module - 2 Mbyte dual bank
   organization (STM32F42xxx and STM32F43xxx)", p.77.
     Bank 1 = 0x0800_0000 .. 0x080F_FFFF  (1 MB, sectors 0..11)
     Bank 2 = 0x0810_0000 .. 0x081F_FFFF  (1 MB, sectors 12..23)
     Last sector 23 = 0x081E_0000 .. 0x081F_FFFF (128 KB) → we use it for config.

   RAM (RM0090 §2.3 memory map): SRAM 0x2000_0000 length 192 KB (112K SRAM1 +
   16K SRAM2 + 64K... no — on F429 the 192K is SRAM1 112K + SRAM2 16K + SRAM3 64K,
   all contiguous from 0x2000_0000). CCM is separate at 0x1000_0000, 64 KB,
   CPU-only (no DMA) — RM0090 §2.3.1.

   OTA A/B design (Web-OTA over Ethernet):
     - SLOT A  = Bank 1  → the running firmware lives here after a normal flash.
     - SLOT B  = Bank 2  → web /firmware writes the new image here, then a tiny
                           bootloader (or the dual-bank boot swap) runs it.
     - CONFIG  = sector 23 (top 128 KB of Bank 2) → static IP etc.

   NOTE: This linker file links the APPLICATION at the start of Bank 1. The A/B
   swap + bootloader is a later step (src/ota.rs). For now FLASH = Slot A so the
   skeleton builds and runs exactly like today; the B/CONFIG regions are reserved
   here so addresses never collide once OTA is wired up.
   ─────────────────────────────────────────────────────────────────────────── */

MEMORY
{
  /* Application = Slot A = Bank 1 (1 MB). RM0090 Table 6, p.77. */
  FLASH   : ORIGIN = 0x08000000, LENGTH = 1024K

  /* Reserved for OTA — do NOT place the app here. Bank 2 minus the config sector.
     Sectors 12..22 = 0x0810_0000 .. 0x081D_FFFF = 896 KB. (Documented, unused by
     the linker; the OTA code addresses it directly.) */
  /* OTA_B : ORIGIN = 0x08100000, LENGTH = 896K */
  /* CONFIG: ORIGIN = 0x081E0000, LENGTH = 128K  (sector 23) */

  /* 192 KB contiguous SRAM. RM0090 §2.3 memory map. DMA-capable (used for the
     ETH descriptors / smoltcp buffers). */
  RAM     : ORIGIN = 0x20000000, LENGTH = 192K

  /* 64 KB Core-Coupled Memory: CPU-only, NOT reachable by DMA. RM0090 §2.3.1.
     We keep its top word as a reset-reason "fault marker" that survives a soft
     reset (cleared on power-on). */
  CCMRAM  : ORIGIN = 0x10000000, LENGTH = 64K
}

/* Reset-reason marker word at the very top of CCM (survives sys_reset). */
_fault_marker = ORIGIN(CCMRAM) + LENGTH(CCMRAM) - 16;

SECTIONS
{
  /* Keep CCM uninitialised (NOLOAD) so the fault marker survives a soft reset. */
  .ccmram (NOLOAD) :
  {
    . = ALIGN(4);
    *(.ccmram .ccmram.*);
    . = ALIGN(4);
  } > CCMRAM
} INSERT BEFORE .bss;
