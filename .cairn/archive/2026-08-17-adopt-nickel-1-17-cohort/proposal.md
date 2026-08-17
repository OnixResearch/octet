# Proposal: Adopt Nickel 1.17 for checked-path configuration

## Why

Octet configuration checks currently resolve to Nickel `1.16.0`. The stack checked path must evaluate policy and evidence configuration with Nickel `1.17.0`.

## What Changes

- Pin Nickel `1.17.0` at upstream commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.
- Use the pin for lint policy, standards profiles, proof rails, fixtures, and Nix checks.
- Add positive and negative compatibility fixtures for contracts, imports, merges, exports, and diagnostics.
- Record the evaluator identity in checked-path evidence where the evaluator affects the result.

## Impact

Octet configuration and evidence rails will use the stack Nickel cohort. Octet retains Rust checking policy and evidence claims.

## Dependencies

This change uses the upstream Nickel `1.17.0` release.

## Non-goals

- Do not change lint or proof meaning only because diagnostics change.
- Do not update unrelated dependencies only to obtain Nickel.
- Do not promote syntax checks into semantic proof.
