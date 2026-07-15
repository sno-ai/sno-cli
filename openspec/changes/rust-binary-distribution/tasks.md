## 1. Contract Hard Cut

- [x] 1.0 Clean-install public `sno` 0.1.0 into a temporary Cargo root and record version, command tree, and fresh-profile Station output as the Rust runtime baseline.
- [x] 1.1 Remove npm and Python/PyPI distribution requirements from the active PRD, probe evidence, README, and contributor instructions.
- [x] 1.2 Document the five native targets, two Linux musl targets, unsupported-target promotion rule, and Rust-only installation paths.
- [x] 1.3 Add a versioned manifest of active workflows, package manifests, installer configuration, and release docs; make CI reject prohibited npm or Python publish actions and wrapper definitions within that explicit scope.
- [x] 1.4 Discover candidate release surfaces from `git ls-files` and fail when any candidate is absent from the manifest or excluded without a non-empty reason.

## 2. Release Configuration

- [x] 2.1 Add `dist-workspace.toml` pinned to `cargo-dist` 0.32.0 with exactly seven targets, Shell and PowerShell installers, SHA-256 checksums, and GitHub attestations.
- [x] 2.2 Increment the crate to `0.1.1`, generate and commit the GitHub Release workflow, and require semantic version tags matching `Cargo.toml` plus locked Cargo builds.
- [x] 2.3 Expand CI to execute tests on Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64; pin native runner labels, assert observed host architecture against the declared target, and validate release configuration drift.
- [x] 2.4 Delete the superseded hand-written release dry-run workflow; keep one generated-then-security-hardened workflow as the sole GitHub artifact authority.
- [x] 2.5 Push the non-publishing runner probe first and retain successful architecture plus real-binary outputs for every final runner label before freezing the release matrix.

## 3. Real Artifact Verification

- [x] 3.1 Make every release build run `--version`, `--help`, and a fresh-profile local Station workflow on its matching native runner.
- [x] 3.2 Make Linux musl builds execute in a pinned Alpine container for the same architecture.
- [x] 3.3 Extract every generated archive into a clean directory and repeat the real-binary smoke before upload.
- [x] 3.4 Verify generated Shell and PowerShell installers against local staged archives, GitHub-downloaded draft assets, and immutable public URLs; do not issue the release-identity receipt until all three stages pass.

## 4. Quality and Publication

- [ ] 4.1 Run format, clippy, all tests, legacy baseline, substitute policy, package inspection, crates.io dry-run, and OpenSpec validation.
- [x] 4.2 Run final test-quality, agentic-debt, and Codex adversarial reviews; fix all material findings and rerun affected gates.
- [x] 4.3 Push `main`, wait for every native CI job to pass, then make the repository public and enable immutable releases; capture both GitHub API results as the pre-tag receipt.
- [ ] 4.4 Protect `refs/tags/v*` with an active organization-administrator-only creation/update/deletion ruleset; require a commit-bound `SNO_RELEASE_AUTHORIZED_SHA` receipt after live repository, immutability, ruleset, remote-main, and CI checks; verify anonymous installers against a one-use public candidate before final publication; and clean up failed candidate or final publication attempts.
- [x] 4.5 From a clean checkout of the reviewed commit, publish crate `0.1.1`, download the registry archive, and require byte-identical SHA-256 against the local package archive.
- [ ] 4.6 Preserve the failed `v0.1.1` through `v0.1.5` attempts without moving their tags, publish the forward-only `0.1.6` crate from the repaired reviewed commit, then verify all seven archives, installers, checksums, manifests, and available attestations from the immutable `v0.1.6` GitHub Release.
- [ ] 4.7 Record reviewed commit, tag commit, local and registry crate hashes, target archive hashes, and workflow run in one release-identity receipt.

## Test Design Gate

- Proof type: real native runner, real extracted archive, real filesystem/SQLite Station flow, and real installer execution.
- Observable assertions: executable reports the crate version, help exposes Account/Station/Starport, local consent state persists and reads back, archive checksum recomputes, and unsupported hosts fail closed.
- Mock Inventory: empty. No fake platform, mocked process, substitute archive, or mocked installer is permitted.
- RED: current repository has no release workflow or downloadable assets, and current active docs still contain npm/PyPI requirements.
- GREEN: generated plan contains exactly seven targets and all native/archive/installer checks pass.
- REFACTOR: remove the obsolete hand-written dry-run workflow after generated release automation proves equivalent crate checks.
