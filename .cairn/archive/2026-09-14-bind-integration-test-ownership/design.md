# Design: compiler-owned test boundaries

## Published baseline

At revision `74a5d6b`, `normalize_architecture_shards` flattens facts from all selected shards without preserving their unit membership.
`GateIndexes` maps each package to one unit. Cargo library, binary, and integration-test units can share a package.
`validate_source_unit_roles` therefore correctly rejects source roles that package-owned decisions cannot enforce.

## Decision and ownership

The Architecture IR core owns deterministic membership construction and validation.
The runner owns bounded Cargo planning and compiler invocation. It must not infer test authority from names or filenames.
The standards core owns role interpretation and production architecture decisions.

Phase one adds a required `unit_memberships` array to normalized IR v2.
Each row contains a selected unit ID and canonical fact/unknown ID lists from that unit's validated shard.
Deduplication retains every observing unit. The IR identity binds the complete membership relation.
Verification rejects missing or noncanonical unit rows, duplicate or unsorted member IDs, orphan IDs, and facts or unknowns with no membership.
These structural checks apply even after resealing the enclosing IR.
Changing one edge of a shared relation changes the IR identity, but a structurally possible resealed relation cannot authenticate which compiler observed it.
Exact observed membership comes from the validated input shards, not from guessing ownership from the flattened facts.
Legacy v1 input remains unsupported by the new selected cohort; there is no invented ownership fallback.
Raw shard and coverage contracts do not change in this first phase.

Phase two binds a unit's source root and test role to the exact Cargo target selected by the compiler adapter.
Library and binary test configurations keep their production roles. An integration-test target can receive `test` only from an explicit matching source role.
A test-named binary, test-looking path, missing root, or production source-role conflict cannot acquire that role.
The required `CompilationUnit.source_path` field binds that root into the unit identity.
Phase two advances the raw shard and unit identity domains to v2 and the collector context to v3.
The Rust generator supplies the matching Nickel contract.
Phase two kept an explicit pending-evaluation rejection until phase three supplied the production view and its controls.

Phase three uses complete validated membership to form a typed evaluation view, not a forged filtered IR with the original identity.
Test-only facts cannot satisfy production ports, authority protocols, no-std obligations, or other positive requirements.
Facts observed in a production unit remain production evidence even when a test unit also observes them.
All original facts, unknowns, target coverage, and test-only dispositions remain bound in emitted receipts and artifact replay.
Required production unknowns remain blocking. Test-only unknowns must have an explicit non-production disposition.
The view validates identities and reference closure before and after partitioning. Missing test-only type or owner evidence cannot become a skipped production check.
Effect and construction checks visit every observing production unit. Each production unit must supply its own no-std configuration evidence.
Definition origin can select only among actual production observations; it cannot invent membership.
Cargo package lookup remains only for package-level dependency diagnostics.

The project receipt schema and identity domain advance to v2. Its required `test_only` field records unit, fact, and unknown IDs.
The field is null only when integrity errors prevent projection. Shared production facts never appear in its excluded subject lists.
Full-input identities remain unchanged. Protocol identity arrays remain input metadata, not evidence that excluded facts satisfied production requirements.
Artifact replay compares the full disposition after reevaluation. Resealing hashes cannot conceal a changed disposition against unchanged inputs.

A real `#[path]` import exposed a source-origin defect in the first phase-three candidate.
Lexical rejection treated `src/../tests/...` as generated source instead of preserving its test-labelled workspace origin.
The collector now resolves physical paths against the selected context's workspace root before assigning source origin.
It does not collapse parent components across symlinks. Existing source-envelope checks reject source symlinks before collection.
Real controls reject production imports in libraries, binaries, and both test configurations. The symlink control rejects before compiler invocation.

The phase-three Noble candidate reached production evaluation, but remained blocked by `no-std-obligation`.
It emitted no `crate:no-std` facts. The pinned compiler represents this built-in as `AttributeKind::NoStd`, not an unparsed named attribute.
The collector must use rustc's parsed crate-attribute query. A source-text search or package-wide witness is not a valid substitute.
Real Cargo controls will cover unconditional and active cfg attributes, inactive cfg attributes, `not(test)`, and integration-only evidence.
They retain the original filesystem-effect controls through an explicit `extern crate std`; no-std attribute evidence is not transitive std-free proof.
The production no-std obligation remains unchanged.
The parsed-attribute correction and all five configuration controls pass. The unchanged Noble candidate now has clean lint and architecture phases.
The [final evidence record](no-std-evidence.md) gives the tested scope and retained failures. It does not establish Noble M1 acceptance.

## Alternatives and audit

These are correlated serial review lenses, not independent reviewers.

| Family | Mechanism | Decision |
|---|---|---|
| Relabel source | Make test files claim the package's production role | Rejected: loses the declared boundary |
| Filename exemption | Ignore test-looking files or any `+test` unit | Rejected: production imports and library test configurations can contain core behavior |
| Compiler membership | Preserve complete unit membership and exact selected target roots | Selected; requires new provenance, role, and replay controls |

Acceptance requires positive production/test separation, shared-fact preservation, stale and resealed malformed membership rejection, missing-shard rejection, and production-import negative controls.
Also test test-only witnesses, fake ports, no-std evidence, and unknowns so test code cannot discharge production obligations.

## Budget and non-claims

Work proceeds in at most two implementation/test rounds per bounded session, with no subagents.
Stop a round on failed evidence integrity; do not publish or repin an incomplete role implementation.
Local tests and compiler observations are not independent acceptance, semantic proof, receipt authenticity, or release eligibility.
Evidence stays under `/home/brittonr/git/noble/.pi/m1-test-ownership/`, outside the disposable worktree.
