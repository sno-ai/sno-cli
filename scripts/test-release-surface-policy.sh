#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
checker="$repo_root/scripts/check-release-surfaces.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

expect_failure() {
  local root="$1"
  if "$checker" "$root" "$root/policy/release-surfaces.tsv" >/dev/null 2>&1; then
    printf 'release-surface mutation unexpectedly passed: %s\n' "$(basename "$root")" >&2
    exit 1
  fi
}

make_repo() {
  local root="$1"
  mkdir -p "$root/.github/workflows" "$root/policy"
  git -C "$root" init -q
  printf '[package]\nname = "fixture"\nversion = "0.1.0"\n' >"$root/Cargo.toml"
  printf 'name: CI\n' >"$root/.github/workflows/ci.yml"
}

make_repo "$tmp_dir/unlisted"
printf 'include\tCargo.toml\n' >"$tmp_dir/unlisted/policy/release-surfaces.tsv"
git -C "$tmp_dir/unlisted" add .
expect_failure "$tmp_dir/unlisted"

make_repo "$tmp_dir/empty-reason"
printf 'include\tCargo.toml\ninclude\t.github/workflows/ci.yml\nexclude\tpolicy/release-surfaces.tsv\n' >"$tmp_dir/empty-reason/policy/release-surfaces.tsv"
git -C "$tmp_dir/empty-reason" add .
expect_failure "$tmp_dir/empty-reason"

make_repo "$tmp_dir/forbidden-action"
printf 'name: publish\njobs:\n  publish:\n    steps:\n      - run: npm %s\n' "publish" >"$tmp_dir/forbidden-action/.github/workflows/ci.yml"
printf 'include\tCargo.toml\ninclude\t.github/workflows/ci.yml\ninclude\tpolicy/release-surfaces.tsv\n' >"$tmp_dir/forbidden-action/policy/release-surfaces.tsv"
git -C "$tmp_dir/forbidden-action" add .
expect_failure "$tmp_dir/forbidden-action"

make_repo "$tmp_dir/non-rust-manifest"
printf '{"name":"wrapper"}\n' >"$tmp_dir/non-rust-manifest/package.json"
printf 'include\tCargo.toml\ninclude\t.github/workflows/ci.yml\ninclude\tpackage.json\ninclude\tpolicy/release-surfaces.tsv\n' >"$tmp_dir/non-rust-manifest/policy/release-surfaces.tsv"
git -C "$tmp_dir/non-rust-manifest" add .
expect_failure "$tmp_dir/non-rust-manifest"

make_repo "$tmp_dir/excluded-publisher"
printf 'name: publish\njobs:\n  publish:\n    steps:\n      - run: npm %s\n' "publish" >"$tmp_dir/excluded-publisher/.github/workflows/ci.yml"
printf 'include\tCargo.toml\nexclude\t.github/workflows/ci.yml\tHistorical workflow.\ninclude\tpolicy/release-surfaces.tsv\n' >"$tmp_dir/excluded-publisher/policy/release-surfaces.tsv"
git -C "$tmp_dir/excluded-publisher" add .
expect_failure "$tmp_dir/excluded-publisher"

"$checker" "$repo_root" "$repo_root/policy/release-surfaces.tsv" >/dev/null
printf 'release-surface policy self-test passed: 5 forbidden mutations rejected and repository accepted\n'
