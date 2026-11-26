use std::fs;
use std::path::Path;

use crate::requirements::hash_bytes;

struct OptionalHashes {
    core: String,
    drv: String,
    final_ir: String,
    catalog: String,
    orig_sched: String,
    reord_sched: String,
    res_map: String,
}

pub(super) fn build_optional_fixture(root: &Path) -> (Vec<u8>, Vec<u8>) {
    let (base_contract, base_evidence) = super::fixture::build_fixture(root);
    let mut contract: serde_json::Value = serde_json::from_slice(&base_contract).expect("parse base contract");
    let mut evidence: serde_json::Value = serde_json::from_slice(&base_evidence).expect("parse base evidence");

    let h = OptionalHashes {
        core: contract["effect_extraction"]["subjects"]["extracted_core"]["blake3"]
            .as_str()
            .expect("core_h")
            .to_owned(),
        drv: contract["effect_extraction"]["subjects"]["effect_driver"]["blake3"]
            .as_str()
            .expect("drv_h")
            .to_owned(),
        final_ir: hash_bytes(b"final-normal-form-ir"),
        catalog: hash_bytes(b"rewrite-rule-catalog-v1"),
        orig_sched: hash_bytes(b"schedule-orig"),
        reord_sched: hash_bytes(b"schedule-reordered"),
        res_map: hash_bytes(b"resource-map"),
    };

    let nf_h = write_optional_artifacts(root, &h);
    append_optional_clauses(&mut contract, &h);
    append_optional_evidence(&mut evidence, &h.core, &nf_h);

    (
        serde_json::to_vec_pretty(&contract).expect("contract"),
        serde_json::to_vec_pretty(&evidence).expect("evidence"),
    )
}

fn write_optional_artifacts(root: &Path, h: &OptionalHashes) -> String {
    let nf_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-extraction-normal-form/v1",
        "status": "passed",
        "final_program_blake3": h.final_ir,
        "rule_catalog_blake3": h.catalog,
        "side_condition_policy": "explicit-pure-guards",
        "contexts": ["seq-step-context"]
    }))
    .expect("nf bytes");
    fs::write(root.join("artifacts/normal-form.json"), &nf_bytes).expect("write nf");
    let nf_h = hash_bytes(&nf_bytes);

    let ind_bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "octet-extraction-effect-independence/v1",
        "status": "passed",
        "extracted_core_blake3": h.core,
        "driver_blake3": h.drv,
        "original_schedule_blake3": h.orig_sched,
        "reordered_schedule_blake3": h.reord_sched,
        "resource_mapping_blake3": h.res_map,
        "observer_identity": "sequential-driver-observer",
        "relation_kind": "weak-bisimulation",
        "is_footprint_soundness_proved": true,
        "is_interchange_theorem_passed": true
    }))
    .expect("ind bytes");
    fs::write(root.join("artifacts/effect-independence.json"), &ind_bytes).expect("write ind");
    let ind_h = hash_bytes(&ind_bytes);

    let mut cards: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("artifacts/proof-cards.json")).expect("read cards"))
            .expect("parse cards");
    cards["cards"].as_array_mut().expect("cards array").push(serde_json::json!({
        "proof_or_property_id": "proof.effect_independence",
        "source_requirement_or_policy_id": "extraction.effect_independence",
        "checked_rust_source_scope": "pilot:core",
        "evidence_tier": "formal_proof",
        "status": "passed",
        "artifact_blake3_identities": [
            { "artifact": "artifacts/effect-independence.json", "blake3": ind_h }
        ],
        "non_claims": ["disjoint footprint swap only"]
    }));
    fs::write(root.join("artifacts/proof-cards.json"), serde_json::to_vec_pretty(&cards).expect("cards"))
        .expect("write cards");
    nf_h
}

fn append_optional_clauses(contract: &mut serde_json::Value, h: &OptionalHashes) {
    let clauses = contract["clauses"].as_array_mut().expect("clauses array");
    clauses.push(serde_json::json!({
        "clause_id": "extraction.rule_normal_form",
        "statement": "extracted core matches rule-relative normal form",
        "required": true,
        "disposition": "empirical_check",
        "checked_scope": ["pilot:core"],
        "expected_evidence_tier": "empirical_check",
        "allowed_evidence_ids": ["check.normal_form"],
        "source_bindings": [{ "locator": "src/core.rs", "blake3": h.core }]
    }));
    clauses.push(serde_json::json!({
        "clause_id": "extraction.effect_independence",
        "statement": "commuting operations satisfy effect-independence certificate",
        "required": true,
        "disposition": "formal_proof",
        "checked_scope": ["pilot:core"],
        "expected_evidence_tier": "formal_proof",
        "allowed_evidence_ids": ["proof.effect_independence"],
        "allowed_source_requirement_ids": ["extraction.effect_independence"],
        "source_bindings": [{ "locator": "src/core.rs", "blake3": h.core }]
    }));

    let eff = &mut contract["effect_extraction"];
    eff["clauses"]["optional_normal_form_clause"] = serde_json::json!("extraction.rule_normal_form");
    eff["clauses"]["effect_independence_clause_id"] = serde_json::json!("extraction.effect_independence");
    eff["optional_normal_form"] = serde_json::json!({
        "final_program_blake3": h.final_ir,
        "rule_catalog_blake3": h.catalog,
        "side_condition_policy": "explicit-pure-guards"
    });
    eff["optional_effect_independence"] = serde_json::json!({
        "original_schedule_blake3": h.orig_sched,
        "reordered_schedule_blake3": h.reord_sched,
        "resource_mapping_blake3": h.res_map,
        "observer_identity": "sequential-driver-observer",
        "relation_kind": "weak-bisimulation"
    });
}

fn append_optional_evidence(evidence: &mut serde_json::Value, core_h: &str, nf_h: &str) {
    evidence["proof_decks"].as_array_mut().expect("decks").push(serde_json::json!({
        "evidence_id": "proof.effect_independence",
        "deck": "artifacts/proof-cards.json",
        "artifact_root": "."
    }));
    evidence["empirical_checks"].as_array_mut().expect("checks").push(serde_json::json!({
        "evidence_id": "check.normal_form",
        "evidence_class": "empirical_check",
        "status": "passed",
        "checker_id": "normalizer",
        "command_argv": ["normalizer"],
        "checked_scope": ["pilot:core"],
        "source_bindings": [{ "locator": "src/core.rs", "blake3": core_h }],
        "artifact": "artifacts/normal-form.json",
        "artifact_blake3": nf_h,
        "non_claims": ["bounded rewrite normal form"]
    }));
}
