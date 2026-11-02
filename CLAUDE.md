# Working on nginx-logfilter

Notes for AI agents. Facts and conventions only; the reasoning is in [`doc/threat-model.md`](doc/threat-model.md). [`REVIEW.md`](REVIEW.md) says what review looks at and what is settled.

## What it is

A syslog receiver that writes the nginx error log without client IP addresses. One binary, Rust standard library only, run by systemd with socket activation.

## Layout

| Path | Role |
|---|---|
| `src/main.rs` | socket (from systemd or bound itself), receive loop |
| `src/syslog.rs` | parse `<PRI>timestamp TAG: TEXT`, validate the tag |
| `src/scrub.rs` | remove client IP addresses |
| `src/logdir.rs` | append to `<dir>/<tag>.err`, reopen on inode change |
| `systemd/` | socket and service units, tmpfiles, logrotate |

## Commands

```bash
cargo fmt && cargo clippy --all-targets -- -W clippy::pedantic && cargo test
```

clippy pedantic must be clean. Unit tests live next to the code in each module.

## Conventions

- **No dependencies.** Anything that needs a crate needs a reason in `REVIEW.md` first.
- **The tag is the only input that reaches a path.** It is validated in `syslog::parse`, nowhere else; keep it that way.
- **Lean towards removing.** A missed address is a privacy leak, a removed word is cosmetic.
- **Version** lives in `Cargo.toml` (and `Cargo.lock`); every release gets a `CHANGELOG.md` section (Keep a Changelog, SemVer).
- **SPDX headers** (`CC0-1.0`) on source and unit files.
- English for code, comments and docs.
- **CI** (`.github/workflows/ci.yml`) runs the commands above on Rust 1.85 and stable; actions are pinned to a commit SHA with a version comment.
