# Proposal: Raise the Aeneas consumer proof budget

## Why

The external-consumer Aeneas job runs the pinned Aeneas/Lean closure with the
pinned Mathlib source. The current 900,000 ms budget expires while the Lean
stage is still building Mathlib (1,325 of 1,632 modules), so the full flake
check fails at `mantle-aeneas-consumer-job` even though the proof stages are
progressing.

## What Changes

- Raise the external-consumer resource policy timeout and the provider authority
  maximum to 1,800,000 ms.
- Raise `aeneas_timeout_ms` in the proof rail manifest so the selected Aeneas
  rail agrees with the consumer resource policy.
- Regenerate the consumer profile and update its identity and expected receipt.
- Update the Aeneas fixture verification, proof cards, and receipt timeout, and
  refresh the receipt identity that binds the changed consumer policy.
- Refresh the advisory self-check baseline for the source line and identity
  shifts introduced by the budget change.
- Record the cold-build observation and the unchanged process, environment, and
  network bounds.

## Non-claims

The budget change does not prove the theorem, make the Lean build deterministic,
or weaken the process, environment, or network restrictions. It extends only
the bounded wall-clock budget for the cold pinned Lean closure.
