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
    let report = super::super::freshness::evaluate(&contract, &index, b"contract", None);
    assert_eq!(report.clauses[0].status, STATUS_STALE_BINDING);
    let mut index = index;
    index.records.clear();
    let report = super::super::freshness::evaluate(&contract, &index, b"contract", None);
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
    let report = super::super::freshness::evaluate(&contract, &index, &contract_bytes, None);
    assert_eq!(report.aggregate_result, STATUS_CURRENT, "{:?}", report.diagnostics);
    assert_eq!(report.clauses[0].status, STATUS_CURRENT);
    assert_eq!(report.clauses[1].status, STATUS_CURRENT);
    assert_eq!(report.clauses[2].status, STATUS_EXCLUDED);
    assert!(report.clauses[0].actual_bindings.iter().any(|binding| binding.stable_id.contains("safe_add")));
    assert!(report.clauses[1].actual_bindings.iter().any(|binding| binding.stable_id.contains("lean_toolchain")));
}

#[test]
fn live_source_root_recomputes_dependency_envelopes_and_enforces_tier_separation() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"spec");
    let caller = write_file(root.path(), SOURCE_PATH, b"pub fn root() -> u32 { ws_dep::pure() }\n");
    write_file(root.path(), "ws_dep/Cargo.toml", b"[package]\nname = \"ws_dep\"\nversion = \"0.1.0\"\n");
    write_file(root.path(), "ws_dep/src/lib.rs", b"pub fn pure() -> u32 { 1 }\n");
    let policy = crate::envelope::policy::embedded_settings().expect("embedded policy");
    let initial_scan =
        crate::envelope::scan::inspect_local_package(&root.path().join("ws_dep/Cargo.toml"), &[], &policy);
    assert_eq!(initial_scan.completeness, crate::envelope::CompletenessStatus::Complete);

    let mut contract = contract(caller.clone(), spec.blake3, DISPOSITION_ACCEPTED_ASSUMPTION);
    contract.clauses[0].expected_evidence_tier = "policy_check".to_owned();
    contract.clauses[0].assumption_id = Some("assumption.ws_dep".to_owned());
    contract.clauses[0].required_bindings.binding_profile = Some("implementation-bound-v1".to_owned());
    contract.clauses[0].required_bindings.evidence_tier = Some("policy_check".to_owned());
    contract.clauses[0]
        .dependency_bindings
        .push(crate::requirements::contract::DependencyEnvelopeBinding {
            provider_name: "ws_dep".to_owned(),
            manifest_path: "ws_dep/Cargo.toml".to_owned(),
            package_envelope_blake3: initial_scan.package_envelope_blake3,
            binding_profile: Some("implementation-bound-v1".to_owned()),
            transitive_manifest_paths: Vec::new(),
            closure_blake3: None,
            dependency_graph_path: None,
        });
    contract.assumptions.push(AcceptedAssumption {
        assumption_id: "assumption.ws_dep".to_owned(),
        statement: "reviewed ws_dep".to_owned(),
        identity: "ws_dep:v1".to_owned(),
        trusted_boundary_identity: "ws_dep-boundary".to_owned(),
        affected_clause_ids: vec![contract.clauses[0].clause_id.clone()],
        required_empirical_evidence: vec![PROOF_CARD_ID.to_owned()],
    });
    let index = EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "index-hash".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: 1,
        record_count: 1,
        records: vec![valence_core::evidence_codebase_index::EvidenceIndexRecord {
            record_id: "record-dep".to_owned(),
            artifact_family: "octet-effect-closure-receipt".to_owned(),
            stable_id: PROOF_CARD_ID.to_owned(),
            artifact_locator: "effect-closure.json".to_owned(),
            artifact_blake3: "hash".to_owned(),
            evidence_tier: "policy_check".to_owned(),
            source_ids: Vec::new(),
            requirement_ids: vec![contract.clauses[0].clause_id.clone()],
            lint_ids: Vec::new(),
            policy_ids: Vec::new(),
            fingerprints: Vec::new(),
            artifact_hashes: Vec::new(),
            provenance_source_ids: Vec::new(),
            replay_commands: Vec::new(),
            parent_links: Vec::new(),
            caveats: Vec::new(),
            summary: "admitted".to_owned(),
        }],
        issues: Vec::new(),
    };

    let fresh = super::super::freshness::evaluate(&contract, &index, b"contract", Some(root.path()));
    assert_eq!(fresh.aggregate_result, STATUS_CURRENT, "{:?}", fresh.diagnostics);
    assert_eq!(fresh.completeness, "live_verified");

    // 1. Mutating dependency implementation on disk makes freshness stale_binding.
    fs::write(root.path().join("ws_dep/src/lib.rs"), b"pub fn pure() -> u32 { let _ = std::fs::read(\"x\"); 1 }\n")
        .expect("mutate ws_dep");
    let stale = super::super::freshness::evaluate(&contract, &index, b"contract", Some(root.path()));
    assert_eq!(stale.clauses[0].status, STATUS_STALE_BINDING, "{:?}", stale.diagnostics);

    // 2. Symlink inside dependency marks live freshness incomplete.
    std::os::unix::fs::symlink(root.path().join("ws_dep/src/lib.rs"), root.path().join("ws_dep/src/link.rs"))
        .expect("symlink");
    let incomplete = super::super::freshness::evaluate(&contract, &index, b"contract", Some(root.path()));
    assert_eq!(incomplete.clauses[0].status, STATUS_INCOMPLETE, "{:?}", incomplete.diagnostics);
    assert_eq!(incomplete.completeness, "live_incomplete");

    // 3. Formal-proof clause refuses policy_check / reviewed-assumption index record.
    let mut proof_clause = contract.clone();
    proof_clause.clauses[0].disposition = DISPOSITION_FORMAL_PROOF.to_owned();
    proof_clause.clauses[0].expected_evidence_tier = "formal_proof".to_owned();
    proof_clause.clauses[0].required_bindings.evidence_tier = Some("formal_proof".to_owned());
    let overclaim = super::super::freshness::evaluate(&proof_clause, &index, b"contract", None);
    assert_eq!(overclaim.clauses[0].status, STATUS_UNSUPPORTED, "{:?}", overclaim.diagnostics);

    // 4. Historical lockfile-only record cannot satisfy implementation-bound-v1 clause.
    let mut legacy_index = index;
    legacy_index.records[0].artifact_family = "cargo-lock-review-v1-lockfile-only".to_owned();
    let legacy = super::super::freshness::evaluate(&contract, &legacy_index, b"contract", None);
    assert_eq!(legacy.clauses[0].status, STATUS_UNSUPPORTED, "{:?}", legacy.diagnostics);
}

#[test]
fn live_dependency_bindings_check_transitive_manifests_profiles_and_monotonic_status() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"spec");
    let caller = write_file(root.path(), SOURCE_PATH, b"pub fn root() -> u32 { ws_dep::pure() }\n");
    write_file(root.path(), "ws_dep/Cargo.toml", b"[package]\nname = \"ws_dep\"\nversion = \"0.1.0\"\n");
    write_file(root.path(), "ws_dep/src/lib.rs", b"pub fn pure() -> u32 { 1 }\n");
    write_file(root.path(), "trans_dep/Cargo.toml", b"[package]\nname = \"trans_dep\"\nversion = \"0.1.0\"\n");
    write_file(root.path(), "trans_dep/src/lib.rs", b"pub fn leaf() -> u32 { 2 }\n");
    let policy = crate::envelope::policy::embedded_settings().expect("embedded policy");
    let ws_scan = crate::envelope::scan::inspect_local_package(&root.path().join("ws_dep/Cargo.toml"), &[], &policy);

    let mut base = contract(caller, spec.blake3, DISPOSITION_ACCEPTED_ASSUMPTION);
    base.clauses[0].expected_evidence_tier = "policy_check".to_owned();
    base.clauses[0].assumption_id = Some("assumption.ws_dep".to_owned());
    base.clauses[0].required_bindings.binding_profile = Some("implementation-bound-v1".to_owned());
    base.clauses[0].required_bindings.evidence_tier = Some("policy_check".to_owned());
    base.clauses[0].dependency_bindings.push(crate::requirements::contract::DependencyEnvelopeBinding {
        provider_name: "ws_dep".to_owned(),
        manifest_path: "ws_dep/Cargo.toml".to_owned(),
        package_envelope_blake3: ws_scan.package_envelope_blake3.clone(),
        binding_profile: Some("implementation-bound-v1".to_owned()),
        transitive_manifest_paths: vec!["trans_dep/Cargo.toml".to_owned()],
        closure_blake3: None,
        dependency_graph_path: None,
    });
    base.assumptions.push(AcceptedAssumption {
        assumption_id: "assumption.ws_dep".to_owned(),
        statement: "reviewed ws_dep".to_owned(),
        identity: "ws_dep:v1".to_owned(),
        trusted_boundary_identity: "ws_dep-boundary".to_owned(),
        affected_clause_ids: vec![base.clauses[0].clause_id.clone()],
        required_empirical_evidence: vec![PROOF_CARD_ID.to_owned()],
    });
    let index = EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "index-hash".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: 1,
        record_count: 1,
        records: vec![valence_core::evidence_codebase_index::EvidenceIndexRecord {
            record_id: "record-dep".to_owned(),
            artifact_family: "octet-effect-closure-receipt".to_owned(),
            stable_id: PROOF_CARD_ID.to_owned(),
            artifact_locator: "effect-closure.json".to_owned(),
            artifact_blake3: "hash".to_owned(),
            evidence_tier: "policy_check".to_owned(),
            source_ids: Vec::new(),
            requirement_ids: vec![base.clauses[0].clause_id.clone()],
            lint_ids: Vec::new(),
            policy_ids: Vec::new(),
            fingerprints: Vec::new(),
            artifact_hashes: Vec::new(),
            provenance_source_ids: Vec::new(),
            replay_commands: Vec::new(),
            parent_links: Vec::new(),
            caveats: Vec::new(),
            summary: "admitted".to_owned(),
        }],
        issues: Vec::new(),
    };

    let mut unknown_profile = base.clone();
    unknown_profile.clauses[0].dependency_bindings[0].binding_profile = Some("custom-v9".to_owned());
    let report = super::super::freshness::evaluate(&unknown_profile, &index, b"contract", Some(root.path()));
    assert_eq!(report.clauses[0].status, STATUS_UNSUPPORTED, "{:?}", report.diagnostics);

    let mut missing_graph = base.clone();
    missing_graph.clauses[0].dependency_bindings[0].closure_blake3 = Some(format!("b3:{}", "a".repeat(64)));
    let report = super::super::freshness::evaluate(&missing_graph, &index, b"contract", Some(root.path()));
    assert_eq!(report.clauses[0].status, STATUS_INCOMPLETE, "{:?}", report.diagnostics);

    std::os::unix::fs::symlink(root.path().join("trans_dep/src/lib.rs"), root.path().join("trans_dep/src/link.rs"))
        .expect("symlink trans_dep");
    let trans_sym = super::super::freshness::evaluate(&base, &index, b"contract", Some(root.path()));
    assert_eq!(trans_sym.clauses[0].status, STATUS_INCOMPLETE, "{:?}", trans_sym.diagnostics);

    let mut multi = base;
    multi.clauses[0]
        .dependency_bindings
        .insert(0, crate::requirements::contract::DependencyEnvelopeBinding {
            provider_name: "legacy_dep".to_owned(),
            manifest_path: "ws_dep/Cargo.toml".to_owned(),
            package_envelope_blake3: ws_scan.package_envelope_blake3,
            binding_profile: Some("lockfile-only-v1".to_owned()),
            transitive_manifest_paths: Vec::new(),
            closure_blake3: None,
            dependency_graph_path: None,
        });
    let monotonic = super::super::freshness::evaluate(&multi, &index, b"contract", Some(root.path()));
    assert_eq!(monotonic.clauses[0].status, STATUS_INCOMPLETE, "{:?}", monotonic.diagnostics);
}
