# Probe Results: SNO CLI Initial Release

Captured: 2026-07-14 PDT
Purpose: Ground the PRD release review in current repository and registry facts.

## Source Repository

Command:

```text
git -C /home/lh/code/nodix-private rev-parse HEAD
git -C /home/lh/code/nodix-private branch --show-current
git -C /home/lh/code/nodix-private status --short --branch
```

Output:

```text
4256aa66aae2dc95edc71f788b456874a789b360
dev
## dev...origin/dev [ahead 3]
```

## Legacy CLI Size and Command Surface

The application contains 22 supported files: 20 code files and 2 documents. Deterministic AST extraction found 121 nodes and 305 edges. The command source is approximately 800 lines; the directly relevant SDK implementation inventory is 3,631 lines before shared schemas, hashing, filesystem utilities, and flush logic.

Observed built-in commands from `apps/nodix-cli/src/index.ts` and command modules:

```text
consent get
consent set
observe pause
observe resume
observe export
register
claim
audit verify
doctor
```

The application imports `@snoai/sno-observe` for every operational behavior.

## Source Artifact Hashes

```text
07a6c4aa1ef5fccd5ea2729e54741db9f13504754539de90ada153d8893c52be  apps/nodix-cli/package.json
488250a0be8ce9792cc13229ecdb33dc32a5107304c2b5cd845169042d3a61bc  apps/nodix-cli/src/index.ts
3d975ff5b9ccec7e4d39bc67d522b19419ed113527c2dadbe4c6027bcc1b6d23  internal/ai-doc/ACTIVE/PRD/nexte-step/nodix-cli-entrypoint.md
9a54f2e595fa5bb54c8834eb545f69a946ae8d0449c97b61b36f2b619d478a9f  openspec/changes/archive/2026-05-02-add-sno-cli/specs/sno-cli/spec.md
```

## Registry and Repository Name Checks

Commands:

```text
cargo info sno
npm view sno-ai name version dist-tags --json
python -m pip index versions sno
gh repo view sno-ai/sno-cli --json name,visibility,url,defaultBranchRef
curl -sS -o /dev/null -w '%{http_code}' https://pypi.org/pypi/sno/json
curl -sS -o /dev/null -w '%{http_code}' -H 'User-Agent: sno-cli-name-check/0.1 contact=info@sno.ai' https://crates.io/api/v1/crates/sno
curl -sS -o /dev/null -w '%{http_code}' https://registry.npmjs.org/sno-ai
```

Observed results:

```text
crates.io: exact crate sno not found
npm: sno-ai returned E404 Not Found
PyPI: no matching distribution found for sno
GitHub: sno-ai/sno-cli could not be resolved
Direct registry APIs: crates.io 404, npm 404, PyPI 404
```

These are time-sensitive observations and must be repeated immediately before publishing or repository creation.

## Local Release Tool Readiness

```text
GitHub CLI: authenticated as LarHope with repo and workflow scopes
Cargo credentials file: present; owning crates.io account not yet proven
npm whoami: unauthorized
PyPI config: missing
Twine: missing
```

No secret value is stored in this repository or recorded in this evidence.

## Updated Release Access and Repository Evidence

Captured: 2026-07-15 PDT

```text
GitHub repository: https://github.com/sno-ai/sno-cli
GitHub visibility: PRIVATE
GitHub authenticated account: LarHope
Git remote: git@github.com:sno-ai/sno-cli.git
Cargo login: accepted a newly generated 365-day crates.io token supplied by the owner
Cargo credential primary: ~/.cargo/credentials.toml, mode 0600
Cargo credential backup: ~/.config/sno-cli/credentials/crates-io-credentials.toml, mode 0600
Cargo credential copies: byte-identical
npm authentication: unavailable (401 Unauthorized)
PyPI authentication: unavailable; packaging may be deferred under the PRD
```

The crates.io token API does not expose an account-identity endpoint to token authentication: `/api/v1/me` returned `403` with `this action can only be performed on the crates.io website`. The owner supplied the new token for company account `SnoInfo`; Cargo accepted and stored it. The final publish remains separately gated by exact package review and explicit owner approval.

The npm authentication gap blocks npm packaging and publication, not the Rust implementation or crates.io dry-run. npm work cannot start until the approved company identity is authenticated. PyPI remains explicitly deferrable rather than shipping an empty package.

## Legacy Contract Test Evidence

Command:

```text
npm test --workspace @snoai/nodix
```

Observed on 2026-07-15 PDT:

```text
tests: 43
passed: 42
failed: 0
skipped: 1
duration: 2.15 seconds
```

The skipped test is the opt-in production CLI end-to-end test because `SNO_CLI_PRODUCTION_E2E=1` was not set. All local command, state, real SQLite, export, identity, and loopback HTTP contract tests passed. The production-shaped smoke remains a pre-publish requirement.

## Mechanical Baseline and Policy Evidence

Commands:

```text
scripts/verify-legacy-baseline.sh /home/lh/code/nodix-private
scripts/test-test-substitute-policy.sh
```

Observed on 2026-07-15 PDT:

```text
legacy baseline verified at 4256aa66aae2dc95edc71f788b456874a789b360: 27 obligations, 57 golden cases, 105 source files; legacy tests passed
test-substitute policy self-test passed: 8 forbidden mutations rejected, allowlist and repository accepted
```

The source manifest contains one hash per file and includes shared hashing, canonical JSON, wire-envelope, flush, identity, filesystem, transport, buffer, export, diagnostics, device-claim, registration, audit, all legacy CLI/observability tests, common identifier sources, package manifests, and the root dependency lockfile. Every matrix row maps to source owners, named legacy tests, and named golden cases. The namespace validator requires `sno station` prefixing for every migrated behavior and requires negative top-level coverage for all six old command nouns.

The package families approved by the owner are Linux x64/ARM64, macOS Intel/Apple Silicon, and Windows x64. Exact binary/package/wheel tuples are candidates, not compatibility claims, until matching-runner artifact and minimum-platform installation evidence exists. An unproven family is deferred instead of shipping an untested compatibility tag.

## Legacy Claim Request Capture

An actual `nodix claim --json` execution against the real loopback fixture server exited `0` with empty stderr and captured:

```text
POST /api/v1/identity/register-machine
Authorization: absent
Body keys: machine_secret_hash, machine_uuid, user_cuid

POST /api/v1/device/code
Authorization: absent
Body keys: machine_uuid, user_cuid

POST /api/v1/device/token
Authorization: absent
Body keys: device_code, grant_type
```

The Rust contract therefore forbids an authorization header on both claim endpoints. Audit verification remains the only migrated request that sends the raw machine secret as bearer, after registration.

## Graph Extraction Degradation

The deterministic AST extraction completed in 0.2 seconds. The separate two-document semantic extraction exceeded 1.5 times its 45-second estimate and was terminated under the long-running process rule before producing output. No semantic result is used as evidence in this PRD.
