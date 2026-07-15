#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
legacy_root="${1:-${NODIX_PRIVATE_ROOT:-/home/lh/code/nodix-private}}"
binary="$repo_root/target/debug/sno"

fail() {
  printf 'cross-language verification failed: %s\n' "$1" >&2
  exit 1
}

[[ -x "$binary" ]] || fail "build the Rust binary first with cargo build"
[[ -f "$legacy_root/apps/nodix-cli/dist/index.js" ]] || fail "build the legacy CLI first"
[[ -f "$legacy_root/packages/sno-observe/dist/internal/buffer-store.js" ]] || fail "build the legacy SDK first"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

rust_profile="$tmp_dir/rust-profile"
SNO_PROFILE_DIR="$rust_profile" "$binary" station telemetry consent set full --json >/dev/null
SNO_PROFILE_DIR="$rust_profile" "$binary" station telemetry export --format jsonl >"$tmp_dir/rust-from-rust.jsonl"
SNO_PROFILE_DIR="$rust_profile" node "$legacy_root/apps/nodix-cli/dist/index.js" observe export --format jsonl >"$tmp_dir/typescript-from-rust.jsonl"
cmp "$tmp_dir/rust-from-rust.jsonl" "$tmp_dir/typescript-from-rust.jsonl"

SNO_BUFFER_PATH="$rust_profile/buffer.db" node --input-type=module - "$legacy_root" <<'NODE'
const legacyRoot = process.argv[2];
const { BufferStore } = await import(`${legacyRoot}/packages/sno-observe/dist/internal/buffer-store.js`);
const store = new BufferStore(process.env.SNO_BUFFER_PATH);
try {
  if (!store.verifyLocalChain()) process.exit(1);
} finally {
  store.close();
}
NODE

typescript_profile="$tmp_dir/typescript-profile"
SNO_PROFILE_DIR="$typescript_profile" node "$legacy_root/apps/nodix-cli/dist/index.js" consent set full --json >/dev/null
SNO_PROFILE_DIR="$typescript_profile" node "$legacy_root/apps/nodix-cli/dist/index.js" observe export --format jsonl >"$tmp_dir/typescript-from-typescript.jsonl"
SNO_PROFILE_DIR="$typescript_profile" "$binary" station telemetry export --format jsonl >"$tmp_dir/rust-from-typescript.jsonl"
cmp "$tmp_dir/typescript-from-typescript.jsonl" "$tmp_dir/rust-from-typescript.jsonl"

printf 'cross-language state verified: Rust and TypeScript read identical SQLite event bytes in both directions\n'
