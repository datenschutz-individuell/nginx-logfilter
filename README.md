# nginx-logfilter

Writes the nginx **error log without client IP addresses**.

`log_format` applies to `access_log` only. `error_log` has no configurable format, and nginx appends `, client: <IP>` to every error line that belongs to a connection. nginx-logfilter receives the error log over syslog, removes that field and appends the line to a file per virtual host. It is a single small binary, uses the Rust standard library only, and needs no signal for log rotation.

## How it works

Each `error_log` sends to a Unix datagram socket:

```nginx
error_log syslog:server=unix:/run/nginx-logfilter/sock,tag=example_com,nohostname warn;
```

The service writes `/var/log/nginx-logfilter/example_com.err`.

- **`tag`** becomes the file name. nginx allows letters, digits and `_` only, up to 32 characters, so `mta-sts` has to become `mta_sts`. The service rejects any other tag.
- **`nohostname`** is required; messages with a hostname are dropped with a note on stderr.
- **What is removed:** the field `, client: <IP>`, and at level `info` the address in "client <IP> closed …". Only text that parses as an IPv4 or IPv6 address is removed; other text containing `client:` – a PHP message passed on by FastCGI, say – stays.
- **Rotation:** files stay open. When a file's inode changes, the service reopens it.
- **Restarts:** systemd holds the socket (`nginx-logfilter.socket`), so it survives restarts and updates of the service, and the kernel buffers what nginx sends in the meantime. If the service is down, nginx keeps running.
- **Not covered:** messages nginx logs before it has read its configuration go to the compiled-in `--error-log-path` (`nginx -V`). Other places an address can appear – a URL or header quoted in a message, `upstream:` – are left alone.

## Install

Build (Rust 1.85 or later):

```bash
cargo test && cargo build --release
sudo install -m 755 target/release/nginx-logfilter /usr/local/bin/
sudo install -m 644 systemd/nginx-logfilter.socket systemd/nginx-logfilter.service /etc/systemd/system/
sudo install -m 644 systemd/nginx-logfilter.tmpfiles /etc/tmpfiles.d/nginx-logfilter.conf
sudo install -m 644 systemd/nginx-logfilter.logrotate /etc/logrotate.d/nginx-logfilter
sudo systemd-tmpfiles --create nginx-logfilter.conf
sudo systemctl daemon-reload && sudo systemctl enable --now nginx-logfilter.socket nginx-logfilter.service
```

The units assume nginx workers run as `http` (Arch Linux). On Debian and Ubuntu, change `User`, `Group`, `SocketUser`, `SocketGroup` and the logrotate `su`/`create` lines to `www-data`.

Then switch each `error_log` to the syslog form above and reload nginx.

**Do not** point the service at `/var/log/nginx` or make that directory writable for the worker user. The nginx master opens its logs there as root; a directory the workers can write to lets them redirect those writes with a symlink (CVE-2016-1247). See [`doc/threat-model.md`](doc/threat-model.md).

## Check

```bash
grep -c "client: " /var/log/nginx-logfilter/*.err   # 0 everywhere, except text that is not an address
journalctl -u nginx-logfilter                       # dropped messages and write errors
```

## Alternatives considered

- **Patching nginx** or a third-party module: has to follow every nginx update.
- **Rewriting `$remote_addr` with the realip module:** the address then reaches access control and `limit_req` too, which breaks rate limiting.
- **ipscrub** and similar modules work on `log_format` variables and so on the access log only.
- **rsyslog `mmanon` or syslog-ng:** work, but bring a full syslog daemon for one rewrite.

## License

[CC0 1.0](LICENSE): public domain, no conditions.
