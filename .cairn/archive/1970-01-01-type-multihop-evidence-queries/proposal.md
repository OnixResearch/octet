## Why

The current `query_evidence_index` query matches exact record, source, requirement, policy, hash, and provenance identities, then adds one linked-record expansion per original match. It never expands a newly selected record. A changed source or policy revision therefore cannot be followed through several explicitly typed relationships in one review query.

The evidence index model already carries parent links, provenance source references, shared requirement/lint/policy labels, content hashes, and fingerprints. Those are not interchangeable. A shared requirement or policy ID is relatedness only, a content identity match is not proof, and the model carries no asserted dependency/support field at all. Any multi-hop experiment must keep those distinctions visible in its output.

## What Changes

- Add an opt-in, bounded typed multi-hop experiment over an already-built evidence codebase index.
- Preserve the exact index hash, normalization version, record occurrences, stable IDs, and per-relation source field/value provenance in a finite snapshot adapter.
- Keep explicit parent links, provenance references, asserted dependency/support, shared labels, and content identity as distinct relation semantics.
- Report asserted dependency/support unavailable instead of inferring it from labels, hashes, or parent links.
- Keep a typed-edge breadth-first reference implementation as the correctness baseline.
- Evaluate a bounded fixed-rule Datalog-equivalent implementation with a fresh runtime per snapshot.
- Return reached record occurrences, typed reasons, and one deterministic selected witness with explicitly classified closures for cycles, self loops, and diamonds.
- Bound depth, facts, tuples, work, estimated memory, and explanation output. Keep analysis truncation distinct from explanation truncation and from an incomplete snapshot.
- Exercise positive and negative fixtures for multi-hop chains, diamonds, cycles, disconnected records, multiple relation families, shared-label non-causality, duplicate occurrences, unknown/ambiguous endpoints, replaced revisions, partial snapshots, and every limit.
- Leave `query_evidence_index`, the starter `cargo octet check` path, artifact discovery, process execution, and publication unchanged.

## Outcome

The experiment stops the external Datalog dependency path. One fixed-rule program over one typed graph does not add value beyond the typed-edge traversal, and the base index supplies no asserted dependency/support field for composed rules to exploit. The typed-edge reference remains the implementation; the bounded fixed-rule evaluator stays as an opt-in comparison surface and regression witness.

## Impact

- **Specs**: `evidence-codebase-index`
- **Files**: `cargo-octet/src/evidence_multihop.rs`, shim re-exports, focused tests, `docs/evidence-codebase-index.md`, `CHANGELOG.md`
- **Consumers**: none; the experiment is opt-in library surface and does not run from `cargo octet check`
- **Dependencies**: none added

## Non-Goals

- Do not add a Crepe or other Datalog dependency.
- Do not change the one-expansion query, its report shape, or its exit behavior.
- Do not add a missing-evidence query, source discovery, filesystem traversal, process execution, publication, or release evaluation to the query.
- Do not infer causal dependency from shared labels or proof support from content identity.
- Do not claim whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, release eligibility, or formal proof.
- Do not enumerate every path or return an unbounded explanation.

## Compatibility Boundaries

- The existing one-expansion query behavior remains unchanged.
- The experiment consumes only an existing `EvidenceCodebaseIndex` and never reads the filesystem.
- Every report names its snapshot hash, index boundary, evaluator, admitted relation kinds, and applied limits.
- Reuse of a runtime against a different snapshot fails instead of returning stale conclusions.
- An index without an asserted dependency/support field reports that relation unavailable; absence of a label is not evidence of absence.

## Verification Expectations

Focused tests must show a chain beyond one expansion, agreement between the typed reference and fixed-rule evaluator, deterministic witness selection for diamonds and cycles, non-causal shared labels, non-proof content identity, unavailable asserted support, explicit unknown/ambiguous/duplicate/partial-snapshot semantics, stale-runtime rejection, and distinct analysis and output truncation. `cairn validate` and the proposal, design, and tasks gates must pass.
