const DIRECT_CALL_PROFILE: &str =
    include_str!("../../../standards/charon-semantic/generated/consumer-direct-call.json");
const TRAIT_HELPER_PROFILE: &str =
    include_str!("../../../standards/charon-semantic/generated/consumer-trait-helper.json");
const DIRECT_CALL_PARENT: &str =
    include_str!("../../../standards/charon-semantic/generated/consumer-direct-call-parent.json");
const CANARY_SNAPSHOT: &str =
    include_str!("../../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/llbc.llbc");

fn direct_call_snapshot_json() -> String {
    let lib_src = include_str!("../../../standards/charon-semantic/fixtures/consumer/direct-call/src/lib.rs");
    let helper_src = include_str!("../../../standards/charon-semantic/fixtures/consumer/direct-call/src/helper.rs");
    serde_json::json!({
        "charon_version": "0.1.0-test",
        "has_errors": false,
        "translated": {
            "crate_name": "direct_call_consumer",
            "options": { "preset": "Aeneas", "skip_borrowck": false, "start_from_pub": false },
            "target_information": [{
                "key": "x86_64-unknown-linux-gnu",
                "value": { "target_pointer_size": 8, "is_little_endian": true }
            }],
            "files": [
                { "id": 0, "name": { "Local": "src/lib.rs" }, "contents": lib_src },
                { "id": 1, "name": { "Local": "src/helper.rs" }, "contents": helper_src }
            ],
            "fun_decls": [
                {
                    "def_id": 0,
                    "item_meta": {
                        "is_local": true,
                        "name": [{ "Ident": ["direct_call_consumer", 0] }, { "Ident": ["combine_steps", 0] }],
                        "span": { "data": { "file_id": 0, "beg": { "line": 3 }, "end": { "line": 6 } } },
                        "opacity": "Transparent"
                    },
                    "signature": { "is_unsafe": false },
                    "body": {
                        "Structured": {
                            "body": {
                                "statements": [{
                                    "id": 0,
                                    "kind": { "Call": { "call": { "func": { "Regular": { "kind": { "Fun": { "Regular": 1 } } } } } } }
                                }]
                            }
                        }
                    }
                },
                {
                    "def_id": 1,
                    "item_meta": {
                        "is_local": true,
                        "name": [
                            { "Ident": ["direct_call_consumer", 0] },
                            { "Ident": ["helper", 0] },
                            { "Ident": ["bounded_step", 0] }
                        ],
                        "span": { "data": { "file_id": 1, "beg": { "line": 1 }, "end": { "line": 7 } } },
                        "opacity": "Transparent"
                    },
                    "signature": { "is_unsafe": false },
                    "body": {
                        "Structured": {
                            "body": {
                                "statements": [{ "id": 0, "kind": { "Assert": { "cond": true } } }]
                            }
                        }
                    }
                }
            ],
            "trait_decls": [],
            "trait_impls": []
        }
    })
    .to_string()
}

#[test]
fn consumer_profiles_validate_and_derive_argv() {
    for (text, expected_id, expected_manifest) in [
        (DIRECT_CALL_PROFILE, "octet-charon-semantic-consumer-direct-call-v1", "Cargo.toml"),
        (TRAIT_HELPER_PROFILE, "octet-charon-semantic-consumer-trait-helper-v1", "Cargo.toml"),
    ] {
        let (identity, argv) = super::validate_profile_text(expected_id, text).expect("consumer profile must validate");
        assert!(super::is_b3_identity(&identity));
        assert!(argv.windows(2).any(|w| w == ["--manifest-path", expected_manifest]));
        assert!(argv.contains(&"--locked".to_owned()));
        assert!(argv.contains(&"--offline".to_owned()));
    }
}

#[test]
fn consumer_profile_rejects_injection_broad_selection_and_provider_drift() {
    let mut profile: super::consumer::WorkspaceProfile =
        serde_json::from_str(DIRECT_CALL_PROFILE).expect("direct-call profile parses");
    profile.source.broad_selection_allowed = true;
    profile.source.root = "../escape".to_owned();
    profile.extraction.features = vec!["--bad;rm".to_owned()];
    profile.extraction.cfg = vec!["foo;bar".to_owned()];
    profile.provider.charon_revision = "0000000000000000000000000000000000000000".to_owned();
    let text = serde_json::to_string(&profile).expect("mutated profile serializes");
    let diagnostics = super::consumer::parse_and_validate(&text).err().expect("invalid profile must fail");
    assert!(diagnostics.iter().any(|d| d.contains("broad")));
    assert!(diagnostics.iter().any(|d| d.contains("unsafe")));
    assert!(diagnostics.iter().any(|d| d.contains("charon_revision")));
}

#[test]
fn consumer_snapshot_separates_roots_from_local_dependencies_and_rejects_unexplained_scope() {
    let snapshot = direct_call_snapshot_json();
    let envelope = super::build_envelope_for_profile(
        snapshot.clone(),
        vec![DIRECT_CALL_PARENT.to_owned()],
        Some(DIRECT_CALL_PROFILE.to_owned()),
    )
    .expect("v2 envelope builds");
    let evaluation = super::evaluate_envelope(&envelope);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("consumer details exist");
    assert_eq!(details.verification_role, super::RECORDED_ONLY_ROLE);
    assert!(details.artifacts.iter().any(|a| a.name == super::PROFILE_ARTIFACT));
    let root = details
        .projection
        .items
        .iter()
        .find(|item| item.qualified_name == "direct_call_consumer::combine_steps")
        .expect("root item");
    let helper = details
        .projection
        .items
        .iter()
        .find(|item| item.qualified_name == "direct_call_consumer::helper::bounded_step")
        .expect("helper item");
    assert_eq!(root.selection_role.as_deref(), Some("requested_root"));
    assert_eq!(helper.selection_role.as_deref(), Some("local_dependency"));
    let comparison = details.comparison.as_ref().expect("comparison exists");
    assert!(!comparison.authority_merged);
    assert_eq!(comparison.summary.disagree, 0);
    assert!(comparison.summary.agree >= 5);

    let mut unexplained: serde_json::Value = serde_json::from_str(&snapshot).expect("snapshot json");
    unexplained["translated"]["fun_decls"]
        .as_array_mut()
        .expect("fun_decls array")
        .push(serde_json::json!({
            "def_id": 2,
            "item_meta": {
                "is_local": true,
                "name": [{ "Ident": ["direct_call_consumer", 0] }, { "Ident": ["stray", 0] }],
                "span": { "data": { "file_id": 0, "beg": { "line": 1 }, "end": { "line": 2 } } },
                "opacity": "Transparent"
            },
            "signature": { "is_unsafe": false },
            "body": null
        }));
    let bad_envelope =
        super::build_envelope_for_profile(unexplained.to_string(), Vec::new(), Some(DIRECT_CALL_PROFILE.to_owned()))
            .expect("envelope builds");
    let bad_eval = super::evaluate_envelope(&bad_envelope);
    assert!(bad_eval.details.is_none());
    assert!(
        bad_eval.diagnostics.iter().any(|d| d.contains("unexplained local scope expansion")),
        "{:?}",
        bad_eval.diagnostics
    );
}

#[test]
fn consumer_comparison_detects_disagreement_and_foreign_source() {
    let mut parent: super::ComparisonParent =
        serde_json::from_str(DIRECT_CALL_PARENT).expect("comparison parent parses");
    parent.facts[0].value = "absent".to_owned();
    let disagree_env = super::build_envelope_for_profile(
        direct_call_snapshot_json(),
        vec![serde_json::to_string(&parent).expect("parent serializes")],
        Some(DIRECT_CALL_PROFILE.to_owned()),
    )
    .expect("envelope builds");
    let disagree_eval = super::evaluate_envelope(&disagree_env);
    let comparison = disagree_eval.details.expect("details exist").comparison.expect("comparison exists");
    assert_eq!(comparison.summary.disagree, 1);

    parent.source_identity = "b3:0000000000000000000000000000000000000000000000000000000000000000".to_owned();
    let foreign_env = super::build_envelope_for_profile(
        direct_call_snapshot_json(),
        vec![serde_json::to_string(&parent).expect("parent serializes")],
        Some(DIRECT_CALL_PROFILE.to_owned()),
    )
    .expect("envelope builds");
    let foreign_eval = super::evaluate_envelope(&foreign_env);
    let foreign_cmp = foreign_eval.details.expect("details exist").comparison.expect("comparison exists");
    assert_eq!(foreign_cmp.summary.unknown, foreign_cmp.rows.len() as u64);
}

#[test]
fn consumer_bundle_detects_tampered_profile_and_missing_root() {
    let envelope = super::build_envelope_for_profile(
        direct_call_snapshot_json(),
        Vec::new(),
        Some(DIRECT_CALL_PROFILE.to_owned()),
    )
    .expect("envelope builds");
    let details = super::evaluate_envelope(&envelope).details.expect("details exist");
    let payloads = super::artifact_payloads(&details, &envelope).expect("payloads render");
    assert!(payloads.contains_key(super::PROFILE_ARTIFACT));
    let mut tampered = details.clone();
    tampered.profile_identity = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();
    let diagnostics = super::validate_details(&tampered, &[
        "does not prove Charon or rustc correctness".into(),
        "does not prove effect absence".into(),
        "does not produce formal-proof".into(),
    ]);
    assert!(diagnostics.iter().any(|d| d.contains("role bindings")));

    let missing_root_env =
        super::build_envelope_for_profile(CANARY_SNAPSHOT.to_owned(), Vec::new(), Some(DIRECT_CALL_PROFILE.to_owned()))
            .expect("envelope builds");
    let missing_root_eval = super::evaluate_envelope(&missing_root_env);
    assert!(missing_root_eval.details.is_none());
    assert!(missing_root_eval.diagnostics.iter().any(|d| d.contains("requested root")));
}
