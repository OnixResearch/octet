use super::*;

// r[verify octet.external_lint.sarif_intake_verification]
const CODEQL_REPORT: &str = r#"{
  "version":"2.1.0",
  "runs":[{
    "tool":{"driver":{"name":"CodeQL","rules":[{"id":"rust/sql-injection"}]}},
    "results":[{
      "ruleId":"rust/sql-injection",
      "level":"warning",
      "message":{"text":"query includes untrusted input"},
      "locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/query.rs"},"region":{"startLine":12,"startColumn":5}}}]
    }]
  }]
}"#;

const SEMGREP_REPORT: &str = r#"{
  "version":"2.1.0",
  "runs":[{
    "tool":{"driver":{"name":"Semgrep","rules":[{"id":"rust.lang.security.audit"}]}},
    "artifacts":[{"location":{"uri":"src/lib.rs"}}],
    "results":[{
      "ruleId":"rust.lang.security.audit",
      "level":"error",
      "message":{"text":"unsafe operation"},
      "locations":[{"physicalLocation":{"artifactLocation":{"index":0},"region":{"startLine":7,"startColumn":3}}}]
    }]
  }]
}"#;

#[test]
fn checked_in_fixtures_cover_distinct_producers_and_unsafe_paths() {
    let codeql = include_str!("../../../tests/fixtures/evidence-rails/sarif-codeql.json");
    let semgrep = include_str!("../../../tests/fixtures/evidence-rails/sarif-semgrep.json");
    let unsafe_report = include_str!("../../../tests/fixtures/evidence-rails/sarif-unsafe.json");
    assert!(evaluate(codeql, "codeql-local").details.is_some());
    assert!(evaluate(semgrep, "semgrep-local").details.is_some());
    assert!(evaluate(unsafe_report, "codeql-local").details.is_none());
}

#[test]
fn distinct_profiles_preserve_source_identity_and_determinism() {
    let first = evaluate(CODEQL_REPORT, "codeql-local");
    let second = evaluate(CODEQL_REPORT, "codeql-local");
    assert_eq!(first, second);
    let codeql = first.details.expect("CodeQL details");
    assert!(first.diagnostics.is_empty());
    assert_eq!(codeql.source_driver, "CodeQL");
    assert_eq!(codeql.findings[0].source_level, "warning");
    assert_eq!(codeql.findings[0].gate_severity, "warning");
    assert_eq!(codeql.findings[0].file, "src/query.rs");

    let semgrep = evaluate(SEMGREP_REPORT, "semgrep-local");
    let details = semgrep.details.expect("Semgrep details");
    assert!(semgrep.diagnostics.is_empty());
    assert_eq!(details.source_driver, "Semgrep");
    assert_eq!(details.findings[0].source_rule_id, "rust.lang.security.audit");
    assert_eq!(details.findings[0].file, "src/lib.rs");
}

#[test]
fn malformed_profile_version_driver_rule_and_location_fail_closed() {
    assert!(evaluate("not-json", "codeql-local").details.is_none());
    assert!(evaluate(CODEQL_REPORT, "unknown").details.is_none());

    for invalid in [
        CODEQL_REPORT.replace("2.1.0", "2.0.0"),
        CODEQL_REPORT.replace("CodeQL", "Other"),
        CODEQL_REPORT.replace("rust/sql-injection\"}", "other\"}"),
        CODEQL_REPORT.replace("src/query.rs", "../escape.rs"),
        CODEQL_REPORT.replace("src/query.rs", "https://example.invalid/source.rs"),
        CODEQL_REPORT.replace("src/query.rs", "C:/escape.rs"),
        CODEQL_REPORT.replace("src/query.rs", "https:source.rs"),
        CODEQL_REPORT.replace("physicalLocation", "missingPhysicalLocation"),
        CODEQL_REPORT.replace("artifactLocation", "missingArtifactLocation"),
        CODEQL_REPORT.replace("region", "missingRegion"),
    ] {
        let result = evaluate(&invalid, "codeql-local");
        assert!(result.details.is_none(), "invalid report passed: {invalid}");
        assert!(!result.diagnostics.is_empty());
    }
}

#[test]
fn duplicate_overlimit_and_unsupported_severity_fail_closed() {
    let duplicate = CODEQL_REPORT
        .replace("{\"id\":\"rust/sql-injection\"}", "{\"id\":\"rust/sql-injection\"},{\"id\":\"rust/sql-injection\"}");
    assert!(evaluate(&duplicate, "codeql-local").details.is_none());
    assert!(evaluate(&CODEQL_REPORT.replace("warning", "critical"), "codeql-local").details.is_none());

    let profile = selected_profile("codeql-local").expect("CodeQL profile");
    let mut artifact_heavy: serde_json::Value = serde_json::from_str(CODEQL_REPORT).expect("parse report fixture");
    artifact_heavy["runs"][0]["artifacts"] =
        serde_json::Value::Array(vec![serde_json::json!({"location":{"uri":"src/lib.rs"}}); profile.max_artifacts + 1]);
    let artifact_heavy = serde_json::to_string(&artifact_heavy).expect("serialize artifact-limit fixture");
    assert!(evaluate(&artifact_heavy, "codeql-local").details.is_none());

    let mut oversized = CODEQL_REPORT.to_owned();
    oversized.push_str(&" ".repeat(1_048_577));
    assert!(evaluate(&oversized, "codeql-local").details.is_none());
}

#[test]
fn missing_result_level_uses_rule_default_configuration() {
    let error_default = CODEQL_REPORT
        .replace(
            "{\"id\":\"rust/sql-injection\"}",
            "{\"id\":\"rust/sql-injection\",\"defaultConfiguration\":{\"level\":\"error\"}}",
        )
        .replace("\"level\":\"warning\",", "");
    let evaluation = evaluate(&error_default, "codeql-local");
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("rule-default details");
    assert_eq!(details.findings[0].source_level, "error");
    assert_eq!(details.findings[0].gate_severity, "error");

    let explicit_wins = error_default.replace("\"ruleId\"", "\"level\":\"note\",\"ruleId\"");
    let details = evaluate(&explicit_wins, "codeql-local").details.expect("explicit-level details");
    assert_eq!(details.findings[0].source_level, "note");

    let no_default = CODEQL_REPORT.replace("\"level\":\"warning\",", "");
    let details = evaluate(&no_default, "codeql-local").details.expect("fallback details");
    assert_eq!(details.findings[0].source_level, "warning");

    let unsupported_default = error_default.replace("\"level\":\"error\"", "\"level\":\"critical\"");
    assert!(evaluate(&unsupported_default, "codeql-local").details.is_none());
}

#[test]
fn non_fail_result_kinds_do_not_gate_and_undecided_kinds_fail_closed() {
    let with_kind = |kind: &str| {
        CODEQL_REPORT.replace("\"level\":\"warning\",", &format!("\"kind\":\"{kind}\",")).replace(
            "{\"id\":\"rust/sql-injection\"}",
            "{\"id\":\"rust/sql-injection\",\"defaultConfiguration\":{\"level\":\"error\"}}",
        )
    };
    for kind in ["pass", "notApplicable", "informational"] {
        let evaluation = evaluate(&with_kind(kind), "codeql-local");
        assert!(evaluation.diagnostics.is_empty(), "{kind}: {:?}", evaluation.diagnostics);
        assert!(evaluation.details.expect("non-gating details").findings.is_empty(), "{kind} result was imported");
    }
    let explicit_none = with_kind("pass").replace("\"kind\"", "\"level\":\"none\",\"kind\"");
    assert!(evaluate(&explicit_none, "codeql-local").details.expect("pass details").findings.is_empty());

    let fail = evaluate(&with_kind("fail"), "codeql-local").details.expect("fail details");
    assert_eq!(fail.findings[0].gate_severity, "error");

    for invalid in [
        with_kind("review"),
        with_kind("open"),
        with_kind("unexpected"),
        with_kind("pass").replace("\"kind\"", "\"level\":\"error\",\"kind\""),
    ] {
        let evaluation = evaluate(&invalid, "codeql-local");
        assert!(evaluation.details.is_none(), "invalid report passed: {invalid}");
        assert!(!evaluation.diagnostics.is_empty());
    }
}

#[test]
fn profile_registry_and_projection_identity_are_fresh() {
    assert_eq!(prefixed_identity(PROFILE_JSON.as_bytes()), PROFILE_IDENTITY);
    for id in ["codeql-local", "semgrep-local"] {
        let profile = selected_profile(id).expect("known profile");
        assert_eq!(profile.supported_version, SUPPORTED_VERSION);
        assert!(profile.require_rule_id);
        assert!(profile.workspace_relative_only);
        assert_eq!(profile.evidence_tier, "static_lint");
    }
    let details = evaluate(CODEQL_REPORT, "codeql-local").details.expect("details");
    assert_eq!(prefixed_identity(&projection_bytes(&details).expect("projection")), details.projection_identity);
}
