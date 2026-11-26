use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;

use super::CellObservation;
use super::CompilerProbe;
use super::EXPECTED_PROFILE_IDENTITY;
use super::Input as DynamicQualificationInput;
use super::LOOM_QUALIFIED_PROFILE_ID;
use super::MATRIX_RECEIPT_SCHEMA;
use super::MIRI_QUALIFIED_PROFILE_ID;
use super::evaluate::run_input as evaluate_input;
use super::probe::ToolRuntime;
use super::selected_profile;
use super::shell::run_with;
use super::validate;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

fn pilot_fixture_dir() -> PathBuf {
    repo_root().join("cargo-octet/tests/fixtures/dynamic-qualification")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("octet-dynamic-qual-{name}-{}", std::process::id()));
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

fn fake_miri_runtime(bin_dir: &Path, outcome: &str) -> ToolRuntime {
    let rust_src = bin_dir.join("rust-src");
    fs::create_dir_all(&rust_src).expect("create fake rust-src");
    let cargo = write_executable(
        bin_dir,
        "cargo",
        &format!(
            "#!/bin/sh
set -eu
if [ \"${{1:-}}\" = \"--version\" ]; then
  echo 'cargo 1.98.0-nightly (test)'
  exit 0
fi
if [ \"${{1:-}}\" = \"miri\" ] && [ \"${{2:-}}\" = \"-V\" ]; then
  echo 'miri 0.1.0 (14210df0e2 2026-05-31)'
  exit 0
fi
if [ -n \"${{RUSTC_WRAPPER:-}}\" ] || [ -n \"${{RUSTUP_TOOLCHAIN:-}}\" ]; then
  echo 'unexpected wrapper/toolchain leakage' >&2
  exit 9
fi
if [ \"{outcome}\" = \"pass\" ]; then
  echo 'running 2 tests'
  echo 'test tests::checked_slice_window_sound ... ok'
  echo 'test tests::raw_pointer_read_aligned ... ok'
  echo 'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
  exit 0
elif [ \"{outcome}\" = \"ub\" ]; then
  echo 'running 1 test'
  echo 'test tests::checked_slice_window_sound ... FAILED'
  echo 'error: Undefined Behavior: pointer arithmetic out of bounds' >&2
  exit 101
elif [ \"{outcome}\" = \"zero\" ]; then
  echo 'running 0 tests'
  echo 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
  exit 0
elif [ \"{outcome}\" = \"ignored\" ]; then
  echo 'running 2 tests'
  echo 'test tests::checked_slice_window_sound ... ignored'
  echo 'test tests::raw_pointer_read_aligned ... ignored'
  echo 'test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out'
  exit 0
elif [ \"{outcome}\" = \"truncated\" ]; then
  head -c 300000 /dev/zero | tr '\\0' 'x'
  exit 0
elif [ \"{outcome}\" = \"sleep\" ]; then
  sleep 4
  exit 0
fi
exit 1
"
        ),
    );
    let rustc = write_executable(
        bin_dir,
        "rustc",
        "#!/bin/sh
echo 'rustc 1.98.0-nightly (31a9463c6 2026-05-25)'
echo 'release: 1.98.0-nightly'
echo 'commit-hash: 31a9463c6e2794a59ce57a8f37abc6966afc2a58'
echo 'host: x86_64-unknown-linux-gnu'
",
    );
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from("/bin:/usr/bin"));
    ToolRuntime {
        cargo,
        rustc,
        xargo_rust_src: Some(rust_src),
        passthrough_env: vec![(OsString::from("PATH"), path)],
        caller_overrides: Vec::new(),
    }
}

fn fake_loom_runtime(bin_dir: &Path, outcome: &str) -> ToolRuntime {
    let cargo = write_executable(
        bin_dir,
        "cargo",
        &format!(
            "#!/bin/sh
set -eu
if [ \"${{1:-}}\" = \"--version\" ]; then
  echo 'cargo 1.96.0-nightly (test)'
  exit 0
fi
if [ \"${{RUSTFLAGS:-}}\" != '--cfg loom' ]; then
  echo \"missing RUSTFLAGS cfg loom: ${{RUSTFLAGS:-unset}}\" >&2
  exit 12
fi
expected_encoded=$(printf '%b' '--cfg\\037loom')
if [ \"${{CARGO_ENCODED_RUSTFLAGS:-}}\" != \"$expected_encoded\" ]; then
  echo 'missing CARGO_ENCODED_RUSTFLAGS cfg loom' >&2
  exit 13
fi
if [ \"{outcome}\" = \"pass\" ]; then
  echo 'running 1 test'
  echo 'loom::model enter bounded_handoff_preserves_order'
  echo 'loom_model_completed permutations=7'
  echo 'test loom_models::bounded_handoff_preserves_order ... ok'
  echo 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
  exit 0
elif [ \"{outcome}\" = \"no_model\" ]; then
  echo 'running 1 test'
  echo 'test loom_models::bounded_handoff_preserves_order ... ok'
  echo 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
  exit 0
elif [ \"{outcome}\" = \"fail\" ]; then
  echo 'running 1 test'
  echo 'loom::model enter bounded_handoff_preserves_order'
  echo 'thread panicked at assertion failed: observed == 1' >&2
  echo 'test loom_models::bounded_handoff_preserves_order ... FAILED'
  exit 101
fi
exit 1
"
        ),
    );
    let rustc = write_executable(
        bin_dir,
        "rustc",
        "#!/bin/sh
echo 'release: 1.96.0-nightly'
echo 'commit-hash: ac7f9ec7da74d37fd28667c86bf117a39ba5b02a'
echo 'host: x86_64-unknown-linux-gnu'
",
    );
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from("/bin:/usr/bin"));
    ToolRuntime {
        cargo,
        rustc,
        xargo_rust_src: None,
        passthrough_env: vec![(OsString::from("PATH"), path)],
        caller_overrides: Vec::new(),
    }
}

#[test]
fn miri_qualified_run_and_negative_controls() {
    let pass_dir = temp_dir("miri-pass");
    let profile = selected_profile(MIRI_QUALIFIED_PROFILE_ID).expect("miri profile");
    let (rendered, exit_code) =
        run_with(&pilot_fixture_dir(), &profile, &fake_miri_runtime(&pass_dir, "pass")).expect("run pass");
    assert_eq!(exit_code, 0);
    let eval = evaluate_input(&rendered, Some(MIRI_QUALIFIED_PROFILE_ID));
    assert_eq!(eval.status, "qualified_passed", "{:?}", eval.diagnostics);
    assert!(eval.diagnostics.is_empty(), "{:?}", eval.diagnostics);
    let details = eval.details.expect("details");
    assert_eq!(details.tests_started, 2);
    assert_eq!(details.tests_passed, 2);
    assert!(validate(&details).is_empty());

    for (mode, expected_status, snippet) in [
        ("ub", "failed", "Undefined Behavior"),
        ("zero", "failed", "zero tests"),
        ("ignored", "failed", "ignored"),
        ("truncated", "incomplete", "truncated"),
    ] {
        let dir = temp_dir(&format!("miri-{mode}"));
        let (out, _) = run_with(&pilot_fixture_dir(), &profile, &fake_miri_runtime(&dir, mode)).expect("run neg");
        let neg = evaluate_input(&out, Some(MIRI_QUALIFIED_PROFILE_ID));
        assert_eq!(neg.status, expected_status, "mode {mode}: {:?}", neg.diagnostics);
        assert!(
            neg.diagnostics.iter().chain(&neg.findings).any(|msg| msg.contains(snippet)),
            "mode {mode} missed `{snippet}`: diag={:?} find={:?}",
            neg.diagnostics,
            neg.findings
        );
    }

    let mut short_limit_profile = profile.clone();
    short_limit_profile.limits.timeout_seconds = 1;
    let sleep_dir = temp_dir("miri-timeout");
    let (sleep_out, _) =
        run_with(&pilot_fixture_dir(), &short_limit_profile, &fake_miri_runtime(&sleep_dir, "sleep")).expect("timeout");
    let timed = evaluate_input(&sleep_out, Some(MIRI_QUALIFIED_PROFILE_ID));
    assert_eq!(timed.status, "timeout");
}

#[test]
fn loom_qualified_run_rejects_missing_model_entry_and_env_override() {
    let dir = temp_dir("loom-pass");
    let profile = selected_profile(LOOM_QUALIFIED_PROFILE_ID).expect("loom profile");
    let runtime = fake_loom_runtime(&dir, "pass");
    let (rendered, exit_code) = run_with(&pilot_fixture_dir(), &profile, &runtime).expect("run loom");
    assert_eq!(exit_code, 0);
    let eval = evaluate_input(&rendered, Some(LOOM_QUALIFIED_PROFILE_ID));
    assert_eq!(eval.status, "qualified_passed", "{:?}", eval.diagnostics);
    let details = eval.details.expect("details");
    assert!(details.is_model_entered);
    assert!(details.is_model_completed);
    assert_eq!(details.permutations_explored, 7);

    let no_model_dir = temp_dir("loom-nomodel");
    let (no_model, _) =
        run_with(&pilot_fixture_dir(), &profile, &fake_loom_runtime(&no_model_dir, "no_model")).expect("run");
    let eval_no_model = evaluate_input(&no_model, Some(LOOM_QUALIFIED_PROFILE_ID));
    assert_eq!(eval_no_model.status, "failed");
    assert!(eval_no_model.diagnostics.iter().any(|d| d.contains("Loom model-entry")));

    let fail_dir = temp_dir("loom-fail");
    let (failed, _) = run_with(&pilot_fixture_dir(), &profile, &fake_loom_runtime(&fail_dir, "fail")).expect("run");
    let eval_fail = evaluate_input(&failed, Some(LOOM_QUALIFIED_PROFILE_ID));
    assert_eq!(eval_fail.status, "failed");

    let mut overridden = runtime;
    overridden.caller_overrides.push("RUSTFLAGS".to_owned());
    let err = run_with(&pilot_fixture_dir(), &profile, &overridden).expect_err("reject override");
    assert!(err.contains("RUSTFLAGS"));
}

#[test]
fn compiler_lab_199_matrix_receipts_handle_all_dispositions_and_staleness() {
    let profile = selected_profile(MIRI_QUALIFIED_PROFILE_ID).expect("miri");
    let mut input = DynamicQualificationInput {
        schema_version: MATRIX_RECEIPT_SCHEMA.to_owned(),
        profile_id: MIRI_QUALIFIED_PROFILE_ID.to_owned(),
        profile_identity: EXPECTED_PROFILE_IDENTITY.to_owned(),
        source_manifest_identity: super::prefixed_identity(b"src"),
        expected_source_manifest_identity: Some(super::prefixed_identity(b"src")),
        lockfile_identity: Some(super::prefixed_identity(b"lock")),
        expected_lockfile_identity: Some(super::prefixed_identity(b"lock")),
        command_argv: profile.command_argv.clone(),
        compiler_probe: CompilerProbe {
            cargo_version: "cargo 1.98.0-nightly".to_owned(),
            rustc_release: "1.98.0-nightly".to_owned(),
            commit_hash: "31a9463c6e2794a59ce57a8f37abc6966afc2a58".to_owned(),
            host_triple: "x86_64-unknown-linux-gnu".to_owned(),
            tool_kind: "miri".to_owned(),
            tool_version: "miri 0.1.0".to_owned(),
            sysroot_identity: Some(super::prefixed_identity(b"sysroot")),
        },
        observed_runs: vec![CellObservation {
            cell_id: "miri-x86_64-tree-borrows-seed1".to_owned(),
            disposition: "passed".to_owned(),
            target_triple: "x86_64-unknown-linux-gnu".to_owned(),
            tool_version: "miri 0.1.0".to_owned(),
            rustc_release: "1.98.0-nightly".to_owned(),
            miri_flags: Some(profile.miri.as_ref().expect("miri").miri_flags.clone()),
            rustflags: None,
            encoded_rustflags: None,
            tests_started: 2,
            tests_passed: 2,
            tests_failed: 0,
            tests_ignored: 0,
            tests_filtered_out: 0,
            executed_tests: profile.test_inventory.clone(),
            filters: None,
            model_entered: Some(false),
            model_completed: Some(false),
            permutations_explored: Some(0),
            exit_code: 0,
            timed_out: Some(false),
            truncated: Some(false),
            bounded_log_identity: super::prefixed_identity(b"log"),
            failure_messages: None,
        }],
    };
    let ok = evaluate_input(&serde_json::to_string(&input).expect("json"), Some(MIRI_QUALIFIED_PROFILE_ID));
    assert_eq!(ok.status, "qualified_passed", "{:?}", ok.diagnostics);

    input.observed_runs[0].disposition = "unsupported".to_owned();
    assert_eq!(evaluate_input(&serde_json::to_string(&input).expect("json"), None).status, "unsupported");
    input.observed_runs[0].disposition = "unavailable".to_owned();
    assert_eq!(evaluate_input(&serde_json::to_string(&input).expect("json"), None).status, "unavailable");

    input.observed_runs[0].disposition = "passed".to_owned();
    let dup = input.observed_runs[0].clone();
    input.observed_runs.push(dup);
    let dup_eval = evaluate_input(&serde_json::to_string(&input).expect("json"), None);
    assert!(dup_eval.diagnostics.iter().any(|d| d.contains("duplicate matrix cell")));

    input.observed_runs.clear();
    let missing_eval = evaluate_input(&serde_json::to_string(&input).expect("json"), None);
    assert_eq!(missing_eval.status, "incomplete");

    input.expected_source_manifest_identity = Some(super::prefixed_identity(b"different"));
    let stale_eval = evaluate_input(&serde_json::to_string(&input).expect("json"), None);
    assert!(stale_eval.diagnostics.iter().any(|d| d.contains("stale")));
}
