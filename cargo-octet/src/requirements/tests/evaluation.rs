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
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
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
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
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
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
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
    let decision = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, root)
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
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
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
        evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
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
        assumptions: &assumptions,
    };
    let decision = super::super::evaluation::clause::evaluate(&evaluator, &contract.clauses[0], &mut diagnostics);
    assert_eq!(decision.status, STATUS_EXCLUDED);
    assert!(!decision.diagnostics.is_empty());
}
