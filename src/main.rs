// SPDX-FileCopyrightText: 2026 Olav Seyfarth
// SPDX-License-Identifier: CC0-1.0

//! Receives nginx `error_log` over syslog on a Unix datagram socket, removes client IP addresses
//! and appends each line to `<dir>/<tag>.err`.
//!
//! nginx cannot format `error_log` (`log_format` applies to `access_log` only) and appends
//! `, client: <IP>` to every error line. Usage: `nginx-logfilter [SOCKET [DIR]]`.

mod logdir;
mod scrub;
mod syslog;

use std::fs;
use std::io::ErrorKind;
use std::os::fd::FromRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::process::exit;

use logdir::LogDir;
use scrub::strip_client_ip;

const SOCKET: &str = "/run/nginx-logfilter/sock";
const LOGDIR: &str = "/var/log/nginx-logfilter";
/// Receive buffer; well above the length to which nginx truncates an error line.
const MAX_DATAGRAM: usize = 65536;
/// First file descriptor passed by systemd socket activation (`SD_LISTEN_FDS_START`).
const LISTEN_FD: i32 = 3;

/// Takes the socket from systemd if it was passed to this process, otherwise binds it.
///
/// With socket activation the socket outlives restarts of the service, and the kernel buffers
/// what nginx sends in the meantime.
fn socket(path: &Path) -> UnixDatagram {
    let for_us = std::env::var("LISTEN_PID").ok().and_then(|p| p.parse::<u32>().ok()) == Some(std::process::id());
    if for_us && std::env::var("LISTEN_FDS").as_deref() == Ok("1") {
        // SAFETY: systemd passed exactly one descriptor, at LISTEN_FD, to this PID, and nothing
        // else in this process owns it.
        return unsafe { UnixDatagram::from_raw_fd(LISTEN_FD) };
    }
    let _ = fs::remove_file(path);
    let socket = UnixDatagram::bind(path).unwrap_or_else(|e| {
        eprintln!("socket {}: {e}", path.display());
        exit(1)
    });
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    socket
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let socket = socket(Path::new(args.get(1).map_or(SOCKET, String::as_str)));
    let mut logs = LogDir::new(args.get(2).map_or(LOGDIR, String::as_str));
    let mut buf = vec![0u8; MAX_DATAGRAM];
    loop {
        let n = match socket.recv(&mut buf) {
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("receive: {e}");
                exit(1) // systemd restarts the service
            }
        };
        handle(&mut logs, &String::from_utf8_lossy(&buf[..n]));
    }
}

/// Parses one syslog message, removes client addresses and appends the line to its file.
fn handle(logs: &mut LogDir, message: &str) {
    let message = message.trim_end_matches(['\n', '\0']);
    if let Some((tag, text)) = syslog::parse(message) {
        let line = strip_client_ip(text);
        if let Err(e) = logs.append(tag, line) {
            eprintln!("{tag}.err: {e}");
        }
    } else {
        let head: String = strip_client_ip(message).chars().take(60).collect();
        eprintln!("unparsable message dropped (nohostname missing?): {head:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_is_scrubbed_on_the_way_to_the_file() {
        let d = std::env::temp_dir().join(format!("nginx-logfilter-handle-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        let mut logs = LogDir::new(&d);
        handle(&mut logs, "<187>Nov  2 15:00:00 ex: 2026/11/02 [error] 1#1: *5 x, client: 9.9.9.9, server: d\n");
        handle(&mut logs, "<187>Nov  2 15:00:00 host ex: dropped, client: 1.2.3.4");
        assert_eq!(fs::read_to_string(d.join("ex.err")).unwrap(), "2026/11/02 [error] 1#1: *5 x, server: d\n");
        fs::remove_dir_all(d).unwrap();
    }
}
