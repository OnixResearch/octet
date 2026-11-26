use std::collections::BTreeSet;

use hegel::generators as gs;

use super::super::CaseTally;

const NODE_COUNT_MIN: usize = 2;
const NODE_COUNT_MAX: usize = 6;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn multihop_bfs_datalog_equivalence(tc: hegel::TestCase, db_path: Option<String>) {
    let count = tc.draw(gs::integers::<usize>().min_value(NODE_COUNT_MIN).max_value(NODE_COUNT_MAX));
    let max_depth = tc.draw(gs::integers::<usize>().min_value(1).max_value(4));
    let max_output = tc.draw(gs::integers::<usize>().min_value(1).max_value(8));
    let is_complete = tc.draw(gs::booleans());
    let has_dangling = tc.draw(gs::booleans());
    let mut edges = Vec::with_capacity(count);
    for idx in 0..count {
        let target = tc.draw(gs::integers::<usize>().min_value(0).max_value(count - 1));
        edges.push((idx, target));
    }
    let _case = CaseTally;

    let mut records = Vec::with_capacity(count);
    for idx in 0..count {
        let mut parent_links = Vec::new();
        for &(from, to) in &edges {
            if from == idx {
                parent_links.push(format!("rec-{to}"));
            }
        }
        if idx == 0 && has_dangling {
            parent_links.push("missing-endpoint".to_owned());
        }
        records.push(valence_core::evidence_codebase_index::EvidenceIndexRecord {
            record_id: format!("rec-{idx}"),
            artifact_family: "octet-proof-card".to_owned(),
            stable_id: format!("stable-{idx}"),
            artifact_locator: format!("art-{idx}.json"),
            artifact_blake3: format!("b3:{idx:064x}"),
            evidence_tier: "formal_proof".to_owned(),
            source_ids: vec![format!("src-{idx}")],
            requirement_ids: vec!["req.shared".to_owned()],
            lint_ids: Vec::new(),
            policy_ids: Vec::new(),
            fingerprints: Vec::new(),
            artifact_hashes: vec![format!("hash-{}", idx % 2)],
            provenance_source_ids: vec![format!("stable-{}", (idx + 1) % count)],
            replay_commands: Vec::new(),
            parent_links,
            caveats: Vec::new(),
            summary: "ok".to_owned(),
        });
    }
    let index = crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex {
        schema_version: "octet-evidence-codebase-index/v1".to_owned(),
        normalization_version: "v1".to_owned(),
        index_hash: "b3:index-a".to_owned(),
        boundary: "identity and linkage only".to_owned(),
        source_artifact_count: records.len(),
        record_count: records.len(),
        records,
        issues: Vec::new(),
    };
    let completeness = if is_complete {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete
    } else {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredPartial {
            reason: "bounded partial fixture".to_owned(),
        }
    };
    let snapshot = crate::evidence_multihop::EvidenceSnapshot::from_index_declared(&index, completeness);
    let request = crate::evidence_multihop::ReachabilityRequest::new("stable-0")
        .with_admitted_relations(BTreeSet::from([
            crate::evidence_multihop::RelationKind::ParentLink,
            crate::evidence_multihop::RelationKind::ProvenanceSource,
            crate::evidence_multihop::RelationKind::SharedRequirement,
            crate::evidence_multihop::RelationKind::ContentIdentity,
        ]))
        .with_limits(crate::evidence_multihop::QueryLimits {
            max_depth,
            max_output,
            ..crate::evidence_multihop::QueryLimits::default()
        });

    let bfs = crate::evidence_multihop::evaluate_typed_reference(&snapshot, &request);
    let datalog = crate::evidence_multihop::evaluate_fixed_rule_datalog(&snapshot, &request);
    assert_eq!(bfs.reached, datalog.reached);
    assert_eq!(bfs.status, datalog.status);
    assert_eq!(bfs.output_truncated, datalog.output_truncated);
    assert_eq!(bfs.witness.closures, datalog.witness.closures);

    // Semantic boundary checks: SharedLabel is relatedness only, AssertedSupport is unavailable.
    assert!(!crate::evidence_multihop::RelationSemantics::SharedLabel.establishes_dependency());
    assert!(!crate::evidence_multihop::RelationSemantics::ContentIdentity.proof_support());
    assert!(
        snapshot
            .availability
            .iter()
            .any(|avail| matches!(avail, crate::evidence_multihop::RelationAvailability::Unavailable { .. }))
    );
    if has_dangling {
        assert!(
            snapshot
                .gaps
                .iter()
                .any(|gap| matches!(gap, crate::evidence_multihop::SnapshotGap::UnknownEndpoint { .. }))
        );
    }

    // Stale snapshot rejection in BoundedDatalogRuntime.
    let mut runtime = crate::evidence_multihop::BoundedDatalogRuntime::for_snapshot(&snapshot);
    let mut newer_index = index;
    newer_index.index_hash = "b3:index-b".to_owned();
    let newer_snapshot = crate::evidence_multihop::EvidenceSnapshot::from_index_declared(
        &newer_index,
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete,
    );
    assert!(matches!(
        runtime.evaluate(&newer_snapshot, &request),
        Err(crate::evidence_multihop::RuntimeError::StaleSnapshot { .. })
    ));
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn pointer_and_session_conservation(tc: hegel::TestCase, db_path: Option<String>) {
    let rev_a = tc.draw(gs::integers::<u32>().min_value(1).max_value(1_000));
    let rev_b = tc.draw(gs::integers::<u32>().min_value(1_001).max_value(2_000));
    let schema_ver = tc.draw(gs::integers::<u64>().min_value(0).max_value(3));
    let _case = CaseTally;

    let temp = tempfile::tempdir().expect("tempdir");
    let registry_path = temp.path().join("pointers.json");
    let receipt_1 = temp.path().join("receipt-1.json");
    let receipt_2 = temp.path().join("receipt-2.json");
    let receipt_3 = temp.path().join("receipt-3.json");
    let hash_1 = format!("b3:{rev_a:064x}");
    let hash_2 = format!("b3:{rev_b:064x}");

    assert_eq!(
        crate::roadmap::test_hooks::pointer_set_and_rollback(&registry_path, "head", "capsule", &hash_1, &receipt_1)
            .expect("set 1"),
        0
    );
    // Rolling back initial pointer creation (no old_hash) must fail.
    assert!(crate::roadmap::test_hooks::pointer_rollback_op(&registry_path, &receipt_1).is_err());

    assert_eq!(
        crate::roadmap::test_hooks::pointer_set_and_rollback(&registry_path, "head", "evidence", &hash_2, &receipt_2)
            .expect("set 2"),
        0
    );
    // Rolling back 2 -> 1 restores hash_1 and kind "capsule".
    assert_eq!(crate::roadmap::test_hooks::pointer_rollback_op(&registry_path, &receipt_2).expect("rollback 2"), 0);
    let restored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&registry_path).expect("read registry")).expect("parse registry");
    assert_eq!(restored["pointers"]["head"]["hash"], hash_1);
    assert_eq!(restored["pointers"]["head"]["artifact_kind"], "capsule");

    // Superseding head again makes receipt_2 non-rollbackable.
    assert_eq!(
        crate::roadmap::test_hooks::pointer_set_and_rollback(&registry_path, "head", "proof", &hash_2, &receipt_3)
            .expect("set 3"),
        0
    );
    assert!(crate::roadmap::test_hooks::pointer_rollback_op(&registry_path, &receipt_2).is_err());

    // Session migration preserves original_identity for supported schema_version and rejects newer.
    let session_in = temp.path().join("session-in.json");
    let session_out = temp.path().join("session-out.json");
    std::fs::write(
        &session_in,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": schema_ver,
            "identity": format!("b3:{rev_a:064x}"),
        }))
        .expect("serialize session"),
    )
    .expect("write session");
    if schema_ver <= 1 {
        assert_eq!(crate::roadmap::test_hooks::session_migrate_op(&session_in, &session_out).expect("migrate"), 0);
        let migrated: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&session_out).expect("read migrated")).expect("parse migrated");
        assert_eq!(migrated["migration_receipt"]["original_identity"], format!("b3:{rev_a:064x}"));
        assert_eq!(migrated["migration_receipt"]["identity_change"], false);
    } else {
        assert!(crate::roadmap::test_hooks::session_migrate_op(&session_in, &session_out).is_err());
    }
}
