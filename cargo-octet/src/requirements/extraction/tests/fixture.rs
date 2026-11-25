use std::fs;
use std::path::Path;

use crate::requirements::hash_bytes;
use crate::semantic_effect::model::CLAIM_TIER;
use crate::semantic_effect::model::CoverageReport;
use crate::semantic_effect::model::RECEIPT_SCHEMA_VERSION;
use crate::semantic_effect::model::SemanticEffectReceipt;
use crate::semantic_effect::model::SubjectIdentities;
use crate::semantic_effect::model::UnwindMode;
use crate::semantic_effect::model::compute_receipt_identity;

pub(super) fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("octet-extraction-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

fn write_and_hash(root: &Path, rel: &str, bytes: &[u8]) -> String {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(&path, bytes).expect("write fixture file");
    hash_bytes(bytes)
}

pub(super) fn build_fixture(root: &Path) -> (Vec<u8>, Vec<u8>) {
    let spec_h = write_and_hash(root, "spec.md", b"# Sequential Extraction Spec\n");
    let orig_h = write_and_hash(root, "src/original.rs", b"pub fn orig(x: u32) -> u32 { x + 1 }\n");
    let core_h = write_and_hash(root, "src/core.rs", b"pub fn core(x: u32) -> u32 { x + 1 }\n");
    let drv_h = write_and_hash(root, "src/driver.rs", b"pub fn run(x: u32) -> u32 { core(x) }\n");
    let lock_h = format!("b3:{}", "1".repeat(64));
    let tool_h = format!("b3:{}", "2".repeat(64));

    let mut sem_receipt = SemanticEffectReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        status: crate::effect_closure::AdmissionStatus::Admitted,
        claim_tier: CLAIM_TIER.to_owned(),
        profile_id: "semantic-mir-core".to_owned(),
        unwind_mode: UnwindMode::ModeledCleanup,
        subjects: SubjectIdentities {
            unit_id: "u1".to_owned(),
            package_name: "pilot".to_owned(),
            target_name: "pilot".to_owned(),
            toolchain_blake3: tool_h.clone(),
            source_blake3: core_h.clone(),
            profile_blake3: "b3:prof".to_owned(),
            cargo_lock_blake3: lock_h.clone(),
            compiler_stage: "rustc-mir-optimized".to_owned(),
        },
        coverage: CoverageReport {
            is_complete: true,
            expected_function_count: 1,
            observed_body_count: 1,
            is_budget_exhausted: false,
            missing_functions: Vec::new(),
            skipped_bodies: Vec::new(),
        },
        observed_effects: Vec::new(),
        trusted_contracts: Vec::new(),
        unsupported_operations: Vec::new(),
        nodes: Vec::new(),
        diagnostics: Vec::new(),
        non_claims: vec!["bounded sequential MIR subset only".to_owned()],
        receipt_identity: String::new(),
    };
    sem_receipt.receipt_identity = compute_receipt_identity(&sem_receipt).expect("sem identity");
    let sem_h = write_and_hash(
        root,
        "artifacts/semantic-effect-receipt.json",
        &serde_json::to_vec_pretty(&sem_receipt).expect("serialize sem receipt"),
    );

    let obs_contract = serde_json::json!({
        "relation": "observational-equivalence",
        "return_values": "exact-value-equality",
        "explicit_errors": "preserved-error-variant",
        "operation_order_multiplicity": "exact-ordered-trace",
        "externally_visible_state": "modeled-state-equality",
        "destruction_treatment": "modeled-normal-and-unwind-drop",
        "panic_unwind_treatment": "preserved-or-excluded",
        "allocation_treatment": "profile-local",
        "termination_treatment": "total-bounded-sequential",
        "excluded_concurrency": "no-threads-atomics-async",
        "input_domain": "u32",
        "preconditions": "x < u32::MAX",
        "causal_environment": "deterministic-sequential-driver"
    });
    let pres_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-extraction-preservation-proof/v1",
        "status": "passed",
        "has_timeout": false,
        "has_checker_error": false,
        "original_blake3": orig_h,
        "extracted_core_blake3": core_h,
        "driver_blake3": drv_h,
        "specification_blake3": spec_h,
        "dependency_closure_blake3": lock_h,
        "target_triple": "x86_64-unknown-linux-gnu",
        "toolchain_blake3": tool_h,
        "observation_contract": obs_contract,
        "assumptions": []
    }))
    .expect("pres bytes");
    let pres_h = write_and_hash(root, "artifacts/preservation-proof.json", &pres_bytes);

    let corr_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-extraction-correspondence/v1",
        "status": "passed",
        "original_blake3": orig_h,
        "extracted_core_blake3": core_h,
        "backend_core_blake3": core_h,
        "checked_steps": ["mir-to-llbc", "llbc-to-lean"],
        "unresolved_boundaries": [],
        "assumptions": []
    }))
    .expect("corr bytes");
    let corr_h = write_and_hash(root, "artifacts/correspondence.json", &corr_bytes);

    let deck_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-proof-cards/v1",
        "cards": [{
            "proof_or_property_id": "proof.preservation",
            "source_requirement_or_policy_id": "extraction.behavior_preservation",
            "checked_rust_source_scope": "pilot:core",
            "evidence_tier": "formal_proof",
            "status": "passed",
            "artifact_blake3_identities": [
                { "artifact": "artifacts/preservation-proof.json", "blake3": pres_h }
            ],
            "non_claims": ["bounded pilot proof only"]
        }]
    }))
    .expect("deck bytes");
    write_and_hash(root, "artifacts/proof-cards.json", &deck_bytes);

    let contract_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-requirement-contract/v1",
        "contract_id": "octet.effect_extraction.pilot",
        "reviewed_specification": { "path": "spec.md", "blake3": spec_h },
        "clauses": [
            {
                "clause_id": "extraction.subjects",
                "statement": "exact subjects frozen",
                "required": true,
                "disposition": "empirical_check",
                "checked_scope": ["pilot:subjects"],
                "expected_evidence_tier": "empirical_check",
                "allowed_evidence_ids": ["check.subjects"],
                "source_bindings": [{ "locator": "src/original.rs", "blake3": orig_h }]
            },
            {
                "clause_id": "extraction.core_effects",
                "statement": "core satisfies semantic effect policy",
                "required": true,
                "disposition": "empirical_check",
                "checked_scope": ["pilot:core"],
                "expected_evidence_tier": "empirical_check",
                "allowed_evidence_ids": ["check.core_effects"],
                "source_bindings": [{ "locator": "src/core.rs", "blake3": core_h }]
            },
            {
                "clause_id": "extraction.behavior_preservation",
                "statement": "original and core+driver satisfy observation contract",
                "required": true,
                "disposition": "formal_proof",
                "checked_scope": ["pilot:core"],
                "expected_evidence_tier": "formal_proof",
                "allowed_evidence_ids": ["proof.preservation"],
                "allowed_source_requirement_ids": ["extraction.behavior_preservation"],
                "source_bindings": [{ "locator": "src/driver.rs", "blake3": drv_h }]
            },
            {
                "clause_id": "extraction.correspondence",
                "statement": "source/model correspondence checked",
                "required": true,
                "disposition": "empirical_check",
                "checked_scope": ["pilot:core"],
                "expected_evidence_tier": "empirical_check",
                "allowed_evidence_ids": ["check.correspondence"],
                "source_bindings": [{ "locator": "src/core.rs", "blake3": core_h }]
            },
            {
                "clause_id": "extraction.freshness",
                "statement": "summaries and proofs fresh",
                "required": true,
                "disposition": "empirical_check",
                "checked_scope": ["pilot:subjects"],
                "expected_evidence_tier": "empirical_check",
                "allowed_evidence_ids": ["check.subjects"],
                "source_bindings": [{ "locator": "src/original.rs", "blake3": orig_h }]
            }
        ],
        "non_claims": ["pilot extraction profile only"],
        "effect_extraction": {
            "schema_version": "octet-effect-extraction-profile/v1",
            "profile_id": "effect-extraction-sequential-v1",
            "pilot_id": "compiler-lab-194",
            "subjects": {
                "original_implementation": { "locator": "src/original.rs", "blake3": orig_h, "role": "original" },
                "extracted_core": { "locator": "src/core.rs", "blake3": core_h, "role": "core" },
                "effect_driver": { "locator": "src/driver.rs", "blake3": drv_h, "role": "driver" },
                "specification": { "locator": "spec.md", "blake3": spec_h, "role": "spec" },
                "dependency_closure_blake3": lock_h,
                "semantic_profile_id": "semantic-mir-core",
                "target_triple": "x86_64-unknown-linux-gnu",
                "toolchain_blake3": tool_h
            },
            "observation_contract": obs_contract,
            "clauses": {
                "subjects_clause_id": "extraction.subjects",
                "core_effects_clause_id": "extraction.core_effects",
                "behavior_preservation_clause_id": "extraction.behavior_preservation",
                "correspondence_clause_id": "extraction.correspondence",
                "freshness_clause_id": "extraction.freshness",
                "rule_normal_form_clause_id": null,
                "effect_independence_clause_id": null
            },
            "approved_assumptions": []
        }
    }))
    .expect("contract bytes");

    let evidence_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-requirement-evidence/v1",
        "contract_id": "octet.effect_extraction.pilot",
        "source_snapshot": [
            { "locator": "src/original.rs", "blake3": orig_h },
            { "locator": "src/core.rs", "blake3": core_h },
            { "locator": "src/driver.rs", "blake3": drv_h }
        ],
        "proof_decks": [{
            "evidence_id": "proof.preservation",
            "deck": "artifacts/proof-cards.json",
            "artifact_root": "."
        }],
        "empirical_checks": [
            {
                "evidence_id": "check.subjects",
                "evidence_class": "empirical_check",
                "status": "passed",
                "checker_id": "b3sum",
                "command_argv": ["b3sum"],
                "checked_scope": ["pilot:subjects"],
                "source_bindings": [{ "locator": "src/original.rs", "blake3": orig_h }],
                "artifact": "artifacts/correspondence.json",
                "artifact_blake3": corr_h,
                "non_claims": ["digest check only"]
            },
            {
                "evidence_id": "check.core_effects",
                "evidence_class": "empirical_check",
                "status": "passed",
                "checker_id": "cargo-octet",
                "command_argv": ["cargo-octet", "admit"],
                "checked_scope": ["pilot:core"],
                "source_bindings": [{ "locator": "src/core.rs", "blake3": core_h }],
                "artifact": "artifacts/semantic-effect-receipt.json",
                "artifact_blake3": sem_h,
                "non_claims": ["bounded sequential MIR"]
            },
            {
                "evidence_id": "check.correspondence",
                "evidence_class": "empirical_check",
                "status": "passed",
                "checker_id": "charon-aeneas",
                "command_argv": ["charon"],
                "checked_scope": ["pilot:core"],
                "source_bindings": [{ "locator": "src/core.rs", "blake3": core_h }],
                "artifact": "artifacts/correspondence.json",
                "artifact_blake3": corr_h,
                "non_claims": ["translation correspondence only"]
            }
        ],
        "non_claims": ["pilot evidence"]
    }))
    .expect("evidence bytes");

    (contract_bytes, evidence_bytes)
}
