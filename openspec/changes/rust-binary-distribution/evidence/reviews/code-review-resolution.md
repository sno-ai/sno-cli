# Codex Adversarial Review Resolution

## Round 1

- Critical: unverified `cargo-dist` bootstrap executed under a write-capable token. Fixed by repository-pinned SHA-256 values for all five host binaries, verified archive extraction, read-only root permissions, and write scope only on release finalization jobs.
- High: installer failures were detected only after publication. Fixed with staged installer execution before hosting, GitHub draft asset download and execution before immutable publication, and an anonymous public-path check after publication.
- Medium: the SBOM generator trusted a checksum downloaded beside the binary. Fixed by pinning the independently recorded `cargo-cyclonedx` 0.5.9 SHA-256 in the repository.

## Round 2

- High: the public installer path was post-publication only. Fixed by inserting a mutable draft phase, downloading the exact uploaded assets from GitHub on all five native runners, and publishing that same draft only after installer success. The anonymous path is repeated after publication because an anonymous public URL cannot exist before publication.
- High: macOS bootstrap used `sha256sum`. Fixed with a verified `shasum -a 256` fallback.
- High: job-level permission escalation was not rejected. Fixed with a per-job write-permission allowlist and mutation test.
- High: mutable action references could pass. Fixed by requiring every remote `uses:` reference to end in a 40-character commit SHA and adding a mutation test.
- High: installer gates could be satisfied by unrelated YAML text. Fixed by checking the exact plan, local-build, host, draft, publication, announcement, and public-smoke job blocks plus the actual shared installer verifier.
- High: excluded workflows skipped prohibited-publisher scanning. Fixed by always scanning workflow candidates and adding an excluded-workflow mutation test.

All authoritative findings were retained. No finding was dismissed as a false positive.
