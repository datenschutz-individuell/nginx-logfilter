# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2026-11-02

### Fixed

- Remove the address from the stream module's UDP field `, udp client: <IP>`
- Remove addresses with a port, as stream and mail log them at level `info` ("client 1.2.3.4:5678 connected to …")
- Remove an address that nginx cut short at the end of a long line
- Remove an address at the end of a message that cannot be parsed, before its start goes to stderr

### Changed

- logrotate: no `compress`/`delaycompress` (with `rotate 1` the old file was never compressed) and `nocreate`, as the service creates the file itself

## [1.0.0] - 2026-11-02

### Added

- Receive nginx `error_log` over syslog on a Unix datagram socket and append each line to `<dir>/<tag>.err`
- Remove `, client: <IP>` and the address in "client <IP> …" messages; leave text that is not an IP address
- Reopen a file when its inode changes, so logrotate needs no signal
- systemd socket activation, so the socket and its buffered messages survive restarts
- systemd units with sandboxing, tmpfiles and logrotate configuration
- Logs in a directory of their own, `/var/log/nginx-logfilter`

[Unreleased]: https://github.com/datenschutz-individuell/nginx-logfilter/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/datenschutz-individuell/nginx-logfilter/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/datenschutz-individuell/nginx-logfilter/releases/tag/v1.0.0
