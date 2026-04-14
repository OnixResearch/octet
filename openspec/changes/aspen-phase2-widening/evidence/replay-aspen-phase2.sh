#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
default_log="$script_dir/aspen-phase2-default-pilot.txt"
coord_log="$script_dir/aspen-phase2-coordination.txt"
meta_log="$script_dir/aspen-phase2-meta.txt"

aspen_repo=/home/brittonr/git/aspen
dns_repo=/home/brittonr/git/aspen-dns
plugin_repo=/home/brittonr/git/aspen-wasm-plugin
sandbox=$(mktemp -d /tmp/aspen-phase2.XXXXXX)
cleanup() {
  if [[ -d "$sandbox/aspen/.git" || -d "$sandbox/aspen" ]]; then
    git -C "$aspen_repo" worktree remove --force "$sandbox/aspen" >/dev/null 2>&1 || true
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT

mkdir -p "$sandbox"
git -C "$aspen_repo" worktree add --detach "$sandbox/aspen" HEAD >/dev/null
ln -s "$dns_repo" "$sandbox/aspen-dns"
ln -s "$plugin_repo" "$sandbox/aspen-wasm-plugin"

{
  printf '$ git -C %q rev-parse HEAD\n' "$aspen_repo"
  git -C "$aspen_repo" rev-parse HEAD
  printf '$ git -C %q rev-parse --short HEAD\n' "$aspen_repo"
  git -C "$aspen_repo" rev-parse --short HEAD
  printf '$ pwd\n%s\n' "$sandbox"
} > "$meta_log"

{
  printf '$ (cd %q && TIGERSTYLE_PATH=%q scripts/tigerstyle-check.sh)\n' "$sandbox/aspen" /home/brittonr/git/tigerstyle
  (cd "$sandbox/aspen" && TIGERSTYLE_PATH=/home/brittonr/git/tigerstyle scripts/tigerstyle-check.sh)
} > "$default_log" 2>&1

{
  printf '$ (cd %q && TIGERSTYLE_PATH=%q scripts/tigerstyle-check.sh -p aspen-coordination -- --lib)\n' "$sandbox/aspen" /home/brittonr/git/tigerstyle
  (cd "$sandbox/aspen" && TIGERSTYLE_PATH=/home/brittonr/git/tigerstyle scripts/tigerstyle-check.sh -p aspen-coordination -- --lib)
} > "$coord_log" 2>&1

python3 "$script_dir/check-aspen-phase2.py" "$default_log" "$coord_log" > "$script_dir/aspen-phase2-check.txt"
