# Legacy Contract Matrix: SNO CLI Initial Release

Status: Frozen baseline
Captured: 2026-07-15 PDT
Source repository: `/home/lh/code/nodix-private`
Source commit: `4256aa66aae2dc95edc71f788b456874a789b360`

This document is the implementation contract for moving the legacy TypeScript CLI into the approved Account and Station command groups. It records behavior, not the old routing name. Every row must map to a Rust parity test and, where shared state is involved, a cross-language fixture test.

## Baseline Integrity

`fixtures/legacy-contract/source-manifest.sha256` contains 105 per-file SHA-256 entries covering the legacy CLI, all TypeScript observability and common identifier sources, all CLI/observability tests and fixtures, dependency manifests, TypeScript configs, the binary entry point, and the root lockfile. Its current SHA-256 is `7776b41324807183189cac761648d1a4381777792a86f501557836cf6d939b95`.

`fixtures/legacy-contract/obligations.json` maps every row below to exact source owners, named legacy tests, and named golden cases. Run:

```text
scripts/verify-legacy-baseline.sh /home/lh/code/nodix-private
```

The command fails on any changed source hash, missing/extra obligation, unhashed mapped file, unknown golden, non-deterministic namespace transformation, or missing hard-cut negative test. A changed file is located by the per-file manifest; all obligations referencing that path are invalid until reviewed and regenerated.

## Namespace Transformation

Every migrated golden stores both `legacy_argv` and `rust_argv`. Consent maps to `sno station telemetry consent ...`; observe maps to `sno station telemetry ...`; register and claim map to `sno account machine ...`; audit and doctor map to `sno station ...`. Root help/version cases preserve their argument array. New root-only capabilities are explicitly marked without claiming a legacy equivalent. Hard-cut negative cases run each old top-level noun against Rust and require usage exit `2`, while recording the canonical migrated array.

## Global Contract

| Row | Input | Success contract | Failure contract | State/network | Required test |
|---|---|---|---|---|---|
| CLI-001 | `sno --version` | Exact package version plus newline; exit `0` | None | No state or network | CLI snapshot |
| CLI-002 | `sno --help` or `-h` | Help contains `station`, `starport`, and external-subcommand explanation; exit `0` | None | No state or network | CLI snapshot |
| CLI-003 | `sno` | Help on stdout; empty stderr; exit `2` | N/A | No state or network | CLI integration |
| CLI-004 | `sno --json` | `{"error":"usage_error","message":"missing command"}` on stdout; empty stderr; exit `2` | N/A | No state or network | Golden `missing_command_json` |
| CLI-005 | Any built-in with `--json` | Exactly one JSON value on stdout; empty stderr; exit matches human mode | JSON error envelope has string `error` and `message` | Handler stderr is not allowed to leak | Every command integration |
| CLI-006 | Parser error | Empty stdout, `error: <message>` on stderr, exit `2` | With `--json`, error envelope on stdout and empty stderr | No state mutation after parse failure | Parser integration |
| CLI-007 | Exit behavior | `0` success, `1` runtime or negative verification, `2` usage | Child external command exit is propagated | No masked errors | Exit matrix |

`--json` is accepted at the root, noun, and leaf command positions. `.snorc` files are ignored.

## Command Contract

| Row | Canonical input | Flags/defaults | Human success | JSON success | Failure and exit | State and side effects | Network/status/retry | Required parity test |
|---|---|---|---|---|---|---|---|---|
| STN-001 | `sno station telemetry consent get` | `--json`; default consent is `metadata-only` | `<consent>\n` | `{"consent":"<value>"}` | Malformed stored consent is runtime failure, exit `1` | Reads consent file only | None | consent default/read |
| STN-002 | `sno station telemetry consent set <value>` | Values: `off`, `metadata-only`, `full`; reason is exactly `sno cli consent set` | `<value>\n` | Consent plus `chain_epoch_advanced`, true iff value changed | `metadata` suggests `metadata-only`; all other invalid/missing values are usage failures, exit `2` | Atomic mode-0600 consent write; chain transition/epoch behavior follows shared buffer contract; may flush transition | Uses normal flush path only when transition rules require it; no command-specific retry | consent values/invalid/chain cross-language |
| STN-003 | `sno station telemetry pause` | `--json`; no arguments | `paused\noff\n`, or `already paused\noff\n` | Consent `off`, `paused:true`, and accurate `already_paused` | State or flush failure is runtime exit `1` | First pause saves prior consent once, sets off, advances chain; repeat pause preserves prior snapshot | Consent-transition flush rules only | pause repeat/state cross-language |
| STN-004 | `sno station telemetry resume` | `--json`; missing saved prior defaults to `metadata-only` | `resumed: <consent>\n` | `{"consent":"<value>"}` | State or flush failure is runtime exit `1` | Restores saved prior, removes prior file; resumes from off in a new chain epoch | Consent-transition flush rules only | resume prior/default cross-language |
| STN-005 | `sno station telemetry export [path]` | `--out <path>` mutually exclusive with positional path; `--format tarball|jsonl|csv`; infer format from `.csv`, `.jsonl`, `.tar.gz`, `.tgz`; default `./sno-export-<unix-seconds>.tar.gz` | Stream JSONL/CSV bytes when no path; file export is silent except tarball prints `exported <n> events to <path>` | `format`, nullable `path`, `event_count`, `bytes`, optional `tarball_sha256`; never binary stdout | Invalid format or two paths: usage exit `2`; write/read failure: runtime exit `1` | Reads all buffer rows; writes atomically when path supplied; never changes shipped/terminal flags | None | export format/path/empty/nonmutation cross-language |
| ACC-001 | `sno account machine register` | `--json`; non-interactive | Four lines: `registered`, `user_cuid=...`, `machine_uuid=...`, `claimed=...` | `registered`, `claimed`, `user_cuid`, `machine_uuid` | Fetch failure maps to `network_error`; server code is preserved when present; otherwise `register_failed`; exit `1` | Bootstraps identity; mode 0600 on Unix; persists valid returned account ID only for matching identity | POST `/api/v1/identity/register-machine`; JSON has user CUID, machine UUID, SHA-256 machine-secret hash; no auth header; exact status `200` plus valid/matching body required; 10s request timeout; no retries | register request/status/identity cross-language |
| ACC-002 | `sno account machine claim` | `--json`; non-interactive device flow; 30-minute total timeout | Prints verification URI, optional complete URI, user code, waiting marker, then claimed/account/identity lines | Claimed/account/identity plus nested code object | Abort, timeout, malformed response, registration error, server error, and network exhaustion are runtime exit `1` with preserved SDK code except fetch text maps to `network_error` | Bootstraps and registers identity; persists only `user_account_id`; no account token is stored | Register first without authorization; POST `/api/v1/device/code` without authorization and body keys `machine_uuid`,`user_cuid`; then POST `/api/v1/device/token` without authorization and body keys `device_code`,`grant_type`. Default poll 5s; clamp 1–30s; `authorization_pending` waits; `slow_down` uses server interval or +5s; transient poll failures retry 3 times with capped doubling delay; each request timeout 10s | claim flow/status/retry/persistence/request-header cross-language |
| STN-008 | `sno station audit verify <event_id>` | `--json`; event ID URL-encoded | Verified: check mark line plus pretty JSON, exit `0`; unverified: tampered line plus pretty JSON, exit `1` | Raw verification object; `verified:false` exits `1` | Missing/unowned maps to `not_found_or_unowned`; fetch maps to `network_error`; other failure `audit_verify_failed`; exit `1`; missing argument exit `2` | Bootstraps/registers identity before verify | Register request, then GET `/api/v1/audit/verify?event_id=...`; machine secret is bearer only on verify; `404` is missing/unowned; exact `200` required; 10s per request; no retries | audit auth/status/result cross-language |
| STN-009 | `sno station doctor` | `--json`; no arguments | Five ordered badge lines: identity, buffer, consent, last ship, lockfile | Object keys in the same order with full check objects | Any warn/fail check causes exit `1`; diagnostic conditions are results, not thrown errors | Read-only; honors all path overrides; never bootstraps identity or buffer | No network request; reports configured normalized base URL only | doctor healthy/warn/fail/read-only cross-language |

## Shared State Contract

| Row | Contract | Required fixture |
|---|---|---|
| DATA-001 | Profile precedence: `SNO_PROFILE_DIR`, then `SNO_HOME`, then `~/.sno` | TypeScript-created profile read by Rust and reverse |
| DATA-002 | Explicit overrides: `SNO_IDENTITY_PATH`, `SNO_BUFFER_PATH`, `SNO_CONSENT_PATH`, `SNO_OBSERVE_BASE_URL` | One fixture per override |
| DATA-003 | Default paths: `identity.json`, `buffer.db`, `state/consent.json`, `state/consent-prior.json`; identity lock beside identity | Path-resolution fixture on all three operating-system families |
| DATA-004 | Identity version 1: valid CUID2 user ID, lowercase canonical UUIDv7 machine ID, 64 lowercase hex secret, ISO timestamp, optional project/account IDs; Unix file mode 0600 and directory mode 0700 | Bidirectional JSON fixture plus permission test |
| DATA-005 | Consent and pause state version 1, ISO timestamps, atomic mode-0600 writes; missing consent means `metadata-only` | Bidirectional state fixture |
| DATA-006 | SQLite schema, row ordering, wire envelope bytes, chain epochs, sequence, self/prev hashes, shipped and terminal flags remain byte/behavior compatible | TypeScript database fixture read/exported by Rust; Rust database read/exported by TypeScript |
| DATA-007 | JSONL is stored envelope bytes plus newline; CSV uses the documented ten-column header and RFC 4180 quoting; tarball contains `events.jsonl` and `MANIFEST.json` with version 1 hashes/chains | Golden export fixtures for empty and populated buffers |

## Security and Transport Contract

| Row | Contract | Required test |
|---|---|---|
| SEC-001 | Never print machine secret, secret hash, registry token, or account token | Output/package recursive secret scan |
| SEC-002 | Production base URL must be HTTPS; HTTP is accepted only for `localhost`, `127.0.0.1`, or IPv6 loopback | URL table test |
| SEC-003 | Registration sends only the secret hash without bearer; both claim endpoints have no authorization header; audit verify sends raw machine secret only as HTTPS bearer | Loopback request capture plus production-shaped smoke |
| SEC-004 | External `sno-<name>` dispatch uses direct process execution, preserves raw arguments/stdout/stderr/exit, and never invokes a shell | Metacharacter and exit propagation integration |

## Test-Substitute Inventory

| Substitute | Scope | Allowed behavior |
|---|---|---|
| Real loopback HTTP server derived from `tests/apps/nodix-cli/fixtures/sno-ai-mock-server.mjs` | External SNO HTTP service only | Bind loopback, accept actual sockets, capture full requests, return scripted protocol responses |

No mocking library, internal module replacement, fake filesystem, fake SQLite implementation, or intercepted process launcher is allowed. Temporary real files, real SQLite databases, real child processes, and the listed loopback server are required.

Enforcement is defined by `policy/test-substitutes.json` and `scripts/check-test-substitutes.sh`. The scan covers all Rust, TypeScript/JavaScript, and Python implementation/test files plus dependency manifests while excluding only dependency/build directories. It rejects named mocking dependencies, mocking or monkey-patching APIs, named fake filesystem/database/process substitutes, and any file that creates a service server outside the exact allowlisted path. The allowlisted Rust server must pass a literal `127.0.0.1` or `[::1]` address directly to each bind expression and must not contain `0.0.0.0`. `scripts/test-test-substitute-policy.sh` mutation-tests dependencies, normal and colocated internal replacement, filesystem/database/process substitutes, undeclared servers, misleading loopback comments, and the allowed path. Handwritten semantic substitutes that evade all enumerated syntax remain a mandatory code-review finding; the static command does not claim semantic detection.
