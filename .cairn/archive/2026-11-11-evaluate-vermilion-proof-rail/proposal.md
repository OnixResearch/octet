# Proposal: Evaluate Vermilion alongside Verus and Aeneas proof rails

## Why

Octet already ships opt-in Verus rails, an Aeneas/Lean consumer contract with
`supported`/`blocked`/`failed` outcomes, proof cards, and artifact
verification. Vermilion keeps Verus's Rust front end and replaces the SMT
backend with explicit Lean theorem obligations. Without a reviewed provider
profile and receipt contract, any comparison between Verus, Vermilion, and
Aeneas would silently conflate different claims, widths, and trusted edges.

The existing addition-bound rail proves a `nat` monotonicity property;
`src/verified/addition.rs` is a `u64` function. That mismatch is a useful pilot
boundary, not a bug report or a Rust refinement proof.

## What Changes

- Add a pinned Vermilion provider profile, typed Nickel contracts, and a
  compatibility matrix with pure and live canaries.
- Add one common semantic contract for a bounded-increment executable contract
  with explicit `value < u64::MAX`, checked overflow, `Option<u64>` results, and
  single-call termination.
- Declare per-route obligations and refinement bridges for Verus, Vermilion,
  and Aeneas; missing bridges remain visible open obligations.
- Add a typed Vermilion receipt and a pure fail-closed validator covering
  source/spec substitution, weakened preconditions, changed overflow semantics,
  `nat`/`u64` confusion, wrong provider, stale generated output, stale proof
  twins, missing or unexpected obligations, unexpected axioms, proof holes,
  interrupted or empty checking, missing dependency audits, tampering, and
  evidence overclaim.
- Add a closed comparison vocabulary with explicit mappings onto the existing
  Aeneas and Vermilion outcome spellings; agreement never merges authority.
- Add a positive contract fixture and a twenty-case negative-control matrix.
- Document the claim/trust matrix, blockers, and a measured go/no-go
  recommendation.

## Impact

- **Standards**: new Nickel profile, contracts, compatibility matrix, pilot
  contract, and generated JSON under `standards/proof-rails/vermilion-lean/`.
- **Code**: opt-in `octet_standards::vermilion` module and focused tests. No
  work is added to starter `cargo octet check`.
- **Evidence**: receipts bind exact source, statement, toolchain, dependency,
  command, artifact, obligation, theorem, and axiom identities. Synthetic
  fixtures are `recorded_only` and cannot admit property evidence.
- **Lifecycle**: policy registry gains two Nickel-authored config families.

## Non-goals

- Do not execute, install, or vendor Vermilion, Lean, Lake, cvc5, Aeneas, or
  Charon in this change.
- Do not claim a live provider run, a measured proof, or a property result.
- Do not prove translator, front-end, kernel, or solver soundness.
- Do not merge Verus, Vermilion, and Aeneas evidence authority or release
  eligibility.
- Do not wire Vermilion into `cargo octet prove`, proof cards, artifact
  verification, or Valence export before the live cohort and refinement
  bridges exist.
