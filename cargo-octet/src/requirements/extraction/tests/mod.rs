mod fixture;
mod optional_fixture;

use std::fs;

use crate::requirements::evaluation::decision::evaluate_contract;
use crate::semantic_effect::model::SemanticEffectReceipt;
use crate::semantic_effect::model::compute_receipt_identity;

#[test]
fn positive_extraction_pilot_passes_and_separates_evidence_classes() {
    let dir = fixture::temp_dir("positive");
    let (contract_bytes, evidence_bytes) = fixture::build_fixture(&dir);
    let contract: crate::requirements::contract::Definition =
        serde_json::from_slice(&contract_bytes).expect("parse contract");
    let evidence: crate::requirements::evidence::Bundle =
        serde_json::from_slice(&evidence_bytes).expect("parse evidence");

    let outcome =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir).expect("evaluate contract");
    assert_eq!(outcome.aggregate_result, "passed", "diagnostics: {:?}", outcome.diagnostics);
    let summary = outcome.extraction_summary.expect("extraction summary");
    assert!(summary.is_cross_clause_verified);
    assert_eq!(summary.strongest_supported_claim, "effect-extraction-verified");
    assert!(summary.static_analysis_evidence_ids.contains(&"check.core_effects".to_owned()));
    assert!(summary.formal_proof_evidence_ids.contains(&"proof.preservation".to_owned()));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn mismatched_core_or_weaker_tier_or_unapproved_assumption_blocks() {
    let dir = fixture::temp_dir("negative");
    let (contract_bytes, evidence_bytes) = fixture::build_fixture(&dir);
    let contract: crate::requirements::contract::Definition =
        serde_json::from_slice(&contract_bytes).expect("parse contract");
    let evidence: crate::requirements::evidence::Bundle =
        serde_json::from_slice(&evidence_bytes).expect("parse evidence");

    let pres_path = dir.join("artifacts/preservation-proof.json");
    let mut pres: serde_json::Value =
        serde_json::from_slice(&fs::read(&pres_path).expect("read pres")).expect("parse pres");
    let orig_pres = pres.clone();
    pres["extracted_core_blake3"] = serde_json::json!("wrong-core-digest");
    fs::write(&pres_path, serde_json::to_vec_pretty(&pres).expect("ser")).expect("write");
    let mismatched =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir).expect("eval mismatched");
    assert_eq!(mismatched.aggregate_result, "failed");
    assert!(!mismatched.extraction_summary.expect("sum").is_cross_clause_verified);

    let mut reordered = orig_pres.clone();
    reordered["observation_contract"]["operation_order_multiplicity"] = serde_json::json!("unordered-multiset");
    fs::write(&pres_path, serde_json::to_vec_pretty(&reordered).expect("ser")).expect("write");
    let order_changed =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir).expect("eval reordered");
    assert_eq!(order_changed.aggregate_result, "failed");

    let mut pres_assumed = orig_pres;
    pres_assumed["assumptions"] = serde_json::json!(["unreviewed-axiom"]);
    fs::write(&pres_path, serde_json::to_vec_pretty(&pres_assumed).expect("ser")).expect("write");
    let unapproved =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir).expect("eval unapproved");
    assert_eq!(unapproved.aggregate_result, "failed");

    let sem_path = dir.join("artifacts/semantic-effect-receipt.json");
    let mut sem: SemanticEffectReceipt =
        serde_json::from_slice(&fs::read(&sem_path).expect("read sem")).expect("parse sem");
    sem.claim_tier = "selected-source-token-effect-closure".to_owned();
    sem.receipt_identity = compute_receipt_identity(&sem).expect("rehash");
    fs::write(&sem_path, serde_json::to_vec_pretty(&sem).expect("ser")).expect("write");
    let weaker = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir).expect("eval weaker");
    assert_eq!(weaker.aggregate_result, "failed");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn optional_normal_form_and_effect_independence_clauses_verify_and_reject_mismatches() {
    let dir = fixture::temp_dir("optional");
    let (contract_bytes, evidence_bytes) = optional_fixture::build_optional_fixture(&dir);
    let contract: crate::requirements::contract::Definition =
        serde_json::from_slice(&contract_bytes).expect("parse contract");
    let evidence: crate::requirements::evidence::Bundle =
        serde_json::from_slice(&evidence_bytes).expect("parse evidence");

    let outcome = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir)
        .expect("evaluate optional clauses");
    assert_eq!(outcome.aggregate_result, "passed", "diagnostics: {:?}", outcome.diagnostics);
    assert!(outcome.extraction_summary.expect("summary").is_cross_clause_verified);

    let ind_path = dir.join("artifacts/effect-independence.json");
    let mut ind: serde_json::Value =
        serde_json::from_slice(&fs::read(&ind_path).expect("read ind")).expect("parse ind");
    ind["is_footprint_soundness_proved"] = serde_json::json!(false);
    fs::write(&ind_path, serde_json::to_vec_pretty(&ind).expect("ser")).expect("write");
    let failed_ind = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &dir)
        .expect("eval failed independence");
    assert_eq!(failed_ind.aggregate_result, "failed");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn retained_effect_extraction_pilot_passes_check_verify_and_freshness() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let contract_path = root.join("standards/requirements/effect-extraction/contract.json");
    let evidence_path = root.join("standards/requirements/fixtures/effect-extraction/evidence.json");
    let contract_bytes = fs::read(&contract_path).expect("read pilot contract");
    let evidence_bytes = fs::read(&evidence_path).expect("read pilot evidence");
    let contract: crate::requirements::contract::Definition =
        serde_json::from_slice(&contract_bytes).expect("parse pilot contract");
    let evidence: crate::requirements::evidence::Bundle =
        serde_json::from_slice(&evidence_bytes).expect("parse pilot evidence");

    let outcome = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, root)
        .expect("evaluate retained pilot");
    assert_eq!(outcome.aggregate_result, "passed", "diagnostics: {:?}", outcome.diagnostics);
    let summary = outcome.extraction_summary.as_ref().expect("extraction summary");
    assert!(summary.is_cross_clause_verified);

    let mut recorded = outcome.clone();
    recorded.decision_blake3 = String::new();
    recorded.decision_blake3 = crate::requirements::evaluation::decision::digest(&recorded).expect("digest");
    let mut verify_errors = Vec::new();
    crate::requirements::evaluation::verification::compare_decisions(&recorded, &outcome, &mut verify_errors);
    assert!(verify_errors.is_empty(), "verify errors: {verify_errors:?}");

    let proof_cards_bytes =
        fs::read(root.join("standards/requirements/fixtures/effect-extraction/proof-cards.json")).expect("cards");
    let index = crate::valence_core_shim::evidence_codebase_index::build_evidence_codebase_index(
        &[
            crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput {
                locator: "standards/requirements/fixtures/effect-extraction/proof-cards.json".to_owned(),
                bytes: proof_cards_bytes,
            },
        ],
        crate::valence_core_shim::evidence_codebase_index::EvidenceIndexBuildMode::Strict,
    );
    assert!(index.issues.is_empty(), "{:?}", index.issues);
    let _freshness = crate::requirements::freshness::evaluate(&contract, &index, &contract_bytes);
}
