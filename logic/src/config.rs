//! Network configuration: the (de)serializable struct + its byte layout.
//!
//! On the Nucleo-F429ZI there is NO EEPROM (the JZF407 board had an AT24C02).
//! Instead the firmware stores this record in a dedicated Flash sector (see
//! src/config.rs in the binary crate for the Flash read/write, and memory.x for
//! the sector address). This crate only owns the *format* + parsing, so the
//! layout is unit-tested on the host (round-trip serialize → deserialize).
//!
//! Validation is magic-only (no CRC): a wrong magic → defaults. A torn write
//! could pass the magic, so anything that fails to parse reverts to defaults
//! rather than being trusted — you can never brick yourself via bad config.

use heapless::String as HString;

/// Magic guarding the config record. Wrong magic ⇒ use `Config::default()`.
pub const CONFIG_MAGIC: [u8; 4] = [0xC0, 0x4F, 0x19, 0x1E];

/// Network + web-login configuration persisted to Flash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub ip: [u8; 4],
    pub prefix_len: u8,
    pub gateway: [u8; 4],
    /// Web dashboard login. Empty user AND pass ⇒ auth disabled (open LAN device).
    pub web_user: HString<32>,
    pub web_pass: HString<32>,
}

impl Default for Config {
    fn default() -> Self {
        // Defaults used on a blank/corrupt Flash sector. Pick a LAN address that
        // matches your router; 192.168.1.x is a common default.
        Config {
            ip: [192, 168, 1, 20],
            prefix_len: 24,
            gateway: [192, 168, 1, 1],
            web_user: HString::new(),
            web_pass: HString::new(),
        }
    }
}

impl Config {
    /// Serialize into a fixed-size record (magic-prefixed). The binary crate
    /// writes these bytes to Flash. Keep the byte offsets documented here so the
    /// Flash layout never drifts from the parser.
    ///
    /// Layout: [0..4) magic, [4..8) ip, [8] prefix_len, [9..13) gateway,
    ///         [13..45) web_user (NUL-padded), [45..77) web_pass (NUL-padded).
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        let mut buf = [0u8; Self::LEN]; // zero-filled → string fields are NUL-padded
        buf[0..4].copy_from_slice(&CONFIG_MAGIC);
        buf[4..8].copy_from_slice(&self.ip);
        buf[8] = self.prefix_len;
        buf[9..13].copy_from_slice(&self.gateway);
        write_str_field(&mut buf[13..45], &self.web_user);
        write_str_field(&mut buf[45..77], &self.web_pass);
        buf
    }

    /// Parse a record back. Returns None if the magic is wrong (caller → defaults).
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < Self::LEN {
            return None; // too short to be a valid record
        }
        if buf[0..4] != CONFIG_MAGIC {
            return None; // wrong magic = blank/corrupt Flash → caller uses default()
        }
        let ip = [buf[4], buf[5], buf[6], buf[7]];
        let prefix_len = buf[8];
        let gateway = [buf[9], buf[10], buf[11], buf[12]];
        let web_user = read_str_field(&buf[13..45])?;
        let web_pass = read_str_field(&buf[45..77])?;
        Some(Config {
            ip,
            prefix_len,
            gateway,
            web_user,
            web_pass,
        })
    }

    /// Total serialized length in bytes (keep in sync with `to_bytes`).
    pub const LEN: usize = 4 /*magic*/ + 4 /*ip*/ + 1 /*prefix*/ + 4 /*gw*/ + 32 + 32;
}

/// Write a string into a fixed-size field: up to field.len() bytes, the rest left
/// as the caller's zeros (NUL termination). Truncates if the string is longer.
fn write_str_field(field: &mut [u8], s: &str) {
    let bytes = s.as_bytes();
    let n = bytes.len().min(field.len());
    field[..n].copy_from_slice(&bytes[..n]);
    // The remaining bytes stay 0 (buf was zero-initialized) → NUL-padded.
}

/// Read a string from a fixed-size field: take bytes up to the first NUL, then
/// validate UTF-8. Returns None on invalid UTF-8 or if it doesn't fit HString<32>.
fn read_str_field(field: &[u8]) -> Option<HString<32>> {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    let s = core::str::from_utf8(&field[..end]).ok()?;
    HString::try_from(s).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let c = Config::default();
        let bytes = c.to_bytes();
        assert_eq!(Config::from_bytes(&bytes), Some(c));
    }

    #[test]
    fn bad_magic_is_none() {
        let bytes = [0u8; Config::LEN];
        assert_eq!(Config::from_bytes(&bytes), None);
    }
}
