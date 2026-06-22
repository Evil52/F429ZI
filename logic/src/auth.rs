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

/// Build the expected auth token `base64("user:pass")`.
/// Empty user AND pass should be handled by the caller as "auth disabled".
pub fn basic_token(user: &str, pass: &str) -> HString<96> {
    todo!("form 'user:pass', base64-encode into a heapless String")
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
