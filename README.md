# SNO CLI

`sno` is the unified command-line entry point for SNO. The first Rust release ports the working machine identity, local telemetry, export, audit verification, and diagnostics flows from the earlier Nodix CLI. It also supports Git-style external subcommands.

The GitHub repository may be private while the crate is public. A crates.io release includes the source files listed in the crate archive and installs a public `sno` binary.

## Install

Before the first registry release, build from this repository:

```sh
cargo install --path .
```

After publication:

```sh
cargo install sno
```

## Commands

```text
sno --help
sno --version

sno account machine register
sno account machine claim

sno station telemetry consent get
sno station telemetry consent set <off|metadata-only|full>
sno station telemetry pause
sno station telemetry resume
sno station telemetry export [PATH] [--format tarball|jsonl|csv]
sno station audit verify <EVENT_ID>
sno station doctor
```

Every built-in command accepts `--json`. Success is exit code `0`, runtime failure is `1`, and invalid usage is `2`.

### External subcommands

An executable named `sno-example` on `PATH` is available as:

```sh
sno example <args...>
```

The CLI starts external commands directly without a shell and preserves their arguments and exit status. Built-in and retired command names never fall through to external dispatch.

## Local state

The default profile is `~/.sno`. These environment variables override its paths:

- `SNO_PROFILE_DIR`, then `SNO_HOME`, selects the profile directory.
- `SNO_IDENTITY_PATH` selects the machine identity file.
- `SNO_BUFFER_PATH` selects the SQLite telemetry buffer.
- `SNO_CONSENT_PATH` selects the consent state file.
- `SNO_OBSERVE_BASE_URL` selects the SNO service endpoint.

Production service endpoints must use HTTPS. Plain HTTP is accepted only for loopback development and tests. Machine secrets are stored locally with owner-only permissions on Unix and are never printed by the CLI.

## Platform scope

Prebuilt release artifacts target:

- Linux x64 and ARM64
- macOS Intel and Apple Silicon
- Windows x64

Linux musl/Alpine and Windows ARM64 prebuilt artifacts are outside the first release.

## Naming guardrail

Code identifiers and package names must use compound SNO forms such as `sno_station`, `sno-station`, `sno_starport`, or `sno-starport`. Bare product nouns such as `station` are allowed only as user-facing command tokens and must not be standalone code identifiers or package names.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and release checks.

## License

Apache-2.0. The license will be reviewed with the formal product requirements before a future major release.
