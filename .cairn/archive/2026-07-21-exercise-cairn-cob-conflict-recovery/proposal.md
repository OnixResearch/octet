## Why

Octet's first signed COB adoption pilot proves preview-first import, update, explicit Git transport, and conflict-free reload. It does not prove that a downstream operator can safely detect two authorized updates from one basis, inspect each exact conflicting value, refuse ordinary mutation, explicitly resolve the complete head set, and retain the losing revision.

## What Changes

- Exercise one real Octet native change through signed creator and delegate updates from the same imported COB basis.
- Transport only explicit Git branch and namespaced COB refs into a local convergence store and retain bounded conflict-inspection receipts.
- Preview and execute an explicit complete-basis resolution, then reload and materialize the selected result from a third local store.
- Add a typed Nickel manifest, retained Git bundle, positive and tampering-negative Rust validation, and a hermetic Nix gate.

## Impact

- **Files**: `fixtures/cairn-cob-conflict-pilot/`, `tests/cairn_cob_conflict_pilot.rs`, `docs/cairn-cob-collaboration.md`, `standards/policies/registry.ncl`, `flake.nix`, and this native Cairn change package.
- **Testing**: focused Rust evidence tests, Nickel freshness, Git bundle and ancestry checks, Cairn lifecycle gates, Octet policy validation, Nextest, Clippy, and applicable Nix checks.
- **Runtime**: no Octet product dependency, Radicle runtime, profile, node, transport, synchronization, or automatic conflict-resolution behavior changes.

## Non-Goals

This change does not prove network publication, authentication, remote authorization, availability, durability, synchronization, distributed convergence, semantic merge correctness, source correctness, whole-program correctness, or release eligibility. It does not add cross-version fixtures or make the deterministic fixture keys production authority.
