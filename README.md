# F429ZI — Rust + Embassy commercial firmware

Industrial-style firmware for the **NUCLEO-F429ZI** (STM32F429ZIT6 + LAN8742A
Ethernet PHY), built on the [Embassy](https://embassy.dev) async runtime. PT100
temperature sensing (MAX31865 over SPI), a web dashboard with live updates over
Server-Sent Events, static-IP networking, and **Web-OTA** firmware updates.

Modeled on the proven [JZF407_RUST](https://github.com/Evil52/JZF407_RUST)
reference and structured per *Making Embedded Systems* (Elecia White): testable
logic is split out into a host-tested `logic/` crate; hardware I/O lives in `src/`.

> **Status: skeleton.** Folders, module boundaries, types, and datasheet-cited
> comments are in place; the bodies marked `todo!()` are being filled in by hand,
> module by module, to understand every line.

Every hardware-specific value in the code carries a citation to its source:
`UM1974 Rev 11, Table 11, p.29` (board) or `RM0090 §7.3.15, p.251` (MCU). All docs
are local in [`docs/`](docs/).

---

## Board identification (verified)

| | |
|---|---|
| Board | NUCLEO-F429ZI, **MB1137-F429ZI-B01** (only board revision — UM1974 Table 24, p.80) |
| Order code | NUF429ZI$AU1 |
| MCU | STM32F429ZIT6 **rev "3"**, errata ES0206 (UM1974 Table 23, p.76) |
| Memory | 2 MB Flash (dual bank), 192 KB SRAM + 64 KB CCM |

---

## Pin map

| Pin | Function | Source |
|-----|----------|--------|
| PB0  | LD1 green (active-HIGH) | UM1974 §7.5, p.25 (SB120) |
| PB7  | LD2 blue  (active-HIGH) | UM1974 §7.5, p.25 (SB139) |
| PB14 | LD3 red   (active-HIGH) | UM1974 §7.5, p.25 (SB118) |
| PC13 | B1 USER button (press→HIGH) | UM1974 §7.6, p.25 (SB173) |
| PF0  | HSE in = 8 MHz ST-LINK MCO | UM1974 §7.8.1, p.26 (SB112+SB149) |
| PA5/PA6/PA7/PA4 | SPI1 SCK/MISO/MOSI + CS → MAX31865 | RM0090 AF map |
| PA1  | RMII REF_CLK | UM1974 Table 11, p.29 (SB13) |
| PA2  | RMII MDIO | UM1974 Table 11, p.29 (SB160) |
| PC1  | RMII MDC | UM1974 Table 11, p.29 (SB164) |
| PA7  | RMII CRS_DV | UM1974 Table 11, p.29 (**JP6**) |
| PC4/PC5 | RMII RXD0/RXD1 | UM1974 Table 11, p.29 (SB178/SB181) |
| PG11 | RMII TX_EN | UM1974 Table 11, p.29 (SB183) |
| PG13 | RMII TXD0 | UM1974 Table 11, p.29 (SB182) |
| PB13 | RMII TXD1 | UM1974 Table 11, p.29 (**JP7**) |

> ⚠ **PA7 conflict:** SPI1_MOSI (MAX31865 SDI) and RMII_CRS_DV are the SAME pin.
> Can't use both. When enabling Ethernet, move the sensor to **SPI4** (PE2 SCK /
> PE5 MISO / PE6 MOSI), which is clear of RMII. See [`src/board.rs`](src/board.rs).

### Jumpers required for Ethernet (UM1974 §7.11, p.29)

**JP6 and JP7 must be ON.** Default solder bridges (SB13/SB160/SB164/SB178/
SB181/SB182/SB183/SB177) are factory-ON for the Ethernet variant.

---

## Flash map (OTA A/B) — RM0090 Table 6, p.77

```
Bank 1  0x0800_0000 .. 0x080F_FFFF  1 MB   Slot A (running app)
Bank 2  0x0810_0000 .. 0x081D_FFFF  ~896K  Slot B (OTA staging)
        0x081E_0000 .. 0x081F_FFFF  128 K  Config sector (sector 23)
CCM     0x1000_0000 .. 0x1000_FFFF  64 K   fault marker at top (reset reason)
```

---

## Task architecture

```
main() — 168 MHz clock, bring-up fixes, then spawns:
 ├── led_task       3-LED state machine (AllOff/SlowBlink/FastBlink/AllOn)
 ├── button_task    PC13 → cycle LED state
 ├── sensor_task    MAX31865 PT100 every 500 ms → publishes temperature
 ├── net_task       embassy-net runner (Ethernet MAC/DMA)        [when wired]
 ├── web_task       HTTP dashboard, socket A                      [when wired]
 ├── web_task_b     HTTP dashboard, socket B (two-socket pool)    [when wired]
 └── watchdog_task  IWDG ~20 s liveness                           [when wired]
```

---

## Project layout

```
F429ZI/
├── Cargo.toml          workspace; embedded deps gated on target_os="none"
├── memory.x            OTA A/B banks + config sector + CCM fault marker
├── build.rs            puts memory.x on the linker search path
├── .cargo/config.toml  target + probe-rs runner via flash.sh
├── flash.sh            CubeIDE-style Flash/RAM size report + flash
├── docs/               local datasheets (RM0090, PM0214, UM1974, the book)
├── src/                firmware (no_std)
│   ├── main.rs         init + bring-up fixes + task spawning
│   ├── board.rs        authoritative pin map with citations
│   ├── clock.rs        168 MHz RCC + firmware-size helpers
│   ├── leds.rs         LED state machine
│   ├── buttons.rs      USER button
│   ├── sensor.rs       MAX31865 PT100 driver + task
│   ├── net.rs          embassy-net runner task
│   ├── web.rs          HTTP server, two-socket pool, router
│   ├── sse.rs          live event stream
│   ├── config.rs       Flash-backed network config
│   ├── ota.rs          Web-OTA A/B staging
│   ├── watchdog.rs     IWDG
│   └── fault.rs        reset-reason + safe_reboot
└── logic/              pure logic, host-tested (cargo test on the Mac)
    ├── src/{config,auth,rtd,parse}.rs
    └── tests/
```

## Build / flash / test

```bash
cargo build --release
cargo run   --release        # flash + RTT logs (flash.sh prints size first)
# host unit tests (pure logic, no hardware):
cargo test -p f429zi-logic --target aarch64-apple-darwin
```
