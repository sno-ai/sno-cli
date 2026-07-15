#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
legacy_root="${1:-${NODIX_PRIVATE_ROOT:-/home/lh/code/nodix-private}}"
manifest="$repo_root/fixtures/legacy-contract/source-manifest.sha256"
obligations="$repo_root/fixtures/legacy-contract/obligations.json"
goldens="$repo_root/fixtures/legacy-contract/cli-goldens.json"
matrix="$repo_root/ai-doc/ACTIVE/PRD/LEGACY-CONTRACT-MATRIX-sno-cli-initial-release.md"

fail() {
  printf 'legacy baseline verification failed: %s\n' "$1" >&2
  exit 1
}

for command in git jq npm sha256sum sed sort comm; do
  command -v "$command" >/dev/null 2>&1 || fail "missing required command: $command"
done

for file in "$manifest" "$obligations" "$goldens" "$matrix"; do
  [[ -r "$file" ]] || fail "missing input: $file"
done
[[ -d "$legacy_root/.git" ]] || fail "legacy repository not found: $legacy_root"

jq -e '.schema_version == 1 and (.obligations | type == "array" and length > 0)' "$obligations" >/dev/null
jq -e '.schema_version == 1 and (.cases | type == "array" and length > 0)' "$goldens" >/dev/null

expected_commit="$(jq -r '.captured_from_commit' "$goldens")"
actual_commit="$(git -C "$legacy_root" rev-parse HEAD)"
[[ "$actual_commit" == "$expected_commit" ]] || fail "legacy commit mismatch: expected $expected_commit, got $actual_commit"

expected_manifest_digest="$(jq -r '.source_manifest_sha256' "$goldens")"
actual_manifest_digest="$(sha256sum "$manifest" | cut -d' ' -f1)"
[[ "$actual_manifest_digest" == "$expected_manifest_digest" ]] || fail "golden corpus is bound to a different source manifest"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

(cd "$legacy_root" && sha256sum --check --strict "$manifest") >"$tmp_dir/hash-check.txt" || {
  sed -n '/FAILED/p' "$tmp_dir/hash-check.txt" >&2
  fail "source hash mismatch"
}

sed -nE 's/^\| ((CLI|ACC|STN|DATA|SEC)-[0-9]{3}) \|.*$/\1/p' "$matrix" | sort -u >"$tmp_dir/matrix-ids.txt"
jq -r '.obligations[].id' "$obligations" | sort -u >"$tmp_dir/obligation-ids.txt"
if ! cmp -s "$tmp_dir/matrix-ids.txt" "$tmp_dir/obligation-ids.txt"; then
  comm -3 "$tmp_dir/matrix-ids.txt" "$tmp_dir/obligation-ids.txt" >&2
  fail "matrix rows and obligation IDs differ"
fi

if [[ "$(jq -r '.obligations[].id' "$obligations" | sort | uniq -d | wc -l)" -ne 0 ]]; then
  fail "duplicate obligation ID"
fi
if [[ "$(jq -r '.cases[].name' "$goldens" | sort | uniq -d | wc -l)" -ne 0 ]]; then
  fail "duplicate golden case name"
fi

jq -e 'all(.obligations[]; (.source_files | length > 0) and (.legacy_tests | length > 0) and (.golden_cases | length > 0))' "$obligations" >/dev/null || fail "an obligation has an empty mapping"

cut -d' ' -f3- "$manifest" | sort -u >"$tmp_dir/manifest-paths.txt"
(
  cd "$legacy_root"
  {
    find apps/nodix-cli/src packages/sno-observe/src packages/common-core/src tests/apps/nodix-cli tests/packages/sno-observe -type f -print
    printf '%s\n' \
      apps/nodix-cli/package.json \
      apps/nodix-cli/bin/nodix.js \
      apps/nodix-cli/tsconfig.json \
      packages/sno-observe/package.json \
      packages/sno-observe/tsconfig.json \
      packages/common-core/package.json \
      packages/common-core/tsconfig.json \
      package-lock.json
  } | LC_ALL=C sort -u
) >"$tmp_dir/derived-paths.txt"
if ! cmp -s "$tmp_dir/manifest-paths.txt" "$tmp_dir/derived-paths.txt"; then
  comm -3 "$tmp_dir/manifest-paths.txt" "$tmp_dir/derived-paths.txt" >&2
  fail "per-file manifest differs from the independently derived source/test/config closure"
fi

jq -r '.obligations[] | .source_files[], .legacy_tests[]' "$obligations" | sort -u >"$tmp_dir/mapped-paths.txt"
if ! comm -23 "$tmp_dir/mapped-paths.txt" "$tmp_dir/manifest-paths.txt" | tee "$tmp_dir/unhashed-paths.txt" | grep -q .; then
  :
else
  cat "$tmp_dir/unhashed-paths.txt" >&2
  fail "mapped source or test is absent from the per-file manifest"
fi

jq -r '.cases[].name' "$goldens" | sort -u >"$tmp_dir/golden-names.txt"
jq -r '.obligations[].golden_cases[]' "$obligations" | sort -u >"$tmp_dir/mapped-goldens.txt"
if ! comm -23 "$tmp_dir/mapped-goldens.txt" "$tmp_dir/golden-names.txt" | tee "$tmp_dir/missing-goldens.txt" | grep -q .; then
  :
else
  cat "$tmp_dir/missing-goldens.txt" >&2
  fail "obligation references an unknown golden case"
fi

jq -e '
  def migrated:
    if .legacy_argv[0] == "consent" then ["station", "telemetry", "consent"] + .legacy_argv[1:]
    elif .legacy_argv[0] == "observe" then ["station", "telemetry"] + .legacy_argv[1:]
    elif .legacy_argv[0] == "register" or .legacy_argv[0] == "claim" then ["account", "machine"] + .legacy_argv
    elif .legacy_argv[0] == "audit" or .legacy_argv[0] == "doctor" then ["station"] + .legacy_argv
    else .legacy_argv
    end;
  all(.cases[];
    (.legacy_argv | type == "array") and
    (.rust_argv | type == "array") and
    (
      ((.scope == "station" or .scope == "station_telemetry" or .scope == "account_machine") and .rust_argv == migrated) or
      (.scope == "root" and .rust_argv == .legacy_argv) or
      (.scope == "new_root" and .rust_argv == .legacy_argv) or
      (.scope == "hard_cut_negative" and .rust_argv == .legacy_argv and .canonical_rust_argv == migrated)
    )
  )
' "$goldens" >/dev/null || fail "legacy-to-Rust argv transformation is not deterministic"

for command_name in consent observe register claim audit doctor; do
  jq -e --arg command_name "$command_name" '
    any(.cases[];
      .scope == "hard_cut_negative" and
      .legacy_argv[0] == $command_name and
      .rust_exit == 2 and
      .collision_must_execute == false and
      (.path_collision_executable | type == "string" and length > 0)
    )
  ' "$goldens" >/dev/null || fail "missing hard-cut negative golden for top-level command: $command_name"
done

if ! (cd "$legacy_root" && npm test --workspace @snoai/nodix) >"$tmp_dir/legacy-tests.txt" 2>&1; then
  tail -80 "$tmp_dir/legacy-tests.txt" >&2
  fail "legacy executable contract tests failed"
fi

printf 'legacy baseline verified at %s: %s obligations, %s golden cases, %s source files; legacy tests passed\n' \
  "$actual_commit" \
  "$(wc -l <"$tmp_dir/obligation-ids.txt" | tr -d ' ')" \
  "$(jq '.cases | length' "$goldens")" \
  "$(wc -l <"$manifest" | tr -d ' ')"
