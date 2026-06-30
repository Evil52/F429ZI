//! Pure parsers shared by the web layer: IPv4 dotted-decimal, TCP port, etc.
//! No Embassy, no sockets — just `&str` → value, so they're unit-tested on the host.

/// Parse "192.168.1.20" → [192,168,1,20]. Returns None on any malformed octet.
pub fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let mut octets = [0u8; 4];
    let mut parts = s.split('.');
    for slot in octets.iter_mut() {
        let part = parts.next()?; // меньше 4 частей -> None
        *slot = part.parse::<u8>().ok()?; // не число или >255  -> None
    }
    if parts.next().is_some() {
        return None;
    }
    Some(octets)
}

/// Parse a TCP/UDP port "1..=65535". Returns None on 0 or overflow.
pub fn parse_port(s: &str) -> Option<u16> {
    let port = s.parse::<u16>().ok()?;
    if port == 0 {
        return None; // порт 0 невалиден
    }
    Some(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_ok() {
        assert_eq!(parse_ipv4("192.168.1.20"), Some([192, 168, 1, 20]));
    }

    #[test]
    fn ipv4_rejects_garbage() {
        assert_eq!(parse_ipv4("192.168.1"), None);
        assert_eq!(parse_ipv4("192.168.1.999"), None);
        assert_eq!(parse_ipv4(""), None);
    }

    #[test]
    fn port_ok_and_rejects_zero() {
        assert_eq!(parse_port("80"), Some(80));
        assert_eq!(parse_port("0"), None);
        assert_eq!(parse_port("70000"), None);
    }
}
