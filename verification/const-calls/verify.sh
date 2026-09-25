#!/bin/sh
set -eu
out=${1:?OUT} driver=${2:?DRIVER} library=${3:?LIBRARY} compiler=${4:?COMPILER}
for path in "$out" "$driver" "$library" "$compiler"; do
  case "$path" in /*) ;; *) exit 2;; esac
done
test ! -e "$out"
umask 077
mkdir -p "$out/home"
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
sh "$here/../const-allocation/verify.sh" "$out/prior" "$driver" "$library" "$compiler"
cp "$here/fixtures/"*.rs "$out/"
alias="$out/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"
ln -s "$library" "$alias"
for name in display local locator free method trait vector inherent constant constructor; do
  expected=0
  case "$name" in inherent|constant|constructor) expected=101;; esac
  env -i PATH="$compiler/bin:/run/current-system/sw/bin" LD_LIBRARY_PATH="$compiler/lib" \
    "$compiler/bin/rustc" --crate-name probe --crate-type lib --edition 2024 --emit=metadata \
    -o "$out/$name-plain.rmeta" "$out/$name.rs" > "$out/$name-plain.stdout" 2> "$out/$name-plain.stderr"
  set +e
  env -i PATH="$compiler/bin:/run/current-system/sw/bin" HOME="$out/home" LD_LIBRARY_PATH="$compiler/lib" \
    DYLINT_LIBS="[\"$alias\"]" DYLINT_NO_DEPS=1 CARGO_PRIMARY_PACKAGE=1 \
    DYLINT_RUSTFLAGS='-D unknown_lints -D missing_const_fn' \
    "$driver" --sysroot "$compiler" --crate-name probe --crate-type lib --edition 2024 --emit=metadata \
    -o "$out/$name.rmeta" "$out/$name.rs" > "$out/$name.stdout" 2> "$out/$name.stderr"
  status=$?
  set -e
  printf '%s\n' "$status" > "$out/$name.exit"
  if grep -q 'internal compiler error' "$out/$name.stderr"; then exit 1; fi
  test "$status" -eq "$expected"
  if test "$expected" -eq 101; then grep -q 'private helper appears const-compatible' "$out/$name.stderr"; fi
  printf '%s: plain rustc 0, driver %s\n' "$name" "$status"
done
printf '%s\n' 'Ten prior controls plus ten new paired controls passed. Diagnostic only; no startup authority.'
