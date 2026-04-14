#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
proof_log="$script_dir/second-consumer-proof.txt"
meta_log="$script_dir/second-consumer-meta.txt"
check_log="$script_dir/second-consumer-check.txt"

consumer_repo=/home/brittonr/git/aspen-verus-metrics
tigerstyle_repo=/home/brittonr/git/tigerstyle
sandbox=$(mktemp -d /tmp/second-consumer.XXXXXX)
cleanup() {
  if [[ -d "$sandbox/aspen-verus-metrics/.git" || -d "$sandbox/aspen-verus-metrics" ]]; then
    git -C "$consumer_repo" worktree remove --force "$sandbox/aspen-verus-metrics" >/dev/null 2>&1 || true
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT

mkdir -p "$sandbox"
git -C "$consumer_repo" worktree add --detach "$sandbox/aspen-verus-metrics" HEAD >/dev/null
mkdir -p "$sandbox/aspen-verus-metrics/scripts"
cp "$tigerstyle_repo/templates/tigerstyle-check.sh" "$sandbox/aspen-verus-metrics/scripts/tigerstyle-check.sh"
cp "$tigerstyle_repo/templates/dylint.toml" "$sandbox/aspen-verus-metrics/dylint.toml"
chmod +x "$sandbox/aspen-verus-metrics/scripts/tigerstyle-check.sh"
python3 - <<'PY' "$sandbox/aspen-verus-metrics/scripts/tigerstyle-check.sh"
from pathlib import Path
import sys
path = Path(sys.argv[1])
text = path.read_text()
old = "default_scope=(\n  # -p your-core-crate\n  # -p your-supporting-crate\n)\n"
new = "default_scope=(\n  -p aspen-verus-metrics\n)\n"
if old not in text:
    raise SystemExit("default_scope block not found")
path.write_text(text.replace(old, new, 1))
PY

{
  printf '$ git -C %q rev-parse HEAD\n' "$consumer_repo"
  git -C "$consumer_repo" rev-parse HEAD
  printf '$ git -C %q rev-parse --short HEAD\n' "$consumer_repo"
  git -C "$consumer_repo" rev-parse --short HEAD
  printf '$ sed -n %q %q\n' '1,120p' "$sandbox/aspen-verus-metrics/scripts/tigerstyle-check.sh"
  sed -n '1,120p' "$sandbox/aspen-verus-metrics/scripts/tigerstyle-check.sh"
} > "$meta_log"

{
  printf '$ nix develop %q -c bash -lc %q\n' "$tigerstyle_repo" "export PATH=\"\$HOME/.cargo/bin:\$PATH\" && cd \"$sandbox/aspen-verus-metrics\" && TIGERSTYLE_PATH=\"$tigerstyle_repo\" TIGERSTYLE_DYLINT_REV=\"0e0a71eefe6f01563d11acc6e1d7af1d505934a9\" scripts/tigerstyle-check.sh"
  nix develop "$tigerstyle_repo" -c bash -lc "export PATH=\"\$HOME/.cargo/bin:\$PATH\" && cd \"$sandbox/aspen-verus-metrics\" && TIGERSTYLE_PATH=\"$tigerstyle_repo\" TIGERSTYLE_DYLINT_REV=\"0e0a71eefe6f01563d11acc6e1d7af1d505934a9\" scripts/tigerstyle-check.sh"
} > "$proof_log" 2>&1

python3 "$script_dir/check-second-consumer.py" "$proof_log" > "$check_log"
