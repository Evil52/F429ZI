//! Host integration tests for parsers + config round-trip.

use f429zi_logic::{config::Config, parse};

#[test]
fn ipv4_roundtrip_via_parser() {
    assert_eq!(parse::parse_ipv4("10.0.0.5"), Some([10, 0, 0, 5]));
    assert_eq!(parse::parse_ipv4("nope"), None);
}

#[test]
fn config_default_round_trip() {
    let c = Config::default();
    let bytes = c.to_bytes();
    assert_eq!(Config::from_bytes(&bytes), Some(c));
}
