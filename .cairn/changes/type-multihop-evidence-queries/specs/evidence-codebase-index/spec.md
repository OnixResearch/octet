# Evidence Codebase Index Specification Delta

## ADDED Requirements

### Requirement: Typed multi-hop snapshot adapter

r[octet.evidence_codebase_index.multihop.snapshot] Octet SHOULD define an opt-in finite snapshot adapter over an already-built evidence codebase index that preserves the exact index hash, schema and normalization versions, source revisions, record occurrences, stable IDs, relation provenance, and an explicit snapshot completeness declaration.

#### Scenario: Snapshot preserves exact revision facts

r[octet.evidence_codebase_index.multihop.snapshot.preserved]

- GIVEN an evidence codebase index with an index hash, records, artifact locators, and BLAKE3 revisions
- WHEN a multi-hop snapshot is built
- THEN every occurrence MUST keep its record ID, stable ID, artifact family, locator, and source revision
- AND every relation MUST name the record field and value that produced it
- AND the report MUST name the snapshot hash and the index boundary verbatim.

#### Scenario: Snapshot forecloses unproven completeness

r[octet.evidence_codebase_index.multihop.snapshot.partial]

- GIVEN a snapshot without a completeness declaration, with a partial declaration, or with a missing index hash
- WHEN the snapshot is built
- THEN it MUST record a deterministic completeness gap
- AND a query over it MUST report an incomplete snapshot rather than a complete result.

### Requirement: Relation semantics stay distinct

r[octet.evidence_codebase_index.multihop.relations] Octet MUST keep explicit parent links, provenance references, asserted dependency/support, shared labels, and content identity distinct in multi-hop output.

#### Scenario: Shared labels remain relatedness

r[octet.evidence_codebase_index.multihop.relations.shared_label]

- GIVEN two records share only a requirement, policy, or lint ID
- WHEN a causal impact query admits parent links only
- THEN neither record MUST be reached from the other
- AND when a relatedness query admits the shared-label kind, the reached reason MUST carry shared-label semantics with no dependency or proof-support claim.

#### Scenario: Content identity is not proof

r[octet.evidence_codebase_index.multihop.relations.content_identity]

- GIVEN two records share an artifact hash or fingerprint
- WHEN the content-identity relation is admitted
- THEN reachability MAY report the identity relation
- AND the reason MUST NOT claim dependency or proof support.

#### Scenario: Asserted support is unavailable

r[octet.evidence_codebase_index.multihop.relations.asserted_support]

- GIVEN the evidence index model carries no asserted dependency/support field
- WHEN a query admits asserted support
- THEN the report MUST mark that relation unavailable with an explicit reason
- AND no parent-link, label, provenance, or identity edge MUST be relabeled as asserted support.

### Requirement: Bounded reachability returns typed reasons and a selected witness

r[octet.evidence_codebase_index.multihop.reachability] Octet SHOULD evaluate bounded recursive reachability over admitted relation kinds and return reached record occurrences, typed reasons, and one deterministic selected witness with explicitly classified closures.

#### Scenario: Multi-hop chain extends beyond one expansion

r[octet.evidence_codebase_index.multihop.reachability.chain]

- GIVEN an index whose records form a parent-link chain longer than one expansion
- WHEN the typed reference evaluates the chain root
- THEN it MUST return every chain record with its depth and typed edge reason
- AND the selected witness MUST contain one supporting edge per reached non-root record
- AND the existing one-expansion query MUST remain unchanged.

#### Scenario: Diamonds, cycles, and self loops are explicit

r[octet.evidence_codebase_index.multihop.reachability.closures]

- GIVEN a diamond, a cycle, and a self edge
- WHEN reachability evaluates from the covered origins
- THEN a diamond MUST select one predecessor and record one diamond closure
- AND a cycle MUST record one cycle closure without re-enqueueing the ancestor
- AND a self edge MUST record one self-loop closure
- AND the origin MUST remain reachable at depth zero.

#### Scenario: Evaluators agree

r[octet.evidence_codebase_index.multihop.reachability.agreement]

- GIVEN the same snapshot, admitted relation kinds, and limits
- WHEN the typed reference and the fixed-rule Datalog-equivalent evaluator run
- THEN their reached occurrences, selected witness edges, closures, and status MUST agree.

### Requirement: Bounds and truncation are distinguishable

r[octet.evidence_codebase_index.multihop.limits] Octet MUST bound depth, facts, tuples, work, estimated memory, and explanation output, and MUST distinguish analysis truncation from explanation truncation and from an incomplete snapshot.

#### Scenario: Analysis limits stop traversal

r[octet.evidence_codebase_index.multihop.limits.analysis]

- GIVEN a reachable chain and a depth, fact, tuple, work, or memory bound below the chain requirement
- WHEN reachability runs
- THEN the report MUST stop under that named limit with an analysis-stopping truncation
- AND the status MUST report truncation
- AND reached records MUST be limited to what was established before the stop.

#### Scenario: Output truncation preserves analysis

r[octet.evidence_codebase_index.multihop.limits.output]

- GIVEN a complete bounded analysis whose explanation exceeds the output bound
- WHEN reachability runs
- THEN the reached record set MUST remain complete
- AND the witness MUST be marked truncated
- AND the status MUST remain complete when no analysis limit and no snapshot gap applies.

### Requirement: Runtime freshness and endpoint semantics are explicit

r[octet.evidence_codebase_index.multihop.freshness] Octet MUST evaluate each snapshot with fresh state and MUST reject a runtime bound to a different snapshot instead of returning stale conclusions.

#### Scenario: Replaced revision is not reused

r[octet.evidence_codebase_index.multihop.freshness.stale_runtime]

- GIVEN a runtime evaluated against one snapshot revision
- WHEN the same runtime is asked to evaluate a snapshot with a replaced source revision, removed link, or different index hash
- THEN it MUST return a stale-snapshot error
- AND a fresh runtime for the new snapshot MUST reflect only the new snapshot.

#### Scenario: Ambiguous and unknown endpoints stay unresolved

r[octet.evidence_codebase_index.multihop.freshness.endpoints]

- GIVEN duplicate stable IDs, a parent link to a missing artifact, or a parent link that resolves to several occurrences
- WHEN a snapshot is built
- THEN duplicate occurrences, unknown endpoints, and ambiguous endpoints MUST each be reported explicitly
- AND no ambiguous endpoint MUST be guessed into a single edge.

#### Scenario: Conflicting revisions are explicit

r[octet.evidence_codebase_index.multihop.freshness.conflict]

- GIVEN two records name the same artifact locator with different BLAKE3 revisions
- WHEN a snapshot is built
- THEN the conflict MUST be reported with both revisions
- AND a query over the snapshot MUST report an incomplete snapshot.

### Requirement: Existing query remains the compatibility baseline

r[octet.evidence_codebase_index.multihop.compatibility] The multi-hop experiment MUST NOT change `query_evidence_index`, its report shape, its return values, source discovery, process execution, or publication.

#### Scenario: Baseline query is unchanged

r[octet.evidence_codebase_index.multihop.compatibility.baseline]

- GIVEN a parent-link chain longer than one expansion
- WHEN the existing query runs from the chain root
- THEN it MUST return only the root and its direct linked record
- AND the multi-hop experiment MUST be reachable only through its opt-in API
- AND no report MUST claim whole-program correctness, semantic equivalence, complete dependency closure, release eligibility, or formal proof.

### Requirement: Dependency decision is recorded

r[octet.evidence_codebase_index.multihop.decision] Octet MUST record a keep, revise, or stop decision for an external Datalog dependency and MUST keep the typed traversal when composed rules do not justify the dependency.

#### Scenario: Decision cites measured value

r[octet.evidence_codebase_index.multihop.decision.stop]

- GIVEN the fixed-rule program and the typed traversal examine the same edges, derive the same occurrences, and return the same witness on the selected fixtures
- WHEN the dependency decision is reviewed
- THEN the recorded decision MUST be stop
- AND no Datalog dependency MUST be added
- AND the typed traversal MUST remain the implementation.

#### Scenario: Decision is reopened only on new evidence

r[octet.evidence_codebase_index.multihop.decision.reopen]

- GIVEN a future request proposes a Datalog dependency
- WHEN review runs
- THEN it MUST name composed rules over several relation families that the typed traversal cannot express simply
- AND it MUST name asserted-support facts that the index actually carries
- AND agreement, limit, and freshness tests MUST cover the new rules before the decision changes.
