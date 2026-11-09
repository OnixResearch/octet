# Tasks: Rank proof candidates from Tiger Style allowances

## Candidate scanning and ranking

- [x] [serial] Add a bounded Rust source scanner for Tiger Style `allow` and `expect` items, including group expansion and comment/string skipping. r[octet.proof_candidates.selection]
- [x] [serial] Add proof-ref discovery, source identity matching, context binding, and drop-off for validated references. r[octet.proof_candidates.selection]
- [x] [serial] Add deterministic severity and trust-boundary ranking with stable JSON ordering. r[octet.proof_candidates.ranking]
- [x] [serial] Add the machine-readable trust-boundary prefix block to `docs/trust-boundary-evidence.md` and parse it in candidate mode. r[octet.proof_candidates.ranking]
- [x] [serial] Map only the existing addition-bound production rail to `unchecked_narrowing` and report unsupported lints explicitly. r[octet.proof_candidates.rail_mapping]
- [x] [parallel] Add positive and negative tests for scanner, ranking, mapping, unsupported reporting, stale references, and stable JSON. r[octet.proof_candidates.validation]

## Negative check

- [x] [serial] Add optional `negative_check` to proof cards and the `--record-negative-check` mode. r[octet.proof_candidates.negative_check]
- [x] [serial] Classify passed, failed, and inconclusive mutant decks and refuse to overwrite the mutant deck. r[octet.proof_candidates.negative_check]
- [x] [parallel] Add positive and negative negative-check tests, including `property_not_demonstrated`. r[octet.proof_candidates.validation]

## Fixtures and skill

- [x] [parallel] Add the `fixtures/proof-candidates` crate with the allowance at `src/codec.rs:42`, a mutant variant, a local trust-boundary document, and a checked-in negative-check deck. r[octet.proof_candidates.validation]
- [x] [serial] Add `docs/skills/prove-a-candidate/SKILL.md` with scaffold, explain, prove, proof-ref, artifact-verify, and negative-check steps. r[octet.proof_candidates.walkthrough]
- [x] [parallel] Add the fixture walkthrough transcript and record `property_not_demonstrated` from the existing rail. r[octet.proof_candidates.walkthrough]
- [x] [serial] Update README and CHANGELOG for the new opt-in modes. r[octet.proof_candidates.walkthrough]
- [x] [serial] Verify focused proof-candidate tests, full workspace check, formatting, Clippy, and Cairn gates. r[octet.proof_candidates.validation]

## Verification Coverage

- `Scenario: Allowance has no proof-ref` -> `cargo octet prove --candidates` fixture transcript and `candidate_is_listed_then_drops_after_validated_proof_ref`
- `Scenario: Validated proof-ref discharges the allowance` -> fixture transcript and `candidate_is_listed_then_drops_after_validated_proof_ref`
- `Scenario: Stale or unbound reference does not discharge` -> `invalid_or_unbound_proof_ref_keeps_candidate_listed_and_blocks`
- `Scenario: Severity and trust boundary order candidates` -> `rank_score_weights_severity_then_trust_boundary` and `candidate_json_is_stable_and_rank_sorted`
- `Scenario: JSON is stable` -> `candidate_json_is_stable_and_rank_sorted`
- `Scenario: Lint has an existing rail` -> fixture transcript and `candidate_is_listed_then_drops_after_validated_proof_ref`
- `Scenario: Lint has no existing rail` -> `unsupported_lints_are_reported_without_claiming_a_rail`
- `Scenario: Mutant still passes the rail` -> `passed_mutant_reports_property_not_demonstrated_in_proof_cards` and fixture transcript
- `Scenario: Mutant fails the rail` -> `failed_mutant_reports_property_demonstrated_for_this_mutation`
- `Scenario: Mutant run is inconclusive` -> `missing_mutant_card_is_inconclusive_and_does_not_overwrite_mutant_deck`
- `Scenario: Skill reaches a validated proof-ref` -> fixture transcript
- `Scenario: Negative check refuses success` -> checked-in `fixtures/proof-candidates/evidence/proof-cards.json`
- `Scenario: Positive and negative tests run` -> focused `cargo test -p cargo-octet --lib proof_candidates`
