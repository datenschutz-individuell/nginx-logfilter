// SPDX-FileCopyrightText: 2026 Olav Seyfarth
// SPDX-License-Identifier: CC0-1.0

//! Removes client IP addresses from nginx error log lines.

use std::net::IpAddr;

/// nginx appends this field to every error line that belongs to a connection.
const FIELD: &str = ", client: ";
/// Messages at level `info` name the client in free text, e.g. "client 1.2.3.4 closed …".
const PHRASE: &str = "client ";

/// Drops `, client: <IP>` and shortens `client <IP> ` to `client `.
///
/// Only text that parses as an IP address is removed, so other text containing
/// `client:` (a PHP message quoted by `FastCGI`, say) stays as it is.
pub fn strip_client_ip(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    loop {
        let field = rest.find(FIELD).map(|p| (p, FIELD.len(), true));
        let phrase = rest.find(PHRASE).map(|p| (p, PHRASE.len(), false));
        let Some((pos, len, is_field)) = earliest(field, phrase) else { break };
        let after = &rest[pos + len..];
        let end = after.find(if is_field { ',' } else { ' ' }).unwrap_or(after.len());
        if after[..end].parse::<IpAddr>().is_ok() {
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

fn earliest<T>(a: Option<(usize, usize, T)>, b: Option<(usize, usize, T)>) -> Option<(usize, usize, T)> {
    match (a, b) {
        (Some(a), Some(b)) => Some(if a.0 < b.0 { a } else { b }),
        (a, b) => a.or(b),
    }
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
