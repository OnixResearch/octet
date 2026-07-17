## 1. Profile And Reference Boundary

- [x] [serial] 1.1 Define a Nickel-authored Charon semantic-analysis profile with explicit provider, target, extraction, comparison, resource, receipt, and non-claim fields. r[octet.charon_semantic_analysis.profile]
- [x] [serial] 1.2 Record exact revisions, licenses, source sentinels, intended use, and non-authority boundaries for `mechanized-llbc` and `charon-rudra` without importing code. r[octet.charon_semantic_analysis.reference_boundary]
- [x] [serial] 1.3 Add policy/catalog registration and operator documentation for the new opt-in rail. r[octet.charon_semantic_analysis.profile]

## 2. Reusable Charon Execution Boundary

- [x] [serial] 2.1 Extract or expose the existing pinned Charon execution stage behind a reusable bounded internal interface. r[octet.charon_semantic_analysis.extraction_contract]
- [x] [serial] 2.2 Ensure that interface records exact provider, Charon, command, target, source, item, options, and resource identities. r[octet.charon_semantic_analysis.extraction_contract]
- [x] [serial] 2.3 Preserve timeout, output-overflow, partial-extraction, fresh-work-root, and network-denial failure behavior. r[octet.charon_semantic_analysis.shell]

## 3. Standalone Rail Integration

- [x] [serial] 3.1 Add a standalone opt-in cargo-octet Charon semantic-analysis command or profile that does not alter the default `cargo octet check` path. r[octet.charon_semantic_analysis.profile]
- [x] [serial] 3.2 Place the semantic-analysis implementation behind a reusable module or crate boundary with typed internal models rather than embedding the logic in CLI orchestration. r[octet.charon_semantic_analysis.pure_core]
- [x] [serial] 3.3 Emit typed receipts and role-separated artifacts for raw snapshots, projections, comparison results, diagnostics, and provider manifests. r[octet.charon_semantic_analysis.evidence]

## 4. Semantic Projection And Analysis

- [x] [serial] 4.1 Project stable item IDs, source spans, declarations, local resolved calls, dependency edges, trait-resolution facts, and control-flow shape from Charon evidence. r[octet.charon_semantic_analysis.pure_core]
- [x] [serial] 4.2 Add conservative effect candidates, unsupported markers, unknown facts, and cycle diagnostics without claiming complete runtime reachability or effect absence. r[octet.charon_semantic_analysis.bounded_analysis]
- [x] [serial] 4.3 Preserve run-local IDs separately from stable IDs and validate graph endpoints deterministically. r[octet.charon_semantic_analysis.bounded_analysis]

## 5. Existing Evidence Comparison

- [x] [serial] 5.1 Normalize comparable Charon and source/token facts into explicit typed rows. r[octet.charon_semantic_analysis.comparison]
- [x] [serial] 5.2 Record agreements, disagreements, one-sided facts, and stale-source unknowns without merging authority. r[octet.charon_semantic_analysis.comparison]
- [x] [serial] 5.3 Keep generated comparison reports BLAKE3-bound and reproducible. r[octet.charon_semantic_analysis.comparison]

## 6. Stack Integration Boundaries

- [x] [serial] 6.1 Record the downstream Valence handoff as typed, identity-preserving, role-separated evidence input. r[octet.charon_semantic_analysis.boundaries]
- [x] [serial] 6.2 Keep Kamacite framed as an interchange boundary rather than the internal compiler IR. r[octet.charon_semantic_analysis.boundaries]
- [x] [serial] 6.3 Keep `mechanized-llbc` and `charon-rudra` recorded-only until separately validated adapters exist. r[octet.charon_semantic_analysis.reference_boundary]

## 7. Validation And Documentation

- [x] [serial] 7.1 Add positive, negative, partial-extraction, stale-identity, unsupported-feature, cycle, and tamper fixtures. r[octet.charon_semantic_analysis.final_validation]
- [x] [serial] 7.2 Add deterministic replay tests and verify artifacts through `cargo octet artifact verify`. r[octet.charon_semantic_analysis.final_validation]
- [x] [serial] 7.3 Run focused tests, policy/profile checks, Cairn validation, and the relevant Nix gate. r[octet.charon_semantic_analysis.final_validation]
- [x] [serial] 7.4 Document exact invocation, artifact roles, limitations, comparison semantics, references, and downstream boundaries. r[octet.charon_semantic_analysis.final_validation]
