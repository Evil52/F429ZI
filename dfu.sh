#!/bin/bash
# Build → .bin → flash over USB DFU (no ST-Link, no probe-rs).
#
# Use this to flash a board "in the field" through the USB OTG FS port
# (CN13 on a Nucleo-144, or your own soldered USB-D+/D- on a custom board)
# using the STM32 *built-in ROM DFU bootloader*. Nothing extra is flashed for
# DFU itself — it already lives in the chip's system memory (ROM).
#
# ── How to put the board into DFU mode FIRST (hardware step, not scriptable) ──
#   1. Set BOOT0 = 1   (Nucleo-144: jumper/pin per UM1974; custom board: your
#      BOOT0 button/jumper pulled to 3V3). BOOT1/PB2 must stay 0.
#   2. Press RESET (or power-cycle) while BOOT0 is high.
#   3. The chip now enumerates as "STM32 BOOTLOADER" (USB VID:PID 0483:df11).
#      Check with:  dfu-util -l
#   4. Run this script. When done, set BOOT0 = 0 and reset to run the new app.
#
# Cargo runner note: `cargo run` passes the ELF path as $1. But in DFU mode you
# usually flash a known build, so this script also works standalone:
#     ./dfu.sh                      # builds release, flashes target/.../f429zi
#     ./dfu.sh path/to/some.elf     # convert+flash that ELF
#     ./dfu.sh some.bin             # flash a raw .bin as-is
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# Flash base = Bank 1 start. RM0090 Table 6, p.77. Same as SLOT_A in src/ota.rs.
FLASH_ORIGIN="0x08000000"
# DFU "Internal Flash" alternate setting on STM32F4 (dfu-util -a). Confirm names
# for your chip with `dfu-util -l`; @Internal Flash is alt 0 on F4.
DFU_ALT="0"
TARGET_BIN="f429zi"

die() { echo "error: $*" >&2; exit 1; }

command -v dfu-util >/dev/null        || die "dfu-util not found. Install: sudo apt install dfu-util"
command -v arm-none-eabi-objcopy >/dev/null \
    || die "arm-none-eabi-objcopy not found (arm-none-eabi-binutils)."

ELF="${1:-}"

if [ -z "$ELF" ]; then
    echo "=== Building release ==="
    cargo build --release
    ELF="target/thumbv7em-none-eabihf/release/${TARGET_BIN}"
fi

[ -f "$ELF" ] || die "input not found: $ELF"

# If we were handed a raw .bin, flash it directly; otherwise objcopy ELF→bin.
case "$ELF" in
    *.bin)
        BIN="$ELF"
        ;;
    *)
        BIN="${ELF}.bin"
        echo "=== objcopy ELF → bin ==="
        arm-none-eabi-objcopy -O binary "$ELF" "$BIN"
        ;;
esac

# ── Size report (same accounting as flash.sh) ────────────────────────────────
if [[ "$ELF" != *.bin ]]; then
    echo ""
    echo "=== Memory Usage ==="
    arm-none-eabi-size -A "$ELF" | awk '
    /^\.text/    { flash += $2 }
    /^\.rodata/  { flash += $2 }
    /^\.data/    { flash += $2; ram += $2 }
    /^\.bss/     { ram += $2 }
    /^\.uninit/  { ram += $2 }
    END {
        printf "Flash: %d bytes (%.1f KB) of 2097152 bytes (2048 KB)\n", flash, flash/1024
        printf "RAM:   %d bytes (%.1f KB) of 262144 bytes (256 KB)\n",   ram,   ram/1024
    }'
    echo "===================="
fi

BIN_SIZE=$(stat -c%s "$BIN")
echo ""
echo "Image: $BIN  (${BIN_SIZE} bytes)"

# ── Verify a DFU device is actually present before trying to flash ───────────
if ! dfu-util -l 2>/dev/null | grep -qi "Found DFU"; then
    echo ""
    echo "No DFU device found. Put the board into DFU mode:" >&2
    echo "  BOOT0 = 1, then press RESET, then re-run. Verify with: dfu-util -l" >&2
    die "no DFU device on USB"
fi

echo ""
echo "=== Flashing over USB DFU (alt ${DFU_ALT} @ ${FLASH_ORIGIN}) ==="
# -a: alternate setting (Internal Flash)
# -s ADDR:leave : download to ADDR, then leave DFU and start the app
# -R is implied by :leave on recent dfu-util; keep :leave explicit for clarity.
dfu-util -a "$DFU_ALT" -s "${FLASH_ORIGIN}:leave" -D "$BIN"

echo ""
echo "Done. If BOOT0 is still high, set BOOT0 = 0 and reset to keep running the app."
