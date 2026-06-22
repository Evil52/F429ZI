//! Clock tree + firmware-size helpers.
//!
//! HSE source on this board = 8 MHz MCO from the ST-LINK (UM1974 §7.8.1, p.26;
//! SB112+SB149 ON, SB8/SB9 OFF). We PLL it up to 168 MHz, the max SYSCLK for the
//! STM32F429 (RM0090 §6/§7 RCC; voltage-scaling/over-drive permitting).
//!
//! PLL math (RM0090 §7.3.2 RCC_PLLCFGR, p.227):
//!   VCO_in  = HSE / M           = 8 MHz / 8   = 1 MHz   (M=PllPreDiv::DIV8)
//!   VCO_out = VCO_in * N        = 1 MHz * 336 = 336 MHz (N=PllMul::MUL336)
//!   SYSCLK  = VCO_out / P       = 336 / 2     = 168 MHz (P=PllPDiv::DIV2)
//!   PLL48   = VCO_out / Q       = 336 / 7     = 48 MHz  (Q=PllQDiv::DIV7, USB/RNG/SDIO)
//!
//! Bus prescalers (RM0090 §7.3.3 RCC_CFGR, p.229; bus max from datasheet):
//!   AHB  = 168 MHz (DIV1)
//!   APB1 = 42 MHz  (DIV4) — APB1 max is 45 MHz; TIM2 kernel clock = 84 MHz
//!   APB2 = 84 MHz  (DIV2) — APB2 max is 90 MHz
//!
//! TIM2 = 84 MHz pairs with embassy-time `tick-hz-1_000_000` (PSC=83, fits u16).

use embassy_stm32::{
    Config,
    rcc::{
        APBPrescaler, Hse, HseMode, Pll, PllMul, PllPDiv, PllPreDiv, PllQDiv, PllSource, Sysclk,
    },
    time::Hertz,
};

// Linker symbols (defined by cortex-m-rt's link.x) for the runtime firmware-size
// report. We compute used Flash/RAM from these so we can log it at boot like the
// CubeIDE "Build Finished" summary. These are addresses, not RM facts.
unsafe extern "C" {
    static __etext: u8; // end of .text+.rodata in Flash
    static __sdata: u8; // start of .data (RAM)
    static __edata: u8; // end of .data   (RAM)
    static __sbss: u8; //  start of .bss  (RAM)
    static __ebss: u8; //  end of .bss    (RAM)
}

/// Bytes of Flash used = end-of-code minus Flash origin (0x0800_0000).
/// Flash origin per RM0090 Table 6, p.77 (Sector 0 base = 0x0800_0000).
pub fn flash_size() -> usize {
    unsafe { &__etext as *const u8 as usize - 0x0800_0000 }
}

/// Bytes of static RAM used = .data + .bss (stack/heap excluded).
pub fn static_ram_size() -> usize {
    unsafe {
        (&__edata as *const u8 as usize - &__sdata as *const u8 as usize)
            + (&__ebss as *const u8 as usize - &__sbss as *const u8 as usize)
    }
}

/// Build the 168 MHz RCC config. (This is your existing, working config —
/// citations added.)
pub fn make_config() -> Config {
    let mut config = Config::default();

    // HSE = 8 MHz from ST-LINK MCO — UM1974 §7.8.1, p.26.
    config.rcc.hse = Some(Hse {
        freq: Hertz(8_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll_src = PllSource::HSE;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV8,   // M=8  → 1 MHz VCO input   (RM0090 §7.3.2, p.227)
        mul: PllMul::MUL336,       // N=336 → 336 MHz VCO
        divp: Some(PllPDiv::DIV2), // P=2  → 168 MHz SYSCLK
        divq: Some(PllQDiv::DIV7), // Q=7  → 48 MHz (USB/RNG/SDIO)
        divr: None,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.apb1_pre = APBPrescaler::DIV4; // 42 MHz (max 45) — RM0090 §7.3.3, p.229
    config.rcc.apb2_pre = APBPrescaler::DIV2; // 84 MHz (max 90)

    config
}
