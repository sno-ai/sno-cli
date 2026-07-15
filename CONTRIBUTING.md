# Contributing to SNO CLI

## Scope

Keep `sno` as one canonical Rust CLI. Do not add legacy `nodix` command aliases, parallel TypeScript routing, placeholder packages, or speculative commands without an approved product contract.

## Naming

Code identifiers and package names always use compound SNO forms such as `sno_station`, `sno-station`, `sno_starport`, or `sno-starport`. The bare word `station` is allowed only as the user-facing token in `sno station`; it is forbidden as a standalone code identifier or package name. Apply the same rule to future product nouns.

## Development checks

Run the complete local gate before requesting review:

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

Registry publication is irreversible. Inspect the exact package archive and pass `cargo publish --dry-run` before running an authorized `cargo publish`. Native binaries are released only through the hardened GitHub Release workflow: archives and local staged installers pass first, GitHub-downloaded draft assets pass before immutable publication, and anonymous public installers pass before the release is declared green. Do not add JavaScript or Python distribution wrappers.
