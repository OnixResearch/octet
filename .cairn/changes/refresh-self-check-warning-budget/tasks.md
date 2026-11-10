# Tasks: Refresh the self-check warning budget

- [x] [serial] Regenerate the baseline with the checked write-baseline path. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Update the accepted identity and warning totals. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Run the canonical self-check with the refreshed baseline. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Run the Nix self-check and the full flake check. r[octet.self_check.warning_budget_refresh]
  - self-check: `nix build path:$PWD#checks.x86_64-linux.cargo-octet-self-check --no-link -L` (exit 0; evidence: `evidence/nix-self-check.json`).
  - full flake: `nix flake check -L` exited 1; the only reported failed attribute was `checks.x86_64-linux.mantle-aeneas-consumer-job` (Aeneas Lean stage exceeded its 900000 ms proof budget). Rerunning that attribute failed identically. All checks that completed, including the self-check, passed. Evidence: `evidence/full-flake-check.json`.

## Verification Coverage

- `Scenario: Refreshed baseline has no stale or new findings` -> canonical self-check
- `Scenario: Counts match the baseline` -> Nix self-check derivation
- `Scenario: Warning budget remains advisory` -> design/proposal non-claims
