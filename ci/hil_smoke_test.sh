#!/usr/bin/env bash
# ci/hil_smoke_test.sh — Hardware-in-the-loop smoke test (HIL stage).
#
# Runs on the self-hosted macOS runner with a NUCLEO-F429ZI attached over USB
# (ST-LINK). Equivalent to the C pipeline's openocd-program + SWD-readback step,
# but for Rust/Embassy we use probe-rs and verify the boot log over RTT/defmt.
#
# Pass criterion: after flashing, the firmware prints its boot banner over RTT
# within a timeout. Seeing "Running." means: clock locked (168 MHz PLL came up,
# else init() would have panicked before RTT), the executor started, and all
# spawned tasks are alive. That's the liveness proof a host unit test cannot give.
#
# Usage:  ci/hil_smoke_test.sh <path-to-elf>
# Env:    CHIP (default STM32F429ZITx), BOOT_MARKER (default "Running."),
#         HIL_TIMEOUT_SECS (default 25)
set -euo pipefail

ELF="${1:?usage: hil_smoke_test.sh <elf>}"
CHIP="${CHIP:-STM32F429ZITx}"
BOOT_MARKER="${BOOT_MARKER:-Running.}"
HIL_TIMEOUT_SECS="${HIL_TIMEOUT_SECS:-25}"
LOG="$(mktemp -t hil_rtt.XXXXXX)"

echo "=== HIL smoke test ==="
echo "ELF        : ${ELF}"
echo "CHIP       : ${CHIP}"
echo "boot marker: '${BOOT_MARKER}'"
echo "timeout    : ${HIL_TIMEOUT_SECS}s"
echo "probe-rs   : $(probe-rs --version 2>/dev/null || echo MISSING)"
echo "probes attached:"
probe-rs list || true
echo "----------------------------------------"

# Flash + attach RTT. `probe-rs run` blocks streaming defmt, so run it in the
# background, capture its output, then look for the boot marker and kill it.
# --catch-hardfault makes a crash visible instead of silently hanging.
probe-rs run --chip "${CHIP}" --catch-hardfault "${ELF}" >"${LOG}" 2>&1 &
RUN_PID=$!

# Make sure we always clean up the probe-rs process.
cleanup() { kill "${RUN_PID}" 2>/dev/null || true; wait "${RUN_PID}" 2>/dev/null || true; }
trap cleanup EXIT

# Poll the log for the boot marker until timeout.
deadline=$(( $(date +%s) + HIL_TIMEOUT_SECS ))
found=0
while [ "$(date +%s)" -lt "${deadline}" ]; do
    if grep -qF "${BOOT_MARKER}" "${LOG}" 2>/dev/null; then
        found=1
        break
    fi
    # Bail early if the firmware panicked / hardfaulted.
    if grep -qiE "panicked|HardFault|stack overflow" "${LOG}" 2>/dev/null; then
        echo "!!! firmware fault detected in RTT log:"
        cat "${LOG}"
        exit 1
    fi
    sleep 1
done

echo "----- captured RTT log -----"
cat "${LOG}"
echo "----------------------------"

if [ "${found}" -eq 1 ]; then
    echo "HIL PASS: boot marker '${BOOT_MARKER}' seen on RTT."
    exit 0
else
    echo "HIL FAIL: boot marker '${BOOT_MARKER}' NOT seen within ${HIL_TIMEOUT_SECS}s."
    exit 1
fi
