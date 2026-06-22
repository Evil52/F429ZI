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
    pub fn to_bytes(&self) -> [u8; Self::LEN] {
        todo!("write CONFIG_MAGIC, ip, prefix_len, gateway, web_user(NUL-term), web_pass(NUL-term)")
    }

    /// Parse a record back. Returns None if the magic is wrong (caller → defaults).
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        todo!("check CONFIG_MAGIC, then read fields back; lenient on trailing 0xFF")
    }

    /// Total serialized length in bytes (keep in sync with `to_bytes`).
    pub const LEN: usize = 4 /*magic*/ + 4 /*ip*/ + 1 /*prefix*/ + 4 /*gw*/ + 32 + 32;
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
