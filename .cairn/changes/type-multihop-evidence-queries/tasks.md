## Phase 1: Snapshot and relation semantics

- [x] [serial] Implement the finite snapshot adapter over the existing evidence index with index hash, schema and normalization versions, source revisions, record occurrences, stable IDs, and relation field/value provenance. r[octet.evidence_codebase_index.multihop.snapshot]
- [x] [serial] Record undeclared and partial completeness, index issues, missing and conflicting source revisions, duplicate stable IDs, and unknown or ambiguous endpoints as explicit gaps. r[octet.evidence_codebase_index.multihop.snapshot.partial]
- [x] [serial] Keep parent links, provenance references, shared requirement/policy/lint labels, and content identity in distinct relation kinds and semantics. r[octet.evidence_codebase_index.multihop.relations]
- [x] [serial] Mark asserted dependency/support unavailable with a fixed reason instead of inferring it from another relation family. r[octet.evidence_codebase_index.multihop.relations.asserted_support]
- [x] [serial] Emit dependency and proof-support claims only where the relation semantics permit them; no relation family claims proof support. r[octet.evidence_codebase_index.multihop.relations.shared_label]

## Phase 2: Reachability, evaluators, and limits

- [x] [serial] Implement the typed-edge breadth-first reference with depth, facts, tuples, work, memory, and output bounds. r[octet.evidence_codebase_index.multihop.reachability]
- [x] [serial] Implement one selected predecessor per reached occurrence and classify back edges as cycle, self loop, or diamond closure without enumerating paths. r[octet.evidence_codebase_index.multihop.reachability.closures]
- [x] [serial] Implement the bounded fixed-rule Datalog-equivalent evaluator with fresh per-snapshot runtime, rule instrumentation, and stale-snapshot rejection. r[octet.evidence_codebase_index.multihop.reachability.agreement]
- [x] [serial] Distinguish analysis-stopping truncation from output-only truncation and from incomplete-snapshot status. r[octet.evidence_codebase_index.multihop.limits]
- [x] [serial] Leave `query_evidence_index`, its report shape, `cargo octet check`, discovery, process execution, and publication unchanged. r[octet.evidence_codebase_index.multihop.compatibility]
- [x] [serial] Record the stop decision for an external Datalog dependency and keep the typed traversal as the implementation. r[octet.evidence_codebase_index.multihop.decision]

## Phase 3: Positive and negative verification

- [x] [parallel] Test a parent-link chain beyond one expansion, independently check each supporting edge against the source record, and assert the one-expansion query still stops after one expansion. r[octet.evidence_codebase_index.multihop.reachability.chain]
- [x] [parallel] Test diamonds, cycles, self loops, self-reachability, disconnected records, and multiple admitted relation families for agreement between the reference and fixed-rule evaluators. r[octet.evidence_codebase_index.multihop.reachability.agreement]
- [x] [parallel] Test that shared requirement, policy, and lint labels do not create causal or proof-support edges, and that content identity is not proof support. r[octet.evidence_codebase_index.multihop.relations.shared_label]
- [x] [parallel] Test replaced source revisions, removed links, stale-runtime rejection, duplicate occurrences, unknown and ambiguous endpoints, conflicting revisions, and partial or undeclared snapshots. r[octet.evidence_codebase_index.multihop.freshness]
- [x] [parallel] Test depth, facts, tuples, work, memory, and output bounds, and assert output truncation leaves analysis status complete. r[octet.evidence_codebase_index.multihop.limits]
- [x] [parallel] Test stable ordering, request and snapshot bindings, deterministic witness selection, availability reporting, and that every witness provenance entry exists in the selected snapshot. r[octet.evidence_codebase_index.multihop.snapshot.preserved]
- [x] [parallel] Test that the index boundary and multi-hop non-claim boundary appear verbatim in the report. r[octet.evidence_codebase_index.multihop.compatibility.baseline]

## Phase 4: Documentation and lifecycle verification

- [x] [serial] Document the opt-in experiment, relation semantics, bounds, freshness behavior, cost comparison, and stop decision in the evidence codebase index guide. r[octet.evidence_codebase_index.multihop.decision.stop]
- [x] [serial] Add the consumer-facing changelog entry naming the experiment and its non-claims. r[octet.evidence_codebase_index.multihop.compatibility]
- [x] [serial] Run the focused `cargo test -p cargo-octet evidence_multihop` filter and the required workspace check. r[octet.evidence_codebase_index.multihop.decision.stop]
- [x] [serial] Run `cargo fmt --check`, focused Clippy, `cairn validate`, and the proposal, design, and tasks gates. r[octet.evidence_codebase_index.multihop.decision.reopen]
