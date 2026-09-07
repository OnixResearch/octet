#!/bin/sh
set -eu
umask 077
[ "$#" -eq 4 ] || { echo 'usage: verify.sh OUT DRIVER LIBRARY COMPILER' >&2; exit 2; }
out=$1 driver=$2 library=$3 compiler=$4
for input in "$@"; do case "$input" in /*) ;; *) exit 2;; esac; done
self=$(CDPATH= cd "$(dirname "$0")" && pwd -P)
test -x "$driver" && test -f "$library" && test -x "$compiler/bin/rustc"
mkdir "$out"
mkdir "$out/home" "$out/lints"
ln -s "$library" "$out/lints/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"
cd "$out"
b3sum "$driver" "$library" "$compiler/bin/rustc" "$self"/fixtures/*.rs > before.txt
probe() {
    name=$1 source=$2 tool=$3 expected=$4
    set +e
    env -i PATH="$compiler/bin:/run/current-system/sw/bin" HOME="$out/home" LD_LIBRARY_PATH="$compiler/lib" \
        CARGO_PRIMARY_PACKAGE=1 DYLINT_NO_DEPS=1 \
        DYLINT_LIBS="[\"$out/lints/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so\"]" \
        DYLINT_RUSTFLAGS='-D unknown_lints -D borrowed_argument_types' \
        "$tool" --sysroot "$compiler" --crate-type lib --edition 2024 --emit=metadata \
        -o "$name.rmeta" "$self/fixtures/$source.rs" > "$name.stdout" 2> "$name.stderr"
    status=$?
    set -e
    printf '%s\n' "$status" > "$name.exit"
    test "$status" -eq "$expected"
    if grep -E 'internal compiler error|thread .*panicked' "$name.stderr"; then exit 1; fi
    printf '%s: expected exit%s\n' "$name" "$status"
}
probe growth-rustc growth "$compiler/bin/rustc" 0
probe slice slice "$compiler/bin/rustc" 1
grep -q 'E0599' slice.stderr
probe growth growth "$driver" 0
test ! -s growth.stderr
probe candidates candidates "$driver" 101
test "$(grep -c 'error: parameter uses' candidates.stderr)" -eq 7
b3sum "$driver" "$library" "$compiler/bin/rustc" "$self"/fixtures/*.rs > after.txt
cmp before.txt after.txt
printf '%s\n' 'PASS: four probes; seven candidates retained; no complete-gate or startup claim.'
