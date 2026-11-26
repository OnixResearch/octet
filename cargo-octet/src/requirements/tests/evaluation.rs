use super::*;

#[test]
fn valid_formal_clause_passes_and_recomputes() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"pub fn add(a: u8, b: u8) -> u8 { a + b }");
    let mut contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_FORMAL_PROOF);
    contract.clauses[0].allowed_source_requirement_ids = vec!["clause.add".to_owned()];
    let mut evidence = evidence(String::new(), source.clone());
    evidence.proof_decks = vec![proof_deck(root.path(), "passed")];
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    evidence.contract_blake3 = Some(hash_bytes(&contract_bytes));
    let decision = evaluate_contract(
        &contract,
        &evidence,
        &contract_bytes,
        &serde_json::to_vec(&evidence).expect("serialize evidence"),
        root.path(),
        None,
    )
    .expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
    assert_eq!(decision.clauses[0].status, STATUS_PASSED);
    let mut recorded = decision.clone();
    recorded.decision_blake3 = String::new();
    recorded.decision_blake3 = digest(&recorded).expect("hash decision");
    let mut diagnostics = Vec::new();
    compare_decisions(&recorded, &decision, &mut diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn missing_evidence_fails_closed() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"source");
    let contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_FORMAL_PROOF);
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let mut evidence = evidence(hash_bytes(&contract_bytes), source);
    evidence.proof_decks = vec![proof_deck(root.path(), "failed")];
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None).expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_FAILED);
    assert_eq!(decision.clauses[0].status, STATUS_MISSING);
}

#[test]
fn changed_source_and_spec_fail_closed() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"source");
    let contract = contract(source, spec.blake3, DISPOSITION_FORMAL_PROOF);
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    fs::write(root.path().join(SPEC_PATH), b"changed spec").expect("change spec");
    fs::write(root.path().join(SOURCE_PATH), b"changed source").expect("change source");
    let evidence = evidence(hash_bytes(&contract_bytes), SourceBinding {
        locator: SOURCE_PATH.to_owned(),
        blake3: hash_bytes(b"source"),
    });
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None).expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_FAILED);
    assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("reviewed specification")));
    assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("source")));
}

#[test]
fn empirical_evidence_cannot_satisfy_formal_clause() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"spec");
    let source = write_file(root.path(), SOURCE_PATH, b"source");
    let mut contract = contract(source, spec.blake3, DISPOSITION_EMPIRICAL_CHECK);
    contract.clauses[0].required_evidence_classes = vec!["formal_proof".to_owned()];
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let mut evidence = evidence(hash_bytes(&contract_bytes), SourceBinding {
        locator: SOURCE_PATH.to_owned(),
        blake3: hash_bytes(b"source"),
    });
    evidence.empirical_checks.push(EmpiricalCheck {
        evidence_id: PROOF_CARD_ID.to_owned(),
        evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
        status: CHECK_STATUS_PASSED.to_owned(),
        checker_id: "test".to_owned(),
        command_argv: vec!["cargo".to_owned(), "test".to_owned()],
        checked_scope: vec!["test".to_owned()],
        source_bindings: vec![SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: hash_bytes(b"source"),
        }],
        artifact: Some("check.txt".to_owned()),
        artifact_blake3: Some(hash_bytes(b"check output")),
        diagnostics: Vec::new(),
        non_claims: Vec::new(),
    });
    fs::write(root.path().join("check.txt"), b"check output").expect("write check artifact");
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None).expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_FAILED, "{:?}", decision.diagnostics);
    assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("formal-proof")));
}

#[test]
fn duplicate_clause_ids_and_empty_required_inventory_fail() {
    let mut contract = contract(
        SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: "0".repeat(64),
        },
        "0".repeat(64),
        DISPOSITION_FORMAL_PROOF,
    );
    contract.clauses.push(contract.clauses[0].clone());
    assert!(validate(&contract).iter().any(|diagnostic| diagnostic.contains("duplicate")));
    contract.clauses.truncate(1);
    contract.clauses[0].required = false;
    let diagnostics = validate(&contract);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("empty required inventory")));
}
#[test]
fn checked_example_contract_passes_and_is_recomputable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let contract_path = root.join("standards/requirements/example-contract.json");
    let evidence_path = root.join("standards/requirements/fixtures/example/evidence.json");
    let contract: Definition = read_json_bounded(&contract_path).expect("read example contract");
    let evidence: Bundle = read_json_bounded(&evidence_path).expect("read example evidence");
    let contract_bytes = fs::read(&contract_path).expect("read contract bytes");
    let evidence_bytes = fs::read(&evidence_path).expect("read evidence bytes");
    let decision = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, root, None)
        .expect("evaluate example contract");
    assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
    assert_eq!(decision.clauses[0].status, STATUS_PASSED);
    assert_eq!(decision.clauses[1].status, STATUS_ASSUMPTION_ACCEPTED);
    assert_eq!(decision.clauses[2].status, STATUS_EXCLUDED);
    assert!(decision.assumptions.iter().all(|assumption| assumption.status == STATUS_ASSUMPTION_ACCEPTED));
    let mut recorded = decision.clone();
    recorded.decision_blake3 = String::new();
    recorded.decision_blake3 = digest(&recorded).expect("hash decision");
    let mut diagnostics = Vec::new();
    compare_decisions(&recorded, &decision, &mut diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
#[test]
fn tampered_decision_bytes_are_rejected() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"source");
    let contract = contract(source.clone(), spec.blake3, DISPOSITION_FORMAL_PROOF);
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let mut evidence = evidence(hash_bytes(&contract_bytes), source);
    evidence.proof_decks = vec![proof_deck(root.path(), "passed")];
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None).expect("evaluate");
    let mut tampered = decision.clone();
    tampered.aggregate_result = STATUS_FAILED.to_owned();
    let mut diagnostics = Vec::new();
    compare_decisions(&tampered, &decision, &mut diagnostics);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded decision bytes are stale")));
    let mut hash_tampered = decision.clone();
    hash_tampered.decision_blake3 = "0".repeat(64);
    let mut diagnostics = Vec::new();
    compare_decisions(&hash_tampered, &decision, &mut diagnostics);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("does not match the recomputed")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded decision bytes are stale")));
}

#[test]
fn changed_assumption_identity_cannot_inherit_acceptance() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"spec");
    let source = write_file(root.path(), SOURCE_PATH, b"source");
    let mut contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_ACCEPTED_ASSUMPTION);
    contract.clauses[0].assumption_id = Some("assumption.test".to_owned());
    contract.assumptions.push(AcceptedAssumption {
        assumption_id: "assumption.test".to_owned(),
        statement: "fixture assumption".to_owned(),
        identity: "identity-a".to_owned(),
        trusted_boundary_identity: "boundary-a".to_owned(),
        affected_clause_ids: vec![contract.clauses[0].clause_id.clone()],
        required_empirical_evidence: vec!["check.test".to_owned()],
    });
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let mut evidence = evidence(hash_bytes(&contract_bytes), source.clone());
    evidence.empirical_checks.push(EmpiricalCheck {
        evidence_id: "check.test".to_owned(),
        evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
        status: CHECK_STATUS_PASSED.to_owned(),
        checker_id: "test".to_owned(),
        command_argv: vec!["cargo".to_owned(), "test".to_owned()],
        checked_scope: vec!["test".to_owned()],
        source_bindings: vec![source],
        artifact: Some("check.txt".to_owned()),
        artifact_blake3: Some(hash_bytes(b"check")),
        diagnostics: Vec::new(),
        non_claims: Vec::new(),
    });
    fs::write(root.path().join("check.txt"), b"check").expect("write artifact");
    evidence.assumption_records.push(AssumptionRecord {
        assumption_id: "assumption.test".to_owned(),
        identity: "identity-b".to_owned(),
        trusted_boundary_identity: "boundary-a".to_owned(),
        status: ASSUMPTION_STATUS_ACCEPTED.to_owned(),
        supporting_evidence_ids: vec!["check.test".to_owned()],
        diagnostics: Vec::new(),
    });
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None).expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_FAILED);
    assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("identity changed")));
}

#[test]
fn required_exclusion_does_not_pass() {
    let mut contract = contract(
        SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: "0".repeat(64),
        },
        "0".repeat(64),
        DISPOSITION_EXCLUDED,
    );
    contract.clauses[0].exclusion_reason = Some("outside scope".to_owned());
    contract.exclusions.push(ExcludedBehavior {
        exclusion_id: "exclusion.test".to_owned(),
        clause_id: contract.clauses[0].clause_id.clone(),
        reason: "outside scope".to_owned(),
        reviewed_specification_blake3: String::new(),
    });
    let mut diagnostics = Vec::new();
    let evidence = Bundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3: None,
        source_snapshot: Vec::new(),
        proof_decks: Vec::new(),
        empirical_checks: Vec::new(),
        assumption_records: Vec::new(),
        non_claims: Vec::new(),
    };
    let assumptions = BTreeMap::new();
    let evaluator = Context {
        contract: &contract,
        evidence: &evidence,
        source_root: Path::new("."),
        artifact_base: None,
        assumptions: &assumptions,
    };
    let decision = super::super::evaluation::clause::evaluate(&evaluator, &contract.clauses[0], &mut diagnostics);
    assert_eq!(decision.status, STATUS_EXCLUDED);
    assert!(!decision.diagnostics.is_empty());
}

fn relative_deck_fixture() -> (tempfile::TempDir, Definition, Bundle) {
    let source = fixture_root();
    let spec = write_file(source.path(), SPEC_PATH, b"reviewed specification");
    let source_binding = write_file(source.path(), SOURCE_PATH, b"pub fn add(a: u8, b: u8) -> u8 { a + b }");
    let contract = contract(source_binding.clone(), spec.blake3, DISPOSITION_FORMAL_PROOF);
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let evidence = Bundle {
        schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3: Some(hash_bytes(&contract_bytes)),
        source_snapshot: vec![source_binding],
        proof_decks: Vec::new(),
        empirical_checks: Vec::new(),
        assumption_records: Vec::new(),
        non_claims: Vec::new(),
    };
    (source, contract, evidence)
}

fn copy_proof_tree(from: &Path, to: &Path) {
    for file in [PROOF_DECK, "proof.receipt"] {
        fs::copy(from.join(file), to.join(file)).expect("copy proof tree file");
    }
}

#[test]
fn relative_deck_resolves_inside_two_artifact_bases_identically() {
    let (source, contract, mut evidence) = relative_deck_fixture();
    let first = fixture_root();
    evidence.proof_decks = vec![proof_deck(first.path(), "passed")];
    let second = fixture_root();
    copy_proof_tree(first.path(), second.path());
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
    let first_decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(first.path()))
            .expect("evaluate first base");
    let second_decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(second.path()))
            .expect("evaluate second base");
    for decision in [&first_decision, &second_decision] {
        assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
        assert_eq!(decision.clauses[0].status, STATUS_PASSED);
    }
    assert_eq!(
        serde_json::to_vec(&first_decision).expect("serialize first decision"),
        serde_json::to_vec(&second_decision).expect("serialize second decision")
    );
    assert_eq!(first_decision.decision_blake3, second_decision.decision_blake3);
}

#[test]
fn escaping_relative_and_base_prefixed_decks_are_refused() {
    let (source, contract, mut evidence) = relative_deck_fixture();
    let base = fixture_root();
    let deck = proof_deck(base.path(), "passed");
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");

    for (locator, field) in [
        ("../proof-cards.json".to_owned(), "proof deck"),
        (format!("{}/../proof-cards.json", base.path().display()), "proof deck"),
    ] {
        let mut escaped = deck.clone();
        escaped.deck = locator.clone();
        evidence.proof_decks = vec![escaped];
        let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(base.path()))
                .expect("evaluate escaping deck");
        assert_eq!(decision.aggregate_result, STATUS_FAILED, "{locator}");
        assert!(
            decision
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains(field) && diagnostic.contains("escapes the artifact base")),
            "{locator}: {:?}",
            decision.diagnostics
        );
    }

    let mut escaped_root = deck.clone();
    escaped_root.artifact_root = "../elsewhere".to_owned();
    evidence.proof_decks = vec![escaped_root];
    let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
    let decision =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(base.path()))
            .expect("evaluate escaping artifact root");
    assert_eq!(decision.aggregate_result, STATUS_FAILED);
    assert!(
        decision
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("artifact root") && diagnostic.contains("escapes the artifact base")),
        "{:?}",
        decision.diagnostics
    );
}

#[test]
fn absolute_deck_locators_keep_their_resolution() {
    let (source, contract, mut evidence) = relative_deck_fixture();
    let tree = fixture_root();
    let mut deck = proof_deck(tree.path(), "passed");
    deck.deck = tree.path().join(PROOF_DECK).to_string_lossy().into_owned();
    deck.artifact_root = tree.path().to_string_lossy().into_owned();
    evidence.proof_decks = vec![deck];
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
    let unrelated_base = fixture_root();
    for artifact_base in [None, Some(unrelated_base.path())] {
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), artifact_base)
                .expect("evaluate absolute deck");
        assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
    }
}

#[test]
fn verify_recomputes_and_reports_changed_proof_card_bytes() {
    let (source, contract, mut evidence) = relative_deck_fixture();
    let first = fixture_root();
    evidence.proof_decks = vec![proof_deck(first.path(), "passed")];
    let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
    let evidence_bytes = serde_json::to_vec(&evidence).expect("serialize evidence");
    let recorded =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(first.path()))
            .expect("evaluate recorded");
    let recomputed =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(first.path()))
            .expect("evaluate recomputed");
    let mut diagnostics = Vec::new();
    compare_decisions(&recorded, &recomputed, &mut diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    let second = fixture_root();
    // Same relative locator, different proof-card bytes.
    let _ = proof_deck(second.path(), "failed");
    let mismatch =
        evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, source.path(), Some(second.path()))
            .expect("evaluate changed base");
    let mut diagnostics = Vec::new();
    compare_decisions(&recorded, &mismatch, &mut diagnostics);
    assert_ne!(recorded.decision_blake3, mismatch.decision_blake3);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.contains("does not match the recomputed identity")),
        "{diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded decision aggregate_result")),
        "{diagnostics:?}"
    );
}

#[test]
fn proof_card_and_empirical_check_gaps_fail_closed() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"pub fn add(a: u8, b: u8) -> u8 { a + b }");

    // 1. Missing checked_rust_source_scope, source_requirement_or_policy_id, or empty
    //    artifact_blake3_identities fails.
    let contract_base = contract(source.clone(), spec.blake3.clone(), DISPOSITION_FORMAL_PROOF);
    let contract_bytes = serde_json::to_vec(&contract_base).expect("serialize contract");
    let bad_deck = serde_json::json!({
        "schema_version": PROOF_CARDS_SCHEMA_VERSION,
        "cards": [{
            "proof_or_property_id": PROOF_CARD_ID,
            "evidence_tier": "formal_proof",
            "status": "passed",
            "non_claims": ["bounded"],
            "diagnostics": [],
            "artifact_blake3_identities": [],
        }],
    });
    fs::write(root.path().join(PROOF_DECK), serde_json::to_vec_pretty(&bad_deck).expect("deck")).expect("write");
    let mut bad_evidence = evidence(hash_bytes(&contract_bytes), source.clone());
    bad_evidence.proof_decks = vec![ProofDeck {
        evidence_id: PROOF_CARD_ID.to_owned(),
        deck: PROOF_DECK.to_owned(),
        artifact_root: ".".to_owned(),
    }];
    let bad_card_decision =
        evaluate_contract(&contract_base, &bad_evidence, &contract_bytes, b"ev", root.path(), None).expect("eval");
    assert_eq!(bad_card_decision.aggregate_result, STATUS_FAILED);
    assert!(bad_card_decision.diagnostics.iter().any(|d| d.contains("checked_rust_source_scope")));
    assert!(bad_card_decision.diagnostics.iter().any(|d| d.contains("source_requirement_or_policy_id")));
    assert!(bad_card_decision.diagnostics.iter().any(|d| d.contains("empty artifact_blake3_identities")));

    // 2. Empirical check with empty checker_id or mismatched checked_scope fails.
    let mut emp_contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_EMPIRICAL_CHECK);
    emp_contract.clauses[0].checked_scope = vec!["scope:expected".to_owned()];
    emp_contract.clauses[0].expected_evidence_tier = DISPOSITION_EMPIRICAL_CHECK.to_owned();
    emp_contract.clauses[0].required_evidence_classes = vec![DISPOSITION_EMPIRICAL_CHECK.to_owned()];
    let emp_bytes = serde_json::to_vec(&emp_contract).expect("emp contract");
    let mut emp_evidence = evidence(hash_bytes(&emp_bytes), source);
    fs::write(root.path().join("emp.txt"), b"ok").expect("write emp");
    emp_evidence.empirical_checks.push(EmpiricalCheck {
        evidence_id: PROOF_CARD_ID.to_owned(),
        evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
        status: CHECK_STATUS_PASSED.to_owned(),
        checker_id: String::new(),
        command_argv: vec!["cargo".to_owned(), "test".to_owned()],
        checked_scope: vec!["scope:other".to_owned()],
        source_bindings: vec![emp_contract.clauses[0].source_bindings[0].clone()],
        artifact: Some("emp.txt".to_owned()),
        artifact_blake3: Some(hash_bytes(b"ok")),
        diagnostics: Vec::new(),
        non_claims: Vec::new(),
    });
    let emp_decision =
        evaluate_contract(&emp_contract, &emp_evidence, &emp_bytes, b"ev", root.path(), None).expect("eval");
    assert_eq!(emp_decision.aggregate_result, STATUS_FAILED);
    assert!(emp_decision.diagnostics.iter().any(|d| d.contains("checker_id")));
    assert!(emp_decision.diagnostics.iter().any(|d| d.contains("checked_scope")));
}

#[test]
fn negative_control_and_exclusion_gaps_fail_closed() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let source = write_file(root.path(), SOURCE_PATH, b"pub fn add(a: u8, b: u8) -> u8 { a + b }");
    let contract_base = contract(source.clone(), spec.blake3, DISPOSITION_FORMAL_PROOF);

    // 1. Required negative control checks mutant_card_status == "failed" and required mutation_id.
    let mut neg_contract = contract_base.clone();
    neg_contract.clauses[0].required_negative_controls = vec!["mut.overflow".to_owned()];
    let neg_bytes = serde_json::to_vec(&neg_contract).expect("neg contract");
    let artifact_bytes = b"{\"status\":\"passed\"}";
    fs::write(root.path().join("proof.receipt"), artifact_bytes).expect("write receipt");
    let mut neg_card = serde_json::json!({
        "proof_or_property_id": PROOF_CARD_ID,
        "source_requirement_or_policy_id": "clause.add",
        "checked_rust_source_scope": "formal-proof:fixture",
        "evidence_tier": "formal_proof",
        "status": "passed",
        "non_claims": ["bounded"],
        "diagnostics": [],
        "artifact_blake3_identities": [{"artifact": "proof.receipt", "blake3": hash_bytes(artifact_bytes)}],
        "negative_check": {
            "schema_version": NEGATIVE_CHECK_SCHEMA_VERSION_V2,
            "status": NEGATIVE_STATUS_DEMONSTRATED,
            "mutant_card_status": "failed",
            "mutant_card_evidence_tier": "formal_proof",
            "evidence": {
                "verifier": "aeneas-lean",
                "proof_or_property_id": PROOF_CARD_ID,
                "approved_specification_id": "spec.1",
                "source_requirement_or_policy_id": "clause.add",
                "checked_rust_source_scope": "formal-proof:fixture",
                "evidence_tier": "formal_proof",
                "baseline": { "outcome": "accepted" },
                "mutant": { "outcome": "rejected", "rejection_attribution": PROOF_CARD_ID },
                "mutation": {
                    "mutation_id": "mut.overflow",
                    "patch_blake3": "a".repeat(64),
                    "baseline_source_blake3": "b".repeat(64),
                    "mutant_source_blake3": "c".repeat(64)
                }
            },
            "measured": {
                "mutation_id": "mut.overflow",
                "patch_blake3": "a".repeat(64),
                "baseline_source_blake3": "b".repeat(64),
                "mutant_source_blake3": "c".repeat(64)
            }
        }
    });
    let write_neg_deck = |card_val: &serde_json::Value| {
        let deck = serde_json::json!({
            "schema_version": PROOF_CARDS_SCHEMA_VERSION,
            "cards": [card_val],
        });
        fs::write(root.path().join(PROOF_DECK), serde_json::to_vec_pretty(&deck).expect("deck")).expect("write");
    };
    write_neg_deck(&neg_card);
    let mut neg_evidence = evidence(hash_bytes(&neg_bytes), source);
    neg_evidence.proof_decks = vec![ProofDeck {
        evidence_id: PROOF_CARD_ID.to_owned(),
        deck: PROOF_DECK.to_owned(),
        artifact_root: ".".to_owned(),
    }];
    let valid_neg =
        evaluate_contract(&neg_contract, &neg_evidence, &neg_bytes, b"ev", root.path(), None).expect("eval");
    assert_eq!(valid_neg.aggregate_result, STATUS_PASSED, "{:?}", valid_neg.diagnostics);

    neg_card["negative_check"]["evidence"]["mutation"]["mutation_id"] = serde_json::json!("mut.other");
    neg_card["negative_check"]["measured"]["mutation_id"] = serde_json::json!("mut.other");
    write_neg_deck(&neg_card);
    let wrong_mut =
        evaluate_contract(&neg_contract, &neg_evidence, &neg_bytes, b"ev", root.path(), None).expect("eval");
    assert_eq!(wrong_mut.aggregate_result, STATUS_FAILED);
    assert!(wrong_mut.diagnostics.iter().any(|d| d.contains("mut.overflow")));

    // 4. Stale exclusion reviewed_specification_blake3 fails contract validation.
    let mut excl_contract = contract_base;
    excl_contract.clauses.push(RequirementClause {
        clause_id: "clause.excl".to_owned(),
        statement: "excluded".to_owned(),
        required: false,
        disposition: DISPOSITION_EXCLUDED.to_owned(),
        checked_scope: Vec::new(),
        expected_evidence_tier: String::new(),
        allowed_evidence_ids: Vec::new(),
        allowed_source_requirement_ids: Vec::new(),
        required_evidence_classes: Vec::new(),
        required_negative_controls: Vec::new(),
        required_bindings: RequirementBindings::default(),
        source_bindings: Vec::new(),
        dependency_bindings: Vec::new(),
        assumption_id: None,
        exclusion_reason: Some("outside scope".to_owned()),
        required_provider_id: None,
    });
    excl_contract.exclusions.push(ExcludedBehavior {
        exclusion_id: "excl.1".to_owned(),
        clause_id: "clause.excl".to_owned(),
        reason: "outside scope".to_owned(),
        reviewed_specification_blake3: "f".repeat(64),
    });
    let excl_diags = validate(&excl_contract);
    assert!(excl_diags.iter().any(|d| d.contains("reviewed_specification_blake3")));
}

#[test]
fn empirical_clause_cannot_promote_to_formal_proof_via_tier_or_bindings() {
    for use_binding_tier in [false, true] {
        for with_empirical_check in [false, true] {
            let root = fixture_root();
            let spec = write_file(root.path(), SPEC_PATH, b"spec");
            let source = write_file(root.path(), SOURCE_PATH, b"source");
            let mut contract = contract(source.clone(), spec.blake3, DISPOSITION_EMPIRICAL_CHECK);
            contract.clauses[0].required_evidence_classes = vec![DISPOSITION_EMPIRICAL_CHECK.to_owned()];
            if use_binding_tier {
                contract.clauses[0].expected_evidence_tier = DISPOSITION_EMPIRICAL_CHECK.to_owned();
                contract.clauses[0].required_bindings.evidence_tier = Some(DISPOSITION_FORMAL_PROOF.to_owned());
            } else {
                contract.clauses[0].expected_evidence_tier = DISPOSITION_FORMAL_PROOF.to_owned();
                contract.clauses[0].required_bindings.evidence_tier = None;
            }
            let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
            let mut evidence = evidence(hash_bytes(&contract_bytes), source.clone());
            if with_empirical_check {
                fs::write(root.path().join("check.txt"), b"check output").expect("write check artifact");
                evidence.empirical_checks.push(EmpiricalCheck {
                    evidence_id: PROOF_CARD_ID.to_owned(),
                    evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
                    status: CHECK_STATUS_PASSED.to_owned(),
                    checker_id: "test".to_owned(),
                    command_argv: vec!["cargo".to_owned(), "test".to_owned()],
                    checked_scope: vec!["test".to_owned()],
                    source_bindings: vec![source],
                    artifact: Some("check.txt".to_owned()),
                    artifact_blake3: Some(hash_bytes(b"check output")),
                    diagnostics: Vec::new(),
                    non_claims: Vec::new(),
                });
            }
            let decision =
                evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path(), None)
                    .expect("evaluate");
            assert_eq!(decision.aggregate_result, STATUS_FAILED, "{:?}", decision.diagnostics);
            assert_eq!(decision.clauses[0].status, STATUS_FAILED);
            assert!(
                decision.diagnostics.iter().any(|d| d.contains("formal_proof") || d.contains("formal-proof")),
                "{:?}",
                decision.diagnostics
            );
        }
    }
}
