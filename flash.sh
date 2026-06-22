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

probe-rs run --chip STM32F429ZITx "$ELF"
