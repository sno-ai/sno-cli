#!/usr/bin/env bash

set -euo pipefail

repo="${1:-sno-ai/sno-cli}"
commit="${2:-$(git rev-parse HEAD)}"
ruleset_name='Protect version release tags'

fail() {
  printf 'release authorization failed: %s\n' "$1" >&2
  exit 1
}

for command in gh jq git; do
  command -v "$command" >/dev/null 2>&1 || fail "missing required command: $command"
done

[[ -z "$(git status --short)" ]] || fail "working tree is not clean"
[[ "$(gh api "repos/$repo" --jq '.visibility')" = public ]] || fail "repository is not public"
[[ "$(gh api -H 'X-GitHub-Api-Version: 2026-03-10' "repos/$repo/immutable-releases" --jq '.enabled')" = true ]] || fail "immutable releases are not enabled"
[[ "$(gh api "repos/$repo/git/ref/heads/main" --jq '.object.sha')" = "$commit" ]] || fail "commit is not the remote main head"

ruleset_id="$(gh api "repos/$repo/rulesets" --paginate --jq ".[] | select(.name == \"$ruleset_name\" and .target == \"tag\" and .enforcement == \"active\") | .id" | head -n 1)"
[[ -n "$ruleset_id" ]] || fail "active version-tag ruleset is missing"
ruleset="$(gh api "repos/$repo/rulesets/$ruleset_id")"
jq -e '
  .conditions.ref_name.include | index("refs/tags/v*") != null
' <<<"$ruleset" >/dev/null || fail "version-tag ruleset target is incomplete"
for rule in creation update deletion; do
  jq -e --arg rule "$rule" 'any(.rules[]; .type == $rule)' <<<"$ruleset" >/dev/null || fail "version-tag ruleset is missing $rule protection"
done
jq -e '
  .bypass_actors | length == 1 and
  .[0].actor_type == "OrganizationAdmin" and
  .[0].bypass_mode == "always"
' <<<"$ruleset" >/dev/null || fail "version-tag ruleset bypass is not limited to organization administrators"

ci_success="$(gh api "repos/$repo/actions/runs?head_sha=$commit&status=completed&per_page=100" --jq '[.workflow_runs[] | select(.name == "CI" and .conclusion == "success")] | length')"
[[ "$ci_success" -gt 0 ]] || fail "no successful CI run exists for the commit"

gh variable set SNO_RELEASE_AUTHORIZED_SHA --repo "$repo" --body "$commit"
printf 'release authorized: repo=%s commit=%s ruleset=%s\n' "$repo" "$commit" "$ruleset_id"
