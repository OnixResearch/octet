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
script_rel=${script_dir#"$repo_root/"}/replay-aspen-comparison.sh
phase1_archive_rel=openspec/changes/archive/2026-04-13-aspen-consumer-rollout/
phase2_archive_rel=openspec/changes/archive/2026-04-14-aspen-phase2-widening/
phase1_archive="$repo_root/${phase1_archive_rel%/}"
phase2_archive="$repo_root/${phase2_archive_rel%/}"

aspen_repo=${ASPEN_PATH:-/home/brittonr/git/aspen}
aspen_dns_repo=${ASPEN_DNS_PATH:-/home/brittonr/git/aspen-dns}
aspen_plugin_repo=${ASPEN_WASM_PLUGIN_PATH:-/home/brittonr/git/aspen-wasm-plugin}
aspen_commit=${ASPEN_COMMIT:-8cddaec93ad1be8c224589e44ea33c6bcedd30b0}

tigerstyle_before_commit=${TIGERSTYLE_BEFORE_COMMIT:-f04333d6640228d2305904b981ce70f7d803424d}
tigerstyle_after_commit=${TIGERSTYLE_AFTER_COMMIT:-2eacd45e17c3f02c5a1e19025b63279981247f12}
dylint_rev=${TIGERSTYLE_DYLINT_REV:-0e0a71eefe6f01563d11acc6e1d7af1d505934a9}

output_prefix=${OUTPUT_PREFIX:-aspen}
meta_log="$script_dir/${output_prefix}-meta.txt"
before_log="$script_dir/${output_prefix}-before.txt"
after_log="$script_dir/${output_prefix}-after.txt"
comparison_md="$script_dir/${output_prefix}-comparison.md"
scope_mode=${COMPARISON_SCOPE_MODE:-workspace_all_targets}
case "$scope_mode" in
  workspace_all_targets)
    scope_display='--workspace -- --all-targets'
    ;;
  widened_all_targets)
    scope_display='-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets'
    ;;
  widened_packages)
    scope_display='-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --lib'
    ;;
  *)
    echo "error: unsupported COMPARISON_SCOPE_MODE: $scope_mode" >&2
    exit 1
    ;;
esac

for path in "$phase1_archive" "$phase2_archive" "$aspen_repo" "$aspen_dns_repo" "$aspen_plugin_repo"; do
  if [[ ! -e "$path" ]]; then
    echo "error: required path missing: $path" >&2
    exit 1
  fi
done

set_wrapper_banner() {
  local script_path=$1
  python3 - "$script_path" <<'PY'
from pathlib import Path
import re
import sys

script_path = Path(sys.argv[1])
text = script_path.read_text()
pattern = re.compile(r"starter_lints=\(\n(?:  [^\n]+\n)*\)\n")
replacement = "starter_lints=(\n  ignored_result\n  no_unwrap\n  no_panic\n  unchecked_narrowing\n  unbounded_loop\n)\n"
updated, count = pattern.subn(replacement, text, count=1)
if count != 1:
    raise SystemExit("starter_lints block not found")
script_path.write_text(updated)
PY
}

set_default_scope() {
  local script_path=$1
  python3 - "$script_path" <<'PY'
from pathlib import Path
import sys

script_path = Path(sys.argv[1])
text = script_path.read_text()
old = "default_scope=(\n  # -p your-core-crate\n  # -p your-supporting-crate\n)\n"
new = "default_scope=(\n  -p aspen-time\n  -p aspen-hlc\n  -p aspen-core\n  -p aspen-coordination\n)\n"
if old not in text:
    raise SystemExit("default_scope block not found")
script_path.write_text(text.replace(old, new, 1))
PY
}

write_config() {
  cat > "$1" <<'EOF'
[tigerstyle]
function_length_max = 100
nesting_max_depth = 4
assertion_density_min_lines = 30
assertion_density_min_assertions = 2
acronym_allowlist = ["IO"]
disabled_lints = [
  "ambient_clock",
  "compound_assertion",
  "contradictory_time",
  "function_length",
  "acronym_style",
  "numeric_units",
  "compound_condition",
  "verified_purity",
  "unjustified_allow",
  "nested_conditionals",
  "assertion_density",
  "multi_lock_ordering",
  "bool_naming",
  "unchecked_division",
  "catch_all_on_enum",
  "platform_dependent_cast",
  "float_for_currency",
  "ambiguous_params",
  "raw_arithmetic_overflow",
  "no_recursion",
]
EOF
}

sandbox=$(mktemp -d /tmp/aspen-lint-precision.XXXXXX)
aspen_worktree="$sandbox/aspen"
tigerstyle_before_worktree="$sandbox/tigerstyle-before"
tigerstyle_after_worktree="$sandbox/tigerstyle-after"
cleanup() {
  set +e
  if [[ -e "$aspen_worktree" ]]; then
    git -C "$aspen_repo" worktree remove --force "$aspen_worktree" >/dev/null 2>&1 || rm -rf "$aspen_worktree"
  fi
  if [[ -e "$tigerstyle_before_worktree" ]]; then
    git -C "$repo_root" worktree remove --force "$tigerstyle_before_worktree" >/dev/null 2>&1 || rm -rf "$tigerstyle_before_worktree"
  fi
  if [[ -e "$tigerstyle_after_worktree" ]]; then
    git -C "$repo_root" worktree remove --force "$tigerstyle_after_worktree" >/dev/null 2>&1 || rm -rf "$tigerstyle_after_worktree"
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT

mkdir -p "$sandbox"
printf '$ git -C %q worktree add --detach %q %q\n' "$aspen_repo" "$aspen_worktree" "$aspen_commit"
git -C "$aspen_repo" worktree add --detach "$aspen_worktree" "$aspen_commit" >/dev/null
printf '$ git -C %q worktree add --detach %q %q\n' "$repo_root" "$tigerstyle_before_worktree" "$tigerstyle_before_commit"
git -C "$repo_root" worktree add --detach "$tigerstyle_before_worktree" "$tigerstyle_before_commit" >/dev/null
printf '$ git -C %q worktree add --detach %q %q\n' "$repo_root" "$tigerstyle_after_worktree" "$tigerstyle_after_commit"
git -C "$repo_root" worktree add --detach "$tigerstyle_after_worktree" "$tigerstyle_after_commit" >/dev/null
printf '$ ln -s %q %q\n' "$aspen_dns_repo" "$sandbox/aspen-dns"
ln -s "$aspen_dns_repo" "$sandbox/aspen-dns"
printf '$ ln -s %q %q\n' "$aspen_plugin_repo" "$sandbox/aspen-wasm-plugin"
ln -s "$aspen_plugin_repo" "$sandbox/aspen-wasm-plugin"

mkdir -p "$aspen_worktree/scripts"
printf '$ cp %q %q\n' "$repo_root/templates/tigerstyle-check.sh" "$aspen_worktree/scripts/tigerstyle-check.sh"
cp "$repo_root/templates/tigerstyle-check.sh" "$aspen_worktree/scripts/tigerstyle-check.sh"
printf '$ cp %q %q\n' "$repo_root/templates/dylint.toml" "$aspen_worktree/dylint.toml"
cp "$repo_root/templates/dylint.toml" "$aspen_worktree/dylint.toml"
printf '$ patch default scope in %q\n' "$aspen_worktree/scripts/tigerstyle-check.sh"
set_default_scope "$aspen_worktree/scripts/tigerstyle-check.sh"
printf '$ patch profile banner in %q\n' "$aspen_worktree/scripts/tigerstyle-check.sh"
set_wrapper_banner "$aspen_worktree/scripts/tigerstyle-check.sh"
printf '$ write precision-comparison config to %q\n' "$aspen_worktree/dylint.toml"
write_config "$aspen_worktree/dylint.toml"

{
  printf '$ bash %s\n' "$script_rel"
  printf '$ git -C %q rev-parse HEAD\n' "$aspen_repo"
  git -C "$aspen_repo" rev-parse HEAD
  printf '$ git -C %q rev-parse %q\n' "$repo_root" "$tigerstyle_before_commit"
  git -C "$repo_root" rev-parse "$tigerstyle_before_commit"
  printf '$ git -C %q rev-parse %q\n' "$repo_root" "$tigerstyle_after_commit"
  git -C "$repo_root" rev-parse "$tigerstyle_after_commit"
  printf '$ pwd\n%s\n' "$sandbox"
  printf '# phase-1 archive: %s\n' "$phase1_archive_rel"
  printf '# phase-2 archive: %s\n' "$phase2_archive_rel"
  printf '# dylint rev: %s\n' "$dylint_rev"
  printf '# comparison scope: %s\n' "$scope_display"
} > "$meta_log"

{
  printf '$ bash %s\n' "$script_rel"
  printf '# tigerstyle commit: %s\n' "$tigerstyle_before_commit"
  printf '# aspen commit: %s\n' "$aspen_commit"
  printf '# phase-1 archive: %s\n' "$phase1_archive_rel"
  printf '# phase-2 archive: %s\n' "$phase2_archive_rel"
  case "$scope_mode" in
    workspace_all_targets)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh --workspace -- --all-targets)\n' \
        "$aspen_worktree" "$tigerstyle_before_worktree" "$dylint_rev" "$sandbox/target-before"
      ;;
    widened_all_targets)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets)\n' \
        "$aspen_worktree" "$tigerstyle_before_worktree" "$dylint_rev" "$sandbox/target-before"
      ;;
    *)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh)\n' \
        "$aspen_worktree" "$tigerstyle_before_worktree" "$dylint_rev" "$sandbox/target-before"
      ;;
  esac
  set +e
  (
    cd "$aspen_worktree"
    case "$scope_mode" in
      workspace_all_targets)
        TIGERSTYLE_PATH="$tigerstyle_before_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-before" \
        scripts/tigerstyle-check.sh --workspace -- --all-targets
        ;;
      widened_all_targets)
        TIGERSTYLE_PATH="$tigerstyle_before_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-before" \
        scripts/tigerstyle-check.sh -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets
        ;;
      *)
        TIGERSTYLE_PATH="$tigerstyle_before_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-before" \
        scripts/tigerstyle-check.sh
        ;;
    esac
  )
  before_status=$?
  set -e
  printf '# exit status: %s\n' "$before_status"
} > "$before_log" 2>&1

{
  printf '$ bash %s\n' "$script_rel"
  printf '# tigerstyle commit: %s\n' "$tigerstyle_after_commit"
  printf '# aspen commit: %s\n' "$aspen_commit"
  printf '# phase-1 archive: %s\n' "$phase1_archive_rel"
  printf '# phase-2 archive: %s\n' "$phase2_archive_rel"
  case "$scope_mode" in
    workspace_all_targets)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh --workspace -- --all-targets)\n' \
        "$aspen_worktree" "$tigerstyle_after_worktree" "$dylint_rev" "$sandbox/target-after"
      ;;
    widened_all_targets)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets)\n' \
        "$aspen_worktree" "$tigerstyle_after_worktree" "$dylint_rev" "$sandbox/target-after"
      ;;
    *)
      printf '$ (cd %q && TIGERSTYLE_PATH=%q TIGERSTYLE_DYLINT_REV=%q CARGO_TARGET_DIR=%q scripts/tigerstyle-check.sh)\n' \
        "$aspen_worktree" "$tigerstyle_after_worktree" "$dylint_rev" "$sandbox/target-after"
      ;;
  esac
  set +e
  (
    cd "$aspen_worktree"
    case "$scope_mode" in
      workspace_all_targets)
        TIGERSTYLE_PATH="$tigerstyle_after_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-after" \
        scripts/tigerstyle-check.sh --workspace -- --all-targets
        ;;
      widened_all_targets)
        TIGERSTYLE_PATH="$tigerstyle_after_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-after" \
        scripts/tigerstyle-check.sh -p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets
        ;;
      *)
        TIGERSTYLE_PATH="$tigerstyle_after_worktree" \
        TIGERSTYLE_DYLINT_REV="$dylint_rev" \
        CARGO_TARGET_DIR="$sandbox/target-after" \
        scripts/tigerstyle-check.sh
        ;;
    esac
  )
  after_status=$?
  set -e
  printf '# exit status: %s\n' "$after_status"
} > "$after_log" 2>&1

python3 "$script_dir/count-aspen-lints.py" \
  "$before_log" \
  "$after_log" \
  "$tigerstyle_before_commit" \
  "$tigerstyle_after_commit" \
  "$aspen_commit" \
  "$phase1_archive_rel" \
  "$phase2_archive_rel" \
  "$script_rel" \
  "$scope_display" > "$comparison_md"

printf 'meta: %s\n' "${meta_log#"$repo_root/"}"
printf 'before transcript: %s\n' "${before_log#"$repo_root/"}"
printf 'after transcript: %s\n' "${after_log#"$repo_root/"}"
printf 'comparison draft: %s\n' "${comparison_md#"$repo_root/"}"
