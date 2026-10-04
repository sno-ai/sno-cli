# AGENTS

## Repository purpose

This is the public binary-distribution and product-catalog repository for Sno CLI. Product source, private specifications, internal evidence, and build inputs remain private.

## Allowed tracked files

Keep tracked files limited to public documentation, Apache-2.0 license, these instructions, deterministic release automation, and public `products.json` recipes. Compiled archives and the canonical installer belong in GitHub Release assets, never commits. Never add Rust source, Cargo manifests/packages, private repository archives, internal paths, model transcripts, test logs, credentials, or private build configuration. GitHub-generated tag archives contain only this public distribution repository.

## Build boundary

The private source repository and owned build hosts compile and test all four targets. This repository never builds product source, checks it out in a workflow, or stores a private-repository token. The private repository owns the single canonical POSIX installer template and its release-data filler. Public automation receives the filler path, not a duplicate template.

## Release contract

The owner's 2026-10-01 independent CLI installation requirement replaces the historical Windows/musl contract. Every release contains exactly:

- `sno-aarch64-apple-darwin.tar.gz`
- `sno-aarch64-unknown-linux-gnu.tar.gz`
- `sno-x86_64-apple-darwin.tar.gz`
- `sno-x86_64-unknown-linux-gnu.tar.gz`
- `sno-installer.sh`

Support GNU Linux x86-64/ARM64, WSL2 x86-64 and macOS Intel/Apple silicon; no native Windows or musl Linux. No generated installer or checksum-sidecar requirement. Each archive contains only its target-named directory, `README.md`, `LICENSE`, and `sno`.

After archive upload, use actual GitHub native `digest` and the asset's final version-tag download URL to fill the canonical installer; translate a supplied temporary `untagged-*` draft URL before embedding it. It installs its own CLI release at `~/.local/bin/sno`, executes candidate `--version` before atomic replacement, and preserves a working binary when the candidate fails. Normal mode initializes only CLI state/stubs/scheduler and provides absolute-path core usage. `--update-only` changes only the binary. No automatic Station installation, prompt or registration merely because CLI exists.

## Publishing

Use only `scripts/release.sh`:

1. `check VERSION STAGING_DIR` checks four archives or the final five release assets locally.
2. `stage VERSION SOURCE_COMMIT STAGING_DIR FILL_SCRIPT` uploads four archives as a draft, finds that draft in the authenticated release list, reads native asset data, runs the private `FILL_SCRIPT VERSION RELEASE_JSON OUTPUT` with final download URLs, and uploads the populated canonical installer.
3. Private build hosts perform native installation checks. Draft access is authorized publisher access, not anonymous download proof.
4. `publish VERSION` downloads and verifies the five draft assets against native digests, then publishes. Stable/prerelease status follows the real CLI version.

Only stage or publish when the owner or authorized private release pipeline explicitly requests that external change. Never replace an existing release. Release notes may name the source commit but never publish source.

## Verification

Run `bash -n scripts/release.sh` and `shellcheck scripts/release.sh` for changes here. Check the focused release contract, including downloader failure, old executable preservation, profile repair and binary-only update. Actual public/native installation evidence remains separate from local fixture checks. Do not run a full unit or integration suite.
