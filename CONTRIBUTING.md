# Contributing to SNO CLI

## Scope

Keep `sno` as one canonical Rust CLI. Do not add legacy `nodix` command aliases, parallel TypeScript routing, placeholder packages, or speculative commands without an approved product contract.

## Naming

Code identifiers and package names always use compound SNO forms such as `sno_station`, `sno-station`, `sno_starport`, or `sno-starport`. The bare word `station` is allowed only as the user-facing token in `sno station`; it is forbidden as a standalone code identifier or package name. Apply the same rule to future product nouns.

## Development checks

GitHub Actions must not run builds, tests, or release jobs. Run the Linux gate on `ci-vm` before requesting review:

```sh
ssh ci-vm 'docker run --rm \
  -v /home/lh/code/sno-cli:/work \
  -w /work \
  rust:1.85-bookworm \
  bash -c "set -e; \
    apt-get update -qq; \
    apt-get install -y -qq jq ripgrep >/dev/null; \
    rustup component add rustfmt clippy >/dev/null; \
    cargo fmt --all --check; \
    cargo clippy --all-targets --all-features -- -D warnings; \
    cargo test --all-targets --all-features --locked; \
    cargo build --profile dist --locked; \
    scripts/check-test-substitutes.sh; \
    scripts/test-test-substitute-policy.sh; \
    cargo package --locked --list"'
```

Run the equivalent native build and CLI smoke checks on `labmba` for macOS Apple Silicon releases. WSL2 uses the Linux x86-64 release and follows [the WSL2 test method](ai-docs/testing/wsl2-test-method.md). GitHub remains the publication host; publish already-built archives with `gh release create` or `gh release upload`.

The expanded local commands are:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
scripts/verify-legacy-baseline.sh /home/lh/code/nodix-private
scripts/check-test-substitutes.sh
scripts/test-test-substitute-policy.sh
cargo package --list
cargo publish --dry-run
```

Tests use real files, real SQLite databases, real child processes, and the allowlisted loopback HTTP service. Do not replace in-repository modules with mocks.

## Command contracts

- Human success output goes to stdout.
- Human errors go to stderr.
- JSON mode emits exactly one JSON value on stdout and no prose on stderr, except machine claim, which emits the documented newline-delimited authorization and result/error records.
- Exit codes are `0` for success, `1` for runtime failure, and `2` for invalid usage.
- External subcommands execute directly without a shell.
- Production HTTP requires HTTPS.
- Credentials and machine secrets must never appear in output, logs, tests, fixtures, package archives, or commits.

## Publishing

Registry publication is irreversible. Inspect the exact package archive and pass `cargo publish --dry-run` before running an authorized `cargo publish`. Build and verify native binaries on owned hosts, then publish the verified archives with the GitHub CLI. Do not run builds, tests, or release jobs on GitHub-hosted runners. Do not add JavaScript or Python distribution wrappers.
