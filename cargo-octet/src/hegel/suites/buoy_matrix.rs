use crate::requirements::tests::hooks;

struct DimensionFixture {
    dir: tempfile::TempDir,
    spec_bytes: Vec<u8>,
    src_bytes: Vec<u8>,
    theorem_bytes: Vec<u8>,
    ext_model_bytes: Vec<u8>,
    llbc_bytes: Vec<u8>,
    contract: hooks::Definition,
    evidence: hooks::Bundle,
}

impl DimensionFixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let spec_bytes = b"buoy 16-dimension spec".to_vec();
        let src_bytes = b"pub fn safe_add(a: u8, b: u8) -> u8 { a.saturating_add(b) }".to_vec();
        let theorem_bytes = b"theorem safe_add_ok : True := trivial".to_vec();
        let ext_model_bytes = b"{\"models\":[\"u8\"]}".to_vec();
        let llbc_bytes = b"def safe_add := 1".to_vec();
        std::fs::create_dir_all(dir.path().join("src")).expect("src dir");
        std::fs::write(dir.path().join("spec.md"), &spec_bytes).expect("spec");
        std::fs::write(dir.path().join("src/lib.rs"), &src_bytes).expect("src");
        std::fs::write(dir.path().join("Theorem.lean"), &theorem_bytes).expect("theorem");
        std::fs::write(dir.path().join("external-models.json"), &ext_model_bytes).expect("ext");
        std::fs::write(dir.path().join("Llbc.lean"), &llbc_bytes).expect("llbc");
        std::fs::write(dir.path().join("emp.log"), b"ok").expect("emp");

        let spec_hash = hooks::hash_bytes(&spec_bytes);
        let src_binding = hooks::SourceBinding {
            locator: "src/lib.rs".to_owned(),
            blake3: hooks::hash_bytes(&src_bytes),
        };
        let thm_binding = hooks::SourceBinding {
            locator: "Theorem.lean".to_owned(),
            blake3: hooks::hash_bytes(&theorem_bytes),
        };
        let contract = Self::build_contract(&spec_hash, &src_binding, &thm_binding);
        Self::write_deck(dir.path(), &ext_model_bytes, &llbc_bytes, &theorem_bytes);
        let contract_bytes = serde_json::to_vec(&contract).expect("contract");
        let evidence = Self::build_evidence(&contract_bytes, src_binding, thm_binding);
        Self {
            dir,
            spec_bytes,
            src_bytes,
            theorem_bytes,
            ext_model_bytes,
            llbc_bytes,
            contract,
            evidence,
        }
    }

    fn formal_clause(
        src_binding: &hooks::SourceBinding,
        thm_binding: &hooks::SourceBinding,
    ) -> hooks::RequirementClause {
        hooks::RequirementClause {
            clause_id: "clause.formal".to_owned(),
            statement: "formal".to_owned(),
            required: true,
            disposition: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
            checked_scope: vec!["scope:formal".to_owned()],
            expected_evidence_tier: hooks::DISPOSITION_FORMAL_PROOF.to_owned(),
            allowed_evidence_ids: vec!["proof.safe_add".to_owned()],
            allowed_source_requirement_ids: vec!["clause.formal".to_owned()],
            required_evidence_classes: vec!["proof_card".to_owned()],
            required_negative_controls: vec!["mut.overflow".to_owned()],
            required_bindings: hooks::RequirementBindings {
                source_ids: vec!["scope:formal".to_owned()],
                policy_ids: vec!["pol.1".to_owned()],
                tool_identity_hashes: vec!["tool-hash".to_owned()],
                artifact_hashes: vec![hooks::hash_bytes(b"{\"models\":[\"u8\"]}")],
                checked_scope: vec!["scope:formal".to_owned()],
                evidence_tier: Some("formal_proof".to_owned()),
                binding_profile: None,
            },
            source_bindings: vec![src_binding.clone(), thm_binding.clone()],
            dependency_bindings: Vec::new(),
            assumption_id: None,
            exclusion_reason: None,
            required_provider_id: Some("aeneas-lean".to_owned()),
        }
    }

    fn build_contract(
        spec_hash: &str,
        src_binding: &hooks::SourceBinding,
        thm_binding: &hooks::SourceBinding,
    ) -> hooks::Definition {
        hooks::Definition {
            schema_version: hooks::CONTRACT_SCHEMA_VERSION.to_owned(),
            contract_id: "octet.buoy.16dim".to_owned(),
            reviewed_specification: hooks::ReviewedSpecification {
                path: "spec.md".to_owned(),
                blake3: spec_hash.to_owned(),
            },
            source_root: None,
            clauses: vec![
                Self::formal_clause(src_binding, thm_binding),
                hooks::RequirementClause {
                    clause_id: "clause.emp".to_owned(),
                    statement: "empirical".to_owned(),
                    required: true,
                    disposition: hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned(),
                    checked_scope: vec!["scope:emp".to_owned()],
                    expected_evidence_tier: hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned(),
                    allowed_evidence_ids: vec!["check.1".to_owned()],
                    allowed_source_requirement_ids: Vec::new(),
                    required_evidence_classes: vec![hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned()],
                    required_negative_controls: Vec::new(),
                    required_bindings: hooks::RequirementBindings::default(),
                    source_bindings: vec![src_binding.clone()],
                    dependency_bindings: Vec::new(),
                    assumption_id: None,
                    exclusion_reason: None,
                    required_provider_id: None,
                },
                hooks::RequirementClause {
                    clause_id: "clause.assump".to_owned(),
                    statement: "assumption".to_owned(),
                    required: true,
                    disposition: hooks::DISPOSITION_ACCEPTED_ASSUMPTION.to_owned(),
                    checked_scope: vec!["scope:assump".to_owned()],
                    expected_evidence_tier: hooks::DISPOSITION_EMPIRICAL_CHECK.to_owned(),
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
                },
                hooks::RequirementClause {
                    clause_id: "clause.excl".to_owned(),
                    statement: "excluded".to_owned(),
                    required: false,
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
                },
            ],
            assumptions: vec![hooks::AcceptedAssumption {
                assumption_id: "assump.1".to_owned(),
                statement: "ok".to_owned(),
                identity: "id:1".to_owned(),
                trusted_boundary_identity: "tb:1".to_owned(),
                affected_clause_ids: vec!["clause.assump".to_owned()],
                required_empirical_evidence: vec!["check.1".to_owned()],
            }],
            exclusions: vec![hooks::ExcludedBehavior {
                exclusion_id: "excl.1".to_owned(),
                clause_id: "clause.excl".to_owned(),
                reason: "out of scope".to_owned(),
                reviewed_specification_blake3: spec_hash.to_owned(),
            }],
            non_claims: Vec::new(),
        }
    }

    fn write_deck(root: &std::path::Path, ext_bytes: &[u8], llbc_bytes: &[u8], theorem_bytes: &[u8]) {
        let deck = serde_json::json!({
            "schema_version": hooks::PROOF_CARDS_SCHEMA_VERSION,
            "cards": [{
                "proof_or_property_id": "proof.safe_add",
                "source_requirement_or_policy_id": "clause.formal",
                "checked_rust_source_scope": "scope:formal",
                "evidence_tier": "formal_proof",
                "status": "passed",
                "non_claims": ["bounded"],
                "diagnostics": [],
                "artifact_blake3_identities": [
                    { "artifact": "external-models.json", "blake3": hooks::hash_bytes(ext_bytes) },
                    { "artifact": "Llbc.lean", "blake3": hooks::hash_bytes(llbc_bytes) },
                    { "artifact": "Theorem.lean", "blake3": hooks::hash_bytes(theorem_bytes) },
                ],
                "formal_proof_envelope": {
                    "schema_version": "octet-formal-proof-envelope/v1",
                    "provider_id": "aeneas-lean",
                    "status": "passed",
                },
                "negative_check": {
                    "schema_version": hooks::NEGATIVE_CHECK_SCHEMA_VERSION_V2,
                    "status": hooks::NEGATIVE_STATUS_DEMONSTRATED,
                    "mutant_card_status": "failed",
                    "mutant_card_evidence_tier": "formal_proof",
                    "evidence": {
                        "verifier": "aeneas-lean",
                        "proof_or_property_id": "proof.safe_add",
                        "approved_specification_id": "spec.1",
                        "source_requirement_or_policy_id": "clause.formal",
                        "checked_rust_source_scope": "scope:formal",
                        "evidence_tier": "formal_proof",
                        "baseline": { "outcome": "accepted" },
                        "mutant": { "outcome": "rejected", "rejection_attribution": "proof.safe_add" },
                        "mutation": {
                            "mutation_id": "mut.overflow",
                            "patch_blake3": "a".repeat(64),
                            "baseline_source_blake3": "b".repeat(64),
                            "mutant_source_blake3": "c".repeat(64),
                        }
                    },
                    "measured": {
                        "mutation_id": "mut.overflow",
                        "patch_blake3": "a".repeat(64),
                        "baseline_source_blake3": "b".repeat(64),
                        "mutant_source_blake3": "c".repeat(64),
                    }
                }
            }],
        });
        std::fs::write(root.join("deck.json"), serde_json::to_vec(&deck).expect("deck")).expect("write deck");
    }

    fn build_evidence(
        contract_bytes: &[u8],
        src_binding: hooks::SourceBinding,
        thm_binding: hooks::SourceBinding,
    ) -> hooks::Bundle {
        hooks::Bundle {
            schema_version: hooks::EVIDENCE_SCHEMA_VERSION.to_owned(),
            contract_id: "octet.buoy.16dim".to_owned(),
            contract_blake3: Some(hooks::hash_bytes(contract_bytes)),
            source_snapshot: vec![src_binding.clone(), thm_binding],
            proof_decks: vec![hooks::ProofDeck {
                evidence_id: "proof.safe_add".to_owned(),
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
                artifact_blake3: Some(hooks::hash_bytes(b"ok")),
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
            non_claims: Vec::new(),
        }
    }

    fn eval(&self) -> hooks::Outcome {
        let c_bytes = serde_json::to_vec(&self.contract).expect("c");
        let e_bytes = serde_json::to_vec(&self.evidence).expect("e");
        hooks::evaluate_contract(&self.contract, &self.evidence, &c_bytes, &e_bytes, self.dir.path()).expect("eval")
    }

    fn index_record(&self) -> valence_core::evidence_codebase_index::EvidenceIndexRecord {
        valence_core::evidence_codebase_index::EvidenceIndexRecord {
            record_id: "rec.formal".to_owned(),
            artifact_family: "octet-proof-card".to_owned(),
            stable_id: "proof.safe_add".to_owned(),
            artifact_locator: "deck.json".to_owned(),
            artifact_blake3: "hash".to_owned(),
            evidence_tier: "formal_proof".to_owned(),
            source_ids: vec!["scope:formal".to_owned()],
            requirement_ids: vec!["clause.formal".to_owned()],
            lint_ids: Vec::new(),
            policy_ids: vec!["pol.1".to_owned()],
            fingerprints: Vec::new(),
            artifact_hashes: vec!["tool-hash".to_owned(), hooks::hash_bytes(&self.ext_model_bytes)],
            provenance_source_ids: Vec::new(),
            replay_commands: Vec::new(),
            parent_links: Vec::new(),
            caveats: Vec::new(),
            summary: "passed".to_owned(),
        }
    }
}

pub(in crate::hegel::tests) fn verify_all_16_dimensions() {
    let matrix = crate::hegel::profile_validation::buoy_mutation_matrix().expect("matrix");
    assert_eq!(matrix.dimensions.len(), 16);
    assert!(crate::hegel::profile_validation::validate_buoy_matrix().is_empty());

    let baseline = DimensionFixture::new();
    let base_outcome = baseline.eval();
    assert_eq!(base_outcome.aggregate_result, hooks::STATUS_PASSED, "{:?}", base_outcome.diagnostics);

    // Dim 1: Requirement/atom revision.
    std::fs::write(baseline.dir.path().join("spec.md"), b"mutated spec").expect("mut spec");
    let d1 = baseline.eval();
    assert_eq!(d1.aggregate_result, hooks::STATUS_FAILED);
    assert!(d1.diagnostics.iter().any(|d| d.contains("reviewed specification")));
    std::fs::write(baseline.dir.path().join("spec.md"), &baseline.spec_bytes).expect("restore spec");

    // Dim 2: Formal-property revision.
    let mut f2 = DimensionFixture::new();
    f2.contract.clauses[0].allowed_evidence_ids = vec!["proof.revised".to_owned()];
    let d2 = f2.eval();
    assert_eq!(d2.clauses[0].status, hooks::STATUS_MISSING);

    // Dim 3: Implementation artifact.
    std::fs::write(baseline.dir.path().join("src/lib.rs"), b"pub fn safe_add(a: u8, b: u8) -> u8 { a + b }")
        .expect("mut src");
    let d3 = baseline.eval();
    assert_eq!(d3.aggregate_result, hooks::STATUS_FAILED);
    assert!(d3.diagnostics.iter().any(|d| d.contains("src/lib.rs")));
    std::fs::write(baseline.dir.path().join("src/lib.rs"), &baseline.src_bytes).expect("restore src");

    // Dim 4: Assumption identities.
    let mut f4 = DimensionFixture::new();
    f4.evidence.assumption_records[0].identity = "id:mutated".to_owned();
    let d4 = f4.eval();
    assert_eq!(d4.aggregate_result, hooks::STATUS_FAILED);
    assert!(d4.diagnostics.iter().any(|d| d.contains("identity changed")));

    // Dim 5: Translator identities (role-confusion).
    let mut rec5 = baseline.index_record();
    rec5.artifact_hashes = vec![hooks::hash_bytes(&baseline.ext_model_bytes)];
    rec5.source_ids.push("tool-hash".to_owned());
    assert!(!hooks::record_satisfies_bindings(&rec5, &baseline.contract.clauses[0]));

    // Dim 6: Verifier identity.
    let mut f6 = DimensionFixture::new();
    f6.evidence.empirical_checks[0].checker_id.clear();
    let d6 = f6.eval();
    assert_eq!(d6.aggregate_result, hooks::STATUS_FAILED);
    assert!(d6.diagnostics.iter().any(|d| d.contains("checker_id")));

    // Dim 7: External-model artifacts.
    std::fs::write(baseline.dir.path().join("external-models.json"), b"{\"models\":[\"u16\"]}").expect("mut ext");
    let d7 = baseline.eval();
    assert_eq!(d7.aggregate_result, hooks::STATUS_FAILED);
    assert!(d7.diagnostics.iter().any(|d| d.contains("external-models.json")));
    std::fs::write(baseline.dir.path().join("external-models.json"), &baseline.ext_model_bytes).expect("restore ext");

    // Dim 8: Generated artifacts.
    std::fs::write(baseline.dir.path().join("Llbc.lean"), b"def safe_add := 2").expect("mut llbc");
    let d8 = baseline.eval();
    assert_eq!(d8.aggregate_result, hooks::STATUS_FAILED);
    assert!(d8.diagnostics.iter().any(|d| d.contains("Llbc.lean")));
    std::fs::write(baseline.dir.path().join("Llbc.lean"), &baseline.llbc_bytes).expect("restore llbc");

    // Dim 9: Proof-source artifacts.
    std::fs::write(baseline.dir.path().join("Theorem.lean"), b"theorem safe_add_ok : False := by sorry")
        .expect("mut thm");
    let d9 = baseline.eval();
    assert_eq!(d9.aggregate_result, hooks::STATUS_FAILED);
    assert!(d9.diagnostics.iter().any(|d| d.contains("Theorem.lean")));
    std::fs::write(baseline.dir.path().join("Theorem.lean"), &baseline.theorem_bytes).expect("restore thm");

    // Dim 10: Passing verdict.
    let mut rec10 = baseline.index_record();
    rec10.summary = "status=failed".to_owned();
    assert!(hooks::record_has_failure(&rec10));

    // Dim 11: Presence of expected-proof context.
    let mut f11 = DimensionFixture::new();
    for clause in &mut f11.contract.clauses {
        clause.required = false;
    }
    let d11 = f11.eval();
    assert_eq!(d11.aggregate_result, hooks::STATUS_FAILED);
    assert!(d11.diagnostics.iter().any(|d| d.contains("empty required inventory")));

    // Dim 12: Verifier configuration.
    let mut cfg_diags = Vec::new();
    let policy = crate::effect_closure::WorkspacePolicy {
        schema_version: crate::effect_closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
        profile_id: "deterministic-core".to_owned(),
        binding_profile: Some(crate::envelope::BindingProfile::ImplementationBoundV1),
        envelope_policy_blake3: Some(format!("b3:{}", "f".repeat(64))),
        function_address_baseline: "b3:base".to_owned(),
        module_roles: vec![crate::effect_closure::ModuleRoleAssignment {
            path: "src/lib.rs".to_owned(),
            role: crate::effect_closure::ModuleRole::Core,
        }],
        call_summaries: Vec::new(),
        non_claims: vec![
            "evaluates declared closure policy and summaries only; does not prove semantic equivalence, whole-program correctness, or release eligibility".to_owned(),
        ],
    };
    cfg_diags.extend(crate::effect_closure::validate_workspace_policy(&policy));
    assert!(cfg_diags.iter().any(|d| d.contains("envelope_policy_blake3")));

    // Dim 13: Checked scope (prose-only scope fails).
    let mut rec13 = baseline.index_record();
    rec13.source_ids.clear();
    rec13.summary = "scope:formal".to_owned();
    assert!(!hooks::record_satisfies_bindings(&rec13, &baseline.contract.clauses[0]));

    // Dim 14: Exclusions.
    let mut f14 = DimensionFixture::new();
    f14.contract.exclusions[0].reviewed_specification_blake3 = "f".repeat(64);
    let d14 = f14.eval();
    assert_eq!(d14.aggregate_result, hooks::STATUS_FAILED);
    assert!(d14.diagnostics.iter().any(|d| d.contains("reviewed_specification_blake3")));

    // Dim 15: Current correspondence derivation.
    let mut tampered = base_outcome.clone();
    tampered.aggregate_result = hooks::STATUS_FAILED.to_owned();
    let mut d15 = Vec::new();
    hooks::compare_decisions(&tampered, &base_outcome, &mut d15);
    assert!(d15.iter().any(|d| d.contains("recorded decision bytes are stale")));

    // Dim 16: Satisfied negative control.
    let mut f16 = DimensionFixture::new();
    f16.contract.clauses[0].required_negative_controls = vec!["mut.missing".to_owned()];
    let d16 = f16.eval();
    assert_eq!(d16.aggregate_result, hooks::STATUS_FAILED);
    assert!(d16.diagnostics.iter().any(|d| d.contains("mut.missing")));
}
