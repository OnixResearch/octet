use std::path::Path;

use hegel::generators as gs;

use super::super::CaseTally;

const MAX_TAG_BYTES: usize = 16;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn canonical_hash_permutation_invariance(tc: hegel::TestCase, db_path: Option<String>) {
    let tag_a = tc.draw(gs::text().max_size(MAX_TAG_BYTES));
    let tag_b = tc.draw(gs::text().max_size(MAX_TAG_BYTES));
    let num = tc.draw(gs::integers::<u32>().min_value(0).max_value(10_000));
    let secret = tc.draw(gs::text().max_size(MAX_TAG_BYTES));
    let schema = tc.draw(gs::integers::<u32>().min_value(1).max_value(5));
    let _case = CaseTally;

    let ws_a = Path::new("/tmp/ws-alpha");
    let ws_b = Path::new("/tmp/ws-beta");
    let home = Path::new("/tmp/home-a");
    let temp = Path::new("/tmp/scratch-a");

    let mut map_a = serde_json::Map::new();
    map_a.insert("alpha".to_owned(), serde_json::json!(tag_a));
    map_a.insert("beta".to_owned(), serde_json::json!(num));
    map_a.insert("path".to_owned(), serde_json::json!("/tmp/ws-alpha/src/lib.rs"));
    map_a.insert("api_token".to_owned(), serde_json::json!(secret));
    map_a.insert("gamma".to_owned(), serde_json::json!(tag_b));

    let mut map_b = serde_json::Map::new();
    map_b.insert("gamma".to_owned(), serde_json::json!(tag_b));
    map_b.insert("path".to_owned(), serde_json::json!("/tmp/ws-beta/src/lib.rs"));
    map_b.insert("beta".to_owned(), serde_json::json!(num));
    map_b.insert("alpha".to_owned(), serde_json::json!(tag_a));

    let (canon_a, hash_a) = crate::roadmap::test_hooks::normalize_and_hash_artifact(
        schema,
        "evidence-summary",
        serde_json::Value::Object(map_a),
        ws_a,
        home,
        temp,
    )
    .expect("hash a");
    let (canon_b, hash_b) = crate::roadmap::test_hooks::normalize_and_hash_artifact(
        schema,
        "evidence-summary",
        serde_json::Value::Object(map_b),
        ws_b,
        home,
        temp,
    )
    .expect("hash b");
    assert_eq!(canon_a, canon_b);
    assert_eq!(hash_a, hash_b);

    // Negative control: changing required `kind` or `schema_version` must change identity.
    let (_canon_diff_kind, hash_diff_kind) = crate::roadmap::test_hooks::normalize_and_hash_artifact(
        schema,
        "different-kind",
        serde_json::json!({ "alpha": tag_a, "beta": num, "gamma": tag_b, "path": "/tmp/ws-alpha/src/lib.rs" }),
        ws_a,
        home,
        temp,
    )
    .expect("hash diff kind");
    assert_ne!(hash_a, hash_diff_kind);

    let outcome = crate::requirements::tests::hooks::Outcome {
        schema_version: "octet-requirement-decision/v1".to_owned(),
        contract_id: format!("contract.{num}"),
        contract_blake3: "a".repeat(64),
        evidence_blake3: "b".repeat(64),
        source_snapshot_blake3: "c".repeat(64),
        reviewed_specification: crate::requirements::tests::hooks::ReviewedSpecification {
            path: "spec.md".to_owned(),
            blake3: "d".repeat(64),
        },
        aggregate_result: "passed".to_owned(),
        clauses: Vec::new(),
        assumptions: Vec::new(),
        exclusions: Vec::new(),
        limitations: vec![tag_a],
        extraction_summary: None,
        diagnostics: Vec::new(),
        non_claims: vec!["bounded".to_owned()],
        decision_blake3: String::new(),
    };
    let digest_a = crate::requirements::tests::hooks::digest(&outcome).expect("digest a");
    let mut mutated_subject = outcome;
    mutated_subject.contract_id = format!("contract.{num}.mutated");
    let digest_b = crate::requirements::tests::hooks::digest(&mutated_subject).expect("digest b");
    assert_ne!(digest_a, digest_b);
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn framed_provider_role_separation(tc: hegel::TestCase, db_path: Option<String>) {
    let p1 = tc.draw(gs::vecs(gs::integers::<u8>()).max_size(MAX_TAG_BYTES));
    let p2 = tc.draw(gs::vecs(gs::integers::<u8>()).max_size(MAX_TAG_BYTES));
    let name = tc.draw(gs::text().max_size(MAX_TAG_BYTES));
    let version = tc.draw(gs::text().max_size(MAX_TAG_BYTES));
    let is_complete = tc.draw(gs::booleans());
    tc.assume(!p2.is_empty());
    let _case = CaseTally;

    let framed = crate::envelope::framed_blake3("octet-identity/v1", &[&p1, &p2]);
    let joined = [p1.as_slice(), p2.as_slice()].concat();
    let shifted = crate::envelope::framed_blake3("octet-identity/v1", &[&joined, &[]]);
    assert_ne!(framed, shifted);

    let provider = crate::effect_closure::ProviderIdentity {
        kind: crate::effect_closure::ProviderKind::Workspace,
        name: name.clone(),
        version,
        source: "workspace".to_owned(),
        target: "x86_64-unknown-linux-gnu".to_owned(),
        features: vec!["default".to_owned()],
    };
    let id_a = crate::effect_closure::provider_identity_blake3(&provider);
    let mut omitted_role = provider;
    omitted_role.features.clear();
    let id_b = crate::effect_closure::provider_identity_blake3(&omitted_role);
    assert_ne!(id_a, id_b);

    let call_a = crate::effect_closure::call_site_identity("fn::a", "src/lib.rs", &name, "direct", 0);
    let call_b = crate::effect_closure::call_site_identity("fn::a", "src/lib.rs", &name, "direct", 1);
    assert_ne!(call_a, call_b);

    let completeness = if is_complete {
        crate::envelope::CompletenessStatus::Complete
    } else {
        crate::envelope::CompletenessStatus::Incomplete
    };
    let source_label = crate::envelope::format_package_source(true, None, completeness, &id_a);
    assert!(source_label.starts_with("workspace-envelope:"));
    if !is_complete {
        assert!(source_label.contains(":incomplete:"));
    }
}
