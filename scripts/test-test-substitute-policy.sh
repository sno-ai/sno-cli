#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
checker="$repo_root/scripts/check-test-substitutes.sh"
policy="$repo_root/policy/test-substitutes.json"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

expect_failure() {
  local name="$1"
  if "$checker" "$tmp_dir/$name" "$policy" >/dev/null 2>&1; then
    printf 'policy mutation unexpectedly passed: %s\n' "$name" >&2
    exit 1
  fi
}

mkdir -p "$tmp_dir/dependency/tests"
printf '[dev-dependencies]\nmockall = "0.13"\n' >"$tmp_dir/dependency/Cargo.toml"
printf 'fn real_test() {}\n' >"$tmp_dir/dependency/tests/basic.rs"
expect_failure dependency

mkdir -p "$tmp_dir/internal/tests"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/internal/Cargo.toml"
printf 'mock!(InternalService {});\n' >"$tmp_dir/internal/tests/internal.rs"
expect_failure internal

mkdir -p "$tmp_dir/colocated/src"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/colocated/Cargo.toml"
printf '#[cfg(test)] mod tests { mock!(InternalService {}); }\n' >"$tmp_dir/colocated/src/lib.rs"
expect_failure colocated

mkdir -p "$tmp_dir/patch/tests"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/patch/Cargo.toml"
printf 'patch("module.internal");\n' >"$tmp_dir/patch/tests/internal.py"
expect_failure patch

for kind in filesystem database process; do
  mkdir -p "$tmp_dir/fake-$kind/src"
  printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/fake-$kind/Cargo.toml"
  case "$kind" in
    filesystem) printf 'struct FakeFileSystem;\n' >"$tmp_dir/fake-$kind/src/lib.rs" ;;
    database) printf 'struct FakeDatabase;\n' >"$tmp_dir/fake-$kind/src/lib.rs" ;;
    process) printf 'struct FakeProcess;\n' >"$tmp_dir/fake-$kind/src/lib.rs" ;;
  esac
  expect_failure "fake-$kind"
done

mkdir -p "$tmp_dir/service/tests"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/service/Cargo.toml"
printf 'let listener = TcpListener::bind("127.0.0.1:0");\n' >"$tmp_dir/service/tests/rogue_server.rs"
expect_failure service

mkdir -p "$tmp_dir/nonloopback/tests/support"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/nonloopback/Cargo.toml"
printf 'let listener = TcpListener::bind("0.0.0.0:0"); // 127.0.0.1\n' >"$tmp_dir/nonloopback/tests/support/sno_service_server.rs"
expect_failure nonloopback

mkdir -p "$tmp_dir/allowed/tests/support"
printf '[package]\nname = "seed"\nversion = "0.0.0"\n' >"$tmp_dir/allowed/Cargo.toml"
printf 'let listener = TcpListener::bind("127.0.0.1:0");\n' >"$tmp_dir/allowed/tests/support/sno_service_server.rs"
"$checker" "$tmp_dir/allowed" "$policy" >/dev/null

"$checker" "$repo_root" "$policy" >/dev/null
printf 'test-substitute policy self-test passed: 9 forbidden mutations rejected, allowlist and repository accepted\n'
