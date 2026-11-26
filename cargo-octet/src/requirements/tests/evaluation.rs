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
