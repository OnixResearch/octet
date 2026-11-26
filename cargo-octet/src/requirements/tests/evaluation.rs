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
        structural_links: Vec::new(),
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

fn write_structural_fixture_bundle(
    root: &Path,
    clause_id: &str,
    locator: &str,
    src_text: &str,
) -> (SourceBinding, crate::requirements::evidence::StructuralLinkEvidence) {
    let source = write_file(root, SOURCE_PATH, src_text.as_bytes());
    let report_bytes = build_cairn_report_payload(clause_id, locator, src_text, &source);
    let profiles_bytes = include_bytes!("../../../../standards/structural-linkage/generated/profiles.json");
    let registry = crate::structural_linkage::parse_profile_registry(profiles_bytes).expect("profiles");
    let profile = crate::structural_linkage::select_profile(&registry, "cairn-structural-trace-v2").expect("profile");
    let (projection, receipt) =
        crate::structural_linkage::receipt::ingest_report(profile, &report_bytes, root).expect("ingest");
    write_file(root, "profiles.json", profiles_bytes);
    write_file(root, "cairn-report.json", &report_bytes);
    write_file(root, "projection.json", &serde_json::to_vec_pretty(&projection).expect("proj"));
    write_file(root, "receipt.json", &serde_json::to_vec_pretty(&receipt).expect("rcpt"));
    (source, crate::requirements::evidence::StructuralLinkEvidence {
        evidence_id: "cairn.link.add".to_owned(),
        evidence_class: DISPOSITION_STRUCTURAL_LINKAGE.to_owned(),
        profile_id: "cairn-structural-trace-v2".to_owned(),
        profiles: "profiles.json".to_owned(),
        report: "cairn-report.json".to_owned(),
        projection: "projection.json".to_owned(),
        receipt: "receipt.json".to_owned(),
        artifact_root: ".".to_owned(),
        checked_scope: vec![locator.to_owned()],
        non_claims: vec!["not formal proof".to_owned()],
    })
}

fn build_cairn_report_payload(clause_id: &str, locator: &str, src_text: &str, source: &SourceBinding) -> Vec<u8> {
    let ast = syn::parse_file(src_text).expect("valid rust source");
    let norm_hash = ast
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Fn(func) if func.sig.ident == "add" => {
                Some(crate::structural_linkage::source::normalized_item_hash(item))
            }
            _ => None,
        })
        .expect("fn add");
    let marker_start = src_text.find("r[impl ").expect("marker") as u64;
    let marker_span =
        serde_json::json!({"start_byte": marker_start, "end_byte": marker_start + 18, "start_line": 1, "end_line": 1});
    let row_hash = hash_bytes(b"row-add");
    let link_hash = hash_bytes(format!("{clause_id}:{}", source.blake3).as_bytes());
    let config = serde_json::json!({
        "source_path": SOURCE_PATH,
        "file_blake3": source.blake3,
        "schema_version": "cairn.rust-item-binding.v1",
        "target": "x86_64-unknown-linux-gnu",
        "features": [],
        "declared_features": [],
        "cfg": [],
    });
    let mut binding = serde_json::json!({
        "link_blake3": link_hash,
        "requirement_id": clause_id,
        "requirement_row_blake3": row_hash,
        "source_path": SOURCE_PATH,
        "source_file_blake3": source.blake3,
        "marker_span": marker_span,
        "item": {
            "source_path": SOURCE_PATH,
            "marker_start_byte": marker_start,
            "start_byte": src_text.find("pub fn add").expect("fn add") as u64,
            "end_byte": src_text.len() as u64,
            "item_kind": "fn",
            "scoped_locator": locator,
            "normalized_item_blake3": norm_hash,
        },
        "configuration": config,
        "binding_blake3": "",
    });
    let binding_hash = crate::structural_linkage::domain_hash_json(
        "cairn.trace-index.rust-item-binding.v1",
        &(
            binding["link_blake3"].as_str().unwrap(),
            binding["requirement_id"].as_str().unwrap(),
            binding["requirement_row_blake3"].as_str().unwrap(),
            binding["source_path"].as_str().unwrap(),
            binding["source_file_blake3"].as_str().unwrap(),
            &binding["marker_span"],
            &binding["item"],
            "cairn.saved-rust-bytes.v1",
            "syn-2.0.117",
            "cairn.rust-item-binding.v1",
            &binding["configuration"],
        ),
    )
    .expect("binding hash");
    binding["binding_blake3"] = serde_json::Value::String(binding_hash);
    let mut report = serde_json::json!({
        "schema_version": "cairn.content-bound-trace-index.v2",
        "boundary": "content-bound trace index",
        "profile": {"name": "cairn-pilot"},
        "profile_blake3": hash_bytes(b"p"),
        "policy_blake3": hash_bytes(b"pol"),
        "requirement_registry_blake3": hash_bytes(b"reg"),
        "requirements": [{"requirement_id": clause_id, "row_blake3": row_hash}],
        "sources": [{"path": SOURCE_PATH, "content_blake3": source.blake3}],
        "links": [{
            "requirement_id": clause_id,
            "requirement_row_blake3": row_hash,
            "role": "impl",
            "source_path": SOURCE_PATH,
            "source_file_blake3": source.blake3,
            "marker_span": marker_span,
            "marker_blake3": hash_bytes(b"// r[impl clause.add]"),
            "link_blake3": link_hash,
        }],
        "non_claims": ["not formal proof"],
        "structural": {
            "adapter": "cairn.saved-rust-bytes.v1",
            "parser": "syn-2.0.117",
            "schema": "cairn.rust-item-binding.v1",
            "configuration": config,
            "bindings": [binding],
            "history": [],
        }
    });
    let index_hash = crate::structural_linkage::domain_hash_json("cairn.trace-index.structural-index.v2", &report)
        .expect("index hash");
    report["index_blake3"] = serde_json::Value::String(index_hash);
    serde_json::to_vec_pretty(&report).expect("report bytes")
}

#[test]
fn structural_linkage_clause_passes_verifies_and_cannot_promote_to_formal_proof() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let src_text = "// r[impl clause.add]\npub fn add(a: u8, b: u8) -> u8 { a + b }\n";
    let (source, link_ev) = write_structural_fixture_bundle(root.path(), "clause.add", "fixture/fn:add#0", src_text);

    let mut contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_STRUCTURAL_LINKAGE);
    contract.clauses[0].checked_scope = vec!["fixture/fn:add#0".to_owned()];
    contract.clauses[0].expected_evidence_tier = DISPOSITION_STRUCTURAL_LINKAGE.to_owned();
    contract.clauses[0].allowed_evidence_ids = vec!["cairn.link.add".to_owned()];
    contract.clauses[0].required_evidence_classes = vec![DISPOSITION_STRUCTURAL_LINKAGE.to_owned()];
    let mut evidence = evidence(String::new(), source);
    evidence.structural_links = vec![link_ev.clone()];
    let contract_bytes = serde_json::to_vec(&contract).expect("contract bytes");
    evidence.contract_blake3 = Some(hash_bytes(&contract_bytes));
    let decision = evaluate_contract(
        &contract,
        &evidence,
        &contract_bytes,
        &serde_json::to_vec(&evidence).expect("evidence"),
        root.path(),
    )
    .expect("evaluate");
    assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
    assert_eq!(decision.clauses[0].status, STATUS_PASSED);

    // Formal-proof clause cannot be satisfied even when structural-linkage evidence is supplied.
    let mut formal_contract = contract.clone();
    formal_contract.clauses[0].disposition = DISPOSITION_FORMAL_PROOF.to_owned();
    formal_contract.clauses[0].expected_evidence_tier = DISPOSITION_FORMAL_PROOF.to_owned();
    formal_contract.clauses[0].required_evidence_classes = vec!["proof_card".to_owned()];
    let formal_bytes = serde_json::to_vec(&formal_contract).expect("formal bytes");
    evidence.contract_blake3 = Some(hash_bytes(&formal_bytes));
    let formal_decision = evaluate_contract(
        &formal_contract,
        &evidence,
        &formal_bytes,
        &serde_json::to_vec(&evidence).expect("ev"),
        root.path(),
    )
    .expect("evaluate formal");
    assert_eq!(formal_decision.aggregate_result, STATUS_FAILED);
    assert_eq!(formal_decision.clauses[0].status, STATUS_MISSING);

    // Claiming formal_proof tier on a structural_linkage clause is rejected by validation & runtime.
    let mut mixed = contract.clone();
    mixed.clauses[0].expected_evidence_tier = DISPOSITION_FORMAL_PROOF.to_owned();
    mixed.clauses[0].required_evidence_classes = vec!["formal_proof".to_owned()];
    let mixed_bytes = serde_json::to_vec(&mixed).expect("mixed bytes");
    evidence.contract_blake3 = Some(hash_bytes(&mixed_bytes));
    let mixed_decision =
        evaluate_contract(&mixed, &evidence, &mixed_bytes, &serde_json::to_vec(&evidence).expect("ev"), root.path())
            .expect("evaluate mixed");
    assert_eq!(mixed_decision.aggregate_result, STATUS_FAILED);
}

#[test]
fn structural_linkage_rejects_prose_only_or_wrong_tier_claim_even_with_valid_receipt() {
    let root = fixture_root();
    let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
    let src_text = "// r[impl clause.add]\npub fn add(a: u8, b: u8) -> u8 { a + b }\n";
    let (source, mut link_ev) =
        write_structural_fixture_bundle(root.path(), "clause.add", "fixture/fn:add#0", src_text);
    link_ev.checked_scope = vec!["unrelated_prose_claim".to_owned()];
    let mut contract = contract(source.clone(), spec.blake3, DISPOSITION_STRUCTURAL_LINKAGE);
    contract.clauses[0].checked_scope = vec!["fixture/fn:other#0".to_owned()];
    contract.clauses[0].expected_evidence_tier = DISPOSITION_STRUCTURAL_LINKAGE.to_owned();
    contract.clauses[0].allowed_evidence_ids = vec!["cairn.link.add".to_owned()];
    contract.clauses[0].required_evidence_classes = vec![DISPOSITION_STRUCTURAL_LINKAGE.to_owned()];
    let mut evidence = evidence(String::new(), source);
    evidence.structural_links = vec![link_ev];
    let contract_bytes = serde_json::to_vec(&contract).expect("contract");
    evidence.contract_blake3 = Some(hash_bytes(&contract_bytes));
    let decision = evaluate_contract(
        &contract,
        &evidence,
        &contract_bytes,
        &serde_json::to_vec(&evidence).expect("ev"),
        root.path(),
    )
    .expect("eval");
    assert_eq!(decision.aggregate_result, STATUS_FAILED);
    assert!(decision.diagnostics.iter().any(|diag| diag.contains("carries no active non-historical anchor")));
}
