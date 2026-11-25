use std::collections::BTreeMap;

use hegel::generators as gs;

use super::super::CaseTally;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn execution_capsule_and_cache_determinism(tc: hegel::TestCase, db_path: Option<String>) {
    let tag = tc.draw(gs::integers::<u32>().min_value(1).max_value(999));
    let is_cache_mismatch = tc.draw(gs::booleans());
    let is_limit_oversized = tc.draw(gs::booleans());
    let has_capsule_drift = tc.draw(gs::booleans());
    let _case = CaseTally;

    let stdout_limit = if is_limit_oversized { 100_000 } else { 512 };
    let limit_res = crate::roadmap::test_hooks::check_effective_limits(stdout_limit);
    assert_eq!(limit_res.is_ok(), !is_limit_oversized);

    let expected = BTreeMap::from([("out.json".to_owned(), format!("b3:{tag:064x}"))]);
    let matching = BTreeMap::from([("out.json".to_owned(), format!("b3:{tag:064x}"))]);
    assert!(crate::roadmap::test_hooks::compare_outputs_op(&expected, &matching).expect("cmp").is_empty());

    let drifted = BTreeMap::from([("out.json".to_owned(), format!("b3:{:064x}", tag + 1))]);
    assert_eq!(crate::roadmap::test_hooks::compare_outputs_op(&expected, &drifted).expect("cmp"), vec![
        "out.json".to_owned()
    ]);

    let manifest = serde_json::json!({
        "command_argv": ["cargo", "octet", "check"],
        "run_kind": "check",
        "octet_binary_identity": "b3:bin",
        "config_hash": "b3:cfg",
        "input_hashes": { "src": "b3:in" },
        "output_hashes": { "out": "b3:out" },
    });
    let mut observed = manifest.clone();
    if has_capsule_drift {
        observed["config_hash"] = serde_json::json!("b3:drifted");
    }
    let mismatches = crate::roadmap::test_hooks::capsule_mismatches_op(&manifest, &observed);
    assert_eq!(mismatches.is_empty(), !has_capsule_drift);
    if has_capsule_drift {
        assert_eq!(mismatches, vec!["config_hash".to_owned()]);
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let key_path = temp.path().join("key.json");
    let store_dir = temp.path().join("store");
    std::fs::write(
        &key_path,
        serde_json::to_vec(&serde_json::json!({
            "command_identity": format!("cmd-{tag}"),
            "input_hashes": { "src": format!("b3:{tag:064x}") },
            "config_hash": "b3:cfg",
            "tool_revision": "0.1.1-rc.1",
            "environment_allowlist": ["PATH"],
        }))
        .expect("serialize key"),
    )
    .expect("write key");
    let out_a = format!("b3:{tag:064x}");
    assert_eq!(crate::roadmap::test_hooks::pure_cache_check_op(&key_path, &store_dir, &out_a).expect("miss"), 0);
    let out_b = if is_cache_mismatch {
        format!("b3:{:064x}", tag + 1)
    } else {
        out_a
    };
    let hit_or_mismatch =
        crate::roadmap::test_hooks::pure_cache_check_op(&key_path, &store_dir, &out_b).expect("check");
    assert_eq!(hit_or_mismatch, i32::from(is_cache_mismatch));
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn negative_control_attribution_and_replay(tc: hegel::TestCase, db_path: Option<String>) {
    let variant = tc.draw(gs::integers::<u8>().min_value(0).max_value(5));
    let _case = CaseTally;

    let temp = tempfile::tempdir().expect("tempdir");
    let base_src_path = temp.path().join("baseline.rs");
    let mut_src_path = temp.path().join("mutant.rs");
    let base_deck_path = temp.path().join("baseline-cards.json");
    let mut_deck_path = temp.path().join("mutant-cards.json");
    let base_src_bytes = b"pub fn safe_add(a: u8, b: u8) -> u8 { a.saturating_add(b) }";
    let mut_src_bytes = if variant == 4 {
        base_src_bytes.as_slice()
    } else {
        b"pub fn safe_add(a: u8, b: u8) -> u8 { a.wrapping_add(b) }"
    };
    std::fs::write(&base_src_path, base_src_bytes).expect("write base src");
    std::fs::write(&mut_src_path, mut_src_bytes).expect("write mut src");
    std::fs::write(&base_deck_path, b"{\"cards\":[]}").expect("write base deck");
    std::fs::write(&mut_deck_path, b"{\"cards\":[\"mutant\"]}").expect("write mut deck");

    let base_hash = blake3::hash(base_src_bytes).to_hex().to_string();
    let mut_hash = blake3::hash(mut_src_bytes).to_hex().to_string();
    let base_deck_hash = blake3::hash(b"{\"cards\":[]}").to_hex().to_string();
    let mut_deck_hash = blake3::hash(b"{\"cards\":[\"mutant\"]}").to_hex().to_string();
    let mutant_outcome = match variant {
        1 => "accepted",
        2 => "crash",
        3 => "timeout",
        _ => "rejected",
    };
    let attribution = if variant == 5 {
        Some("wrong.property".to_owned())
    } else {
        Some("prop.safe_add".to_owned())
    };
    let mutant_card_status = if variant == 1 { "passed" } else { "failed" };

    let mut record = crate::proof_ref::ProofCardNegativeCheck {
        schema_version: crate::proof_ref::NEGATIVE_CHECK_SCHEMA_VERSION_V2.to_owned(),
        status: crate::proof_ref::NEGATIVE_STATUS_DEMONSTRATED.to_owned(),
        mutant_source: mut_src_path.to_string_lossy().into_owned(),
        mutant_card_status: mutant_card_status.to_owned(),
        mutant_card_evidence_tier: "formal_proof".to_owned(),
        mutant_cards_blake3: mut_deck_hash.clone(),
        non_claims: vec!["bounded negative control".to_owned()],
        diagnostics: Vec::new(),
        evidence: Some(crate::proof_ref::NegativeControlEvidence {
            schema_version: crate::proof_ref::NEGATIVE_CHECK_EVIDENCE_SCHEMA_VERSION.to_owned(),
            verifier: "aeneas-lean@0.1.0".to_owned(),
            toolchain: "leanprover/lean4:v4.31.0".to_owned(),
            verification_mode: "fixture".to_owned(),
            command_argv: vec!["lake".to_owned(), "build".to_owned()],
            verification_configuration_blake3: "4".repeat(64),
            approved_specification_id: "spec.1".to_owned(),
            proof_or_property_id: "prop.safe_add".to_owned(),
            source_requirement_or_policy_id: "req.safe_add".to_owned(),
            checked_rust_source_scope: "formal-proof:fixture".to_owned(),
            evidence_tier: "formal_proof".to_owned(),
            mutation: crate::proof_ref::NegativeControlMutation {
                mutation_id: "mut.overflow".to_owned(),
                patch_blake3: "5".repeat(64),
                baseline_source_blake3: base_hash.clone(),
                mutant_source_blake3: mut_hash.clone(),
            },
            baseline: crate::proof_ref::NegativeControlVerifierRun {
                outcome: "accepted".to_owned(),
                rejection_attribution: None,
                approved_specification_id: "spec.1".to_owned(),
                proof_or_property_id: "prop.safe_add".to_owned(),
                source_requirement_or_policy_id: "req.safe_add".to_owned(),
                checked_rust_source_scope: "formal-proof:fixture".to_owned(),
                source_path: base_src_path.to_string_lossy().into_owned(),
                source_blake3: base_hash.clone(),
                deck_path: base_deck_path.to_string_lossy().into_owned(),
                deck_blake3: base_deck_hash.clone(),
                raw_output: None,
            },
            mutant: crate::proof_ref::NegativeControlVerifierRun {
                outcome: mutant_outcome.to_owned(),
                rejection_attribution: attribution,
                approved_specification_id: "spec.1".to_owned(),
                proof_or_property_id: "prop.safe_add".to_owned(),
                source_requirement_or_policy_id: "req.safe_add".to_owned(),
                checked_rust_source_scope: "formal-proof:fixture".to_owned(),
                source_path: mut_src_path.to_string_lossy().into_owned(),
                source_blake3: mut_hash.clone(),
                deck_path: mut_deck_path.to_string_lossy().into_owned(),
                deck_blake3: mut_deck_hash.clone(),
                raw_output: None,
            },
            non_claims: vec!["bounded".to_owned()],
        }),
        measured: Some(crate::proof_ref::NegativeControlMeasurement {
            baseline_source_blake3: base_hash,
            mutant_source_blake3: mut_hash,
            mutation_id: "mut.overflow".to_owned(),
            patch_blake3: "5".repeat(64),
            baseline_deck_blake3: base_deck_hash,
            mutant_deck_blake3: mut_deck_hash,
            binding: crate::proof_ref::NegativeControlCardBinding {
                baseline_proof_or_property_id: "prop.safe_add".to_owned(),
                mutant_proof_or_property_id: "prop.safe_add".to_owned(),
                baseline_source_requirement_or_policy_id: "req.safe_add".to_owned(),
                mutant_source_requirement_or_policy_id: "req.safe_add".to_owned(),
                baseline_policy_id: "pol.1".to_owned(),
                mutant_policy_id: "pol.1".to_owned(),
                baseline_checked_rust_source_scope: "formal-proof:fixture".to_owned(),
                mutant_checked_rust_source_scope: "formal-proof:fixture".to_owned(),
                baseline_evidence_tier: "formal_proof".to_owned(),
                mutant_evidence_tier: "formal_proof".to_owned(),
                baseline_verifier_mode: "fixture".to_owned(),
                mutant_verifier_mode: "fixture".to_owned(),
                baseline_verifier_identity_summary: "aeneas-lean@0.1.0".to_owned(),
                mutant_verifier_identity_summary: "aeneas-lean@0.1.0".to_owned(),
                baseline_toolchain_identity_summary: "leanprover/lean4:v4.31.0".to_owned(),
                mutant_toolchain_identity_summary: "leanprover/lean4:v4.31.0".to_owned(),
                baseline_source_paths: vec![base_src_path.to_string_lossy().into_owned()],
                mutant_source_paths: vec![mut_src_path.to_string_lossy().into_owned()],
            },
        }),
    };

    let baseline_card = crate::proof_ref::ProofCard {
        proof_or_property_id: "prop.safe_add".to_owned(),
        source_requirement_or_policy_id: "req.safe_add".to_owned(),
        policy_id: "pol.1".to_owned(),
        checked_rust_source_scope: "formal-proof:fixture".to_owned(),
        source_paths: vec![base_src_path.to_string_lossy().into_owned()],
        evidence_tier: "formal_proof".to_owned(),
        evidence_origin: "octet_rerun".to_owned(),
        status: "passed".to_owned(),
        verifier_mode: "fixture".to_owned(),
        verifier_identity_summary: "aeneas-lean@0.1.0".to_owned(),
        toolchain_identity_summary: "leanprover/lean4:v4.31.0".to_owned(),
        artifact_blake3_identities: Vec::new(),
        non_claims: vec!["bounded".to_owned()],
        diagnostics: Vec::new(),
        negative_check: None,
    };

    let expected = crate::proof_ref::NegativeControlExpectation {
        proof_or_property_id: "prop.safe_add",
        source_requirement_or_policy_id: "req.safe_add",
        policy_id: "pol.1",
        checked_rust_source_scope: "formal-proof:fixture",
        evidence_tier: "formal_proof",
        baseline_card_status: "passed",
        baseline_card_diagnostics: &[],
        verifier_mode: "fixture",
        verifier_identity_summary: "aeneas-lean@0.1.0",
        toolchain_identity_summary: "leanprover/lean4:v4.31.0",
        baseline_source_paths: &baseline_card.source_paths,
        require_readable_artifacts: false,
    };
    let verdict = crate::proof_ref::negative_control_verdict(&record, &expected);
    match variant {
        0 => assert_eq!(verdict.status, crate::proof_ref::NEGATIVE_STATUS_DEMONSTRATED, "{:?}", verdict.problems),
        1 => assert_eq!(verdict.status, crate::proof_ref::NEGATIVE_STATUS_NOT_DEMONSTRATED),
        _ => assert_eq!(verdict.status, crate::proof_ref::NEGATIVE_STATUS_INCONCLUSIVE),
    }

    // Exercise proof_candidates::apply_negative_verdict with live disk artifacts.
    crate::proof_candidates::test_hooks::apply_negative_verdict(&mut record, &baseline_card, Vec::new());
    match variant {
        0 => assert_eq!(record.status, crate::proof_ref::NEGATIVE_STATUS_DEMONSTRATED),
        1 => assert_eq!(record.status, crate::proof_ref::NEGATIVE_STATUS_NOT_DEMONSTRATED),
        _ => assert_eq!(record.status, crate::proof_ref::NEGATIVE_STATUS_INCONCLUSIVE),
    }
}
