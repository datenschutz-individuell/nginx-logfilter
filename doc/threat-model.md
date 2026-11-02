# Threat model

## What it promises

The `.err` files contain no client IP address in the places nginx puts one: the `, client: <IP>` field and "client <IP> …" at level `info`.

It does not promise that no address appears anywhere. A request line, a header or an upstream message quoted by nginx can contain one, and so can a PHP message passed on by FastCGI. Those are not touched, because telling an address in free text apart from, say, a version number or a server address is guesswork.

## Who can write to the socket

The socket has mode 0600 and belongs to the nginx worker user. Anyone who can send to it can already run code as that user. Such a sender can

- write arbitrary text into any `<tag>.err` (log spoofing, not prevented);
- create up to as many files as there are valid tags (`[A-Za-z0-9_]{1,32}`), nothing outside the log directory.

## Why a directory of its own

The nginx master runs as root and opens its access and error logs in `/var/log/nginx` at start, reload and `USR1`, following symlinks. If the worker user can write to that directory, code running as that user – a compromised PHP application, for example – can replace a log with a symlink to any file, and root will create or append to it (CVE-2016-1247). Writing the `.err` files into `/var/log/nginx` would have required exactly that permission, so the service uses `/var/log/nginx-logfilter`, which nginx never opens as root.

`fs.protected_symlinks` does not help here: it applies only to sticky, world-writable directories.

## Sandbox

The service runs as the worker user with `ProtectSystem=strict` (only the log directory is writable), no network (`PrivateNetwork`, `AF_UNIX` only), `NoNewPrivileges` and `MemoryDenyWriteExecute`. It parses untrusted text and nothing else; there is no `unsafe` beyond taking the socket from systemd.

## Availability

If the service stops, nginx keeps running. The socket stays open under systemd and buffers datagrams until the kernel's queue is full; after that, nginx's sends fail and those lines are lost. Lost error lines are accepted, a blocked nginx would not be.
