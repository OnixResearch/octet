use super::*;
use crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex;

#[test]
fn stale_binding_and_missing_freshness_are_distinct() {
    let index = EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "index-hash".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: 1,
        record_count: 1,
        records: vec![valence_core::evidence_codebase_index::EvidenceIndexRecord {
            record_id: "record".to_owned(),
            artifact_family: "octet-proof-card".to_owned(),
            stable_id: "clause.add".to_owned(),
            artifact_locator: "proof-cards.json".to_owned(),
            artifact_blake3: "hash".to_owned(),
            evidence_tier: "formal_proof".to_owned(),
            source_ids: vec!["old-source".to_owned()],
            requirement_ids: vec!["clause.add".to_owned()],
            lint_ids: Vec::new(),
            policy_ids: vec!["old-policy".to_owned()],
            fingerprints: Vec::new(),
            artifact_hashes: Vec::new(),
            provenance_source_ids: Vec::new(),
            replay_commands: Vec::new(),
            parent_links: Vec::new(),
            caveats: Vec::new(),
            summary: "passed".to_owned(),
        }],
        issues: Vec::new(),
    };
    let mut contract = contract(
        SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: "0".repeat(64),
        },
        "0".repeat(64),
        DISPOSITION_FORMAL_PROOF,
    );
    contract.clauses[0].required_bindings = RequirementBindings {
        source_ids: vec!["new-source".to_owned()],
        policy_ids: vec!["new-policy".to_owned()],
        ..RequirementBindings::default()
    };
    let report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(report.clauses[0].status, STATUS_STALE_BINDING);
    let mut index = index;
    index.records.clear();
    let report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(report.clauses[0].status, STATUS_MISSING);
}
#[test]
fn checked_example_contract_freshness_reports_current_obligations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let contract_path = root.join("standards/requirements/example-contract.json");
    let contract: Definition = read_json_bounded(&contract_path).expect("read example contract");
    let contract_bytes = fs::read(&contract_path).expect("read contract bytes");
    let proof_cards = root.join("standards/requirements/fixtures/example/proof-cards.json");
    let finding = root.join("standards/requirements/fixtures/example/index/check-finding.json");
    let inputs = vec![
        crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput {
            locator: "standards/requirements/fixtures/example/proof-cards.json".to_owned(),
            bytes: fs::read(&proof_cards).expect("read proof cards"),
        },
        crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput {
            locator: "standards/requirements/fixtures/example/index/check-finding.json".to_owned(),
            bytes: fs::read(&finding).expect("read finding"),
        },
    ];
    let index = crate::valence_core_shim::evidence_codebase_index::build_evidence_codebase_index(
        &inputs,
        crate::valence_core_shim::evidence_codebase_index::EvidenceIndexBuildMode::Strict,
    );
    assert!(index.issues.is_empty(), "{:?}", index.issues);
    let report = super::super::freshness::evaluate(&contract, &index, &contract_bytes);
    assert_eq!(report.aggregate_result, STATUS_CURRENT, "{:?}", report.diagnostics);
    assert_eq!(report.clauses[0].status, STATUS_CURRENT);
    assert_eq!(report.clauses[1].status, STATUS_CURRENT);
    assert_eq!(report.clauses[2].status, STATUS_EXCLUDED);
    assert!(report.clauses[0].actual_bindings.iter().any(|binding| binding.stable_id.contains("safe_add")));
    assert!(report.clauses[1].actual_bindings.iter().any(|binding| binding.stable_id.contains("lean_toolchain")));
}

#[test]
fn historical_wrong_role_and_incomplete_freshness_stay_non_current() {
    let base_record = valence_core::evidence_codebase_index::EvidenceIndexRecord {
        record_id: "link.rec".to_owned(),
        artifact_family: "octet-structural-linkage".to_owned(),
        stable_id: "cairn.link.add".to_owned(),
        artifact_locator: "receipt.json".to_owned(),
        artifact_blake3: "1".repeat(64),
        evidence_tier: "structural_linkage".to_owned(),
        source_ids: vec!["src/pilot.rs".to_owned()],
        requirement_ids: vec!["clause.add".to_owned()],
        lint_ids: Vec::new(),
        policy_ids: vec!["cairn-structural-trace-v2".to_owned()],
        fingerprints: Vec::new(),
        artifact_hashes: vec!["2".repeat(64)],
        provenance_source_ids: Vec::new(),
        replay_commands: Vec::new(),
        parent_links: Vec::new(),
        caveats: vec!["historical".to_owned()],
        summary: "checked_add".to_owned(),
    };
    let mut index = EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "index-hash".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: 1,
        record_count: 1,
        records: vec![base_record],
        issues: Vec::new(),
    };
    let mut contract = contract(
        SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: "0".repeat(64),
        },
        "0".repeat(64),
        DISPOSITION_STRUCTURAL_LINKAGE,
    );
    contract.clauses[0].expected_evidence_tier = DISPOSITION_STRUCTURAL_LINKAGE.to_owned();
    contract.clauses[0].required_bindings = RequirementBindings {
        source_ids: vec!["src/pilot.rs".to_owned()],
        policy_ids: vec!["cairn-structural-trace-v2".to_owned()],
        artifact_hashes: vec!["2".repeat(64)],
        evidence_tier: Some(DISPOSITION_STRUCTURAL_LINKAGE.to_owned()),
        checked_scope: vec!["checked_add".to_owned()],
        ..RequirementBindings::default()
    };

    // Historical caveat cannot satisfy current freshness.
    let hist_report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(hist_report.clauses[0].status, STATUS_STALE_BINDING);

    // Removing historical caveat passes for structural_linkage tier,
    // but cannot satisfy formal_proof expected tier.
    index.records[0].caveats.clear();
    let current_report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(current_report.clauses[0].status, STATUS_CURRENT);

    contract.clauses[0].required_bindings.evidence_tier = Some(DISPOSITION_FORMAL_PROOF.to_owned());
    let wrong_tier_report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(wrong_tier_report.clauses[0].status, STATUS_STALE_BINDING);

    // Unsupported caveat reports STATUS_UNSUPPORTED.
    contract.clauses[0].required_bindings.evidence_tier = Some(DISPOSITION_STRUCTURAL_LINKAGE.to_owned());
    index.records[0].caveats = vec!["unsupported".to_owned()];
    let unsup_report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(unsup_report.clauses[0].status, STATUS_UNSUPPORTED);

    // Empty index_hash reports incomplete instead of missing when records are absent.
    index.records.clear();
    index.index_hash.clear();
    let incomplete_report = super::super::freshness::evaluate(&contract, &index, b"contract");
    assert_eq!(incomplete_report.clauses[0].status, STATUS_INCOMPLETE);
}
