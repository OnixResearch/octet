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
script_rel=${script_dir#"$repo_root/"}
aspen_root=${ASPEN_PATH:-/home/brittonr/git/aspen}
aspen_dns_root=${ASPEN_DNS_PATH:-/home/brittonr/git/aspen-dns}
aspen_wasm_plugin_root=${ASPEN_WASM_PLUGIN_PATH:-/home/brittonr/git/aspen-wasm-plugin}
aspen_phase1_commit=${ASPEN_PHASE1_COMMIT:-909e5e9fa}
phase1_archive_rel=openspec/changes/archive/2026-04-13-aspen-consumer-rollout
phase1_archive="$repo_root/$phase1_archive_rel"
phase1_rollout_notes="$phase1_archive/rollout-notes.md"
phase1_validation_counts="$phase1_archive/evidence/validation-counts.txt"
wrapper_evidence_rel=openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt
wrapper_evidence="$repo_root/$wrapper_evidence_rel"
dylint_rev=${TIGERSTYLE_DYLINT_REV:-0e0a71eefe6f01563d11acc6e1d7af1d505934a9}
boundary_log="$script_dir/aspen-boundary-first.txt"
smoke_log="$script_dir/aspen-smoke.txt"
inventory_log="$script_dir/aspen-inventory-only.txt"
inventory_config_snapshot="$script_dir/aspen-inventory-only.dylint.toml"
checkpoint_note="$script_dir/aspen-validation-checkpoint.txt"
replay_rel="$script_rel/replay-aspen-profiles.sh"
boundary_log_rel=${boundary_log#"$repo_root/"}
smoke_log_rel=${smoke_log#"$repo_root/"}
inventory_log_rel=${inventory_log#"$repo_root/"}
inventory_config_snapshot_rel=${inventory_config_snapshot#"$repo_root/"}
checkpoint_note_rel=${checkpoint_note#"$repo_root/"}

for path in "$aspen_root" "$aspen_dns_root" "$aspen_wasm_plugin_root"; do
  if [[ ! -d "$path/.git" && ! -f "$path/.git" ]]; then
    echo "error: expected git repo at $path" >&2
    exit 1
  fi
done

for path in "$phase1_archive" "$phase1_rollout_notes" "$phase1_validation_counts" "$wrapper_evidence"; do
  if [[ ! -e "$path" ]]; then
    echo "error: required reference artifact missing: $path" >&2
    exit 1
  fi
done

write_boundary_config() {
  cat > "$1" <<'EOF'
# Aspen-specific Tiger Style tuning for starter smoke/boundary-first replay.
[tigerstyle]
function_length_max = 100
nesting_max_depth = 4
assertion_density_min_lines = 30
assertion_density_min_assertions = 2
acronym_allowlist = ["IO"]
disabled_lints = [
  "function_length",
  "acronym_style",
  "numeric_units",
  "unbounded_loop",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_unwrap",
  "no_panic",
  "no_recursion",
]
EOF
}

write_inventory_config() {
  cat > "$1" <<'EOF'
# Aspen-specific Tiger Style tuning for inventory-only replay.
[tigerstyle]
function_length_max = 100
nesting_max_depth = 4
assertion_density_min_lines = 30
assertion_density_min_assertions = 2
acronym_allowlist = ["IO"]
disabled_lints = [
  "compound_assertion",
  "function_length",
  "acronym_style",
  "numeric_units",
  "unbounded_loop",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "bool_naming",
  "ignored_result",
  "unchecked_narrowing",
  "unchecked_division",
  "contradictory_time",
  "ambient_clock",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_panic",
  "no_recursion",
]

[tigerstyle.lint_levels]
no_unwrap = "warn"
EOF
}

set_wrapper_banner() {
  local script_path=$1
  shift
  python3 - "$script_path" "$@" <<'PY'
from pathlib import Path
import re
import sys

script_path = Path(sys.argv[1])
entries = sys.argv[2:]
text = script_path.read_text()
pattern = re.compile(r"starter_lints=\(\n(?:  [^\n]+\n)*\)\n")
replacement = "starter_lints=(\n" + "".join(f"  {entry}\n" for entry in entries) + ")\n"
updated, count = pattern.subn(replacement, text, count=1)
if count != 1:
    raise SystemExit("starter_lints block not found in template copy")
script_path.write_text(updated)
PY
}

sandbox=$(mktemp -d /tmp/aspen-per-lint-config.XXXXXX)
worktree="$sandbox/aspen"
cleanup() {
  set +e
  if [[ -e "$worktree" ]]; then
    git -C "$aspen_root" worktree remove --force "$worktree" >/dev/null 2>&1 || rm -rf "$worktree"
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT

cat > "$checkpoint_note" <<EOF
Exact replay command: bash $replay_rel
Pinned Aspen revision: $aspen_phase1_commit
Phase-1 archive: $phase1_archive_rel/
Phase-1 rollout notes: $phase1_archive_rel/rollout-notes.md
Phase-1 validation counts: $phase1_archive_rel/evidence/validation-counts.txt
Wrapper evidence path: $wrapper_evidence_rel
Authoritative repo template path: templates/tigerstyle-check.sh
Boundary-first transcript: $boundary_log_rel
Smoke transcript: $smoke_log_rel
Inventory-only transcript: $inventory_log_rel
Inventory-only config snapshot: $inventory_config_snapshot_rel
EOF

echo "Exact replay command: bash $replay_rel"
echo "Phase-1 archive reference: $phase1_archive_rel/"
echo "Phase-1 rollout notes: $phase1_archive_rel/rollout-notes.md"
echo "Phase-1 validation counts: $phase1_archive_rel/evidence/validation-counts.txt"
echo "Wrapper evidence path: $wrapper_evidence_rel"
printf '$ ln -s %q %q\n' "$aspen_dns_root" "$sandbox/aspen-dns"
ln -s "$aspen_dns_root" "$sandbox/aspen-dns"
printf '$ ln -s %q %q\n' "$aspen_wasm_plugin_root" "$sandbox/aspen-wasm-plugin"
ln -s "$aspen_wasm_plugin_root" "$sandbox/aspen-wasm-plugin"
printf '$ git -C %q worktree add --detach %q %q\n' "$aspen_root" "$worktree" "$aspen_phase1_commit"
git -C "$aspen_root" worktree add --detach "$worktree" "$aspen_phase1_commit"

printf '$ cp %q %q\n' "$repo_root/templates/tigerstyle-check.sh" "$worktree/scripts/tigerstyle-check.sh"
mkdir -p "$worktree/scripts"
cp "$repo_root/templates/tigerstyle-check.sh" "$worktree/scripts/tigerstyle-check.sh"

printf '$ cp %q %q\n' "$repo_root/templates/dylint.toml" "$worktree/dylint.toml"
cp "$repo_root/templates/dylint.toml" "$worktree/dylint.toml"

printf '$ python3 - %q\n' "$worktree/scripts/tigerstyle-check.sh"
python3 - "$worktree/scripts/tigerstyle-check.sh" <<'PY'
from pathlib import Path
import sys
script_path = Path(sys.argv[1])
script = script_path.read_text()
old = "default_scope=(\n  # -p your-core-crate\n  # -p your-supporting-crate\n)\n"
new = "default_scope=(\n  -p aspen-time\n  -p aspen-hlc\n  -p aspen-core\n)\n"
if old not in script:
    raise SystemExit("default_scope block not found in template copy")
script_path.write_text(script.replace(old, new, 1))
PY

printf '$ write boundary-first config to %q\n' "$worktree/dylint.toml"
write_boundary_config "$worktree/dylint.toml"
printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q scripts/tigerstyle-check.sh) > %q 2>&1\n' \
  "$worktree" "$repo_root" "$dylint_rev" "$boundary_log"
(
  cd "$worktree"
  TIGERSTYLE_PATH="$repo_root" \
  TIGERSTYLE_DYLINT_REV="$dylint_rev" \
  scripts/tigerstyle-check.sh
) > "$boundary_log" 2>&1

grep -q 'direct system clock read in production code' "$boundary_log"
! grep -q 'production code will panic on failure' "$boundary_log"

printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q scripts/tigerstyle-check.sh -p aspen-time -- --lib) > %q 2>&1\n' \
  "$worktree" "$repo_root" "$dylint_rev" "$smoke_log"
(
  cd "$worktree"
  TIGERSTYLE_PATH="$repo_root" \
  TIGERSTYLE_DYLINT_REV="$dylint_rev" \
  scripts/tigerstyle-check.sh -p aspen-time -- --lib
) > "$smoke_log" 2>&1

grep -q 'direct system clock read in production code' "$smoke_log"
! grep -q 'production code will panic on failure' "$smoke_log"

printf '$ write inventory-only config to %q\n' "$worktree/dylint.toml"
write_inventory_config "$worktree/dylint.toml"
printf '$ cp %q %q\n' "$worktree/dylint.toml" "$inventory_config_snapshot"
cp "$worktree/dylint.toml" "$inventory_config_snapshot"
printf '$ python3 - %q inventory-only no_unwrap\n' "$worktree/scripts/tigerstyle-check.sh"
set_wrapper_banner "$worktree/scripts/tigerstyle-check.sh" inventory-only no_unwrap
printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q scripts/tigerstyle-check.sh) > %q 2>&1\n' \
  "$worktree" "$repo_root" "$dylint_rev" "$inventory_log"
(
  printf '# replay profile: inventory-only\n'
  printf '# enabled lint banner: no_unwrap\n'
  printf '# config snapshot: %s\n' "$inventory_config_snapshot_rel"
  printf '# archive reference: %s/\n' "$phase1_archive_rel"
  cd "$worktree"
  TIGERSTYLE_PATH="$repo_root" \
  TIGERSTYLE_DYLINT_REV="$dylint_rev" \
  scripts/tigerstyle-check.sh
) > "$inventory_log" 2>&1

grep -q '^# replay profile: inventory-only$' "$inventory_log"
grep -q '^\[tigerstyle\] profile:   inventory-only no_unwrap$' "$inventory_log"
! grep -q 'production code will panic on failure' "$inventory_log"

printf 'checkpoint note: %s\n' "$checkpoint_note"
printf 'boundary-first transcript: %s\n' "$boundary_log"
printf 'smoke transcript: %s\n' "$smoke_log"
printf 'inventory-only transcript: %s\n' "$inventory_log"
printf 'inventory-only config snapshot: %s\n' "$inventory_config_snapshot"
