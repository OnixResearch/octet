#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
script_rel=${script_dir#"$repo_root"/}
aspen_repo=/home/brittonr/git/aspen
dns_repo=/home/brittonr/git/aspen-dns
plugin_repo=/home/brittonr/git/aspen-wasm-plugin
meta_log="$script_dir/aspen-new-lints-meta.txt"
sandbox=$(mktemp -d /tmp/aspen-new-lints.XXXXXX)
target_dir="$sandbox/target"
cleanup() {
  if [ -d "$sandbox/aspen/.git" ] || [ -d "$sandbox/aspen" ]; then
    git -C "$aspen_repo" worktree remove --force "$sandbox/aspen" >/dev/null 2>&1 || true
  fi
  rm -rf "$sandbox"
}
trap cleanup EXIT INT HUP TERM

cargo_tigerstyle_store=$(cd "$repo_root" && nix build --no-link --print-out-paths .#cargo-tigerstyle)
cargo_tigerstyle_bin="$cargo_tigerstyle_store/bin/cargo-tigerstyle"

require_path() {
  if [ ! -e "$1" ]; then
    echo "missing required path: $1" >&2
    exit 1
  fi
}

require_path "$aspen_repo"
require_path "$dns_repo"
require_path "$plugin_repo"
require_path "$cargo_tigerstyle_bin"

git -C "$aspen_repo" worktree add --detach "$sandbox/aspen" HEAD >/dev/null
ln -s "$dns_repo" "$sandbox/aspen-dns"
ln -s "$plugin_repo" "$sandbox/aspen-wasm-plugin"

python3 - "$sandbox/aspen/Cargo.toml" <<'PY'
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

cat > "$meta_log" <<EOF
$ bash $script_rel/replay-aspen-new-lints.sh
aspen_head: $(git -C "$aspen_repo" rev-parse HEAD)
aspen_head_short: $(git -C "$aspen_repo" rev-parse --short HEAD)
cargo_tigerstyle_store: $cargo_tigerstyle_store
cargo_tigerstyle_bin: $cargo_tigerstyle_bin
sandbox_root: $sandbox
default_scope: -p aspen-time -p aspen-hlc -p aspen-core -- --lib
coordination_scope: -p aspen-coordination -- --lib
EOF

write_config() {
  lint_name=$1
  config_path=$2
  python3 - "$lint_name" "$config_path" <<'PY'
from pathlib import Path
import sys

target = sys.argv[1]
path = Path(sys.argv[2])
all_lints = [
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
    "explicit_defaults",
    "unbounded_channel",
    "unbounded_collection_growth",
    "usize_in_public_api",
    "too_many_parameters",
    "negated_predicate",
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
    "no_unwrap",
    "no_panic",
    "unjustified_no_panic_allow",
    "no_recursion",
]
disabled = [lint for lint in all_lints if lint != target]
lines = [
    "# Aspen probe config for {} only.".format(target),
    "[tigerstyle]",
    "function_length_max = 100",
    "nesting_max_depth = 4",
    "assertion_density_min_lines = 30",
    "assertion_density_min_assertions = 2",
    'acronym_allowlist = ["IO"]',
    "disabled_lints = [",
]
lines.extend('  "{}",'.format(lint) for lint in disabled)
lines.append("]")
lines.append("")
path.write_text("\n".join(lines))
PY
}

run_probe() {
  lint_name=$1
  default_log="$script_dir/aspen-$lint_name-default.txt"
  coord_log="$script_dir/aspen-$lint_name-coordination.txt"
  config_snapshot="$script_dir/config-$lint_name.toml"

  write_config "$lint_name" "$config_snapshot"
  cp "$config_snapshot" "$sandbox/aspen/dylint.toml"

  {
    printf '$ cat %q\n' "$sandbox/aspen/dylint.toml"
    cat "$sandbox/aspen/dylint.toml"
    printf '\n$ (cd %q && CARGO_TARGET_DIR=%q %q check)\n' "$sandbox/aspen" "$target_dir" "$cargo_tigerstyle_bin"
    (cd "$sandbox/aspen" && CARGO_TARGET_DIR="$target_dir" "$cargo_tigerstyle_bin" check)
  } > "$default_log" 2>&1

  {
    printf '$ cat %q\n' "$sandbox/aspen/dylint.toml"
    cat "$sandbox/aspen/dylint.toml"
    printf '\n$ (cd %q && CARGO_TARGET_DIR=%q %q check -p aspen-coordination -- --lib)\n' "$sandbox/aspen" "$target_dir" "$cargo_tigerstyle_bin"
    (cd "$sandbox/aspen" && CARGO_TARGET_DIR="$target_dir" "$cargo_tigerstyle_bin" check -p aspen-coordination -- --lib)
  } > "$coord_log" 2>&1
}

for lint_name in \
  explicit_defaults \
  unbounded_channel \
  unbounded_collection_growth \
  usize_in_public_api \
  too_many_parameters \
  negated_predicate

do
  run_probe "$lint_name"
done

python3 "$script_dir/check-aspen-new-lints.py" "$script_dir" > "$script_dir/validation-counts.txt"

printf 'saved %s\n' "$meta_log"
for lint_name in \
  explicit_defaults \
  unbounded_channel \
  unbounded_collection_growth \
  usize_in_public_api \
  too_many_parameters \
  negated_predicate

do
  printf 'saved %s\n' "$script_dir/config-$lint_name.toml"
  printf 'saved %s\n' "$script_dir/aspen-$lint_name-default.txt"
  printf 'saved %s\n' "$script_dir/aspen-$lint_name-coordination.txt"
done
printf 'saved %s\n' "$script_dir/validation-counts.txt"
