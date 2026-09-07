#!/bin/sh
# Exact-marker probes only; absolute inputs and fresh OUT are required.
set -eu
umask 077
[ "$#" -eq 5 ] || { echo 'usage: verify.sh OUT DRIVER LIBRARY COMPILER NICKEL' >&2; exit 2; }
out=$1 driver=$2 library=$3 compiler=$4 nickel=$5
for input in "$@"; do case "$input" in /*) ;; *) exit 2;; esac; done
self=$(CDPATH= cd "$(dirname "$0")" && pwd -P)
test -x "$driver" && test -f "$library" && test -x "$compiler/bin/rustc" && test -x "$nickel"
mkdir "$out"
mkdir "$out/home" "$out/lints"
ln -s "$library" "$out/lints/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"
cd "$out"
b3sum "$driver" "$library" "$compiler/bin/rustc" "$nickel" "$self"/*.rs "$self"/*.ncl > before.txt
"$nickel" --version > nickel-version.txt
custom=$("$nickel" export --format toml "$self/custom.ncl")
empty=$("$nickel" export --format toml "$self/empty.ncl")
printf '%s\n' "$custom" > custom.toml
printf '%s\n' "$empty" > empty.toml
config=''
probe() {
    name=$1 source=$2 tool=$3 expected=$4
    shift 4
    set +e
    env -i PATH="$compiler/bin:/run/current-system/sw/bin" HOME="$out/home" LD_LIBRARY_PATH="$compiler/lib" \
        CARGO_PRIMARY_PACKAGE=1 DYLINT_NO_DEPS=1 DYLINT_TOML="$config" \
        DYLINT_LIBS="[\"$out/lints/liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so\"]" \
        DYLINT_RUSTFLAGS='-D unknown_lints -D fragile_exhaustive_enum_match -D state_machine_exhaustiveness' \
        "$tool" --sysroot "$compiler" --crate-type lib --edition 2024 --emit=metadata \
        -o "$name.rmeta" "$self/$source.rs" "$@" > "$name.stdout" 2> "$name.stderr"
    status=$?
    set -e
    printf '%s\n' "$status" > "$name.exit"
    test "$status" -eq "$expected"
    if grep -E 'internal compiler error|thread .*panicked' "$name.stderr"; then exit 1; fi
    if [ "$expected" -eq 0 ]; then test ! -s "$name.stderr"; fi
    printf '%s: expected exit%s\n' "$name" "$status"
}
probe plain enum "$compiler/bin/rustc" 0
probe unmarked enum "$driver" 101
grep -q 'same-crate enum' unmarked.stderr
probe real enum "$driver" 0 --cfg real
probe fsm-marker enum "$driver" 0 --cfg fsm
for variant in doc wrong payload prefix nested; do
    probe "$variant" enum "$driver" 101 --cfg "$variant"
    grep -q 'same-crate enum' "$variant.stderr"
done
probe future enum "$compiler/bin/rustc" 1 --cfg real --cfg future
grep -q 'E0004' future.stderr
config=$custom
probe custom-positive enum "$driver" 0 --cfg wrong
probe custom-default-rejected enum "$driver" 101 --cfg real
grep -q 'same-crate enum' custom-default-rejected.stderr
config=$empty
probe empty-rejected enum "$driver" 101 --cfg real
grep -q 'same-crate enum' empty-rejected.stderr
config=''
probe fsm-valid fsm "$driver" 0 --cfg real_core --cfg real_enums
for variant in doc_enums wrong_enums; do
    probe "$variant" fsm "$driver" 101 --cfg real_core --cfg "$variant"
    grep -q 'state and event enums must carry' "$variant.stderr"
done
probe fsm-wildcard fsm "$driver" 101 --cfg real_core --cfg real_enums --cfg wildcard
grep -q 'wildcard' fsm-wildcard.stderr
probe doc-core-not-opt-in fsm "$driver" 0 --cfg doc_core --cfg real_enums --cfg wildcard
probe wrong-core-not-opt-in fsm "$driver" 0 --cfg wrong_core --cfg real_enums --cfg wildcard
b3sum "$driver" "$library" "$compiler/bin/rustc" "$nickel" "$self"/*.rs "$self"/*.ncl > after.txt
cmp before.txt after.txt
printf '%s\n' 'PASS: 19 probes; exact marker admission; no startup authority.'
