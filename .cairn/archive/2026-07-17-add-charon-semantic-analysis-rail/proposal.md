## Why

Octet's selected-source effect-closure rail intentionally uses conservative source-token analysis and exact reviewed dependency summaries. That boundary is useful and fail-closed, but it cannot provide resolved Rust item, trait, call, dependency, control-flow, or compiler-target facts. Charon already extracts a typed semantic Rust representation for the Aeneas provider, so Octet can reuse the pinned extraction boundary for a separate bounded analysis rail without pretending the existing source-token closure is semantic Rust resolution.

The rail must remain optional and evidence-oriented. Charon is alpha, has unsupported Rust features, and can emit partial/errorful output under some settings. A semantic snapshot is therefore useful only when the effective extraction contract, exact selected items, target metadata, and output bytes are bound and validated.

## What Changes

- Add an opt-in `charon-semantic` evidence rail separate from `cargo octet check`, strict-profile admission, source-token effect closure, and formal-proof providers.
- Reuse the reviewed Charon package and extraction-contract machinery established by `add-aeneas-lean-proof-rail`, including exact pin, effective serialized options, target metadata, non-errorful output, and borrow-check preservation.
- Add a pure semantic projection and analysis core over already-loaded Charon artifacts for exact selected items, resolved local calls, item dependencies, trait-resolution facts, structured/unstructured control-flow metadata, source spans, opacity/unsupported flags, and a closed conservative effect-candidate vocabulary.
- Emit a deterministic comparison report against existing source-token closure without replacing, silently upgrading, or merging either evidence authority.
- Bind stack-owned artifacts and receipts with BLAKE3 and keep semantic inventory/comparison results recorded-only unless a later reviewed policy defines a narrower promotion.
- Use `mechanized-llbc` and `charon-rudra` only as exact recorded design/fixture references; do not import their proof or implementation authority.
- Add a small exact-item canary corpus plus negative fixtures for partial extraction, `skip_borrowck`, target/profile drift, unsupported constructs, opaque items, missing dependencies, stale spans, malformed graphs, reference drift, and overclaiming receipts.

## Impact

- **Core**: pure Charon snapshot DTO validation, semantic projection, bounded analysis, source-token comparison, deterministic diagnostics, and receipt construction.
- **Shell**: pinned argv execution and confined artifact loading only; no semantic decisions in process or filesystem code.
- **Policy**: typed Nickel profiles for exact source selection, extraction settings, supported fact vocabulary, limits, reference metadata, and non-claims.
- **Compatibility**: no default `cargo octet check` or `cargo octet admit --closure` behavior changes; no source-token evidence is relabeled as compiler-semantic evidence.
- **Testing**: positive exact-item fixtures and paired negative fixtures for every fail-closed boundary.
- **Claims**: the rail records bounded facts from one exact Charon snapshot. It does not prove Charon or rustc correctness, complete Rust coverage, runtime reachability, effect absence, semantic equivalence, whole-program correctness, formal proof, or release eligibility.
