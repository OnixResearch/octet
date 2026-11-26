//! Unit tests for Cairn structural trace receipts and pinned Buoy comparison pilot intake.

use std::fs;
use std::path::Path;

use super::fixture_gen::build_cairn_v2_report;

fn write_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, content).expect("write test fixture file");
}

fn load_profiles() -> super::IntakeProfileRegistry {
    let bytes = include_bytes!("../../../standards/structural-linkage/generated/profiles.json");
    super::parse_profile_registry(bytes).expect("checked-in profiles must parse")
}

pub(super) fn build_buoy_pilot_report(semantic: &str, upstream_executed: bool) -> Vec<u8> {
    let fixture_hash = super::blake3_hex(b"pilot-fixture");
    let report_hash = super::blake3_hex(format!("buoy-{semantic}").as_bytes());
    let manifest_hash = super::blake3_hex(b"manifest");
    serde_json::to_vec_pretty(&serde_json::json!({
        "schema": "cairn.buoy-comparison.receipt.v1",
        "command": "traceability compare",
        "upstream": {
            "producer": "buoy",
            "repository": "https://github.com/Chelis-Lang/buoy",
            "revision": "01096fbdd0b9bf39b2fec468f90e7b1db958f615",
            "source_content_hash": "sha256-1+/x0hY3+YMYgLVXs0uoUmA1VPPlg5i/7qt19NrBruc=",
            "source_content_hash_algorithm": "sha256",
            "source_content_hash_mode": "recursive-nar",
            "consumer_package": "cairn",
            "rust_toolchain": "1.88.0",
            "digest_feature": "default",
            "commands": ["buoy check"],
            "command_behavior": ["declared_only"],
            "report_schemas": ["buoy.check.v1"],
            "evidence": ["pilot-fixture"],
            "executed": upstream_executed,
            "artifact_available": true,
            "disposition": "artifact_available_not_executed",
            "reason": "pinned comparison fixture"
        },
        "mapping": {
            "id": "octet-buoy-comparison-pilot-v1",
            "semantic_subset": "current-agreement-only",
            "projection_version": "1",
            "original_authority": "cairn",
            "preserves_original_identities": true,
            "external_projection_is_authority": false
        },
        "decision": {
            "disposition": "agreement",
            "runtime_adoption": false,
            "reason": "bounded comparison pilot"
        },
        "manifest_blake3": manifest_hash,
        "policy_blake3": super::blake3_hex(b"policy"),
        "comparison_identity": super::blake3_hex(b"cmp"),
        "execution": {
            "native_executed": true,
            "upstream_executed": upstream_executed,
            "case_count": 1,
            "executed_case_count": 1,
            "declared_only_case_count": 0
        },
        "cases": [{
            "id": "pilot-case-1",
            "fixture_blake3": fixture_hash,
            "execution_mode": "verify_base_index",
            "native": {"executed": true, "valid": true},
            "native_semantic": "current",
            "declared_upstream": {
                "disposition": "declared_current",
                "semantic": semantic,
                "report_schema": "buoy.check.v1",
                "report_blake3": report_hash,
                "reason": "current pilot case"
            },
            "comparison": "agreement",
            "outcome": "agreement",
            "reason": "matches Cairn current binding"
        }],
        "valid": true,
        "complete": true,
        "verdict": "pass",
        "boundary": "bounded pilot comparison only",
        "non_claims": ["not upstream execution", "not formal proof"],
        "issues": []
    }))
    .expect("serialize buoy pilot")
}

#[test]
fn cairn_v2_intake_preserves_distinct_schemas_identities_and_history_separation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = "// r[impl octet.structural_linkage.pilot.checked_add]\npub fn checked_add(a: u32, b: u32) -> Option<u32> { a.checked_add(b) }\n";
    write_file(dir.path(), "src/pilot.rs", src);
    let report = build_cairn_v2_report("src/pilot.rs", src, true);
    let profiles = load_profiles();
    let profile = super::select_profile(&profiles, "cairn-structural-trace-v2").expect("cairn profile");

    let (projection, receipt) =
        super::receipt::ingest_report(profile, &report, dir.path()).expect("ingest cairn report");
    assert_eq!(receipt.status, super::STATUS_VALID);
    assert!(!receipt.command_executed);
    assert!(receipt.replay_deterministic);
    assert_eq!(projection.schema_version, super::PROJECTION_SCHEMA_V1);
    assert_eq!(receipt.schema_version, super::RECEIPT_SCHEMA_V1);
    assert_eq!(receipt.original_report_schema, "cairn.content-bound-trace-index.v2");
    assert_ne!(receipt.original_report_blake3, projection.projection_blake3);
    assert_ne!(projection.projection_blake3, receipt.receipt_blake3);
    assert_eq!(projection.records.len(), 1);
    assert_eq!(projection.historical_records.len(), 1);
    assert!(projection.historical_records[0].historical_only);

    let proj_bytes = serde_json::to_vec(&projection).expect("projection bytes");
    let rcpt_bytes = serde_json::to_vec(&receipt).expect("receipt bytes");
    let verified = super::receipt::verify_conversion(&profiles, &report, &proj_bytes, &rcpt_bytes, dir.path())
        .expect("readonly verify");
    assert_eq!(verified.status, super::STATUS_VALID);
}

#[test]
fn formatting_only_vs_body_edit_distinguishes_normalized_item_hash_and_invalidates_exact_receipt() {
    let dir = tempfile::tempdir().expect("tempdir");
    let orig = "// r[impl octet.structural_linkage.pilot.checked_add]\npub fn checked_add(a: u32, b: u32) -> Option<u32> {\n    a.checked_add(b)\n}\n";
    write_file(dir.path(), "src/pilot.rs", orig);
    let report = build_cairn_v2_report("src/pilot.rs", orig, false);
    let profiles = load_profiles();
    let profile = super::select_profile(&profiles, "cairn-structural-trace-v2").expect("cairn profile");
    let (projection, receipt) = super::receipt::ingest_report(profile, &report, dir.path()).expect("ingest");
    let proj_bytes = serde_json::to_vec(&projection).expect("proj");
    let rcpt_bytes = serde_json::to_vec(&receipt).expect("rcpt");

    // Formatting-only edit preserves normalized_item_blake3 but invalidates exact source receipt.
    let reformatted = "// r[impl octet.structural_linkage.pilot.checked_add]\npub fn checked_add(a: u32, b: u32) -> Option<u32> { a.checked_add(b) }\n";
    write_file(dir.path(), "src/pilot.rs", reformatted);
    let fmt_check =
        super::receipt::verify_conversion(&profiles, &report, &proj_bytes, &rcpt_bytes, dir.path()).expect("verify");
    assert_eq!(fmt_check.status, super::STATUS_STALE_BINDING);
    assert_eq!(fmt_check.source_checks[0].status, super::SOURCE_STATUS_FORMATTING_ONLY);
    assert_eq!(
        fmt_check.source_checks[0].expected_normalized_item_blake3,
        fmt_check.source_checks[0].actual_normalized_item_blake3
    );
    assert_ne!(fmt_check.source_checks[0].expected_file_blake3, fmt_check.source_checks[0].actual_file_blake3);

    // Item body edit changes normalized_item_blake3 too.
    let body_changed = "// r[impl octet.structural_linkage.pilot.checked_add]\npub fn checked_add(a: u32, b: u32) -> Option<u32> { Some(a.saturating_add(b)) }\n";
    write_file(dir.path(), "src/pilot.rs", body_changed);
    let body_check =
        super::receipt::verify_conversion(&profiles, &report, &proj_bytes, &rcpt_bytes, dir.path()).expect("verify");
    assert_eq!(body_check.status, super::STATUS_STALE_BINDING);
    assert_eq!(body_check.source_checks[0].status, super::SOURCE_STATUS_BODY_CHANGED);
    assert_ne!(
        body_check.source_checks[0].expected_normalized_item_blake3,
        body_check.source_checks[0].actual_normalized_item_blake3
    );
}

#[test]
fn buoy_pilot_accepts_current_agreement_and_rejects_unsupported_or_executed_claims() {
    let dir = tempfile::tempdir().expect("tempdir");
    let profiles = load_profiles();
    let profile = super::select_profile(&profiles, "buoy-comparison-pilot-v1").expect("buoy profile");

    let valid_buoy = build_buoy_pilot_report("current", false);
    let (proj, rcpt) = super::receipt::ingest_report(profile, &valid_buoy, dir.path()).expect("buoy valid");
    assert_eq!(rcpt.status, super::STATUS_VALID);
    assert_eq!(proj.namespace, "buoy");
    assert!(!rcpt.command_executed);

    let unsupported_buoy = build_buoy_pilot_report("unsupported", false);
    let (_, bad_rcpt) = super::receipt::ingest_report(profile, &unsupported_buoy, dir.path()).expect("buoy unsup");
    assert_eq!(bad_rcpt.status, super::STATUS_UNSUPPORTED);

    let exec_buoy = build_buoy_pilot_report("current", true);
    let (_, exec_rcpt) = super::receipt::ingest_report(profile, &exec_buoy, dir.path()).expect("buoy exec");
    assert_eq!(exec_rcpt.status, super::STATUS_FAILED);
}

#[test]
fn unsupported_schema_and_tampered_index_or_parser_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = "// r[impl octet.structural_linkage.pilot.checked_add]\npub fn checked_add(a: u32, b: u32) -> Option<u32> { a.checked_add(b) }\n";
    write_file(dir.path(), "src/pilot.rs", src);
    let profiles = load_profiles();
    let profile = super::select_profile(&profiles, "cairn-structural-trace-v2").expect("cairn profile");

    let wrong_schema = br#"{"schema_version": "cairn.content-bound-trace-index.v99"}"#;
    let (_, rcpt) = super::receipt::ingest_report(profile, wrong_schema, dir.path()).expect("unsup schema");
    assert_eq!(rcpt.status, super::STATUS_UNSUPPORTED);

    let mut report_val: serde_json::Value =
        serde_json::from_slice(&build_cairn_v2_report("src/pilot.rs", src, false)).expect("parse");
    report_val["structural"]["parser"] = serde_json::Value::String("syn-1.0.0".to_owned());
    let tampered_bytes = serde_json::to_vec(&report_val).expect("serialize");
    let (_, tampered_rcpt) =
        super::receipt::ingest_report(profile, &tampered_bytes, dir.path()).expect("tampered parser");
    assert_eq!(tampered_rcpt.status, super::STATUS_FAILED);
}
