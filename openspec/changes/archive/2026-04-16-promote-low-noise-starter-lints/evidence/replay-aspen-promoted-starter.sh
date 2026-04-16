#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
script_rel=${script_dir#"$repo_root"/}
# Override these if your Aspen clones live somewhere else.
aspen_repo=${ASPEN_REPO:-/home/brittonr/git/aspen}
aspen_dns_repo=${ASPEN_DNS_REPO:-/home/brittonr/git/aspen-dns}
aspen_plugin_repo=${ASPEN_PLUGIN_REPO:-/home/brittonr/git/aspen-wasm-plugin}
phase1_archive=openspec/changes/archive/2026-04-13-aspen-consumer-rollout
phase1_commit=909e5e9fa
evidence_config="$script_dir/aspen-promoted-starter-config.toml"
default_log="$script_dir/aspen-promoted-starter-default.txt"
coordination_log="$script_dir/aspen-promoted-starter-coordination.txt"
counts_log="$script_dir/aspen-promoted-starter-counts.txt"
sandbox=$(mktemp -d /tmp/aspen-promoted-starter.XXXXXX)
default_target_dir="$sandbox/target-default"
coordination_target_dir="$sandbox/target-coordination"
worktree="$sandbox/aspen"
cleanup() {
  if [ -d "$worktree/.git" ] || [ -d "$worktree" ]; then
    git -C "$aspen_repo" worktree remove --force "$worktree" >/dev/null 2>&1 || true
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT INT HUP TERM

require_path() {
  if [ ! -e "$1" ]; then
    echo "missing required path: $1" >&2
    exit 1
  fi
}

require_path "$aspen_repo"
require_path "$aspen_dns_repo"
require_path "$aspen_plugin_repo"
require_path "$repo_root/$phase1_archive"
git -C "$aspen_repo" cat-file -e "$phase1_commit^{commit}"

cargo_tigerstyle_store=$(cd "$repo_root" && nix build --no-link --print-out-paths .#cargo-tigerstyle)
cargo_tigerstyle_bin="$cargo_tigerstyle_store/bin/cargo-tigerstyle"
require_path "$cargo_tigerstyle_bin"

git -C "$aspen_repo" worktree add --detach "$worktree" "$phase1_commit" >/dev/null
ln -s "$aspen_dns_repo" "$sandbox/aspen-dns"
ln -s "$aspen_plugin_repo" "$sandbox/aspen-wasm-plugin"

python3 - "$worktree/Cargo.toml" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text()
section = """
[workspace.metadata.tigerstyle]
default_scope = ["-p", "aspen-time", "-p", "aspen-hlc", "-p", "aspen-core"]
cargo_check_args = ["--lib"]
""".lstrip()
pattern = re.compile(r"\n\[workspace\.metadata\.tigerstyle\]\n(?:[^\[]*\n?)*", re.MULTILINE)
if pattern.search(text):
    text = pattern.sub("\n" + section + "\n", text).rstrip() + "\n"
else:
    text = text.rstrip() + "\n\n" + section + "\n"
path.write_text(text)
PY

python3 - "$evidence_config" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
disabled = [
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
    "unbounded_collection_growth",
    "too_many_parameters",
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
    "unjustified_no_panic_allow",
    "no_recursion",
]
lines = [
    "# Aspen promoted starter profile.",
    "# Phase-1 reference: openspec/changes/archive/2026-04-13-aspen-consumer-rollout/ on Aspen commit 909e5e9fa.",
    "# Enabled starter families: ambient_clock, compound_assertion, contradictory_time, explicit_defaults, unbounded_channel, usize_in_public_api, negated_predicate.",
    "# Still deferred: unbounded_collection_growth, too_many_parameters.",
    "[tigerstyle]",
    "function_length_max = 100",
    "nesting_max_depth = 4",
    "assertion_density_min_lines = 30",
    "assertion_density_min_assertions = 2",
    'acronym_allowlist = ["IO"]',
    "disabled_lints = [",
]
lines.extend(f'  "{lint}",' for lint in disabled)
lines.append("]")
lines.append("")
path.write_text("\n".join(lines))
PY

cp "$evidence_config" "$worktree/dylint.toml"

write_run_log() {
  output_path=$1
  target_dir=$2
  cargo_args=$3

  {
    printf '# phase1_reference: %s\n' "$phase1_archive/"
    printf '# aspen_commit: %s\n' "$phase1_commit"
    printf '# promoted_families: explicit_defaults, unbounded_channel, usize_in_public_api, negated_predicate\n'
    printf '# deferred_families: unbounded_collection_growth, too_many_parameters\n'
    printf '# starter_scope: -p aspen-time -p aspen-hlc -p aspen-core -- --lib\n'
    printf '$ cat %s\n' "$worktree/dylint.toml"
    cat "$worktree/dylint.toml"
    printf '\n$ (cd %s && git rev-parse --short HEAD)\n' "$worktree"
    (cd "$worktree" && git rev-parse --short HEAD)
    printf '\n$ (cd %s && CARGO_TARGET_DIR=%s %s check%s)\n' "$worktree" "$target_dir" "$cargo_tigerstyle_bin" "$cargo_args"
    (cd "$worktree" && CARGO_TARGET_DIR="$target_dir" "$cargo_tigerstyle_bin" check $cargo_args)
  } > "$output_path" 2>&1
}

write_run_log "$default_log" "$default_target_dir" ""
write_run_log "$coordination_log" "$coordination_target_dir" " -p aspen-coordination -- --lib"

{
  printf '$ bash %s/replay-aspen-promoted-starter.sh\n' "$script_rel"
  printf '$ python3 %s/check-aspen-promoted-starter.py %s\n' "$script_rel" "$script_rel"
  python3 "$script_dir/check-aspen-promoted-starter.py" "$script_dir"
} > "$counts_log"

printf 'saved %s\n' "$evidence_config"
printf 'saved %s\n' "$default_log"
printf 'saved %s\n' "$coordination_log"
printf 'saved %s\n' "$counts_log"
