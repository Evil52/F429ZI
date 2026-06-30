//! PT100 RTD math — Callendar–Van Dusen equation (IEC 60751).
//!
//! The MAX31865 gives us a 15-bit ratio of (RTD resistance / Rref). We turn that
//! into resistance, then resistance into temperature. This is pure math, so it
//! lives here and is unit-tested on the host (known resistance → known °C).
//!
//! Hardware constants for THIS board (Adafruit-style MAX31865 breakout, PT100):
//!   - Rref = 430 Ω  (reference resistor fitted on the breakout for PT100)
//!   - R0   = 100 Ω  (PT100 nominal resistance at 0 °C, by definition)
//! These are board/sensor facts, not MCU register facts — no RM page applies.
//!
//! Callendar–Van Dusen (for t >= 0 °C):  R(t) = R0 * (1 + A*t + B*t^2)
//!   IEC 60751 coefficients:  A = 3.9083e-3,  B = -5.775e-7  (C = -4.183e-12, t<0)
//! Inverting the quadratic for t >= 0 °C gives the closed form used in `resistance_to_celsius`.

/// PT100 nominal resistance at 0 °C (IEC 60751 definition).
pub const R0: f32 = 100.0;
/// Reference resistor on the MAX31865 breakout (PT100 variant).
pub const RREF: f32 = 430.0;

/// IEC 60751 Callendar–Van Dusen coefficient A.
pub const CVD_A: f32 = 3.9083e-3;
/// IEC 60751 Callendar–Van Dusen coefficient B.
pub const CVD_B: f32 = -5.775e-7;

// sqrt source: core has no f32::sqrt on bare-metal, so on the MCU we use
// libm::sqrtf; on the host the std build has f32::sqrt. One `sqrt()` either way.
#[cfg(target_os = "none")]
#[inline]
fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

#[cfg(not(target_os = "none"))]
#[inline]
fn sqrt(x: f32) -> f32 {
    x.sqrt()
}

/// Convert the MAX31865 15-bit RTD register value into RTD resistance (ohms).
///
/// The MAX31865 RTD result register (MSB+LSB) holds the ratio in bits [15:1];
/// bit 0 is the fault flag and must be shifted out by the caller before passing
/// `raw15` here. resistance = raw15 / 32768 * Rref.
pub fn raw_to_resistance(raw15: u16) -> f32 {
    // raw15 is already the 15-bit ratio numerator (fault bit removed).
    (raw15 as f32) / 32768.0 * RREF
}

/// Convert RTD resistance (ohms) to temperature in °C using the inverted
/// Callendar–Van Dusen quadratic (valid for t >= 0 °C). For sub-zero work the
/// cubic C-term is needed; add it later if the product must read below 0 °C.
///
/// Closed form: t = (-A + sqrt(A^2 - 4B(1 - R/R0))) / (2B)
///
/// `sqrtf` comes from the `libm` crate in the firmware; the host test can use
/// `f32::sqrt`. Keep the sqrt call behind the caller so this crate stays
/// dependency-light, OR add libm here too — decide when you write it.
pub fn resistance_to_celsius(r_ohms: f32) -> f32 {
    let c = 1.0 - r_ohms / R0;
    let discriminant = CVD_A * CVD_A - 4.0 * CVD_B * c;
    (-CVD_A + sqrt(discriminant)) / (2.0 * CVD_B)
}

#[cfg(test)]
mod tests {
    use super::*;

    // PT100 reference points (IEC 60751): 100.00 Ω = 0 °C, 138.51 Ω = 100 °C.
    #[test]
    fn zero_celsius_is_100_ohms() {
        let t = resistance_to_celsius(100.0);
        assert!((t - 0.0).abs() < 0.1, "got {t}");
    }

    #[test]
    fn hundred_celsius_is_138_51_ohms() {
        let t = resistance_to_celsius(138.51);
        assert!((t - 100.0).abs() < 0.2, "got {t}");
    }
}
