# Review guidance

What is worth flagging here, and what has been decided. Project facts are in [`CLAUDE.md`](CLAUDE.md).

## Worth flagging

- **Anything that lets the tag reach a path unvalidated**, or that writes outside the log directory.
- **An address that survives.** A new nginx message shape that carries the client address, an IPv6 notation `IpAddr` does not parse, a change to `strip_client_ip` that skips a match.
- **Text that is removed although it is not an address.** Only parsed IP addresses go.
- **Permissions.** The log directory must not be `/var/log/nginx`, and nothing may make that directory writable for the worker user (CVE-2016-1247).
- **A line split across writes.** Each line goes out in one `write_all` on an `O_APPEND` file.
- **Unsafe code** beyond the one `from_raw_fd` for socket activation.

## Already decided – please do not flag

- **No traits or abstraction layers.** There is one input (a datagram socket) and one output (files). An interface for a second one would have no second implementation. The split into modules is by responsibility, which is as much SOLID as a program of this size needs.
- **No `libc`, no `O_NOFOLLOW`.** The log directory belongs to the service user; whoever could plant a symlink there already has the service's rights.
- **The service runs as the nginx worker user.** A separate user would only protect the `.err` files from the workers, which write that content anyway.
- **The fallback `bind` sets mode 0600 after binding.** It is for manual runs and tests; in production systemd creates the socket with `SocketMode`.
- **Lines are dropped, not stored raw, when the header does not parse.** A raw copy would keep the address.
- **`String::from_utf8_lossy`.** nginx error lines may carry bytes from requests; replacing invalid UTF-8 is fine for a log.
- **No timestamp of our own.** nginx's timestamp is part of the text; the syslog one is dropped.
