## Why

The published Rust crate is usable through `cargo install`, but the repository does not yet produce downloadable, target-specific binaries. SNO needs a Rust-only release path that ordinary users can install without Rust, Node.js, or Python while preserving verifiable provenance and platform-specific runtime evidence.

## What Changes

- Add production GitHub Release automation for five native targets: Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64.
- Add static Linux musl x64/ARM64 artifacts for Alpine and portable Linux environments.
- Generate Shell and PowerShell installers, archive checksums, Cargo Binstall metadata, and GitHub artifact attestations.
- Require each supported target to run the real binary and a local Station smoke before its artifact can ship.
- **BREAKING**: Remove npm and Python/PyPI distribution from the active product contract. GitHub Releases and crates.io become the only release channels.
- Keep Windows ARM64 outside the supported matrix until its hosted runner is generally available and the binary passes native runtime tests.

## Capabilities

### New Capabilities

- `rust-binary-distribution`: Defines supported targets, release artifacts, installers, integrity metadata, provenance, and runtime verification.

### Modified Capabilities

None. No existing OpenSpec capability is present in this repository.

## Impact

- Adds a pinned `cargo-dist` configuration and a generated-then-security-hardened GitHub release workflow.
- Changes the active release PRD, README, and contributor release instructions.
- Adds release-only CI work across Linux, macOS, and Windows runners.
- Does not change the `sno` command tree, local state, network protocol, or crates.io package behavior.
