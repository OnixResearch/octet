## Why

Octet owns checked Rust rails and deterministic receipts, but it has no reviewed
contract for checking a supplied four-version merge candidate. The existing
Charon semantic rail is a one-item, recorded-only analysis and comparison path;
its receipts retain recorded-only roles and refuse merged parent authority.
Relabeling Charon comparisons, function hashes, lint results, or tests as merge
proofs would overstate them.

Verified Three-Way Program Merge (OOPSLA 2018) motivates a per-observable
preservation relation: for the same admissible input, a merge must match each
variant that changed an observation, and must preserve the base where neither
variant changed it. Incompatible changes can make the relation unsatisfiable.
The relation is sufficient but can reject intentional integrations, and it does
not reconstruct developer intent. SafeMerge-style checking consumes a supplied
candidate; it does not synthesize the merge.

## What Changes

- Add a reviewed, versioned profile for a bounded four-version semantic merge
  relation over an explicit deterministic Rust subset.
- Define exact observables, preconditions, arithmetic and abnormal outcomes,
  memory/aliasing assumptions, dependency closure, and explicit exclusions.
- Define four-version item/interface matching and role-tagged O/A/B/M source and
  semantic-subject identities. Renames, signature changes, changed
  dependencies, and missing items require validated mappings or an
  `unsupported` result.
- Define deterministic outcomes: `verified`, `counterexample`, `unknown`,
  `unsupported`, `timeout`, and `tool-error`. A solver warning or failure to
  prove is not a semantic conflict.
- Add an opt-in pure-core prototype crate with a typed scalar IR, bounded
  evaluator, relation checker, deterministic receipt construction and
  validation, and an injected backend contract.
- Add an executed fixture corpus for identical and independent changes,
  equivalent edits with different syntax, incompatible changes, lost changes,
  and text-clean semantic interference.
- Add paired negative controls for missing observables, vacuous preconditions,
  changed callees, wrong bindings, swapped roles, stale and tampered receipts,
  unsupported constructs, domain overflow, interrupted verification, and
  backend failure.
- Compare the executed corpus against an independently authored expectation
  table and report precision, coverage, deterministic work, and unsupported or
  unknown counts.
- Record a go/no-go decision: go for the bounded prototype, no-go for production
  admission or any claim of Rust source-to-IR correspondence.
- Preserve Pijul native change identities as optional recorded-only provenance.

## Impact

- **Files**: new `crates/octet-merge-rail/` prototype crate and tests, new
  `docs/semantic-merge-verification-rail.md`, `CHANGELOG.md`, root
  `Cargo.toml`, `Cargo.lock`, and the Cairn delta specification.
- **Testing**: focused crate tests for the positive corpus and negative
  controls, workspace `cargo check`, formatting, and Clippy for the checked
  packages.
- **Boundary**: the prototype checks a supplied candidate over declared finite
  domains only. It does not synthesize or apply a merge, prove whole-program
  correctness, prove semantic equivalence, establish release authority, or
  transfer trust to providers. Its typed IR is authored by hand, so Rust
  source-to-IR correspondence is unproven.
- **Ordering**: the profile, executable prototype, fixtures, and non-claims land
  together so no consumer can read the bounded experiment as a production
  merge verifier.

## Out of Scope

- No Rust frontend extraction, no Universal or symbolic backend, and no solver.
- No production `cargo octet check` wiring or gate authority.
- No concurrency, unsafe, FFI, macro, dispatch, memory, or external-effect
  modeling.
- No imported performance, precision, or soundness claim from the paper.
- No change to Charon semantic rail roles, receipts, or authority.
