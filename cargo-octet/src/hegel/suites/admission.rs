use std::collections::BTreeMap;

use hegel::generators as gs;

use super::super::CaseTally;
use crate::requirements::tests::hooks;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn contract_evaluation_permutation_and_tier_separation(tc: hegel::TestCase, db_path: Option<String>) {
    let reverse_clauses = tc.draw(gs::booleans());
    let emp_tier_variant = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));
    let promote_assumption = tc.draw(gs::booleans());
    let require_excluded = tc.draw(gs::booleans());
    let empty_required = tc.draw(gs::booleans());
    let _case = CaseTally;

    let temp = tempfile::tempdir().expect("tempdir");
    let spec_bytes = b"reviewed specification";
    let src_bytes = b"pub fn safe_add(a: u8, b: u8) -> u8 { a.saturating_add(b) }";
    let emp_bytes = b"cargo test passed";
    let proof_bytes = b"{\"status\":\"passed\"}";
    std::fs::write(temp.path().join("spec.md"), spec_bytes).expect("spec");
    std::fs::create_dir_all(temp.path().join("src")).expect("src dir");
    std::fs::write(temp.path().join("src/lib.rs"), src_bytes).expect("src");
    std::fs::write(temp.path().join("emp.log"), emp_bytes).expect("emp");
    std::fs::write(temp.path().join("proof.receipt"), proof_bytes).expect("proof");

    let spec_hash = hooks::hash_bytes(spec_bytes);
    let src_binding = hooks::SourceBinding {
        locator: "src/lib.rs".to_owned(),
        blake3: hooks::hash_bytes(src_bytes),
    };
    let deck = serde_json::json!({
        "schema_version": hooks::PROOF_CARDS_SCHEMA_VERSION,
        "cards": [{
            "proof_or_property_id": "proof.add",
            "source_requirement_or_policy_id": "clause.formal",
            "checked_rust_source_scope": "scope:formal",
            "evidence_tier": "formal_proof",
            "status": "passed",
            "non_claims": ["bounded"],
            "diagnostics": [],
            "artifact_blake3_identities": [{ "artifact": "proof.receipt", "blake3": hooks::hash_bytes(proof_bytes) }],
        }],
    });
    std::fs::write(temp.path().join("deck.json"), serde_json::to_vec(&deck).expect("deck")).expect("write deck");

    let formal_clause = hooks::RequirementClause {
        clause_id: "clause.formal".to_owned(),
        statement: "formal add".to_owned(),
        required: !empty_required,
        disposition: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
        checked_scope: vec!["scope:formal".to_owned()],
        expected_evidence_tier: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
        allowed_evidence_ids: vec!["proof.add".to_owned()],
        allowed_source_requirement_ids: vec!["clause.formal".to_owned()],
        required_evidence_classes: vec!["proof_card".to_owned()],
        required_negative_controls: Vec::new(),
        required_bindings: hooks::RequirementBindings::default(),
        source_bindings: vec![src_binding.clone()],
        dependency_bindings: Vec::new(),
        assumption_id: None,
        exclusion_reason: None,
        required_provider_id: None,
    };
    let emp_clause = hooks::RequirementClause {
        clause_id: "clause.emp".to_owned(),
        statement: "empirical check".to_owned(),
        required: !empty_required,
        disposition: hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned(),
        checked_scope: vec!["scope:emp".to_owned()],
        expected_evidence_tier: if emp_tier_variant == 1 {
            hooks::DISPOSITION_FORMAL_PROOF.to_owned()
        } else {
            hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned()
        },
        allowed_evidence_ids: vec!["check.1".to_owned()],
        allowed_source_requirement_ids: Vec::new(),
        required_evidence_classes: if emp_tier_variant == 3 {
            vec![hooks::DISPOSITION_FORMAL_PROOF.to_owned()]
        } else {
            vec![hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned()]
        },
        required_negative_controls: Vec::new(),
        required_bindings: hooks::RequirementBindings {
            evidence_tier: (emp_tier_variant == 2).then(|| hooks::DISPOSITION_FORMAL_PROOF.to_owned()),
            ..hooks::RequirementBindings::default()
        },
        source_bindings: vec![src_binding.clone()],
        dependency_bindings: Vec::new(),
        assumption_id: None,
        exclusion_reason: None,
        required_provider_id: None,
    };
    let assumption_clause = hooks::RequirementClause {
        clause_id: "clause.assumption".to_owned(),
        statement: "assumption check".to_owned(),
        required: !empty_required,
        disposition: hooks::DISPOSITION_ACCEPTED_ASSUMPTION.to_owned(),
        checked_scope: vec!["scope:assump".to_owned()],
        expected_evidence_tier: if promote_assumption {
            hooks::DISPOSITION_FORMAL_PROOF.to_owned()
        } else {
            hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned()
        },
        allowed_evidence_ids: vec!["check.1".to_owned()],
        allowed_source_requirement_ids: Vec::new(),
        required_evidence_classes: Vec::new(),
        required_negative_controls: Vec::new(),
        required_bindings: hooks::RequirementBindings::default(),
        source_bindings: vec![src_binding.clone()],
        dependency_bindings: Vec::new(),
        assumption_id: Some("assump.1".to_owned()),
        exclusion_reason: None,
        required_provider_id: None,
    };
    let excluded_clause = hooks::RequirementClause {
        clause_id: "clause.excluded".to_owned(),
        statement: "excluded".to_owned(),
        required: require_excluded,
        disposition: hooks::DISPOSITION_EXCLUDED.to_owned(),
        checked_scope: Vec::new(),
        expected_evidence_tier: String::new(),
        allowed_evidence_ids: Vec::new(),
        allowed_source_requirement_ids: Vec::new(),
        required_evidence_classes: Vec::new(),
        required_negative_controls: Vec::new(),
        required_bindings: hooks::RequirementBindings::default(),
        source_bindings: Vec::new(),
        dependency_bindings: Vec::new(),
        assumption_id: None,
        exclusion_reason: Some("out of scope".to_owned()),
        required_provider_id: None,
    };

    let mut clauses = vec![formal_clause, emp_clause, assumption_clause, excluded_clause];
    if reverse_clauses {
        clauses.reverse();
    }
    let contract = hooks::Definition {
        schema_version: hooks::CONTRACT_SCHEMA_VERSION.to_owned(),
        contract_id: "octet.hegel.admission".to_owned(),
        reviewed_specification: hooks::ReviewedSpecification {
            path: "spec.md".to_owned(),
            blake3: spec_hash.clone(),
        },
        source_root: None,
        clauses,
        assumptions: vec![hooks::AcceptedAssumption {
            assumption_id: "assump.1".to_owned(),
            statement: "ok".to_owned(),
            identity: "id:1".to_owned(),
            trusted_boundary_identity: "tb:1".to_owned(),
            affected_clause_ids: vec!["clause.assumption".to_owned()],
            required_empirical_evidence: vec!["check.1".to_owned()],
        }],
        exclusions: vec![hooks::ExcludedBehavior {
            exclusion_id: "excl.1".to_owned(),
            clause_id: "clause.excluded".to_owned(),
            reason: "out of scope".to_owned(),
            reviewed_specification_blake3: spec_hash,
        }],
        non_claims: Vec::new(),
        effect_extraction: None,
    };
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let evidence = hooks::Bundle {
        schema_version: hooks::EVIDENCE_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3: Some(hooks::hash_bytes(&contract_bytes)),
        source_snapshot: vec![src_binding.clone()],
        proof_decks: vec![hooks::ProofDeck {
            evidence_id: "proof.add".to_owned(),
            deck: "deck.json".to_owned(),
            artifact_root: ".".to_owned(),
        }],
        empirical_checks: vec![hooks::EmpiricalCheck {
            evidence_id: "check.1".to_owned(),
            evidence_class: hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned(),
            status: hooks::CHECK_STATUS_PASSED.to_owned(),
            checker_id: "cargo-test".to_owned(),
            command_argv: vec!["cargo".to_owned(), "test".to_owned()],
            checked_scope: vec!["scope:emp".to_owned()],
            source_bindings: vec![src_binding],
            artifact: Some("emp.log".to_owned()),
            artifact_blake3: Some(hooks::hash_bytes(emp_bytes)),
            diagnostics: Vec::new(),
            non_claims: vec!["bounded".to_owned()],
        }],
        assumption_records: vec![hooks::AssumptionRecord {
            assumption_id: "assump.1".to_owned(),
            identity: "id:1".to_owned(),
            trusted_boundary_identity: "tb:1".to_owned(),
            status: "accepted".to_owned(),
            supporting_evidence_ids: vec!["check.1".to_owned()],
            diagnostics: Vec::new(),
        }],
        structural_links: Vec::new(),
        non_claims: Vec::new(),
    };
    let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
    let decision = hooks::evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, temp.path())
        .expect("evaluate contract");

    let mut unpermuted = contract;
    unpermuted.clauses.sort_by(|a, b| a.clause_id.cmp(&b.clause_id));
    let unpermuted_bytes = serde_json::to_vec(&unpermuted).expect("serialize unpermuted");
    let mut unpermuted_ev = evidence;
    unpermuted_ev.contract_blake3 = Some(hooks::hash_bytes(&unpermuted_bytes));
    let unpermuted_ev_bytes = serde_json::to_vec(&unpermuted_ev).expect("serialize unpermuted ev");
    let baseline =
        hooks::evaluate_contract(&unpermuted, &unpermuted_ev, &unpermuted_bytes, &unpermuted_ev_bytes, temp.path())
            .expect("eval baseline");

    assert_eq!(decision.aggregate_result, baseline.aggregate_result);
    let map_a = decision
        .clauses
        .iter()
        .map(|c| (c.clause_id.as_str(), c.status.as_str()))
        .collect::<BTreeMap<_, _>>();
    let map_b = baseline
        .clauses
        .iter()
        .map(|c| (c.clause_id.as_str(), c.status.as_str()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(map_a, map_b);

    let should_pass = emp_tier_variant == 0 && !promote_assumption && !require_excluded && !empty_required;
    if should_pass {
        assert_eq!(decision.aggregate_result, hooks::STATUS_PASSED, "{:?}", decision.diagnostics);
    } else {
        assert_eq!(decision.aggregate_result, hooks::STATUS_FAILED);
    }
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn effect_closure_summary_and_receipt_integrity(tc: hegel::TestCase, db_path: Option<String>) {
    let promote_assumption = tc.draw(gs::booleans());
    let unresolved = tc.draw(gs::booleans());
    let tamper_receipt = tc.draw(gs::booleans());
    let _case = CaseTally;

    let ev_hash = format!("b3:{}", "a".repeat(64));
    let lock_hash = format!("b3:{}", "e".repeat(64));
    let dep_provider = crate::effect_closure::ProviderIdentity {
        kind: crate::effect_closure::ProviderKind::Toolchain,
        name: "alloc".to_owned(),
        version: "nightly".to_owned(),
        source: "rustc-toolchain".to_owned(),
        target: "x86_64-unknown-linux-gnu".to_owned(),
        features: Vec::new(),
    };
    let shape = "call:Vec::new";
    let scope = crate::effect_closure::prelude_call_scope("src/core.rs", shape);
    let call_site_id = crate::effect_closure::call_site_identity(&ev_hash, "src/core.rs", &scope, shape, 0);
    let justification = if unresolved {
        crate::envelope::SummaryJustification::UnresolvedObligation
    } else if promote_assumption {
        crate::envelope::SummaryJustification::CheckedProof
    } else {
        crate::envelope::SummaryJustification::ReviewedAssumption
    };
    let pkg_hash = crate::effect_closure::provider_identity_blake3(&dep_provider);
    let build_ctx_hash = format!("b3:{}", "d".repeat(64));
    let closure_hash = crate::envelope::framed_blake3("octet-effect-closure-toolchain-closure/v1", &[
        pkg_hash.as_bytes(),
        build_ctx_hash.as_bytes(),
    ]);

    let policy = crate::effect_closure::WorkspacePolicy {
        schema_version: crate::effect_closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
        profile_id: "deterministic-core".to_owned(),
        binding_profile: Some(crate::envelope::BindingProfile::ImplementationBoundV1),
        envelope_policy_blake3: Some(crate::envelope::policy::embedded_json_blake3()),
        function_address_baseline: "evidence/functions.json".to_owned(),
        module_roles: vec![crate::effect_closure::ModuleRoleAssignment {
            path: "src/core.rs".to_owned(),
            role: crate::effect_closure::ModuleRole::Core,
        }],
        call_summaries: vec![crate::effect_closure::CallSummary {
            call_site_id,
            review_label: "reviewed Vec::new call".to_owned(),
            callee: "Vec::new".to_owned(),
            provider: dep_provider.clone(),
            effects: vec![crate::effect_closure::Effect::Alloc],
            evidence: crate::effect_closure::EvidenceReference {
                kind: "toolchain-api-review-v1".to_owned(),
                locator: "rustc-version#alloc".to_owned(),
                blake3: pkg_hash.clone(),
            },
            implementation_binding: Some(crate::envelope::ImplementationBinding {
                profile: crate::envelope::BindingProfile::ImplementationBoundV1,
                package_envelope_blake3: pkg_hash.clone(),
                closure_blake3: closure_hash.clone(),
                build_context_blake3: build_ctx_hash.clone(),
                justification,
                generated_inputs: Vec::new(),
            }),
        }],
        non_claims: vec![
            "evaluates declared closure policy and summaries only; does not prove semantic equivalence, whole-program correctness, or release eligibility".to_owned(),
        ],
    };

    let catalog = crate::effect_closure::embedded_catalog().expect("embedded catalog");
    let source_src = "pub fn entry() { let _v: Vec<u8> = Vec::new(); }\n";
    let source_id = format!("b3:{}", blake3::hash(source_src.as_bytes()).to_hex());
    let mut receipt = crate::effect_closure::compute_closure(crate::effect_closure::ClosureInput {
        profile_id: "deterministic-core".to_owned(),
        profile_hash: ev_hash.clone(),
        config_hash: ev_hash.clone(),
        command_blake3: ev_hash.clone(),
        check_status_blake3: ev_hash.clone(),
        function_address_artifact_blake3: ev_hash.clone(),
        function_address_record_root: ev_hash.clone(),
        workspace_policy_blake3: ev_hash.clone(),
        cargo_graph_blake3: ev_hash.clone(),
        cargo_lock_blake3: lock_hash,
        build_context_blake3: build_ctx_hash.clone(),
        target: "x86_64-unknown-linux-gnu".to_owned(),
        toolchain: "nightly".to_owned(),
        required_binding_profile: crate::envelope::BindingProfile::ImplementationBoundV1,
        sources: vec![crate::effect_closure::SourceEnvelope {
            path: "src/core.rs".to_owned(),
            source: source_src.to_owned(),
            source_identity: source_id.clone(),
        }],
        function_addresses: vec![crate::effect_closure::FunctionAddressRecord {
            path: "src/core.rs".to_owned(),
            name: "entry".to_owned(),
            function_address: ev_hash,
            source_identity: source_id,
        }],
        resolved_providers: vec![dep_provider.clone()],
        resolved_envelopes: vec![crate::envelope::ProviderRecord {
            provider: dep_provider,
            source_envelope_kind: crate::envelope::SourceKind::Toolchain,
            completeness: crate::envelope::CompletenessStatus::Complete,
            completeness_diagnostics: Vec::new(),
            package_envelope_blake3: pkg_hash,
            build_context_blake3: build_ctx_hash,
            implementation_closure_blake3: closure_hash,
            generated_inputs: Vec::new(),
            transitive_providers: Vec::new(),
        }],
        catalog,
        policy,
    });
    assert!(
        crate::effect_closure::verify_receipt_identity(&receipt).is_empty(),
        "{:?}",
        crate::effect_closure::verify_receipt_identity(&receipt)
    );
    if unresolved {
        assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Denied);
        assert!(receipt.diagnostics.iter().any(|d| d.code == "summary_unresolved_obligation"));
    } else if promote_assumption {
        assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Denied);
        assert!(receipt.diagnostics.iter().any(|d| d.code == "summary_justification_invalid"));
    } else {
        assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Admitted);
    }
    if tamper_receipt {
        receipt.claim_scope = "tampered".to_owned();
        assert!(!crate::effect_closure::verify_receipt_identity(&receipt).is_empty());
    }
}
