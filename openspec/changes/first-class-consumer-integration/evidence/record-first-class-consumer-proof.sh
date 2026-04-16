#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/../../../.." && pwd)
fixture_template_root="$repo_root/tests/fixtures/consumer-integration/nix-consumer"
system=$(nix eval --impure --expr builtins.currentSystem --raw)
tmp_root=$(mktemp -d /tmp/tigerstyle-first-class-consumer-proof.XXXXXX)
cli_target_dir="$tmp_root/cli-target"
fixture_dir="$tmp_root/consumer"
meta_log="$script_dir/consumer-meta.txt"
cli_default_log="$script_dir/cli-default-roundtrip.txt"
cli_override_log="$script_dir/cli-override-contrast.txt"
nix_parity_log="$script_dir/nix-run-parity.txt"
cleanup() {
  rm -rf "$tmp_root"
}
trap cleanup EXIT

require_contains() {
  local file=$1
  local needle=$2
  if ! grep -Fq "$needle" "$file"; then
    echo "error: expected '$needle' in $file" >&2
    exit 1
  fi
}

require_not_contains() {
  local file=$1
  local needle=$2
  if grep -Fq "$needle" "$file"; then
    echo "error: unexpected '$needle' in $file" >&2
    exit 1
  fi
}

cp -r "$fixture_template_root/." "$fixture_dir"
chmod -R u+w "$fixture_dir"
python3 - "$fixture_dir/flake.nix" "$repo_root" "$system" <<'PY'
from pathlib import Path
import sys

flake_path = Path(sys.argv[1])
repo_root = sys.argv[2]
system = sys.argv[3]
text = flake_path.read_text()
text = text.replace("@TIGERSTYLE_PATH@", repo_root)
text = text.replace("@SYSTEM@", system)
flake_path.write_text(text)
PY

nix develop -c bash -lc "cd '$fixture_dir' && cargo generate-lockfile >/dev/null"

cat > "$meta_log" <<EOF
$ bash openspec/changes/first-class-consumer-integration/evidence/record-first-class-consumer-proof.sh
system: $system
repo_root: $repo_root
fixture_template: tests/fixtures/consumer-integration/nix-consumer
fixture_default_scope: -p good
fixture_override_scope: -p bad -- --all-targets
cli_driver_bootstrap_rev: 0e0a71eefe6f01563d11acc6e1d7af1d505934a9
EOF

cat > "$tmp_root/cli-default.sh" <<EOF
set -euo pipefail
cd "$repo_root"
export CARGO_TARGET_DIR="$cli_target_dir"
cargo build -q --package cargo-tigerstyle --bin cargo-tigerstyle
export PATH="$cli_target_dir/debug:\$PATH"
export TIGERSTYLE_DYLINT_REV='0e0a71eefe6f01563d11acc6e1d7af1d505934a9'
cd "$fixture_dir"
cp Cargo.lock Cargo.lock.before
echo '$ export TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9'
echo '$ cargo tigerstyle check'
echo '$ rustup show active-toolchain'
rustup show active-toolchain
cargo tigerstyle check
echo '$ cmp -s Cargo.lock Cargo.lock.before'
cmp -s Cargo.lock Cargo.lock.before
echo 'Cargo.lock unchanged'
EOF
nix develop -c bash "$tmp_root/cli-default.sh" > "$cli_default_log" 2>&1
require_contains "$cli_default_log" '$ cargo tigerstyle check'
require_contains "$cli_default_log" 'good v0.1.0'
require_contains "$cli_default_log" 'Cargo.lock unchanged'
require_not_contains "$cli_default_log" 'bad crate should be skipped by metadata default scope'

cat > "$tmp_root/cli-override.sh" <<EOF
set -euo pipefail
cd "$repo_root"
export CARGO_TARGET_DIR="$cli_target_dir"
cargo build -q --package cargo-tigerstyle --bin cargo-tigerstyle
export PATH="$cli_target_dir/debug:\$PATH"
export TIGERSTYLE_DYLINT_REV='0e0a71eefe6f01563d11acc6e1d7af1d505934a9'
cd "$fixture_dir"
cp Cargo.lock Cargo.lock.before
echo '$ export TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9'
echo '$ cargo tigerstyle check -p bad -- --all-targets'
set +e
cargo tigerstyle check -p bad -- --all-targets
status=\$?
set -e
echo "exit_code=\$status"
test "\$status" -eq 1
echo '$ cmp -s Cargo.lock Cargo.lock.before'
cmp -s Cargo.lock Cargo.lock.before
echo 'Cargo.lock unchanged'
EOF
nix develop -c bash "$tmp_root/cli-override.sh" > "$cli_override_log" 2>&1
require_contains "$cli_override_log" '$ cargo tigerstyle check -p bad -- --all-targets'
require_contains "$cli_override_log" 'bad crate should be skipped by metadata default scope'
require_contains "$cli_override_log" 'exit_code=1'
require_contains "$cli_override_log" 'Cargo.lock unchanged'

cat > "$tmp_root/nix-parity.sh" <<EOF
set -euo pipefail
cd "$fixture_dir"
cp Cargo.lock Cargo.lock.before
echo '$ nix run path:$repo_root#cargo-tigerstyle -- check'
nix run "path:$repo_root#cargo-tigerstyle" -- check
echo '$ cmp -s Cargo.lock Cargo.lock.before'
cmp -s Cargo.lock Cargo.lock.before
echo 'Cargo.lock unchanged'
EOF
bash "$tmp_root/nix-parity.sh" > "$nix_parity_log" 2>&1
require_contains "$nix_parity_log" "$ nix run path:$repo_root#cargo-tigerstyle -- check"
require_contains "$nix_parity_log" 'good v0.1.0'
require_contains "$nix_parity_log" 'Cargo.lock unchanged'

printf 'saved %s\n' "$meta_log"
printf 'saved %s\n' "$cli_default_log"
printf 'saved %s\n' "$cli_override_log"
printf 'saved %s\n' "$nix_parity_log"
