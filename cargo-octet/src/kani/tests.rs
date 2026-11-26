use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;

use super::ASSUMPTION_REVIEW_SCHEMA;
use super::AssumptionReviewRecord;
use super::EXPECTED_PROFILE_IDENTITY;
use super::EXPORT_SCHEMA;
use super::OBSERVATION_SCHEMA;
use super::Observation as KaniProducerObservation;
use super::PRODUCER_PROFILE_ID;
use super::ToolchainObservation as KaniToolchainObservation;
use super::adapter::check_observation as evaluate_observation;
use super::probe::ToolRuntime;
use super::probe::compute_source_manifest_identity;
use super::selected_profile;
use super::shell::run_with;
use super::validate;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

fn kani_pilot_dir() -> PathBuf {
    repo_root().join("cargo-octet/tests/fixtures/kani-producer")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("octet-kani-prod-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_executable(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).expect("write script");
    let mut perms = fs::metadata(&path).expect("stat script").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod script");
    path
}

fn export_json_payload(mode: &str) -> String {
    match mode {
        "pass" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "SUCCESS",
    "unwind_bound": 8,
    "properties": [
      { "name": "checked_bounds_no_panic", "check_type": "assert", "status": "SUCCESS", "location": { "file": "src/lib.rs", "function": "check_checked_window_bounds", "line": 44, "column": 13 } },
      { "name": "cover_valid_window_reachable", "check_type": "cover", "status": "SATISFIED", "location": { "file": "src/lib.rs", "function": "check_checked_window_bounds", "line": 42, "column": 9 } },
      { "name": "unwind_assertion_loop", "check_type": "unwind", "status": "SUCCESS", "location": { "file": "src/lib.rs", "function": "bounded_window_checksum", "line": 22, "column": 5 } }
    ]
  }]
}"#.to_owned(),
        "fail_assert" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "FAILURE",
    "unwind_bound": 8,
    "properties": [
      { "name": "checked_bounds_no_panic", "check_type": "assert", "status": "FAILURE", "description": "assertion failed: result.is_some()" },
      { "name": "cover_valid_window_reachable", "check_type": "cover", "status": "SATISFIED" },
      { "name": "unwind_assertion_loop", "check_type": "unwind", "status": "SUCCESS" }
    ]
  }]
}"#.to_owned(),
        "contradictory_cover" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "SUCCESS",
    "unwind_bound": 8,
    "properties": [
      { "name": "checked_bounds_no_panic", "check_type": "assert", "status": "SUCCESS" },
      { "name": "cover_valid_window_reachable", "check_type": "cover", "status": "UNREACHABLE" },
      { "name": "unwind_assertion_loop", "check_type": "unwind", "status": "SUCCESS" }
    ]
  }]
}"#.to_owned(),
        "empty_props" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "SUCCESS",
    "unwind_bound": 8,
    "properties": []
  }]
}"#.to_owned(),
        "unsupported" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "UNSUPPORTED",
    "unwind_bound": 8,
    "properties": [{ "name": "inline_asm", "check_type": "assert", "status": "UNDETERMINED" }]
  }]
}"#.to_owned(),
        "tool_error" => r#"{
  "schema_version": "kani-export/v1",
  "kani_version": "0.68.0",
  "harnesses": [{
    "package": "octet-kani-boundary-pilot",
    "target": "lib",
    "name": "check_checked_window_bounds",
    "verification_status": "ERROR",
    "unwind_bound": 8,
    "properties": [{ "name": "cbmc_crash", "check_type": "assert", "status": "UNDETERMINED" }]
  }]
}"#.to_owned(),
        _ => String::new(),
    }
}

fn fake_kani_runtime(bin_dir: &Path, mode: &str, kani_ver: &str, rustc_rel: &str) -> ToolRuntime {
    let payload = export_json_payload(mode);
    let cargo = write_executable(
        bin_dir,
        "cargo",
        &format!(
            "#!/bin/sh
set -eu
if [ \"${{1:-}}\" = \"kani\" ] && [ \"${{2:-}}\" = \"--version\" ]; then
  echo 'cargo-kani {kani_ver}'
  exit 0
fi
if [ \"{mode}\" = \"sleep\" ]; then
  sleep 4
  exit 0
fi
if [ \"{mode}\" = \"truncate\" ]; then
  head -c 600000 /dev/zero | tr '\\0' 'k'
  exit 0
fi
export_path=''
for arg in \"$@\"; do
  export_path=\"$arg\"
done
cat > \"$export_path\" <<'KANI_JSON'
{payload}
KANI_JSON
if [ \"{mode}\" = \"fail_assert\" ] || [ \"{mode}\" = \"tool_error\" ]; then
  exit 1
fi
exit 0
"
        ),
    );
    let kani_rustc = write_executable(
        bin_dir,
        "rustc",
        &format!(
            "#!/bin/sh
echo 'rustc {rustc_rel} (fake-kani)'
echo 'release: {rustc_rel}'
echo 'commit-hash: 1111111111111111111111111111111111111111'
"
        ),
    );
    let cbmc = write_executable(
        bin_dir,
        "cbmc",
        "#!/bin/sh
echo '6.5.0 (cbmc-6.5.0)'
",
    );
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from("/bin:/usr/bin"));
    ToolRuntime {
        cargo,
        kani_rustc,
        cbmc,
        passthrough_env: vec![(OsString::from("PATH"), path)],
    }
}

#[test]
fn kani_producer_run_binds_assumption_review_cover_witness_and_negative_controls() {
    let profile = selected_profile(PRODUCER_PROFILE_ID).expect("kani profile");
    let pilot_dir = kani_pilot_dir();
    let (src_id, _) = compute_source_manifest_identity(&pilot_dir, &profile.limits).expect("source id");

    let pass_dir = temp_dir("pass");
    let review_path = pass_dir.join("assumption-review.json");
    let review = AssumptionReviewRecord {
        schema_version: ASSUMPTION_REVIEW_SCHEMA.to_owned(),
        source_manifest_identity: src_id.clone(),
        harness_names: vec!["check_checked_window_bounds".to_owned()],
        is_assumptions_reviewed: true,
        assumption_status: "reviewed".to_owned(),
        reviewer: "assurance-reviewer".to_owned(),
        notes: "Bounds and bounded_window_checksum assumptions reviewed".to_owned(),
    };
    fs::write(&review_path, serde_json::to_vec(&review).expect("json")).expect("write review");

    let export_dir = pass_dir.join("exports");
    let runtime = fake_kani_runtime(&pass_dir, "pass", "0.68.0", "1.87.0-nightly");
    let (obs_json, exit_code) =
        run_with(&pilot_dir, &profile, &runtime, Some(&review_path), Some(&export_dir)).expect("run");
    assert_eq!(exit_code, 0);
    let eval = evaluate_observation(&obs_json);
    assert_eq!(eval.status, "passed", "{:?}", eval.diagnostics);
    let details = eval.details.expect("details");
    assert_eq!(details.execution_status, "passed");
    assert_eq!(details.property_totals.cover_satisfied, 1);
    assert_eq!(details.evidence_tier, "transcript_assertion");
    assert!(validate(&details).is_empty());
    assert!(eval.normalized_summary_json.as_deref().is_some_and(|s| s.contains("cargo-kani-summary/v1")));

    // Stale export directory must be rejected before spawning Kani.
    let stale_err =
        run_with(&pilot_dir, &profile, &runtime, Some(&review_path), Some(&export_dir)).expect_err("reject stale");
    assert!(stale_err.contains("stale JSON artifact"));

    // Omitting assumption review yields manual_review_required, never passed.
    let (no_rev_json, _) = run_with(&pilot_dir, &profile, &runtime, None, None).expect("run without review");
    let no_rev_eval = evaluate_observation(&no_rev_json);
    assert_eq!(no_rev_eval.status, "manual_review_required");

    // Negative controls: failed assertion, contradictory cover, empty props, unsupported, tool_error,
    // truncated.
    for (mode, expected_exec) in [
        ("fail_assert", "failed"),
        ("contradictory_cover", "failed"),
        ("empty_props", "incomplete_analysis"),
        ("unsupported", "unsupported"),
        ("tool_error", "tool_error"),
        ("truncate", "incomplete_analysis"),
    ] {
        let dir = temp_dir(mode);
        let rt = fake_kani_runtime(&dir, mode, "0.68.0", "1.87.0-nightly");
        let (out, _) = run_with(&pilot_dir, &profile, &rt, Some(&review_path), None).expect("run mode");
        let res = evaluate_observation(&out);
        assert_eq!(res.status, "failed", "mode {mode}");
        assert_eq!(res.details.expect("details").execution_status, expected_exec, "mode {mode}");
    }

    // Wrong Kani / rustc version fails probe.
    let bad_ver_dir = temp_dir("bad-ver");
    let bad_rt = fake_kani_runtime(&bad_ver_dir, "pass", "0.65.0", "1.87.0-nightly");
    assert!(run_with(&pilot_dir, &profile, &bad_rt, None, None).is_err());

    let bad_rustc_dir = temp_dir("bad-rustc");
    let bad_rustc = fake_kani_runtime(&bad_rustc_dir, "pass", "0.68.0", "1.80.0-nightly");
    assert!(run_with(&pilot_dir, &profile, &bad_rustc, None, None).is_err());

    // Timeout case.
    let mut short_profile = profile;
    short_profile.limits.timeout_seconds = 1;
    let sleep_dir = temp_dir("sleep");
    let (sleep_out, _) = run_with(
        &pilot_dir,
        &short_profile,
        &fake_kani_runtime(&sleep_dir, "sleep", "0.68.0", "1.87.0-nightly"),
        None,
        None,
    )
    .expect("timeout run");
    let sleep_eval = evaluate_observation(&sleep_out);
    assert_eq!(sleep_eval.details.expect("details").execution_status, "timeout");
}

#[test]
fn kani_adapter_reconciles_harnesses_order_independently_and_rejects_vacuous_or_stale_review() {
    let mut obs = KaniProducerObservation {
        schema_version: OBSERVATION_SCHEMA.to_owned(),
        profile_id: PRODUCER_PROFILE_ID.to_owned(),
        profile_identity: EXPECTED_PROFILE_IDENTITY.to_owned(),
        source_manifest_identity: super::prefixed_identity(b"src"),
        expected_source_manifest_identity: Some(super::prefixed_identity(b"src")),
        lockfile_identity: None,
        expected_lockfile_identity: None,
        command_argv: vec!["cargo".to_owned(), "kani".to_owned()],
        toolchain: KaniToolchainObservation {
            kani_version: "cargo-kani 0.68.0".to_owned(),
            provider_revision: "kani-0.68.0-release".to_owned(),
            closure_identity: "nix:kani-0.68.0-x86_64-unknown-linux-gnu".to_owned(),
            rustc_release: "1.87.0-nightly".to_owned(),
            rustc_commit_hash: "1111111111111111111111111111111111111111".to_owned(),
            cbmc_version: "6.5.0 (cbmc-6.5.0)".to_owned(),
            solver_mode: "cbmc_builtin".to_owned(),
            solver_identity: "cbmc-builtin-cadical".to_owned(),
        },
        exit_code: 0,
        timed_out: Some(false),
        truncated: Some(false),
        bounded_log_identity: super::prefixed_identity(b"log"),
        raw_export_json: serde_json::json!({
            "schema_version": EXPORT_SCHEMA,
            "kani_version": "0.68.0",
            "harnesses": [{
                "package": "octet-kani-boundary-pilot",
                "target": "lib",
                "name": "check_checked_window_bounds",
                "verification_status": "SUCCESS",
                "unwind_bound": 8,
                "properties": [
                    { "name": "a1", "check_type": "assert", "status": "SUCCESS" },
                    { "name": "cover_valid_window_reachable", "check_type": "cover", "status": "SATISFIED" },
                    { "name": "u1", "check_type": "unwind", "status": "SUCCESS" }
                ]
            }]
        })
        .to_string(),
        assumption_review: Some(AssumptionReviewRecord {
            schema_version: ASSUMPTION_REVIEW_SCHEMA.to_owned(),
            source_manifest_identity: super::prefixed_identity(b"stale"),
            harness_names: vec!["check_checked_window_bounds".to_owned()],
            is_assumptions_reviewed: true,
            assumption_status: "vacuous".to_owned(),
            reviewer: "rev".to_owned(),
            notes: "vacuous test".to_owned(),
        }),
    };
    let eval = evaluate_observation(&serde_json::to_string(&obs).expect("json"));
    assert_eq!(eval.status, "failed");
    assert!(eval.diagnostics.iter().any(|d| d.contains("vacuous")));
    assert!(eval.diagnostics.iter().any(|d| d.contains("source_manifest_identity")));

    // Reordering properties gives identical totals.
    obs.assumption_review = None;
    let first_totals = evaluate_observation(&serde_json::to_string(&obs).expect("json"))
        .details
        .expect("details")
        .property_totals;
    obs.raw_export_json = serde_json::json!({
        "schema_version": EXPORT_SCHEMA,
        "kani_version": "0.68.0",
        "harnesses": [{
            "package": "octet-kani-boundary-pilot",
            "target": "lib",
            "name": "check_checked_window_bounds",
            "verification_status": "SUCCESS",
            "unwind_bound": 8,
            "properties": [
                { "name": "u1", "check_type": "unwind", "status": "SUCCESS" },
                { "name": "cover_valid_window_reachable", "check_type": "cover", "status": "SATISFIED" },
                { "name": "a1", "check_type": "assert", "status": "SUCCESS" }
            ]
        }]
    })
    .to_string();
    let reordered_totals = evaluate_observation(&serde_json::to_string(&obs).expect("json"))
        .details
        .expect("details")
        .property_totals;
    assert_eq!(first_totals, reordered_totals);
}
