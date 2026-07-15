# PRD: SNO CLI Initial Release

Status: Release Candidate
Date: 2026-07-14 PDT
ADLC project ID: `sno-cli-initial-release`
Scope: Create the unified Rust `sno` CLI, migrate the legacy Nodix operator commands into the approved Account and Station command groups, and release only real, usable packages.

## Decision Summary

SNO ships one Rust binary named `sno`. Version `0.1.0` is a functional first release, not a placeholder: it includes the existing Nodix identity workflows under `sno account machine ...`, its local telemetry workflows under `sno station ...`, top-level help/version behavior, a `sno starport` noun scaffold, and Git-style external subcommand dispatch for executables named `sno-<name>` on `PATH`.

The GitHub repository starts private. Publishing to crates.io is allowed only after package inspection, dry-run success, full review, and explicit owner approval. Publishing makes the Rust source included in the `.crate` archive public even while GitHub remains private.

## Problem

SNO needs one memorable CLI entry point. The current operator CLI is a private TypeScript application named `nodix` inside another repository. Shipping a new empty Rust binary would reserve a name but discard working behavior; keeping the TypeScript CLI beside the new Rust CLI would create two command authorities and permanent drift.

The release must therefore solve three problems together:

1. Establish the canonical `sno` binary and repository.
2. Move the existing operator behavior into the new two-level command namespace.
3. Publish only packages that install and run the real binary.

## Source-Grounded Current State

The source snapshot is `nodix-private` commit `4256aa66aae2dc95edc71f788b456874a789b360` on branch `dev`. The checkout is three commits ahead of its remote; this project must not overwrite or discard those changes.

- `nodix-private/apps/nodix-cli/package.json` defines private package `@snoai/nodix` version `0.9.74` and binary `nodix`.
- `nodix-private/apps/nodix-cli/src/index.ts` exposes `consent`, `observe`, `register`, `claim`, `audit`, and `doctor` as top-level commands.
- Each command delegates to `@snoai/sno-observe`; the command layer does not own identity generation, SQLite buffering, consent transitions, exports, machine registration, device claim, or audit verification.
- The user-visible command layer is about 800 source lines. The directly required SDK implementation spans identity, consent, buffer, export, diagnostics, registration, claim, and audit code; a correct port is not a parser-only translation.
- The archived May 2026 CLI specification describes the existing behavior but is reference material only. Its old top-level naming and Node-only packaging decisions are superseded by this PRD.
- The draft “Nodix CLI as the Unified Local AI Setup Entry” is not authoritative. Its memory/gateway/scale wizard remains future product work.
- Current name checks on 2026-07-14 PDT found no exact crates.io package `sno`, no public npm package `sno-ai`, no PyPI distribution `sno`, and no GitHub repository `sno-ai/sno-cli`. These checks must run again immediately before each irreversible publish.

Probe evidence is recorded in `ai-doc/ACTIVE/PRD/PROBE-RESULTS-sno-cli-initial-release.md`.

## Target Users and Operators

- End users who install one SNO CLI and need local account/observe operations.
- Support and operations staff who need deterministic JSON output, diagnostics, exports, and audit verification.
- Product teams that distribute additional `sno-<name>` executables without modifying the root CLI.
- Release owners who publish GitHub, crates.io, npm, and optionally PyPI artifacts.

## Goals

### PRD-GOAL-1 — Canonical Rust CLI

Create crate `sno` version `0.1.0` with binary `sno`, built with Rust and `clap`. `sno --version` and `sno --help` must work from a clean installation.

### PRD-GOAL-2 — Functional Account and Station Namespaces

Move the current operator commands without retaining the old top-level command path:

| Legacy command | Canonical command |
|---|---|
| `nodix consent get` | `sno station telemetry consent get` |
| `nodix consent set <off|metadata-only|full>` | `sno station telemetry consent set <off|metadata-only|full>` |
| `nodix observe pause` | `sno station telemetry pause` |
| `nodix observe resume` | `sno station telemetry resume` |
| `nodix observe export ...` | `sno station telemetry export ...` |
| `nodix register` | `sno account machine register` |
| `nodix claim` | `sno account machine claim` |
| `nodix audit verify <event_id>` | `sno station audit verify <event_id>` |
| `nodix doctor` | `sno station doctor` |

The Rust implementation must preserve supported flags, JSON schemas, exit codes, local state behavior, security rules, and production API behavior unless this PRD explicitly changes them.

### PRD-GOAL-3 — External Subcommands

When no built-in or reserved top-level command matches, `sno <name> [args...]` must execute `sno-<name> [args...]` through direct process execution and `PATH` lookup. It must not invoke a shell. Arguments and the child exit result must be preserved. Missing executables must produce a usage error. The retired top-level nouns `consent`, `observe`, `register`, `claim`, `audit`, and `doctor` are permanently reserved: they must return a usage error before external lookup even when matching `sno-<name>` executables exist on `PATH`.

### PRD-GOAL-4 — Product Noun Skeleton

Expose built-in noun commands `sno account`, `sno station`, and `sno starport`. `sno account machine` contains registration and claim. `sno station telemetry`, `sno station audit`, and `sno station doctor` contain the migrated local workflows. `sno starport` may expose help and an explicit “no verbs released yet” result; it must not claim unavailable product behavior.

### PRD-GOAL-5 — Real Distribution

Prepare and publish real packages only:

- crates.io: `sno` `0.1.0`, owned by the company account `SnoInfo`.
- npm: `sno-ai` `0.1.0`, with `bin.sno` selecting an installed platform-specific package that contains the real compiled binary. Platform packages are allowed only when they contain a tested binary for their declared target.
- PyPI: `sno` `0.1.0`, using platform wheels that contain or install the same tested binary. This item may be explicitly deferred if real wheels are not ready; an empty Python console script is forbidden.

The first-release prebuilt operating-system and architecture families are frozen to Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64. Exact package tuples remain provisional until target-specific artifact inspection and clean-install evidence proves each compatibility floor:

| Operating system | Architecture | Rust target candidate | npm package candidate | PyPI tag candidate |
|---|---|---|---|---|
| Linux | x64 | `x86_64-unknown-linux-gnu` | `sno-ai-linux-x64` | `manylinux_2_17_x86_64` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `sno-ai-linux-arm64` | `manylinux_2_17_aarch64` |
| macOS | Intel | `x86_64-apple-darwin` | `sno-ai-darwin-x64` | `macosx_11_0_x86_64` |
| macOS | Apple Silicon | `aarch64-apple-darwin` | `sno-ai-darwin-arm64` | `macosx_11_0_arm64` |
| Windows | x64 | `x86_64-pc-windows-msvc` | `sno-ai-win32-x64` | `win_amd64` |

Linux musl/Alpine and Windows ARM64 prebuilt packages are not part of version `0.1.0`. Root wrappers must fail closed on unsupported platforms. Cargo source installation may work on additional Rust targets, but those targets are not release claims. A candidate tuple becomes a release claim only after a matching-runner build, artifact inspection, clean installation, and help/version/local-station smoke at the declared minimum platform; otherwise that family is explicitly deferred rather than relabeled with a weaker untested tag.

### PRD-GOAL-6 — Repository and Public-Release Readiness

Create private repository `github.com/sno-ai/sno-cli`, Apache-2.0 license, English README and contributing guide, repository metadata, continuous integration, release workflow stubs, and naming guardrails. The repository may become public later without changing package behavior.

### PRD-GOAL-7 — Clean Migration

After the Rust implementation passes parity and production-shaped checks, retire the old TypeScript CLI application and its CLI-only tests in the source repository in the same migration workstream. Update active user docs and live callers from `nodix ...` to `sno station ...`. Archived specifications remain immutable historical records.

## Protected Behavior

### PRD-AUTH-1 — Human Publish Authority

No crates.io publish occurs before the owner reviews the exact package contents and review report, then explicitly approves `cargo publish`. npm and PyPI publishes likewise require valid company credentials and a pre-publish package inspection.

### PRD-AUTH-2 — Local Identity Authority

Machine registration, machine claim, and audit verification use the local machine identity. Environment variables must not substitute an account token or another auth path. Claim remains an optional user-initiated browser approval flow.

### PRD-AUTH-3 — Source Repository Protection

The old repository is a source and migration target, not a disposable staging tree. Existing unpushed commits and unrelated changes must be preserved. Archived change artifacts are read-only.

### PRD-DATA-1 — Existing State Compatibility

The migrated Account and Station commands must read and write the current `~/.sno` state contract and documented overrides: `SNO_PROFILE_DIR`, `SNO_HOME`, `SNO_IDENTITY_PATH`, `SNO_BUFFER_PATH`, `SNO_CONSENT_PATH`, and `SNO_OBSERVE_BASE_URL`. Identity JSON, consent JSON, pause state, and SQLite buffer behavior must remain interoperable with the current TypeScript SDK.

### PRD-SEC-1 — Secret Containment

The CLI must never print or package a machine secret, machine-secret hash, registry token, npm token, PyPI token, or GitHub token. Identity files use owner-only permissions on Unix. Machine registration sends only the secret hash without authorization. Claim sends no authorization header: the device-code request carries the machine UUID and user CUID, and the device-token request carries the device code and device grant type. Audit verification alone sends the raw machine secret as bearer after registration.

### PRD-SEC-2 — Transport Security

Production API requests require HTTPS. Plain HTTP is permitted only for loopback fixture servers. External subcommands execute directly without a shell.

### PRD-CONTRACT-1 — Output and Exit Stability

Human mode uses stdout for successful output and stderr for errors. JSON mode writes one parseable JSON value to stdout and no human prose to stderr. Exit codes remain `0` success, `1` runtime failure, and `2` usage failure.

### PRD-CONTRACT-2 — Consent and Buffer Integrity

Consent transitions, pause/resume, exports, chain epochs, off-period retention, and shipped/terminal flags must match the existing behavior. Export must never mutate event shipment state.

### PRD-CONTRACT-3 — Cross-Language Contract Ownership

The Rust CLI becomes the only owner of end-user CLI routing. The TypeScript observability SDK remains required by Node products, so shared state and protocol behavior are owned by the derived OpenSpec contract rather than by either language implementation. Rust implements only the operations required by `sno station`; both implementations must pass the same compatibility fixtures. This release must not expose a second general-purpose observability SDK.

### PRD-CONTRACT-4 — Hash-Backed Legacy Baseline

Implementation must not begin until `scripts/verify-legacy-baseline.sh` proves that `LEGACY-CONTRACT-MATRIX-sno-cli-initial-release.md`, the 105-file `source-manifest.sha256`, `obligations.json`, and `cli-goldens.json` cover every migrated command, flag, default, success output, failure output, exit code, state read/write, HTTP request, status mapping, timeout, and retry rule. Each matrix row maps to exact source owners, named legacy tests, and named golden cases. Any per-file hash change fails verification and identifies the changed file; the obligation map identifies every row requiring review.

### PRD-AUTH-4 — Release Access Before Each Distribution Stage

Rust and crates.io implementation requires non-secret evidence that GitHub repository creation is authorized and a newly generated crates.io token for company account `SnoInfo` has been accepted by Cargo. npm packaging cannot begin until npm authentication resolves to the approved company account with publish access for the root and platform package names. PyPI packaging cannot begin until it is explicitly deferred or authenticated under the approved company account with permission to publish `sno`. Tokens must never be copied into project files, logs, command arguments captured as evidence, or CI artifacts.

## Non-Goals

- `PRD-NG-1`: No old `nodix` binary or top-level aliases in the new repository.
- `PRD-NG-2`: No compatibility wrapper, feature flag, or dual routing between old and new CLI implementations.
- `PRD-NG-3`: No full public Rust observability SDK in this release; only the internal support required by the migrated Account and Station commands is in scope.
- `PRD-NG-4`: No interactive setup wizard, memory installer, gateway configuration, scale configuration, or broad product orchestration from the obsolete draft PRD.
- `PRD-NG-5`: No `sno watch` command and no `.snorc` configuration file.
- `PRD-NG-6`: No publication of `sno-station` or `sno-starport` placeholder crates or packages.
- `PRD-NG-7`: Do not touch the disputed npm package name `sno`.
- `PRD-NG-8`: Do not alter previously published company packages named in the owner prompt.
- `PRD-NG-9`: No source-hiding claim. crates.io publication exposes the source included in the crate archive.
- `PRD-NG-10`: No prebuilt Linux musl/Alpine or Windows ARM64 package in version `0.1.0`.

## Workflow

1. User installs `sno` through Cargo, npm, PyPI, or a GitHub release artifact.
2. User runs `sno --help` or `sno --version`.
3. User runs a migrated workflow under `sno account machine ...` or `sno station ...`; the CLI uses compatible local state and production APIs.
4. User runs `sno <name>` for an extension; the CLI resolves and executes `sno-<name>` from `PATH`.
5. Release owner inspects package contents, dry-run output, test evidence, and review findings before authorizing each registry publish.

All routing, parsing, validation, retries, status handling, schema checks, package selection, and process exit propagation are deterministic code. No model is part of runtime behavior.

## Failure Signals

- `PRD-FAIL-1`: Any legacy operator command is missing, silently weakened, or exposed outside its approved Account or Station command group.
- `PRD-FAIL-2`: A Rust command corrupts or cannot read TypeScript-generated identity, consent, pause, or buffer state.
- `PRD-FAIL-3`: JSON mode emits prose, multiple values, or secret material.
- `PRD-FAIL-4`: A registry package installs without a runnable `sno` binary for its declared platform.
- `PRD-FAIL-5`: External subcommand dispatch invokes a shell, changes arguments, or masks the child failure.
- `PRD-FAIL-6`: A package is published before its required review and owner approval.
- `PRD-FAIL-7`: The old CLI remains an active parallel implementation after the migration is accepted.
- `PRD-FAIL-8`: The crate archive contains unintended files or secrets.
- `PRD-FAIL-9`: Rust and TypeScript independently evolve shared state or protocol behavior without updating and testing the common contract.
- `PRD-FAIL-10`: A retired top-level noun reaches external-subcommand lookup or executes a colliding `sno-<name>` program.

## Test and Evidence Strategy

- `TEST-1`: CLI parser and help snapshot tests cover the complete built-in command tree, missing arguments, invalid values, and exit codes.
- `TEST-2`: External-subcommand integration tests create temporary `sno-<name>` executables and prove argument, stdout, stderr, and exit propagation without a shell.
- `TEST-3`: Cross-language compatibility tests generate identity, consent, pause, and SQLite fixtures with the current TypeScript SDK, then exercise them with Rust. Rust-generated state is read back by the TypeScript SDK.
- `TEST-4`: Registration, claim, and audit tests use the real local HTTP fixture server and assert request paths, bodies, bearer placement, HTTPS rejection, persisted state, and secret absence.
- `TEST-5`: Export tests assert JSONL, CSV, and tarball content, manifest hashes, empty buffers, and unchanged shipment flags.
- `TEST-6`: Package tests inspect `cargo package --list`, unpack the `.crate`, install it into a clean temporary root, and run help/version plus one local migrated workflow.
- `TEST-7`: Platform-package tests install each built npm package in its declared operating-system/architecture runner and execute the real binary. The root npm wrapper must fail closed on unsupported platforms.
- `TEST-8`: PyPI wheel tests, when in scope, install each wheel in a clean environment and execute the same real binary checks.
- `TEST-9`: A production-shaped pre-publish smoke uses the normal binary against the canonical production service for anonymous registration and diagnostics without printing secrets. Audit verification runs only when a real event is available.
- `TEST-10`: Migration negative searches prove no active `nodix` CLI invocation, old app, or old package remains outside immutable archives and explicitly unrelated Nodix product names.
- `TEST-11`: `scripts/check-test-substitutes.sh` enforces the versioned `policy/test-substitutes.json` inventory across dependency manifests and test sources. `scripts/test-test-substitute-policy.sh` must prove that seeded mocking dependencies, internal-module replacement, and undeclared service replacement fail while only the declared real loopback HTTP server passes.
- `TEST-12`: Every approved platform family must freeze an exact tuple only after its binary target, npm platform package, and PyPI wheel when PyPI is in scope pass artifact inspection, clean-install help/version, and one local `sno station` smoke on a runner at the declared minimum platform.

Mocks are not permitted for in-repository code. The allowed test substitute is a real loopback HTTP server standing in for the external production service; the production smoke separately validates the real service path. The repository must include a deterministic CI policy check that fails when a mocking dependency, internal-module replacement, or unreviewed test substitute is introduced.

## Rollout, Monitoring, and Rollback

- The GitHub repository is created private and pushed only after required git verification and owner confirmation for the protected default branch.
- CI runs formatting, linting, tests, package inspection, and supported-target builds.
- Release artifacts are checksummed. Package wrappers select only artifacts produced for the same version.
- crates.io versions are permanent. A faulty release may be yanked and superseded but not overwritten or deleted.
- npm and PyPI releases must not be published until their platform installation smoke passes.
- The old TypeScript CLI remains active until replacement packages have been published, installed through their public registry paths on the frozen platform matrix, and passed the production-shaped smoke. Legacy deletion is merged only after that evidence exists. If publication fails before cutover, keep the old CLI active and do not merge its deletion.

## Naming Guardrails

- Code identifiers and package names use compound forms such as `sno_station`, `sno-station`, `sno_starport`, or `sno-starport`.
- The bare word `station` is allowed only as the user-facing command token in `sno station`; it is forbidden as a standalone code identifier or package name.
- The same rule applies to future product nouns: prefer explicit `sno-<noun>` identifiers.

## Implementation Directives

Always:

- Prefer one canonical implementation and remove the replaced CLI in the same migration workstream.
- Preserve real state and protocol behavior through compatibility fixtures and integration tests.
- Inspect exact package contents before every publish.
- Keep code, comments, docs, commit messages, and registry metadata in English.

Ask:

- Before every irreversible registry publish.
- Before changing a settled command, state, auth, or JSON contract.
- Before weakening or deferring any Release Green-Light criterion.

Never:

- Publish an empty package.
- Add old command aliases or compatibility shims.
- Copy the TypeScript implementation without retiring its CLI ownership and adding parity tests.
- Shell-expand external-subcommand arguments.
- expose secrets in output, logs, package archives, CI artifacts, or review evidence.

## Release Green-Light

Every item is `risky: true` and requires recorded evidence.

- `PRD-GL-1`: A clean install runs `sno --version`, `sno --help`, the full Account and Station command trees, and external `sno-<name>` dispatch on supported development platforms.
- `PRD-GL-2`: Every legacy operator workflow has behavior-level Rust coverage under its approved Account or Station command path, including JSON output and exit parity.
- `PRD-GL-3`: TypeScript-to-Rust and Rust-to-TypeScript state compatibility passes for identity, consent, pause state, and SQLite buffer/export data.
- `PRD-GL-4`: Security tests prove no secret output, HTTPS enforcement, direct external process execution, owner-only identity permissions, and package archive cleanliness.
- `PRD-GL-5`: GitHub repository `sno-ai/sno-cli` exists as private, CI is green, Apache-2.0 and repository metadata are present, and the naming guardrail is documented.
- `PRD-GL-6`: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, `cargo package --list`, and `cargo publish --dry-run` pass from a clean tree.
- `PRD-GL-7`: The exact `.crate` contents and final source review are presented to the owner; `cargo publish` runs only after explicit approval and publishes `sno` `0.1.0` under `SnoInfo`.
- `PRD-GL-8`: npm `sno-ai` `0.1.0` is published only after real platform packages install and run the compiled `sno` binary; npm authentication and two-factor requirements are satisfied.
- `PRD-GL-9`: PyPI `sno` `0.1.0` is either published as tested real-binary wheels or explicitly recorded as deferred; no empty package is uploaded.
- `PRD-GL-10`: The old TypeScript CLI is retired and active callers/docs are updated after parity passes; immutable archives remain unchanged.
- `PRD-GL-11`: The hash-backed legacy contract matrix and golden corpus cover every migrated behavior, and every row maps to a passing parity test.
- `PRD-GL-12`: The five approved operating-system/architecture families either freeze an exact proven tuple after target build, package construction, clean registry-style install, and executable smoke at the declared floor, or are explicitly deferred; no unsupported prebuilt platform is advertised.
- `PRD-GL-13`: Non-secret release-access evidence proves the approved GitHub and crates.io authority before Rust implementation, npm authority before npm packaging, and PyPI authority or explicit deferral before PyPI packaging.
- `PRD-GL-14`: The reviewed test-substitute inventory and deterministic CI policy check pass with the loopback external-service fixture as the only exception.
- `PRD-GL-15`: Replacement packages are publicly installable and pass production-shaped smoke before any legacy CLI deletion is merged.

## Stable ID Registry

| ID family | IDs | Coverage |
|---|---|---|
| Goals | `PRD-GOAL-1` through `PRD-GOAL-7` | `TEST-1` through `TEST-10`, release evidence |
| Authority | `PRD-AUTH-1` through `PRD-AUTH-4` | review receipts, git/publish approvals, non-secret account evidence |
| Data | `PRD-DATA-1` | `TEST-3`, `TEST-5` |
| Security | `PRD-SEC-1`, `PRD-SEC-2` | `TEST-2`, `TEST-4`, `TEST-6`, `TEST-9` |
| Contracts | `PRD-CONTRACT-1` through `PRD-CONTRACT-4` | `TEST-1`, `TEST-3`, `TEST-5`, `TEST-11` |
| Non-goals | `PRD-NG-1` through `PRD-NG-10` | negative searches and package inspection |
| Failure signals | `PRD-FAIL-1` through `PRD-FAIL-9` | `TEST-1` through `TEST-10` |
| Green-Light | `PRD-GL-1` through `PRD-GL-15` | `EVID-*` placeholders created during OpenSpec derivation |

## Release Packet

- Scenario: greenfield product plus cross-repository migration and multi-registry release.
- Source PRD: this file.
- Reference implementation: `nodix-private/apps/nodix-cli` at the recorded source commit.
- Reference contract: archived `add-sno-cli` OpenSpec artifacts; reference only, not authoritative naming or packaging.
- Required human gates: PRD release, crates.io publish approval, protected-branch push confirmation, and archive readiness.
- Phase-boundary commits: require standing owner authorization at PRD release.
- Next artifact after approval: derived OpenSpec change `sno-cli-initial-release` with hash-backed traceability.
