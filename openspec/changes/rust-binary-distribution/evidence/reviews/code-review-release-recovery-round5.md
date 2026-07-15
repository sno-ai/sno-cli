# Codex Adversarial Review

Target: `.github/workflows/release-preflight.yml`, `.github/workflows/release.yml`, `scripts/check-release-workflow.sh`, `scripts/test-release-workflow-policy.sh`
Verdict: needs-attention

Do not treat this as a secure release boundary yet: a movable tag can race the final publication check, and the policy checker can be bypassed with valid YAML that grants a pull-request job write access.

Findings:
- [high] Tag can move after the final commit check (`.github/workflows/release.yml`:423-431, confidence 0.94)
  Trigger: A tag is force-moved after the workflow resolves and compares its commit, but before it promotes the draft release. The workflow already recognizes tag movement as a realistic condition, but this check-and-publish sequence is not atomic.
  Impact: Assets built from the authorized commit can be published under a tag now resolving to a different commit. Users receive a release whose binaries and source tag disagree; correcting it requires revoking a public release.
  Recommendation: Enforce a repository ruleset that forbids updates and deletion of release tags before this workflow runs, restricting tag creation to the release authority.

- [high] Policy checker accepts flow-style write permissions (`scripts/check-release-workflow.sh`:32-49, confidence 0.92)
  Trigger: A workflow change adds valid YAML such as `permissions: { contents: write }` to the `plan` job. The checker only detects `write-all` or an indented block mapping, so it exits successfully.
  Impact: A same-repository pull request can execute its modified checkout/bootstrap code with a repository-write token while the stated policy check remains green, bypassing the intended release-token boundary.
  Recommendation: Parse the workflow as YAML and validate every job's effective permission mapping; add a mutation test for flow-style permission mappings.

Next steps:
- Lock release-tag mutation through repository rules before relying on commit-bound authorization.
- Replace the regex-based permission scan with schema-aware validation and cover alternate YAML forms.
