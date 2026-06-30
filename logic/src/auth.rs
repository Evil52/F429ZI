//! Web login token helper. Pure function so it's unit-tested on the host.
//!
//! The dashboard uses a form-login + session-cookie scheme (see src/web.rs). The
//! "expected token" is derived once at boot from the configured user/pass and
//! compared against what the login form submits. Building it as base64(user:pass)
//! mirrors HTTP Basic Auth's encoding but we never put it in a WWW-Authenticate
//! header (that would trigger the browser's native popup and break logout).
//!
//! NOTE: base64 over plain http:// is NOT encryption — fine on a trusted LAN,
//! not a substitute for TLS on the public internet.

use heapless::String as HString;
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Build the expected auth token `base64("user:pass")`.
/// Empty user AND pass should be handled by the caller as "auth disabled".
pub fn basic_token(user: &str, pass: &str) -> HString<96> {
    let mut input: heapless::Vec<u8, 64> = heapless::Vec::new();
    let _ = input.extend_from_slice(user.as_bytes());
    let _ = input.push(b':');
    let _ = input.extend_from_slice(pass.as_bytes());

    let mut out: HString<96> = HString::new();
    for chunck in input.chunks(3) {
        let b0 = chunck[0] as u32;
        let b1 = if chunck.len() > 1 {
            chunck[1] as u32
        } else {
            0
        };
        let b2 = if chunck.len() > 2 {
            chunck[2] as u32
        } else {
            0
        };
        let n = (b0 << 16) | (b1 << 8) | b2;

        let i0 = ((n >> 18) & 0x3F) as usize;
        let i1 = ((n >> 12) & 0x3F) as usize;
        let i2 = ((n >> 6) & 0x3F) as usize;
        let i3 = (n & 0x3F) as usize;

        // Первые два символа есть всегда; 3-й и 4-й — padding, если байтов <3.
        let _ = out.push(B64[i0] as char);
        let _ = out.push(B64[i1] as char);
        let _ = out.push(if chunck.len() > 1 {
            B64[i2] as char
        } else {
            '='
        });
        let _ = out.push(if chunck.len() > 2 {
            B64[i3] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector() {
        // base64("admin:secret") == "YWRtaW46c2VjcmV0"
        assert_eq!(basic_token("admin", "secret").as_str(), "YWRtaW46c2VjcmV0");
    }
}
