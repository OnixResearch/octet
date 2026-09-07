#!/bin/sh
# Diagnostic controls only. No tool builds or production admission.
set -eu
out=${1:?OUT} driver=${2:?DRIVER} library=${3:?LIBRARY_FILE} compiler=${4:?COMPILER_PACKAGE}
for path in "$out" "$driver" "$library" "$compiler"; do
  case "$path" in /*) ;; *) echo 'absolute paths required' >&2; exit 2;; esac
done
test ! -e "$out"
mkdir -p "$out/home"
fixtures=$(CDPATH= cd -- "$(dirname -- "$0")/fixtures" && pwd)
cp "$fixtures/"*.rs "$out/"
alias="$out/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"
ln -s "$library" "$alias"
run_case() {
  name=$1 expected=$2 flags=$3
  set +e
  env -i PATH="$compiler/bin:/run/current-system/sw/bin" HOME="$out/home" \
    LD_LIBRARY_PATH="$compiler/lib" DYLINT_LIBS="[\"$alias\"]" DYLINT_NO_DEPS=1 \
    CARGO_PRIMARY_PACKAGE=1 DYLINT_RUSTFLAGS="$flags" \
    "$driver" --sysroot "$compiler" --crate-name probe --crate-type lib \
    --edition 2024 --emit=metadata -o "$out/$name.rmeta" "$out/$name.rs" \
    > "$out/$name.stdout" 2> "$out/$name.stderr"
  status=$?
  set -e
  printf '%s\n' "$status" > "$out/$name.exit"
  if grep -q 'internal compiler error' "$out/$name.stderr"; then
    echo "$name: compiler crash is not a lint result" >&2
    exit 1
  fi
  test "$status" -eq "$expected"
  printf '%s: expected exit %s\n' "$name" "$status"
}
flags='-D unknown_lints -D missing_const_fn'
for name in nested qualified alias constant; do run_case "$name" 0 "$flags"; done
for name in positive text custom associated; do
  run_case "$name" 101 "$flags"
  grep -q 'private helper appears const-compatible' "$out/$name.stderr"
done
run_case bare 101 '-D unknown_lints -D no_todo'
grep -q 'marks unfinished production code' "$out/bare.stderr"
set +e
env -i PATH="$compiler/bin:/run/current-system/sw/bin" LD_LIBRARY_PATH="$compiler/lib" \
  "$compiler/bin/rustc" --crate-name probe --crate-type lib --edition 2024 \
  --emit=metadata -o "$out/illegal.rmeta" "$out/illegal.rs" \
  > "$out/illegal.stdout" 2> "$out/illegal.stderr"
status=$?
set -e
printf '%s\n' "$status" > "$out/illegal.exit"
test "$status" -eq 1
grep -q 'error\[E0015\]' "$out/illegal.stderr"
printf '%s\n' 'illegal: rustc rejects const formatting; ten diagnostic controls passed.'
