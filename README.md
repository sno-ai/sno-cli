# SNO CLI

`sno` is the unified command-line entry point for SNO. The first Rust release ports the working machine identity, local telemetry, export, audit verification, and diagnostics flows from the earlier Nodix CLI. It also supports Git-style external subcommands.

The CLI is distributed only as Rust source through crates.io and as native binaries through GitHub Releases. It has no Node.js or Python runtime dependency.

## Install

Build from this repository:

```sh
cargo install --path .
```

Install from crates.io when a Rust toolchain is available:

```sh
cargo install sno
```

GitHub Releases provide precompiled archives plus Shell and PowerShell installers for systems without a Rust toolchain. Each release includes SHA-256 checksums and verifiable GitHub build provenance when available.

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
sno station rem-start --type noop --scope <SCOPE>
sno station rem-status <JOB_ID> [--wait [--timeout <SECONDS>]]
```

Every built-in command accepts `--json`. Commands emit one JSON value except `sno account machine claim`, which emits newline-delimited JSON: an `authorization` record before waiting and a final `result` or `error` record. This lets users and automation receive the browser verification code before approval. Success is exit code `0`, runtime failure is `1`, and command-line usage errors exit with `2`.

### Local REM jobs

REM jobs use the local Sno Station sidecar and are asynchronous. Start returns immediately with a
job id:

```sh
sno station rem-start --type noop --scope persona:demo
```

Read the current state once, or wait for a terminal state:

```sh
sno station rem-status <JOB_ID>
sno station rem-status <JOB_ID> --wait --timeout 60
```

Human-readable status output includes both the job id and state.
The default wait timeout is 60 seconds. The `station rem-*` family uses this exit-code contract:

| Outcome class | Exit code | Machine-readable error codes |
|---|---:|---|
| success | `0` | — |
| unclassified failure | `1` | Any error code absent from the declaration |
| invalid usage | `2` | `usage_error` |
| job failed | `3` | `rem_job_failed` |
| wait deadline passed | `4` | `rem_timeout` |
| state vocabulary mismatch | `5` | `rem_state_unrecognised` |
| malformed or truncated response | `6` | `sidecar_response_invalid`, `sidecar_response_truncated` |
| sidecar failure | `7` | `sidecar_not_running`, `sidecar_unauthorized`, `sidecar_client_error`, `sidecar_discovery_error`, `sidecar_discovery_invalid`, `sidecar_response_error` |
| local environment failure | `8` | `profile_error`, `rem_trace_error` |
| unknown job identifier | `9` | `rem_job_not_found` |

Add `--json` to either command for stable JSON output. The CLI rereads local sidecar discovery on
every request and poll, so an active wait can reconnect after a sidecar restart.
If the audit stream is unavailable, the job remains non-terminal and the sidecar reports the
write failure; `--wait` exits `4` when its timeout expires instead of claiming an unaudited result.

REM JSON start output includes both `job_id` and `correlation_id`. The CLI sends the correlation
id to the sidecar and writes a durable, token-free JSONL trace for each parsed command, discovery
read, request, response, poll, and transient failure. A caller can set `SNO_REM_CORRELATION_ID` to
join start and status under one id. Otherwise, the CLI creates an id. The process trace is fixed at
`$OPENCLAW_STATE_DIR/mem-claw/rem-trace.jsonl`, defaulting to
`~/.openclaw/mem-claw/rem-trace.jsonl`.
`SNO_REM_TRACE=0` disables this process trace only; it does not control the sidecar's independent
memory audit stream.

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

The five formally supported native platforms are:

- Linux x64 and ARM64
- macOS Intel and Apple Silicon
- Windows x64

Linux additionally ships static musl x64 and ARM64 artifacts for Alpine and portable Linux use. Windows ARM64 remains unsupported until a generally available native runner passes the same build, extraction, installer, and Station smoke gates.

## Naming guardrail

Code identifiers and package names must use compound SNO forms such as `sno_station`, `sno-station`, `sno_starport`, or `sno-starport`. Bare product nouns such as `station` are allowed only as user-facing command tokens and must not be standalone code identifiers or package names.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and release checks.

## License

Apache-2.0. The license will be reviewed with the formal product requirements before a future major release.
