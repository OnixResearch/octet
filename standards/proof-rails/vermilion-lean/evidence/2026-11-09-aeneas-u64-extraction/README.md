# Aeneas extraction of the u64 checked-increment pilot

Date: 2026-11-09

This is the Aeneas/Charon extraction step for the plain-Rust pilot
`increment_checked(value: u64) -> Option<u64>`. It uses the pinned provider
closure (`Aeneas bc2eb6f9`, `Charon cb50ff16`).

## Commands

```console
charon cargo --preset aeneas --dest-file llbc.llbc
aeneas -backend lean -emit-json -dest . -no-progress-bar llbc.llbc
```

Observed result: Charon extracted the crate and Aeneas generated `Llbc.lean`
and `translation.json`.

## Generated model

```lean
def increment_checked (value : Std.U64) : Result (Option Std.U64) := do
  if value < core.num.U64.MAX
  then let i ← value + 1#u64
       ok (some i)
  else ok none
```

## Open obligation

The generated model is extracted, but the universal Lean theorem that relates
this model to the checked contract (`value < u64::MAX` implies
`Some(value + 1)`, and `None` at the maximum) has not been checked yet. The
Aeneas Lean library build and theorem remain the next step. No refinement,
provider-soundness, whole-program, or release claim follows from extraction
alone.
