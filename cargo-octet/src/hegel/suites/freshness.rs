use hegel::generators as gs;

use super::super::CaseTally;
use crate::requirements::tests::hooks;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn buoy_single_role_mutation_and_role_confusion(tc: hegel::TestCase, db_path: Option<String>) {
    let salt = tc.draw(gs::integers::<u32>().min_value(1).max_value(10_000));
    let mutant_kind = tc.draw(gs::integers::<u8>().min_value(0).max_value(5));
    let _case = CaseTally;

    let tool_hash = format!("tool-{salt:08x}");
    let art_hash = format!("art-{salt:08x}");
    let mut contract = hooks::Definition {
        schema_version: hooks::CONTRACT_SCHEMA_VERSION.to_owned(),
        contract_id: format!("octet.freshness.{salt}"),
        reviewed_specification: hooks::ReviewedSpecification {
            path: "spec.md".to_owned(),
            blake3: "0".repeat(64),
        },
        source_root: None,
        clauses: vec![hooks::RequirementClause {
            clause_id: format!("clause.{salt}"),
            statement: "freshness law".to_owned(),
            required: true,
            disposition: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
            checked_scope: vec![format!("scope-a-{salt}"), format!("scope-b-{salt}")],
            expected_evidence_tier: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
            allowed_evidence_ids: vec![format!("proof.{salt}")],
            allowed_source_requirement_ids: vec![format!("clause.{salt}")],
            required_evidence_classes: vec!["proof_card".to_owned()],
            required_negative_controls: Vec::new(),
            required_bindings: hooks::RequirementBindings {
                source_ids: vec![format!("scope-a-{salt}")],
                policy_ids: vec![format!("policy.{salt}")],
                tool_identity_hashes: vec![tool_hash.clone()],
                artifact_hashes: vec![art_hash.clone()],
                checked_scope: vec![format!("scope-a-{salt}"), format!("scope-b-{salt}")],
                evidence_tier: Some("formal_proof".to_owned()),
                binding_profile: None,
            },
            source_bindings: vec![hooks::SourceBinding {
                locator: "src/lib.rs".to_owned(),
                blake3: "1".repeat(64),
            }],
            dependency_bindings: Vec::new(),
            assumption_id: None,
            exclusion_reason: None,
            required_provider_id: None,
        }],
        assumptions: Vec::new(),
        exclusions: Vec::new(),
        non_claims: Vec::new(),
    };

    let mut primary = valence_core::evidence_codebase_index::EvidenceIndexRecord {
        record_id: format!("rec.{salt}"),
        artifact_family: "octet-proof-card".to_owned(),
        stable_id: format!("proof.{salt}"),
        artifact_locator: "proof-cards.json".to_owned(),
        artifact_blake3: "hash".to_owned(),
        evidence_tier: "formal_proof".to_owned(),
        source_ids: vec![format!("scope-a-{salt}"), format!("scope-b-{salt}")],
        requirement_ids: vec![format!("clause.{salt}")],
        lint_ids: Vec::new(),
        policy_ids: vec![format!("policy.{salt}")],
        fingerprints: Vec::new(),
        artifact_hashes: vec![tool_hash.clone(), art_hash],
        provenance_source_ids: Vec::new(),
        replay_commands: Vec::new(),
        parent_links: Vec::new(),
        caveats: vec!["does not reject invalid external axioms".to_owned()],
        summary: "not status=failed; passed".to_owned(),
    };
    let distractor = valence_core::evidence_codebase_index::EvidenceIndexRecord {
        record_id: format!("distractor.{salt}"),
        artifact_family: "octet-finding".to_owned(),
        stable_id: format!("finding.{salt}"),
        artifact_locator: "finding.json".to_owned(),
        artifact_blake3: "hash".to_owned(),
        evidence_tier: "static_lint".to_owned(),
        source_ids: Vec::new(),
        requirement_ids: Vec::new(),
        lint_ids: Vec::new(),
        policy_ids: vec![format!("policy.{salt}")],
        fingerprints: Vec::new(),
        artifact_hashes: Vec::new(),
        provenance_source_ids: Vec::new(),
        replay_commands: Vec::new(),
        parent_links: Vec::new(),
        caveats: Vec::new(),
        summary: format!("mentions proof.{salt} in prose"),
    };

    match mutant_kind {
        1 => {
            // Wrong-role substitution: move tool digest from artifact_hashes to source_ids.
            primary.artifact_hashes.retain(|h| h != &tool_hash);
            primary.source_ids.push(tool_hash);
        }
        2 => {
            // Prose-only clause match: strip requirement_ids and stable_id, leave in summary.
            primary.requirement_ids.clear();
            primary.stable_id = "other".to_owned();
            primary.summary = format!("proof.{salt}");
        }
        3 => {
            // Prose-only / partial scope.
            primary.source_ids = vec![format!("scope-a-{salt}")];
            primary.summary = format!("scope-a-{salt} scope-b-{salt}");
        }
        4 => {
            // Structured failure verdict.
            primary.summary = "status=failed".to_owned();
        }
        5 => {
            // Empty required inventory.
            contract.clauses[0].required = false;
        }
        _ => {}
    }

    let index = crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "idx-hash".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: 2,
        record_count: 2,
        records: vec![primary, distractor],
        issues: Vec::new(),
    };
    let report = hooks::evaluate_freshness(&contract, &index, b"contract", None);
    match mutant_kind {
        0 => assert_eq!(report.aggregate_result, hooks::STATUS_CURRENT, "{:?}", report.diagnostics),
        1 | 3 => {
            assert_eq!(report.clauses[0].status, hooks::STATUS_STALE_BINDING);
            assert_eq!(report.aggregate_result, hooks::STATUS_FAILED);
        }
        2 => {
            assert_eq!(report.clauses[0].status, hooks::STATUS_MISSING);
            assert_eq!(report.aggregate_result, hooks::STATUS_FAILED);
        }
        4 => {
            assert_eq!(report.clauses[0].status, hooks::STATUS_FAILED);
            assert_eq!(report.aggregate_result, hooks::STATUS_FAILED);
        }
        _ => assert_eq!(report.aggregate_result, hooks::STATUS_FAILED),
    }
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn envelope_relocation_and_transitive_drift(tc: hegel::TestCase, db_path: Option<String>) {
    let val = tc.draw(gs::integers::<u32>().min_value(1).max_value(10_000));
    let mutate_dep = tc.draw(gs::booleans());
    let add_symlink = tc.draw(gs::booleans());
    let _case = CaseTally;

    let settings = crate::envelope::policy::embedded_settings().expect("settings");
    let root_a = tempfile::tempdir().expect("root a");
    let root_b = tempfile::tempdir().expect("root b");

    let write_pkg = |root: &std::path::Path, body_val: u32| {
        std::fs::create_dir_all(root.join("dep/src")).expect("dir");
        std::fs::write(
            root.join("dep/Cargo.toml"),
            "[package]\nname = \"dep\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("cargo toml");
        std::fs::write(root.join("dep/src/lib.rs"), format!("pub fn val() -> u32 {{ {body_val} }}\n")).expect("lib.rs");
    };
    write_pkg(root_a.path(), val);
    write_pkg(root_b.path(), val);

    let scan_a = crate::envelope::scan::inspect_local_package(&root_a.path().join("dep/Cargo.toml"), &[], &settings);
    let scan_b = crate::envelope::scan::inspect_local_package(&root_b.path().join("dep/Cargo.toml"), &[], &settings);
    assert_eq!(scan_a.completeness, crate::envelope::CompletenessStatus::Complete);
    assert_eq!(scan_a.package_envelope_blake3, scan_b.package_envelope_blake3);

    if mutate_dep {
        write_pkg(root_b.path(), val.saturating_add(1));
        let mutated =
            crate::envelope::scan::inspect_local_package(&root_b.path().join("dep/Cargo.toml"), &[], &settings);
        assert_ne!(scan_a.package_envelope_blake3, mutated.package_envelope_blake3);
    }

    if add_symlink {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root_b.path().join("dep/src/lib.rs"), root_b.path().join("dep/src/link.rs"))
                .expect("symlink");
            let sym_scan =
                crate::envelope::scan::inspect_local_package(&root_b.path().join("dep/Cargo.toml"), &[], &settings);
            assert_eq!(sym_scan.completeness, crate::envelope::CompletenessStatus::Incomplete);
        }
    }
}
