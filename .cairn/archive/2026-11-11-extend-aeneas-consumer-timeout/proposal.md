# Proposal: Extend the Aeneas consumer proof budget

## Why

The external-consumer Lean stage now reaches 1,549 of 1,632 Mathlib modules at
the 1,800,000 ms budget on the current host and is terminated before the final
modules. The cold pinned Lean closure needs a larger bounded wall-clock budget.

## What Changes

- Raise the external-consumer timeout to 2,700,000 ms.
- Raise the provider authority maximum and the selected Aeneas rail timeout to
  match.
- Raise the live-provider test budgets.
- Regenerate the consumer profile and update its identity, expected receipt,
  and fixture timeout fields.

## Non-claims

The budget extension does not prove the theorem, make the Lean build faster or
deterministic, or weaken process, environment, output, or network bounds. It
only extends the bounded wall-clock allowance for the cold pinned Lean closure.
