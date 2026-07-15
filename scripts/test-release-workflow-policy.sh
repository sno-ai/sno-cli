#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
checker="$repo_root/scripts/check-release-workflow.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

make_fixture() {
  local name="$1"
  local root="$tmp_dir/$name"
  mkdir -p "$root"
  cp -R "$repo_root/.github" "$repo_root/scripts" "$repo_root/tools" "$root/"
  printf '%s\n' "$root"
}

expect_failure() {
  local root="$1"
  if "$checker" "$root" >/dev/null 2>&1; then
    printf 'release-workflow mutation unexpectedly passed: %s\n' "$(basename "$root")" >&2
    exit 1
  fi
}

root="$(make_fixture mutable-action)"
sed -i '0,/@[0-9a-f]\{40\}/s//@latest/' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture write-all)"
sed -i '/^  plan:$/a\    permissions: write-all' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture flow-style-write)"
sed -i '/^  plan:$/a\    permissions: { contents: write }' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture bootstrap-bypass)"
sed -i '0,/run: scripts\/install-cargo-dist.sh/s//run: true/' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture portable-checksum-bypass)"
sed -i '/shasum -a 256/d' "$root/scripts/install-cargo-dist.sh"
expect_failure "$root"

root="$(make_fixture zip-extraction-bypass)"
sed -i '/unzip -q/d' "$root/scripts/install-cargo-dist.sh"
expect_failure "$root"

root="$(make_fixture archive-checksum-bypass)"
sed -i '0,/recorded="${recorded#\\\*}"/d' "$root/.github/workflows/release-archive-smoke.yml"
expect_failure "$root"

root="$(make_fixture host-bypass)"
sed -i '0,/^      - custom-release-installer-smoke$/d' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture installer-bypass)"
sed -i '/& "artifacts\/\${{ matrix.script }}"/d' "$root/.github/workflows/release-installer-verify.yml"
expect_failure "$root"

root="$(make_fixture receipt-bypass)"
sed -i '/test "\${RELEASE_AUTHORIZED_SHA}" = "\${GITHUB_SHA}"/d' "$root/.github/workflows/release-preflight.yml"
expect_failure "$root"

root="$(make_fixture immutable-result-bypass)"
sed -i '/state=unknown/d' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture tag-move-bypass)"
sed -i '0,/test "\$resolved_commit" = "\$GITHUB_SHA"/d' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture candidate-bypass)"
sed -i '0,/^      - custom-release-candidate-installer-smoke$/d' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture publication-cleanup-bypass)"
sed -i '0,/needs.verify-published-release.outputs.state == '\''mutable'\''/s//needs.verify-published-release.outputs.state == '\''unknown'\''/' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture preflight-reachability-bypass)"
sed -i '0,/^      - custom-release-preflight$/d' "$root/.github/workflows/release.yml"
expect_failure "$root"

root="$(make_fixture public-smoke-cleanup-bypass)"
sed -i '/needs.custom-release-public-installer-smoke.result == '\''failure'\''/s/failure/success/' "$root/.github/workflows/release.yml"
expect_failure "$root"

"$checker" "$repo_root" >/dev/null
printf 'release-workflow policy self-test passed: 16 security mutations rejected and repository accepted\n'
