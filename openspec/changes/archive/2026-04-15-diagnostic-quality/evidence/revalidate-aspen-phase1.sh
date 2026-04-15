#!/usr/bin/env bash
set -euo pipefail
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing required command: $1" >&2
    exit 1
  fi
}

for cmd in bash cargo rustc python3 cargo-dylint dylint-link git mktemp sed; do
  need_cmd "$cmd"
done

repo_root=$(git rev-parse --show-toplevel)
script_dir=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
reference_change='openspec/changes/archive/2026-04-13-aspen-consumer-rollout'
support_dir="$repo_root/openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence"
prepare_py="$support_dir/prepare-aspen-phase1-worktree.py"
counts_py="$script_dir/check-aspen-phase1-revalidation.py"
aspen_root=${ASPEN_PATH:-/home/brittonr/git/aspen}
aspen_dns_root=${ASPEN_DNS_PATH:-/home/brittonr/git/aspen-dns}
aspen_wasm_plugin_root=${ASPEN_WASM_PLUGIN_PATH:-/home/brittonr/git/aspen-wasm-plugin}
aspen_phase1_commit=${ASPEN_PHASE1_COMMIT:-909e5e9fa}
dylint_rev=${TIGERSTYLE_DYLINT_REV:-0e0a71eefe6f01563d11acc6e1d7af1d505934a9}
default_log="$script_dir/aspen-phase1-default-run.txt"
coord_log="$script_dir/aspen-phase1-coordination-spotcheck.txt"

for path in "$prepare_py" "$counts_py"; do
  if [[ ! -f "$path" ]]; then
    echo "error: expected helper at $path" >&2
    exit 1
  fi
done

for path in "$aspen_root" "$aspen_dns_root" "$aspen_wasm_plugin_root"; do
  if [[ ! -d "$path/.git" && ! -f "$path/.git" ]]; then
    echo "error: expected git repo at $path" >&2
    exit 1
  fi
done

printf 'reference archive: %s\n' "$reference_change"

sandbox=$(mktemp -d /tmp/aspen-phase1-revalidation.XXXXXX)
worktree="$sandbox/aspen"
cleanup() {
  set +e
  if [[ -e "$worktree" ]]; then
    git -C "$aspen_root" worktree remove --force "$worktree" >/dev/null 2>&1 || rm -rf "$worktree"
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT

printf '$ ln -s %q %q\n' "$aspen_dns_root" "$sandbox/aspen-dns"
ln -s "$aspen_dns_root" "$sandbox/aspen-dns"
printf '$ ln -s %q %q\n' "$aspen_wasm_plugin_root" "$sandbox/aspen-wasm-plugin"
ln -s "$aspen_wasm_plugin_root" "$sandbox/aspen-wasm-plugin"
printf '$ git -C %q worktree add --detach %q %q\n' "$aspen_root" "$worktree" "$aspen_phase1_commit"
git -C "$aspen_root" worktree add --detach "$worktree" "$aspen_phase1_commit"

printf '$ cp templates/tigerstyle-check.sh %q\n' "$worktree/scripts/tigerstyle-check.sh"
mkdir -p "$worktree/scripts"
cp "$repo_root/templates/tigerstyle-check.sh" "$worktree/scripts/tigerstyle-check.sh"

printf '$ cp templates/dylint.toml %q\n' "$worktree/dylint.toml"
cp "$repo_root/templates/dylint.toml" "$worktree/dylint.toml"

printf '$ python3 %q %q %q\n' "$prepare_py" "$worktree/scripts/tigerstyle-check.sh" "$worktree/dylint.toml"
python3 "$prepare_py" "$worktree/scripts/tigerstyle-check.sh" "$worktree/dylint.toml"

printf '$ sed -n '\''1,80p'\'' %q\n' "$worktree/scripts/tigerstyle-check.sh"
sed -n '1,80p' "$worktree/scripts/tigerstyle-check.sh"

printf '$ cat %q\n' "$worktree/dylint.toml"
cat "$worktree/dylint.toml"

printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q scripts/tigerstyle-check.sh) > %q 2>&1\n' \
  "$worktree" "$repo_root" "$dylint_rev" "$default_log"
(
  cd "$worktree"
  TIGERSTYLE_PATH="$repo_root" \
  TIGERSTYLE_DYLINT_REV="$dylint_rev" \
  scripts/tigerstyle-check.sh
) > "$default_log" 2>&1

printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q scripts/tigerstyle-check.sh -p aspen-coordination -- --lib) > %q 2>&1\n' \
  "$worktree" "$repo_root" "$dylint_rev" "$coord_log"
(
  cd "$worktree"
  TIGERSTYLE_PATH="$repo_root" \
  TIGERSTYLE_DYLINT_REV="$dylint_rev" \
  scripts/tigerstyle-check.sh -p aspen-coordination -- --lib
) > "$coord_log" 2>&1

printf '$ python3 %q %q %q\n' "$counts_py" "$default_log" "$coord_log"
python3 "$counts_py" "$default_log" "$coord_log"

echo
printf 'raw default run: %s\n' "$default_log"
printf 'raw coordination run: %s\n' "$coord_log"
