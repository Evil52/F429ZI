//! Host integration tests for the PT100 RTD math.
//! Run with: cargo test -p f429zi-logic --target aarch64-apple-darwin
//! (the firmware's .cargo/config forces the embedded triple; override it here).

use f429zi_logic::rtd;

#[test]
fn pt100_reference_points() {
    // IEC 60751: 100.00 Ω = 0 °C, 138.51 Ω = 100 °C.
    assert!((rtd::resistance_to_celsius(100.0) - 0.0).abs() < 0.1);
    assert!((rtd::resistance_to_celsius(138.51) - 100.0).abs() < 0.2);
}

#[test]
fn raw_ratio_maps_to_resistance() {
    // Full-scale ratio (raw15 = 0x7FFF ≈ 32767) → ~Rref.
    let r = rtd::raw_to_resistance(0x7FFF);
    assert!((r - rtd::RREF).abs() < 1.0, "got {r}");
}
