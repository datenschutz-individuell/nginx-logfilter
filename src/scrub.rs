// SPDX-FileCopyrightText: 2026 Olav Seyfarth
// SPDX-License-Identifier: CC0-1.0

//! Removes client IP addresses from nginx error log lines.

use std::net::{IpAddr, SocketAddr};

/// Fields nginx appends to an error line that belongs to a connection (stream logs UDP as `udp client`).
const FIELDS: [&str; 2] = [", client: ", ", udp client: "];
/// Messages at level `info` name the client in free text: "client 1.2.3.4 closed …" (http),
/// "client 1.2.3.4:5678 connected to …" (stream, mail).
const PHRASE: &str = "client ";

/// Drops the client field and shortens `client <address> ` to `client `.
///
/// Only an IP address, with or without a port, is removed, so other text containing `client:`
/// (a PHP message passed on by `FastCGI`, say) stays. At the very end of a line, where nginx cuts
/// long lines, a field also goes if what is left could be the start of an address.
pub fn strip_client_ip(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 1);
    let mut rest = line;
    while let Some((pos, len, is_field)) = next_match(rest) {
        let after = &rest[pos + len..];
        let end = after.find(if is_field { ',' } else { ' ' }).unwrap_or(after.len());
        if is_address(&after[..end]) || (is_field && end == after.len() && looks_cut(after)) {
            out.push_str(&rest[..pos]);
            if !is_field {
                out.push_str("client");
            }
            rest = &after[end..];
        } else {
            out.push_str(&rest[..pos + len]);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Earliest field or phrase in `s`: (position, length, is a field).
fn next_match(s: &str) -> Option<(usize, usize, bool)> {
    let fields = FIELDS.iter().filter_map(|f| s.find(f).map(|p| (p, f.len(), true)));
    let phrase = s.find(PHRASE).map(|p| (p, PHRASE.len(), false));
    fields.chain(phrase).min_by_key(|m| m.0)
}

/// What is left of an address that nginx cut short: only characters an address can contain.
fn looks_cut(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit() || b"[].:".contains(&b))
}

fn is_address(s: &str) -> bool {
    s.parse::<IpAddr>().is_ok() || s.parse::<SocketAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::strip_client_ip as strip;

    #[test]
    fn ipv4_in_the_middle() {
        assert_eq!(
            strip("open() failed (2: x), client: 1.2.3.4, server: a.de, request: \"GET / HTTP/1.1\""),
            "open() failed (2: x), server: a.de, request: \"GET / HTTP/1.1\""
        );
    }

    #[test]
    fn ipv6_and_end_of_line() {
        assert_eq!(
            strip("SSL_do_handshake() failed while SSL handshaking, client: 2001:db8::1"),
            "SSL_do_handshake() failed while SSL handshaking"
        );
        assert_eq!(strip("x, client: 2001:db8::1, server: 0.0.0.0:443"), "x, server: 0.0.0.0:443");
        assert_eq!(strip("x, client: ::ffff:1.2.3.4, server: d"), "x, server: d");
    }

    #[test]
    fn stream_udp_and_port() {
        assert_eq!(strip("x, udp client: 203.0.113.7, server: 0.0.0.0:53"), "x, server: 0.0.0.0:53");
        assert_eq!(
            strip("*1 client 198.51.100.4:51234 connected to 0.0.0.0:443"),
            "*1 client connected to 0.0.0.0:443"
        );
        assert_eq!(strip("*1 client [2001:db8::1]:51234 connected to [::]:443"), "*1 client connected to [::]:443");
    }

    #[test]
    fn field_cut_short_at_end() {
        assert_eq!(strip("FastCGI sent in stderr: \"long\", client: 2a0a"), "FastCGI sent in stderr: \"long\"");
        assert_eq!(strip("the client add."), "the client add."); // free text, not a field
        assert_eq!(strip("login failed, client: unknown user"), "login failed, client: unknown user");
    }

    #[test]
    fn other_text_stays() {
        let line =
            "FastCGI sent in stderr: \"PHP message: foo, client: bar, baz\" while reading, client: 9.9.9.9, server: d";
        assert_eq!(
            strip(line),
            "FastCGI sent in stderr: \"PHP message: foo, client: bar, baz\" while reading, server: d"
        );
    }

    #[test]
    fn info_message() {
        assert_eq!(strip("client 1.2.3.4 closed keepalive connection"), "client closed keepalive connection");
        assert_eq!(strip("client sent invalid header"), "client sent invalid header");
    }

    #[test]
    fn line_without_client() {
        assert_eq!(strip("worker process 12 exited on signal 9"), "worker process 12 exited on signal 9");
    }
}
