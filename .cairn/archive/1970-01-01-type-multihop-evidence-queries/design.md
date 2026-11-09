## Context

`query_evidence_index` is a one-expansion query. For every exact identity match it returns the match plus records linked directly to that match. A record selected only by that expansion is never expanded again, so a three-record chain `A -> B -> C` returns `A` and `B`, never `C`.

The evidence index model carries several kinds of link that are easy to conflate:

- `parent_links` names a parent artifact, stable ID, or record ID.
- `provenance_source_ids` names a provenance source identity.
- `requirement_ids`, `lint_ids`, and `policy_ids` are shared labels.
- `artifact_hashes` and `fingerprints` are content identity.
- The model has no asserted dependency/support field.

The frozen Octet-hosted Valence fixture is not the canonical implementation. `cargo-octet` consumes the pinned standalone `valence-core` dependency through `valence_core_shim`. The experiment therefore lives in the Octet composition root and composes the existing model rather than forking or rewriting the frozen fixture.

## Completion Contract

Completion requires an opt-in query that returns a multi-hop chain beyond one expansion with typed reasons, a deterministic selected witness, and explicit limit and snapshot-completeness reporting. A typed-edge reference implementation must remain the correctness baseline. A fixed-rule Datalog-equivalent evaluator must agree with it on reachability, selected witness edges, and closures. Shared labels must never become causal dependency, content identity must never become proof support, and asserted support must be reported unavailable.

A working-only happy path is false completion. So is a query that hides unknown, ambiguous, duplicate, conflicting, truncated, or partial conditions behind an empty or successful result.

## Decisions

### Decision: Keep the experiment opt-in in the composition root

**Choice:** Add `cargo-octet/src/evidence_multihop.rs` as a public library module. Do not add a CLI action, do not call it from `cargo octet check`, and do not change `query_evidence_index`.

**Rationale:** The issue is an experiment proposal. An opt-in API lets tests and future callers exercise the boundary without increasing starter check cost or changing any existing report shape. Placing it beside the shim keeps the frozen fixture frozen and the canonical dependency untouched.

### Decision: Preserve one exact finite snapshot

**Choice:** `EvidenceSnapshot::from_index_declared` records the index hash, schema and normalization versions, index boundary, source revisions, record occurrences with stable IDs, relation provenance, and a completeness declaration. When the supplied index has no hash, the snapshot derives an identity from the record set and still records `MissingIndexHash` as a gap.

**Rationale:** A conclusion is meaningful only against a named revision. Occurrences are indexed separately from stable IDs so duplicate stable IDs remain visible rather than being merged. Conflicting source revisions for one locator become an explicit gap.

### Decision: Keep relation semantics distinct

**Choice:** Parent links, provenance references, asserted support, shared labels, and content identity are separate `RelationKind` values with separate `RelationSemantics`. Only asserted support can `establish_dependency`. No relation family reports `proof_support`.

**Rationale:** A shared requirement or policy ID says two records are related to the same label. It does not say one caused, supports, or proves the other. A shared hash says the bytes or fingerprints match. It does not prove a claim.

### Decision: Report asserted support unavailable

**Choice:** The adapter emits one `RelationAvailability::Unavailable` entry for `AssertedSupport` with a fixed reason. It never infers support from parent links, provenance, labels, or identity.

**Rationale:** The model has no asserted support field. Inventing an edge would turn relatedness into a causal claim. Reporting unavailability is the honest bounded result the issue asks for.

### Decision: Use a deterministic selected witness, not all paths

**Choice:** Breadth-first discovery records one selected predecessor edge per newly reached occurrence. Edges back onto reached occurrences are recorded once and classified as `Cycle`, `SelfLoop`, or `DiamondClosure`. Diamonds record one selected path plus one closure edge. Cycles and self loops terminate.

**Rationale:** Reachability needs a proof-shaped explanation, not path enumeration. One deterministic predecessor tree plus classified closures is bounded, stable, and sufficient to check each claimed link. It also makes diamond and cycle behavior distinguishable instead of collapsing both into "cycle".

### Decision: Bound five dimensions plus explanation output

**Choice:** Apply `max_depth`, `max_facts`, `max_tuples`, `max_work`, `max_memory_bytes`, and `max_output`. Limits that stop traversal produce `Truncation { analysis_stopped: true }` and a truncated status. The output bound only shortens the witness: reached records stay complete and the status stays complete when nothing else failed.

**Rationale:** A reviewer must be able to tell "there are more reachable records than I asked to explore" from "the analyzer stopped early" from "the witness is shorter than the analysis". Conflating those would overstate or understate what the query established.

### Decision: Reject stale runtimes

**Choice:** `BoundedDatalogRuntime` is bound to one snapshot ID and index hash. Evaluating it against another snapshot returns `RuntimeError::StaleSnapshot` before reading facts. Each fresh evaluation uses fresh local state.

**Rationale:** A cache keyed only by record IDs would silently carry replaced revisions and removed links. Binding the runtime to the exact index identity prevents stale conclusions by construction.

### Decision: Keep the typed-edge reference and stop the external Datalog dependency

**Choice:** Do not add Crepe or any Datalog crate. Keep `evaluate_typed_reference` as the implementation. Retain `evaluate_fixed_rule_datalog` as an opt-in comparison surface and regression witness.

**Rationale:** The fixed-rule program has two rules: seed the origins and extend reachability through admitted edges. On the canonical four-record chain, both evaluators perform the same three edge examinations, derive the same three new occurrences, and produce the same selected witness and memory estimate. The Datalog path additionally needs rule instrumentation, a runtime identity check, and a second traversal to keep in agreement. Composed rules over several relation families would be needed to justify the dependency, and the base index supplies no asserted support facts for those rules to compose. The experiment therefore records **stop** for the dependency and keeps the small typed traversal.

## Cost Comparison

Measured on the four-record parent-link chain fixture (`rail-1#rule -> rail-1 -> artifacts/evidence-log.json -> prov-1`):

| Dimension | Typed-edge reference | Fixed-rule evaluator |
|---|---|---|
| Snapshot projection | One pass over records; relation families grouped in `BTreeMap`s | Same snapshot, same projection |
| Traversal | 3 edge examinations, 3 derived occurrences | 3 edge examinations, 3 derived occurrences |
| Explanation | 4 witness nodes, 3 selected edges, 0 closures | Same |
| Memory estimate | Same deterministic estimate | Same estimate |
| Instrumentation | None | 2 rules, 3 premise names, evaluation and derivation counters |
| Maintenance | One traversal loop and one closure classifier | Second traversal loop must stay in agreement plus rule bookkeeping |

The projection cost is not bounded by query limits: pairwise families (shared labels and content identity) enumerate every ordered pair in a group before any request is applied. The query memory bound detects an oversized projected snapshot and truncates before traversal, but it cannot undo the projection allocation. This is acceptable for the opt-in experiment and is a reason to keep it out of starter check paths.

## Limit and Completeness Semantics

- `Depth`, `Facts`, `Tuples`, `Work`, and `Memory` stop traversal. Reachability may be incomplete.
- `Output` truncates witness nodes, selected edges, and closures entry by entry. Reachability and status are unaffected.
- Snapshot gaps make the analysis incomplete even when traversal completes. Gaps include undeclared or partial completeness, index issues, missing or conflicting source revisions, duplicate stable IDs, unknown or ambiguous endpoints, and unknown or ambiguous starts.
- Status is derived from both axes: `Complete`, `Truncated`, `IncompleteSnapshot`, or `TruncatedAndIncompleteSnapshot`.
- A missing record is never proof of absence. This experiment does not implement a missing-evidence query.

## Cycle and Self-Reachability

- An origin is reachable at depth zero with `ReachReason::Origin`; this is self-reachability even without a self edge.
- A self edge is recorded once as `SelfLoop` and does not re-enqueue.
- An edge to an ancestor in the selected witness tree is a `Cycle`.
- Any other edge to an already-reached occurrence is a `DiamondClosure`; it does not replace the selected predecessor.
- Closure entries share the output bound and are sorted deterministically.

## Risks

- Pairwise label and identity projection is quadratic in the largest group size. A future caller with a large index should measure projection before using the experiment.
- The fixed-rule evaluator currently has one rule program. Agreement with the reference proves this program on fixture graphs, not generic Datalog correctness.
- Deterministic ordering is defined for this model's sorted vectors and `BTreeMap` iteration. A caller that mutates records after snapshot construction invalidates the snapshot identity contract.

## Non-Claims

- No whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, release eligibility, or formal proof.
- No causal dependency from shared labels.
- No proof support from content identity.
- No asserted dependency or support where the index carries none.
- No evidence of absence from a partial snapshot.
- No claim that the fixed-rule evaluator or the reference traversal is complete beyond the bounded run and declared snapshot.
