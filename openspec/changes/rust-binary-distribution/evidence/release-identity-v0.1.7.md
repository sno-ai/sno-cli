# Release Identity Receipt — `sno` 0.1.7

Status: released and independently verified

Published at: 2026-07-15T12:51:21Z

## Registry and Repository Identity

- Crate: `sno` 0.1.7
- crates.io publisher: `SnoInfo`
- crates.io state: not yanked
- GitHub repository: <https://github.com/sno-ai/sno-cli>
- GitHub repository visibility: public
- GitHub Release: <https://github.com/sno-ai/sno-cli/releases/tag/v0.1.7>
- GitHub release ID: `354435136`
- GitHub release state: published, non-prerelease, immutable
- Immutable releases setting: enabled
- Protected version-tag ruleset: `18984000`, active for `refs/tags/v*`, with organization-administrator bypass

## Source and Authorization Identity

- Reviewed source commit: `05c9f5b6e7e24b08347e9c605b8620eb80974ba9`
- Remote `main` commit at authorization: `05c9f5b6e7e24b08347e9c605b8620eb80974ba9`
- Annotated `v0.1.7` tag commit: `05c9f5b6e7e24b08347e9c605b8620eb80974ba9`
- Commit-bound `SNO_RELEASE_AUTHORIZED_SHA`: `05c9f5b6e7e24b08347e9c605b8620eb80974ba9`
- Successful eight-job CI run: <https://github.com/sno-ai/sno-cli/actions/runs/29415885098>
- Successful production Release run: <https://github.com/sno-ai/sno-cli/actions/runs/29416145988>
- Final adversarial review evidence: `evidence/reviews/code-review-release-recovery-round5-resolution.md`

## Crate Identity

- Local package archive SHA-256: `4af592f08dcd7a8551db252f1cf44cc52765b539197a9748f343daa45d5c13a4`
- Registry package archive SHA-256: `4af592f08dcd7a8551db252f1cf44cc52765b539197a9748f343daa45d5c13a4`
- Package contents: 14 files; Rust source, `Cargo.toml`, `Cargo.lock`, README, and Apache-2.0 license only
- `cargo publish --dry-run --locked`: passed
- Registry archive byte comparison: passed

## Target Archive Identity

| Supported target | Archive | SHA-256 |
| --- | --- | --- |
| macOS Apple Silicon | `sno-aarch64-apple-darwin.tar.gz` | `90ec0da15ebf80360f9478840c44607684cbab03d017c5105998053738b6baa7` |
| Linux ARM64 glibc | `sno-aarch64-unknown-linux-gnu.tar.gz` | `9ae0c68f0009ad5f146ee9a820e20b55b4962e73cbc2a9e5c0e23601946b1a3c` |
| Linux ARM64 musl | `sno-aarch64-unknown-linux-musl.tar.gz` | `66b94b40742a0fcb1015d3ce2757d9d379a9b97040477db3d61991b354e5dd64` |
| macOS Intel | `sno-x86_64-apple-darwin.tar.gz` | `54301c098caadd47f9b1b5ece6fdb465bb638660529c81c2b0cbbe498d71abcd` |
| Windows x64 | `sno-x86_64-pc-windows-msvc.zip` | `e696b916b341cbd9d3f566648282ad568cd52c8201188f52fc7f90998cd26b54` |
| Linux x64 glibc | `sno-x86_64-unknown-linux-gnu.tar.gz` | `8f0e02886ad9b9b8297bd85dcd918bfae9fbbe94c30a4795656da9d7f9276416` |
| Linux x64 musl | `sno-x86_64-unknown-linux-musl.tar.gz` | `56d27cc77dea8c94bacc774de8fb36d09ea47ed3089f53fce38e9ffa5782b79a` |

All seven public archives were downloaded anonymously from the immutable Release and passed `sha256.sum`. All seven also passed GitHub attestation verification constrained to repository `sno-ai/sno-cli`, signer workflow `.github/workflows/release.yml`, source digest `05c9f5b6e7e24b08347e9c605b8620eb80974ba9`, and source ref `refs/tags/v0.1.7`.

Additional published identities:

- Shell installer SHA-256: `4c7cc1b20a9b5581e3ecb0bc359683d2436684b4de6e9773603fb0e046dd6851`
- PowerShell installer SHA-256: `257a465762538b410eb4c9e13cc32dfc505a519f6715279500fd5035d86fdfaf`
- CycloneDX software bill of materials SHA-256: `3803ea006050f8fc8f064edff1ee2816b3d8d5db913ae9b8f4d0f9167bb205a6`
- Distribution manifest SHA-256: `7fa2e348401bac4e27e2631770deb14e5c50555b8d8ddbd5052ed8b9313ba989`

## Runtime Evidence

- Five native runner families passed tests, optimized build, architecture assertion, version, help, and fresh-profile Station consent execution.
- Two architecture-matched Alpine jobs executed the static Linux musl binaries.
- Seven extracted archives passed real-binary smoke tests.
- Five staged installers, five authenticated draft installers, five anonymous public-candidate installers, and five anonymous immutable-public installers passed.
- A separate anonymous Linux x64 Shell installer run installed `sno 0.1.7` and passed version, help, and `station telemetry consent get --json`.
- A separate clean `cargo install sno --version 0.1.7 --locked` run downloaded the registry crate, compiled it in 46.24 seconds, and passed the same runtime checks.

## Forward-Only History

Versions 0.1.1 through 0.1.6 remain valid, functional crates.io releases but have no matching public GitHub binary Release. Their protected tags remain fixed. The production workflow failed closed at a successively later gate for each version and deleted any mutable draft before public publication. No tag or immutable asset was replaced.
