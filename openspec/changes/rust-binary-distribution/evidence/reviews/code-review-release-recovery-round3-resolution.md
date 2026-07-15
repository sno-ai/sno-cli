# Release Recovery Review Resolution

## Critical candidate-cleanup finding

Status: `[Possible FP]`, refuted by a live repository probe and vendor contract.

The reviewer correctly required evidence because the workflow makes final publication depend on deleting an immutable one-use candidate. GitHub's immutable-release contract permits deleting an immutable release and then deleting its tag, while forbidding reuse of that tag name. A unique candidate tag therefore remains disposable.

Live probe on 2026-07-15:

1. Created public prerelease tag `sno-release-candidate-probe-1784112875` at reviewed commit `47c673641a89ad3945d20a86b16531eeccb26cfd`.
2. `GET /repos/sno-ai/sno-cli/releases/tags/{tag}` returned `.immutable == true`.
3. `gh release delete {tag} --yes --cleanup-tag` succeeded.
4. Both the release-by-tag API lookup and `git ls-remote origin refs/tags/{tag}` confirmed absence afterward.

The production candidate uses a run-unique tag and the same deletion sequence. No workflow change is required for this finding.

## High preflight-reachability finding

Status: fixed.

`scripts/check-release-workflow.sh` now requires the reusable authorization-preflight job and requires artifact builds to depend on it. The policy self-test removes that dependency and proves the checker fails closed.
