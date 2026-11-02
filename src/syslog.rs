// SPDX-FileCopyrightText: 2026 Olav Seyfarth
// SPDX-License-Identifier: CC0-1.0

//! Parses the BSD syslog messages (RFC 3164) that nginx sends with `nohostname`.

/// Splits `<PRI>Mmm dd hh:mm:ss TAG: TEXT` into `(TAG, TEXT)`.
///
/// The tag becomes a file name, so it must be what nginx itself allows:
/// 1–32 characters of `[A-Za-z0-9_]`. Anything else is rejected.
pub fn parse(message: &str) -> Option<(&str, &str)> {
    let s = message.strip_prefix('<')?;
    let s = &s[s.find('>')? + 1..];
    // The syslog timestamp has a fixed width of 15 plus a space; nginx's own timestamp is part of TEXT.
    let s = s.get(16..)?;
    let (tag, text) = s.split_once(": ")?;
    let valid = (1..=32).contains(&tag.len()) && tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
    valid.then_some((tag, text))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn header() {
        let m = "<187>Nov  2 15:00:00 example_com: 2026/11/02 15:00:00 [error] 1#1: *5 x, server: d";
        assert_eq!(parse(m), Some(("example_com", "2026/11/02 15:00:00 [error] 1#1: *5 x, server: d")));
    }

    #[test]
    fn bad_tag() {
        assert_eq!(parse("<187>Nov  2 15:00:00 ../etc/x: a"), None);
        assert_eq!(parse("<187>Nov  2 15:00:00 host tag: a"), None); // nohostname missing
        assert_eq!(parse(&format!("<187>Nov  2 15:00:00 {}: a", "a".repeat(33))), None);
        assert_eq!(parse("<187>Nov  2 15:00:00 : a"), None);
        assert_eq!(parse("garbage"), None);
    }
}
