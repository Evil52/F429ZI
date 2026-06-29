#!/bin/bash
set -e

ELF="$1"

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
echo ""

# Flash and RELEASE the board so it runs standalone — no USB re-plug needed.
#
# `probe-rs run` flashes but keeps the core under the debugger (halted in the
# debug domain, RTT held open). On exit the core is left suspended, so the app
# only really starts after a power-on reset (unplug/replug USB). For a "flash and
# go" workflow we instead download, then issue a reset that lets the core run on
# its own and detach.
#
# For live defmt logs during development use `probe-rs run --chip STM32F429ZITx
# "$ELF"` directly, or `cargo embed`.
CHIP="STM32F429ZITx"

echo "=== Flashing ==="
probe-rs download --chip "$CHIP" "$ELF"

echo "=== Reset (run standalone) ==="
probe-rs reset --chip "$CHIP"

echo "Done — board is running. No USB re-plug needed."
