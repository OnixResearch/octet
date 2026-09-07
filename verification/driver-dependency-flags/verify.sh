#!/bin/sh
# Uses existing tools only. All inputs and the fresh output root must be absolute.
set -eu
umask 077
[ "$#" -eq 7 ] || { echo 'usage: verify.sh OUT DRIVER CLI RUNTIME COMPILER LIBRARY HOOK' >&2; exit 2; }
out=$1 driver=$2 cli=$3 runtime=$4 compiler=$5 library=$6 hook=$7
for input in "$@"; do
  case "$input" in /*) ;; *) echo 'absolute inputs required' >&2; exit 2;; esac
done
self=$(CDPATH= cd "$(dirname "$0")" && pwd -P)
test -x "$driver" && test -x "$cli/bin/cargo-octet"
test -x "$compiler/bin/rustc" && test -f "$library" && test -f "$hook"
mkdir "$out"
mkdir "$out/home" "$out/cargo-home" "$out/tmp" "$out/lints"
cp -R "$self/fixture" "$out/source"
toolchain=nightly-2026-03-21-x86_64-unknown-linux-gnu
alias=$out/lints/liboctet@$toolchain.so
ln -s "$library" "$alias"
cd "$out/source"
workspace_case() {
  name=$1
  shift
  env -i PATH="$runtime/bin:$cli/bin:/run/current-system/sw/bin" \
    HOME="$out/home" CARGO_HOME="$out/cargo-home" TMPDIR="$out/tmp" \
    CARGO_TARGET_DIR="$out/target-$name" CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true \
    RUSTUP_TOOLCHAIN="$toolchain" LD_LIBRARY_PATH="$compiler/lib" \
    TIGERSTYLE_TOOLCHAIN="$toolchain" TIGERSTYLE_DRIVER_PATH="$driver" \
    TIGERSTYLE_LINT_LIB="$alias" OCTET_PRECOMMIT_USE_INSTALLED=true \
    /run/current-system/sw/bin/sh "$hook" --artifact-dir "$out/artifacts-$name" "$@" \
    > "$out/$name.stdout.log" 2> "$out/$name.stderr.log"
  grep -q '"status": "clean"' "$out/artifacts-$name/status.json"
  grep -q '"total_findings": 0,' "$out/artifacts-$name/status.json"
  printf '%s: exit0, clean, zero findings\n' "$name"
}
workspace_case dependency
workspace_case workspace --workspace
workspace_case core -p registration-core
cd "$self/direct"
direct_case() {
  name=$1 source=$2 expected=$3 no_deps=$4 libs=$5 flags=$6
  shift 6
  set +e
  env -i PATH="$compiler/bin:/run/current-system/sw/bin" HOME="$out/home" \
    LD_LIBRARY_PATH="$compiler/lib" DYLINT_LIBS="[\"$libs\"]" DYLINT_NO_DEPS="$no_deps" \
    DYLINT_RUSTFLAGS="$flags" "$@" "$driver" --sysroot "$compiler" \
    --crate-name registration --crate-type lib --edition 2024 --emit=metadata \
    -o "$out/$name.rmeta" "$source" > "$out/$name.stdout.log" 2> "$out/$name.stderr.log"
  status=$?
  set -e
  printf '%s\n' "$status" > "$out/$name.exit"
  test "$status" -eq "$expected"
  printf '%s: expected exit%s\n' "$name" "$status"
}
flags='-D unknown_lints -D no_todo'
direct_case direct-dependency clean.rs 0 1 "$alias" "$flags"
direct_case direct-primary clean.rs 0 1 "$alias" "$flags" CARGO_PRIMARY_PACKAGE=1
direct_case direct-bare bare.rs 101 1 "$alias" "$flags" CARGO_PRIMARY_PACKAGE=1
grep -q 'marks unfinished production code' "$out/direct-bare.stderr.log"
direct_case deps-enabled-bare bare.rs 101 0 "$alias" "$flags"
grep -q 'marks unfinished production code' "$out/deps-enabled-bare.stderr.log"
direct_case primary-unknown clean.rs 101 1 "$alias" '-D unknown_lints -D nonexistent_probe_lint' CARGO_PRIMARY_PACKAGE=1
grep -q 'unknown lint: `nonexistent_probe_lint`' "$out/primary-unknown.stderr.log"
direct_case primary-missing-library clean.rs 101 1 "$out/libmissing@$toolchain.so" "$flags" CARGO_PRIMARY_PACKAGE=1
grep -q 'could not load library' "$out/primary-missing-library.stderr.log"
printf '%s\n' 'PASS: workspace selection and six direct-driver controls; no production admission.'
