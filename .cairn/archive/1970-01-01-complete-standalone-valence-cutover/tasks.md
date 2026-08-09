## Phase 1: Canonical dependency

- [x] [serial] Require the archived Valence `harden-preserves-integrity-boundaries` receipt and select an exact immutable standalone revision. r[octet.valence_project_extraction.phase5.hardened_upstream_dependency]
- [x] [serial] Define and validate the standalone Valence dependency manifest, source coordinate, revision, module rows, and compatibility dispositions. r[octet.valence_project_extraction.phase5.standalone_dependency_manifest]
- [x] [serial] Make manifest and lockfile source identities agree and reject floating release dependencies. r[octet.valence_project_extraction.phase5.exact_source_pin]

## Phase 2: Cutover and package identity

- [x] [serial] Route Valence-owned pure operations through standalone Valence while retaining Octet-owned shells. r[octet.valence_project_extraction.phase5.shell_consumes_standalone]
- [x] [serial] Remove or distinctly rename the Octet-hosted package so it cannot share canonical `valence-core` package identity. r[octet.valence_project_extraction.phase5.unique_package_identity]
- [x] [serial] Keep any compatibility shim free of forked Valence semantic implementation. r[octet.valence_project_extraction.phase5.compatibility_shim]

## Phase 3: Compatibility and migration

- [x] [parallel] Add positive Function Object, evidence-index, and evidence-codebase-index compatibility fixtures. r[octet.valence_project_extraction.phase5.compatibility.positive]
- [x] [parallel] Add negative fixtures for wrong revision, stale hosted code, missing rows, weakened non-claims, lock drift, duplicate package identity, and hosted fallback. r[octet.valence_project_extraction.phase5.compatibility.negative]
- [x] [parallel] Document Aspen and other downstream migration to the exact standalone target. r[octet.valence_project_extraction.phase5.consumer_migration_docs]

## Phase 4: Verification

- [x] [serial] Run focused cutover, compatibility, package-graph, and lock-drift tests. r[octet.valence_project_extraction.phase5.cutover_validation]
- [x] [serial] Add a checked-in CI workflow that runs only `nix flake check`. r[octet.valence_project_extraction.phase5.flake_check_ci]
- [x] [serial] Run `nix flake check` and Cairn validation/gates before sync and archive. Evidence: Cairn validation and proposal/design/tasks gates pass; focused cutover, compatibility, object-corpus, formatting, root/standards tests, UI fixtures, and strict release Clippy rails pass; `nix flake check path:$PWD -L` reports `all checks passed!`. The former pedantic and fixture-drift blockers were repaired without policy suppression. r[octet.valence_project_extraction.phase5.cutover_validation]

## Closeout evidence

- Exact standalone Valence source, package identity, compatibility shim, positive and negative fixtures, and downstream migration documentation remain checked by the release graph.
- Workspace-wide Clippy, state-machine catalog/fixture coverage, pre-commit strictness, UI support locks, and object-corpus receipts are synchronized and green.
- The passing rails retain the documented non-claims; they do not promote linkage and compatibility evidence into behavioral correctness or release eligibility.
