# Tasks: Bounded four-version semantic merge verification rail

## Profile and design

- [x] [serial] Record the exact bounded Rust subset, observables, preconditions, arithmetic and abnormal outcomes, aliasing model, dependency closure, and exclusions in the design and guide. r[octet.semantic_merge_verification.profile]
- [x] [serial] Specify the per-observable O/A/B/M preservation relation, unsatisfiable incompatibility, and the sufficient-criterion non-claim. r[octet.semantic_merge_verification.relation]
- [x] [parallel] Specify role-tagged source and subject identities, validated item mappings, and fail-closed handling for renames, signatures, dependencies, missing items, wrong bindings, and swapped roles. r[octet.semantic_merge_verification.identity]
- [x] [parallel] Specify all six deterministic outcome classes and the rule that solver, budget, and backend failures are not semantic conflicts. r[octet.semantic_merge_verification.outcomes]
- [x] [serial] Record the direct-checking versus summary/product-construction decision, the prototype trusted base, and the distinction between bounded evidence, universal proof, and Rust correspondence. r[octet.semantic_merge_verification.profile]

## Prototype

- [x] [serial] Add the typed scalar IR, assumptions, roles, mappings, observables, domains, budgets, and outcomes. r[octet.semantic_merge_verification.profile]
- [x] [parallel] Add canonical admission, item resolution, binding and role validation, dependency closure, signature checks, and bounded domain enumeration. r[octet.semantic_merge_verification.identity]
- [x] [parallel] Add the pure scalar evaluator, injected backend contract, explicit checked and wrapping arithmetic, and abnormal outcomes. r[octet.semantic_merge_verification.relation]
- [x] [parallel] Add the per-observable relation checker with incompatible, lost-A, lost-B, and interference counterexample classes. r[octet.semantic_merge_verification.relation]
- [x] [parallel] Add deterministic receipt construction and validation binding profile, tool identities, assumptions, roles, mappings, domains, dependencies, budgets, command, work, outcome, provenance, and non-claims. r[octet.semantic_merge_verification.receipts]
- [x] [parallel] Keep the rail opt-in and outside starter `cargo octet check`. r[octet.semantic_merge_verification.claim_boundary]

## Fixtures and evidence

- [x] [parallel] Add executed positive fixtures for identical and independent changes, equivalent edits with different syntax, and bounded-domain coverage. r[octet.semantic_merge_verification.fixtures]
- [x] [parallel] Add executed negative fixtures for incompatible changes, lost changes, and text-clean semantic interference. r[octet.semantic_merge_verification.fixtures]
- [x] [parallel] Add negative controls for missing observables, vacuous preconditions, changed callees, wrong bindings, swapped roles, stale receipts, tampered receipts, unsupported constructs, domain overflow, interrupted verification, backend failure, and malformed role or binding inventories. r[octet.semantic_merge_verification.fixtures]
- [x] [serial] Add an independently authored expectation table and a deterministic experiment report with precision, coverage, work, and outcome counts. r[octet.semantic_merge_verification.fixtures]
- [x] [serial] Verify that every corpus receipt validates against its own admission identity and that repeated runs produce byte-identical experiment reports. r[octet.semantic_merge_verification.receipts]

## Documentation and decision

- [x] [serial] Add `docs/semantic-merge-verification-rail.md` with the profile, relation, identities, outcomes, receipts, observed experiment results, go/no-go decision, and consumer non-claims. r[octet.semantic_merge_verification.claim_boundary]
- [x] [serial] Record the change in `CHANGELOG.md` without claiming production or universal verification. r[octet.semantic_merge_verification.claim_boundary]
- [x] [serial] State that the typed IR is hand-authored, Rust source-to-IR correspondence is unproven, and Charon comparisons, function hashes, lint results, and tests are not merge proofs. r[octet.semantic_merge_verification.claim_boundary]

## Open obligations

- [x] [serial] Implement and review a real Rust source-to-IR frontend with an explicit correspondence obligation and evidence. r[octet.semantic_merge_verification.identity] r[octet.semantic_merge_verification.frontend]
  - Bounded `syn` frontend in `crates/octet-merge-rail/src/frontend.rs`; tests in `crates/octet-merge-rail/tests/frontend.rs`; guide section. It maps the documented `i64` subset exactly once, rejects unsupported constructs fail-closed, and emits a syntactic correspondence report. It does not prove Rust semantic correspondence; the external verifier and production gating obligations remain blocked.
  - Initial frontend refresh: 3,120 identities / 4,284 warnings, hash `b3:5cc32c…`; evidence: `evidence/rust-frontend.json`. The later Lean backend added findings and refreshed the baseline again.
- [x] [serial] Implement and review a symbolic or external verifier backend with a stated trusted base, proof obligations, universal-versus-bounded evidence, and solver-failure semantics. r[octet.semantic_merge_verification.profile] r[octet.semantic_merge_verification.external_backend]
  - Opt-in `LeanBackend` in `crates/octet-merge-rail/src/external_lean.rs`; tests in `crates/octet-merge-rail/tests/lean_backend.rs`; pinned Lean binary supplied by the Nix nextest check. It checks a concrete encoded equality per input with the Lean kernel, supports checked arithmetic only, and maps Lean rejection, timeout, spawn failure, or nonzero exit to `tool-error`. Evidence is bounded per input; no universal claim.
  - The advisory self-check baseline was refreshed for the new backend findings: 3,148 identities / 4,325 warnings, hash `b3:8557d2…`; evidence: `evidence/lean-backend.json`. The full flake check reports all checks passed.
- [x] [serial] Retain the reviewed production-gating no-go and keep production admission or consumer authority out of scope. The Rust frontend and Lean external verifier obligations are complete; production admission and any whole-program, semantic-equivalence, or release-authority claim remain a separately reviewed decision. r[octet.semantic_merge_verification.claim_boundary]

- `Scenario: Admitted Rust source maps to the scalar IR` -> `frontend::parses_scalar_arithmetic_and_binds_correspondence`, `frontend::frontend_output_admits_identical_roles_and_verifies`
- `Scenario: Unsupported Rust construct fails closed` -> `frontend::rejects_unsupported_constructs_fail_closed`

- `Scenario: Concrete evaluation is Lean-checked` -> `lean_backend::lean_backend_verifies_frontend_request_when_available`, `lean_backend::lean_backend_verifies_checked_overflow_when_available`
- `Scenario: Lean rejection or timeout is a tool error` -> `lean_backend::lean_backend_reports_tool_error_on_rejection`, `lean_backend::lean_backend_times_out_as_tool_error`
- `Scenario: Wrapping is outside the backend profile` -> `lean_backend::lean_backend_rejects_wrapping_arithmetic`

## Verification Coverage

- `Scenario: Admitted request carries the reviewed assumptions` -> receipt and admission identity tests
- `Scenario: Construct outside the profile is rejected` -> `unsupported_construct_is_reported_without_evaluation`
- `Scenario: Vacuous precondition never verifies` -> `vacuous_precondition_is_unknown_not_verified`
- `Scenario: Independent changes are preserved` -> `independent-changes` corpus case
- `Scenario: Incompatible changes are reported` -> `incompatible_change_binds_both_conflicting_values`
- `Scenario: A lost change is reported` -> `lost_change_class_is_bound_to_the_changed_role`
- `Scenario: Text-clean interference is reported` -> `text_clean_interference_is_detected_with_exact_values`
- `Scenario: Renamed item has an explicit mapping` -> `missing_call_mapping_is_unsupported` plus design mapping rules
- `Scenario: Wrong or swapped binding fails closed` -> `wrong_source_binding_is_rejected`, `swapped_roles_are_detected`
- `Scenario: Missing observable fails closed` -> `missing_observable_is_unsupported_not_verified`
- `Scenario: Changed dependency fails closed` -> `changed_callee_is_unsupported_without_an_observable_mapping`
- `Scenario: Textual correspondence is insufficient` -> role binding tests and design non-claim
- `Scenario: Interrupted run is a timeout` -> `interrupted_verification_reports_timeout_not_a_conflict`
- `Scenario: Backend failure is a tool error` -> `backend_failure_is_a_tool_error_not_a_semantic_conflict`
- `Scenario: Structural request failure issues no verdict` -> `duplicate_role_is_an_admission_error_without_a_verdict`, `missing_binding_is_an_admission_error_without_a_verdict`, `missing_role_is_an_admission_error_without_a_verdict`
- `Scenario: Fresh receipt validates` -> `every_corpus_receipt_validates_against_its_admission`
- `Scenario: Stale receipt is rejected` -> `stale_receipt_is_rejected_against_a_changed_request`
- `Scenario: Tampered payload is rejected` -> `tampered_receipt_identity_is_rejected`
- `Scenario: Pijul provenance stays recorded-only` -> design and receipt non-claims
- `Scenario: Corpus matches an independent expectation table` -> `corpus_matches_every_independent_expectation`, `corpus_expectations_are_not_derived_from_actual_outcomes`
- `Scenario: Negative controls do not verify` -> `tests/negative_controls.rs`
- `Scenario: Bounded prototype is authorized` -> guide go/no-go section
- `Scenario: No downstream authority is implied` -> receipt non-claims and guide consumer section
