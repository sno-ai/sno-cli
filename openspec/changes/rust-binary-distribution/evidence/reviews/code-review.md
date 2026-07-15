# Codex Adversarial Review — Round 1

Verdict: needs-attention

## Authoritative Findings

### Critical — Unverified release builder executes with repository-write permissions

- Trigger: Every tag release downloads a remote shell script and pipes it directly to `sh`; the workflow globally grants `contents: write`.
- Impact: A compromised upstream release asset or delivery path can execute arbitrary code with the release token, allowing malicious release publication or repository mutation.
- Recommendation: Download a versioned `cargo-dist` archive and verify it against a repository-pinned SHA-256 before extracting or executing it.

### High — Installer failures are discovered after the release is public

- Trigger: The workflow creates the GitHub Release, then installer smoke runs only after announcement.
- Impact: Fresh installs fail after immutable publication and require a new release.
- Recommendation: Run installer checks against staged release artifacts before public release creation.

### Medium — SBOM generator integrity check trusts a checksum from the same source

- Trigger: The workflow downloads both `cargo-cyclonedx` and its SHA-256 from the same upstream release.
- Impact: A compromised generator can forge the SBOM and its adjacent checksum.
- Recommendation: Pin the expected SHA-256 in the repository.

The second review wave approved the release-policy self-test and Rust CLI integration tests with no material findings.
