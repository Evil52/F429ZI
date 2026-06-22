//! Single source of truth for the Nucleo-F429ZI (MB1137-F429ZI-B01) pin map.
//!
//! Embassy hands out the `Peripherals` struct in `main()` and each pin is a
//! move-only singleton, so we can't store them in `const`s here. Instead this
//! module DOCUMENTS every pin with its datasheet citation, and `main.rs` reads
//! pins from `p.<PIN>` in the same order. Keep this table and `main.rs` in sync —
//! this is the one place to look up "which pin is what and why".
//!
//! Board: NUCLEO-F429ZI, order code NUF429ZI$AU1, MCU STM32F429ZIT6 rev "3"
//! (errata ES0206), board MB1137-F429ZI-B01 (only board revision —
//! UM1974 Rev 11, Table 24, p.80). 2 MB Flash, 192 KB SRAM + 64 KB CCM.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//!  USER LEDs                       UM1974 Rev 11, §7.5 "LEDs", p.25
//!    LD1 green = PB0   (SB120 ON)  "on when I/O is HIGH" → active-HIGH
//!    LD2 blue  = PB7   (SB139 ON)
//!    LD3 red   = PB14  (SB118 ON)
//!
//!  USER BUTTON                     UM1974 Rev 11, §7.6 "Push-buttons", p.25
//!    B1 USER   = PC13  (SB173 ON default). Idle LOW, pressed HIGH (board pulls
//!                       to GND; press connects to VDD) → wait_for_rising_edge,
//!                       Pull::None.
//!    B2 RESET  = NRST  (not a GPIO).
//!
//!  CLOCK                           UM1974 Rev 11, §7.8.1 "OSC clock supply", p.26
//!    HSE = MCO from ST-LINK, fixed 8 MHz, into PF0/OSC_IN.
//!    Default SBs: SB112 + SB149 ON (MCO connected), SB8/SB9 OFF (no X3 crystal).
//!    → see clock.rs: PLL M=8 N=336 P=2 → 168 MHz SYSCLK.
//!
//!  SPI1 → MAX31865 (PT100)         pins from RM0090 AF map / datasheet; the
//!                                  MAX31865 is an external module, not on the board.
//!    SCK  = PA5   (SPI1_SCK, AF5)  — also shared with LD1 alt routing, fine here
//!    MISO = PA6   (SPI1_MISO, AF5) = SDO of MAX31865
//!    MOSI = PA7   (SPI1_MOSI, AF5) = SDI of MAX31865
//!       ⚠ PA7 is ALSO RMII_CRS_DV via JP6. If you enable Ethernet you CANNOT use
//!         PA7 for SPI MOSI at the same time. See "CONFLICT" note below.
//!    CS   = PA4   (GPIO out, software CS)
//!    RDY  = (optional) DRDY interrupt — wire to a spare EXTI pin if used.
//!    SPI Mode 1 (CPOL=0, CPHA=1), <= 5 MHz (MAX31865 datasheet).
//!    DMA: SPI1_TX = DMA2 (TX), SPI1_RX = DMA2 (RX) — RM0090 §10 DMA request map.
//!
//!  ETHERNET RMII → LAN8742A PHY    UM1974 Rev 11, Table 11 "Ethernet pins", p.29
//!    "JP6 and JP7 must be ON when using Ethernet" (UM1974 §7.11, p.29)
//!    REF_CLK = PA1   (SB13 ON)     50 MHz from PHY
//!    MDIO    = PA2   (SB160 ON)
//!    MDC     = PC1   (SB164 ON)
//!    CRS_DV  = PA7   (JP6 ON)      ⚠ CONFLICT with SPI1_MOSI above
//!    RXD0    = PC4   (SB178 ON)
//!    RXD1    = PC5   (SB181 ON)
//!    TX_EN   = PG11  (SB183 ON)    ← differs from JZF407 (which used PB11)
//!    TXD0    = PG13  (SB182 ON)    ← differs from JZF407 (PB12)
//!    TXD1    = PB13  (JP7 ON)      ← differs from JZF407 (PB13 too, coincidence)
//!    PHY nRST= NRST via SB177 (Ethernet PHY reset) — UM1974 Table 12, p.32
//!    PHY address of LAN8742A on this board = 0 (default strap).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//!  ⚠ PA7 CONFLICT (read this before wiring the sensor):
//!  PA7 is used by BOTH SPI1_MOSI (MAX31865 SDI) AND RMII_CRS_DV (Ethernet).
//!  You cannot have both on PA7. Options:
//!    (a) Move the MAX31865 to a different SPI bus / different MOSI pin
//!        (e.g. SPI4 on PE6=MOSI, PE2=SCK, PE5=MISO — all free of RMII), OR
//!    (b) Run the sensor on SPI1 but on a non-RMII MOSI alternate, OR
//!    (c) Use the sensor only when Ethernet is down.
//!  Decide this when wiring sensor.rs + main.rs. The skeleton leaves SPI on
//!  SPI1/PA5/PA6/PA7 (as today) and Ethernet stubbed, so they don't clash yet.
//! ─────────────────────────────────────────────────────────────────────────────

// This module is documentation-only for now: no code, just the authoritative map
// above. If you later want typed pin aliases, add them here. Keeping it as a
// module means `mod board;` in main.rs surfaces this map in the crate.
