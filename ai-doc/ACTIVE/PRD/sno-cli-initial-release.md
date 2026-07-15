# PRD: SNO CLI Initial Release

Status: Implementing
Date: 2026-07-15 PDT
ADLC project ID: `sno-cli-initial-release`
Scope: Create the unified Rust `sno` CLI, migrate the legacy Nodix operator commands into the approved Account and Station command groups, and release only real, usable packages.

## Decision Summary

SNO ships one Rust binary named `sno`. Version `0.1.0` established the functional crate rather than a placeholder: it includes the existing Nodix identity workflows under `sno account machine ...`, its local telemetry workflows under `sno station ...`, top-level help/version behavior, a `sno starport` noun scaffold, and Git-style external subcommand dispatch for executables named `sno-<name>` on `PATH`. Versions `0.1.1`, `0.1.2`, and `0.1.3` preserved that implementation on crates.io, but their GitHub workflows failed closed before artifact hosting. Version `0.1.4` is the forward-only synchronized production-distribution release that adds verified native artifacts without changing the command contract.

The GitHub repository is public before downloadable binaries are released. Publishing to crates.io is allowed only after package inspection, dry-run success, and full review. The owner has authorized forward-only publication through `0.1.4` after those gates pass. The Rust source included in the `.crate` archive is public regardless of GitHub repository visibility.

## Problem

SNO needs one memorable CLI entry point. The current operator CLI is a private TypeScript application named `nodix` inside another repository. Shipping a new empty Rust binary would reserve a name but discard working behavior; keeping the TypeScript CLI beside the new Rust CLI would create two command authorities and permanent drift.

The release must therefore solve three problems together:

1. Establish the canonical `sno` binary and repository.
2. Move the existing operator behavior into the new two-level command namespace.
3. Publish only Rust source or native artifacts that install and run the real binary.

## Source-Grounded Current State

The source snapshot is `nodix-private` commit `4256aa66aae2dc95edc71f788b456874a789b360` on branch `dev`. The checkout is three commits ahead of its remote; this project must not overwrite or discard those changes.

- `nodix-private/apps/nodix-cli/package.json` defines private package `@snoai/nodix` version `0.9.74` and binary `nodix`.
- `nodix-private/apps/nodix-cli/src/index.ts` exposes `consent`, `observe`, `register`, `claim`, `audit`, and `doctor` as top-level commands.
- Each command delegates to `@snoai/sno-observe`; the command layer does not own identity generation, SQLite buffering, consent transitions, exports, machine registration, device claim, or audit verification.
- The user-visible command layer is about 800 source lines. The directly required SDK implementation spans identity, consent, buffer, export, diagnostics, registration, claim, and audit code; a correct port is not a parser-only translation.
- The archived May 2026 CLI specification describes the existing behavior but is reference material only. Its old top-level naming and Node-only packaging decisions are superseded by this PRD.
- The draft “Nodix CLI as the Unified Local AI Setup Entry” is not authoritative. Its memory/gateway/scale wizard remains future product work.
- Current release state on 2026-07-15 PDT: crates.io packages `sno` `0.1.0` through `0.1.3` are public under `SnoInfo`; GitHub repository `sno-ai/sno-cli` is public with immutable releases enabled; no GitHub binary release exists yet. The `0.1.2` workflow failed closed during cross-platform bootstrap, and the `0.1.3` workflow failed closed while parsing valid checksum files with a trailing blank line; both stopped before hosting assets and their tags remain fixed.

Probe evidence is recorded in `ai-doc/ACTIVE/PRD/PROBE-RESULTS-sno-cli-initial-release.md`.

## Target Users and Operators

- End users who install one SNO CLI and need local account/observe operations.
- Support and operations staff who need deterministic JSON output, diagnostics, exports, and audit verification.
- Product teams that distribute additional `sno-<name>` executables without modifying the root CLI.
- Release owners who publish crates.io source packages and immutable GitHub native-binary releases.

## Goals

### PRD-GOAL-1 — Canonical Rust CLI

Maintain crate `sno` with binary `sno`, built with Rust and `clap`. `sno --version` and `sno --help` must work from a clean installation. Release `0.1.4` synchronizes crates.io source, the Git tag, and GitHub binary assets.

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

Prepare and publish only the real Rust implementation:

- crates.io: `sno` `0.1.4`, owned by company account `SnoInfo`; `0.1.0` through the registry-only `0.1.3` remain valid published predecessors.
- GitHub Releases: native archives, Shell and PowerShell installers, Cargo Binstall metadata, SHA-256 checksums, and available GitHub artifact attestations.

The five formally supported operating-system and architecture families are Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64. Linux additionally ships static musl variants for x64 and ARM64, producing seven target archives:

| Support class | Operating system | Architecture | Rust target |
|---|---|---|---|
| Native | Linux | x64 | `x86_64-unknown-linux-gnu` |
| Native | Linux | ARM64 | `aarch64-unknown-linux-gnu` |
| Portable Linux | Linux musl/Alpine | x64 | `x86_64-unknown-linux-musl` |
| Portable Linux | Linux musl/Alpine | ARM64 | `aarch64-unknown-linux-musl` |
| Native | macOS | Intel | `x86_64-apple-darwin` |
| Native | macOS | Apple Silicon | `aarch64-apple-darwin` |
| Native | Windows | x64 | `x86_64-pc-windows-msvc` |

Each target becomes a release claim only after a matching-architecture build, real-binary execution, archive extraction, and help/version/local-Station smoke. Musl artifacts also execute inside a pinned Alpine image. Final Shell and PowerShell installers must install the exact local staged archives, repeat after the same bytes are downloaded from a GitHub draft, repeat anonymously through a one-use public candidate containing those same bytes, and repeat from the immutable final public URLs before the release is declared green. Windows ARM64 remains unsupported while its standard GitHub-hosted runner is public preview and outside the service-level guarantee; cross-compilation alone cannot promote a target.

### PRD-GOAL-6 — Repository and Public-Release Readiness

Create repository `github.com/sno-ai/sno-cli`, Apache-2.0 license, English README and contributing guide, repository metadata, continuous integration, production release automation, and naming guardrails. The repository becomes public, release immutability is enabled, and organization-administrator-only version-tag protection is active before the first public GitHub binary release.

### PRD-GOAL-7 — Clean Migration

After the Rust implementation passes parity and production-shaped checks, retire the old TypeScript CLI application and its CLI-only tests in the source repository in the same migration workstream. Update active user docs and live callers from `nodix ...` to `sno station ...`. Archived specifications remain immutable historical records.

## Protected Behavior

### PRD-AUTH-1 — Human Publish Authority

No crates.io publish occurs before the exact package contents, review report, and required gates are inspected. The owner has explicitly authorized forward-only publication through `0.1.4` after those gates pass; a new approval is required only if the package scope or settled release contract changes. GitHub creates only a mutable draft after local archive and installer checks; it publishes and freezes that draft only after the GitHub-downloaded assets pass, and the release is not declared green until anonymous public Shell and PowerShell checks also pass.

### PRD-AUTH-2 — Local Identity Authority

Machine registration, machine claim, and audit verification use the local machine identity. Environment variables must not substitute an account token or another auth path. Claim remains an optional user-initiated browser approval flow. Claim persistence is non-overwriting: the first account binding wins, repeating the same binding is idempotent, and a different returned account fails with `claim_account_conflict`.

### PRD-AUTH-3 — Source Repository Protection

The old repository is a source and migration target, not a disposable staging tree. Existing unpushed commits and unrelated changes must be preserved. Archived change artifacts are read-only.

### PRD-DATA-1 — Existing State Compatibility

The migrated Account and Station commands must read and write the current `~/.sno` state contract and documented overrides: `SNO_PROFILE_DIR`, `SNO_HOME`, `SNO_IDENTITY_PATH`, `SNO_BUFFER_PATH`, `SNO_CONSENT_PATH`, and `SNO_OBSERVE_BASE_URL`. Identity JSON, consent JSON, pause state, and SQLite buffer behavior must remain interoperable with the current TypeScript SDK.

### PRD-SEC-1 — Secret Containment

The CLI must never print or package a machine secret, machine-secret hash, registry token, or GitHub token. Identity files use owner-only permissions on Unix. Machine registration sends only the secret hash without authorization. Claim sends no authorization header: the device-code request carries the machine UUID and user CUID, and the device-token request carries the device code and device grant type. Audit verification alone sends the raw machine secret as bearer after registration.

### PRD-SEC-2 — Transport Security

Production API requests require HTTPS. Plain HTTP is permitted only for loopback fixture servers. External subcommands execute directly without a shell.

### PRD-CONTRACT-1 — Output and Exit Stability

Human mode uses stdout for successful output and stderr for errors. JSON mode writes one parseable JSON value to stdout and no human prose to stderr, except `sno account machine claim`: it writes one newline-delimited `authorization` JSON record before polling, then one `result` or `error` record. This exception is required because browser approval cannot begin until the caller receives the device code. Exit codes remain `0` success, `1` runtime failure, and `2` usage failure.

### PRD-CONTRACT-2 — Consent and Buffer Integrity

Consent transitions, pause/resume, exports, chain epochs, off-period retention, and shipped/terminal flags must match the existing behavior except that resume fails closed when no saved pause exists; it must never reverse an explicit opt-out. A consent-specific operating-system lock covers recovery, state read, the immediate SQLite write transaction for all audit-chain events, the durable transition journal, consent-file replacement, and journal cleanup; lock files persist as inert coordination files and are never stolen by age. Agent discovery is scoped to the current machine identity so a retained buffer cannot import agents from a prior identity. Unix state replacement synchronizes the containing directory before success. Existing parent directories selected by environment overrides retain their permissions. Export uses a consistent SQLite read transaction and streams data without retaining the full event buffer or spooling it to temporary storage; tarball export makes two passes over the same snapshot. JSONL contains exactly one compact JSON value per line and export never mutates shipment state. JSON metadata mode requires an output path for JSONL and CSV so successful status output can never replace the requested data.

### PRD-CONTRACT-3 — Cross-Language Contract Ownership

The Rust CLI becomes the only owner of end-user CLI routing. The TypeScript observability SDK remains required by Node products, so shared state and protocol behavior are owned by the derived OpenSpec contract rather than by either language implementation. Rust implements only the operations required by `sno station`; both implementations must pass the same compatibility fixtures. This release must not expose a second general-purpose observability SDK.

### PRD-CONTRACT-4 — Hash-Backed Legacy Baseline

Implementation must not begin until `scripts/verify-legacy-baseline.sh` proves that `LEGACY-CONTRACT-MATRIX-sno-cli-initial-release.md`, the 105-file `source-manifest.sha256`, `obligations.json`, and `cli-goldens.json` cover every migrated command, flag, default, success output, failure output, exit code, state read/write, HTTP request, status mapping, timeout, and retry rule. Each matrix row maps to exact source owners, named legacy tests, and named golden cases. Any per-file hash change fails verification and identifies the changed file; the obligation map identifies every row requiring review.

### PRD-AUTH-4 — Release Access Before Each Distribution Stage

Rust and crates.io implementation requires non-secret evidence that GitHub repository creation is authorized and a crates.io token for company account `SnoInfo` has been accepted by Cargo. GitHub binary publication requires repository release permission and a passing tag-triggered workflow. Tokens must never be copied into project files, logs, command arguments captured as evidence, or CI artifacts.

## Non-Goals

- `PRD-NG-1`: No old `nodix` binary or top-level aliases in the new repository.
- `PRD-NG-2`: No compatibility wrapper, feature flag, or dual routing between old and new CLI implementations.
- `PRD-NG-3`: No full public Rust observability SDK in this release; only the internal support required by the migrated Account and Station commands is in scope.
- `PRD-NG-4`: No interactive setup wizard, memory installer, gateway configuration, scale configuration, or broad product orchestration from the obsolete draft PRD.
- `PRD-NG-5`: No `sno watch` command and no `.snorc` configuration file.
- `PRD-NG-6`: No publication of `sno-station` or `sno-starport` placeholder crates or packages.
- `PRD-NG-7`: No npm, PyPI, Node.js, or Python distribution wrapper; Rust is the sole CLI implementation.
- `PRD-NG-8`: Do not alter previously published company packages named in the owner prompt.
- `PRD-NG-9`: No source-hiding claim. crates.io publication exposes the source included in the crate archive.
- `PRD-NG-10`: No Windows ARM64 or 32-bit prebuilt package until native production-grade runner evidence exists.

## Workflow

1. User installs `sno` through Cargo or a GitHub Release archive/installer.
2. User runs `sno --help` or `sno --version`.
3. User runs a migrated workflow under `sno account machine ...` or `sno station ...`; the CLI uses compatible local state and production APIs.
4. User runs `sno <name>` for an extension; the CLI resolves and executes `sno-<name>` from `PATH`.
5. Release owner inspects package contents, target runtime evidence, integrity metadata, and review findings before publication.

All routing, parsing, validation, retries, status handling, schema checks, package selection, and process exit propagation are deterministic code. No model is part of runtime behavior.

## Failure Signals

- `PRD-FAIL-1`: Any legacy operator command is missing, silently weakened, or exposed outside its approved Account or Station command group.
- `PRD-FAIL-2`: A Rust command corrupts or cannot read TypeScript-generated identity, consent, pause, or buffer state.
- `PRD-FAIL-3`: JSON mode emits prose, multiple values, or secret material.
- `PRD-FAIL-4`: A GitHub artifact or installer lacks a runnable `sno` binary for its declared target.
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
- `TEST-7`: Matching-architecture release jobs execute each built binary, extract its archive, and repeat version/help/local-Station smoke using real filesystem and SQLite state.
- `TEST-8`: Shell and PowerShell installer tests install into clean temporary prefixes, execute the installed binary, and fail closed on unsupported platforms.
- `TEST-9`: A production-shaped pre-publish smoke uses the normal binary against the canonical production service for anonymous registration and diagnostics without printing secrets. Audit verification runs only when a real event is available.
- `TEST-10`: Migration negative searches prove no active `nodix` CLI invocation, old app, or old package remains outside immutable archives and explicitly unrelated Nodix product names.
- `TEST-11`: `scripts/check-test-substitutes.sh` enforces the versioned `policy/test-substitutes.json` inventory across dependency manifests and test sources. `scripts/test-test-substitute-policy.sh` must prove that seeded mocking dependencies, internal-module replacement, and undeclared service replacement fail while only the declared real loopback HTTP server passes.
- `TEST-12`: Every approved target must pass target build, archive inspection, clean extraction, help/version, and one local `sno station` smoke on the matching runner; musl targets additionally execute in pinned Alpine.

Mocks are not permitted for in-repository code. The allowed test substitute is a real loopback HTTP server standing in for the external production service; the production smoke separately validates the real service path. The repository must include a deterministic CI policy check that fails when a mocking dependency, internal-module replacement, or unreviewed test substitute is introduced.

## Rollout, Monitoring, and Rollback

- The GitHub repository is created private and pushed only after required git verification and owner confirmation for the protected default branch.
- CI runs formatting, linting, tests, package inspection, and supported-target builds.
- Release artifacts are checksummed and carry build provenance when GitHub attestations are available. Installers select only artifacts produced for the exact version and host target.
- crates.io versions are permanent. A faulty release may be yanked and superseded but not overwritten or deleted.
- GitHub release tags and assets are immutable. A faulty release is superseded by a new version and never replaced in place.
- The old TypeScript CLI remains active until the Rust crate and public GitHub artifacts have passed the production-shaped smoke. Legacy deletion is merged only after that evidence exists. If publication fails before cutover, keep the old CLI active and do not merge its deletion.

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

- Before publishing if the reviewed package scope or settled release contract changes; forward-only `0.1.4` is already authorized after its gates pass.
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
- `PRD-GL-5`: GitHub repository `sno-ai/sno-cli` is public before binary publication, CI is green, release immutability is enabled, Apache-2.0 and repository metadata are present, and the naming guardrail is documented.
- `PRD-GL-6`: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, `cargo package --list`, and `cargo publish --dry-run` pass from a clean tree.
- `PRD-GL-7`: The exact `.crate` contents and final source review are presented to the owner; the synchronized release publishes `sno` `0.1.4` under `SnoInfo` and tags the same reviewed source version.
- `PRD-GL-8`: Seven GitHub target archives are published only after native build, real-binary execution, clean extraction, and local-Station smoke; musl assets additionally pass pinned-Alpine execution.
- `PRD-GL-9`: Shell and PowerShell installers, Cargo Binstall metadata, SHA-256 checksums, and available GitHub artifact attestations match the exact released version and assets.
- `PRD-GL-10`: The old TypeScript CLI is retired and active callers/docs are updated after parity passes; immutable archives remain unchanged.
- `PRD-GL-11`: The hash-backed legacy contract matrix and golden corpus cover every migrated behavior, and every row maps to a passing parity test.
- `PRD-GL-12`: The five native platform families and two Linux musl targets freeze exact tuples only after matching-runner build, archive construction, clean extraction, and executable smoke; no unsupported prebuilt platform is advertised.
- `PRD-GL-13`: Non-secret release-access evidence proves approved GitHub and crates.io authority before publication.
- `PRD-GL-14`: The reviewed test-substitute inventory and deterministic CI policy check pass with the loopback external-service fixture as the only exception.
- `PRD-GL-15`: The crates.io package and GitHub native artifacts are publicly installable and pass production-shaped smoke before any legacy CLI deletion is merged.

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

- Scenario: greenfield product plus cross-repository migration and Rust-native release.
- Source PRD: this file.
- Reference implementation: `nodix-private/apps/nodix-cli` at the recorded source commit.
- Reference contract: archived `add-sno-cli` OpenSpec artifacts; reference only, not authoritative naming or packaging.
- Required human gates: PRD release, crates.io publish approval, and archive readiness. The owner has granted standing authorization for normal commits and pushes.
- Phase-boundary commits: require standing owner authorization at PRD release.
- Next artifact after approval: derived OpenSpec change `sno-cli-initial-release` with hash-backed traceability.
