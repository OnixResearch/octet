use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

mod support {
    #[allow(dead_code)]
    pub mod consumer_fixture;
}

use support::consumer_fixture::TempDir;
use support::consumer_fixture::bash_path;
use support::consumer_fixture::built_cargo_octet_bin;
use support::consumer_fixture::write_executable;
use support::consumer_fixture::write_file;

const FIXTURE_REL_DIR: &str = "tests/fixtures/cargo-native-lint";
const OCTET_RUNNER_ENV: [&str; 8] = [
    "OCTET_TOOLCHAIN",
    "OCTET_DRIVER_PATH",
    "OCTET_LINT_LIB",
    "OCTET_DYLINT_REV",
    "TIGERSTYLE_TOOLCHAIN",
    "TIGERSTYLE_DRIVER_PATH",
    "TIGERSTYLE_LINT_LIB",
    "TIGERSTYLE_DYLINT_REV",
];

struct CaptureFixture {
    cargo_version: String,
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_REL_DIR)
}

fn load_capture(name: &str) -> CaptureFixture {
    let path = fixture_root().join("captures").join(format!("{name}.json"));
    let text = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read capture {}: {err}", path.display()));
    let json: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse capture {}: {err}", path.display()));
    CaptureFixture {
        cargo_version: json["cargo_version"].as_str().expect("cargo_version").to_owned(),
        exit_code: i32::try_from(json["exit_code"].as_i64().expect("exit_code")).expect("exit_code range"),
        stdout: json["stdout"].as_str().expect("stdout").to_owned(),
        stderr: json["stderr"].as_str().expect("stderr").to_owned(),
    }
}

fn copy_fixture_workspace(dest: &Path) {
    copy_dir_filtered(&fixture_root(), dest);
}

fn copy_dir_filtered(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create target fixture dir");
    for entry in fs::read_dir(from).expect("read source fixture dir") {
        let entry = entry.expect("dir entry");
        let name = entry.file_name();
        if name == OsStr::new("captures") || name == OsStr::new("target") {
            continue;
        }
        let src_path = entry.path();
        let dst_path = to.join(&name);
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_filtered(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy fixture file");
        }
    }
}

fn cargo_octet_bin() -> PathBuf {
    match env::var("CARGO_OCTET_BIN") {
        Ok(bin) if !bin.is_empty() => PathBuf::from(bin),
        _ => built_cargo_octet_bin().expect("build cargo-octet"),
    }
}

fn setup_fake_cargo_capture(tmp: &Path, ws: &Path, capture: &CaptureFixture) -> (PathBuf, PathBuf, PathBuf) {
    let bash = bash_path().expect("bash");
    let fakebin = tmp.join("fakebin");
    let driver = tmp.join("bin/dylint-driver");
    let lint_lib = tmp.join("lib/liboctet.so");
    let stdout_file = tmp.join("capture.stdout");
    let stderr_file = tmp.join("capture.stderr");
    let target_dir = tmp.join("target");
    fs::create_dir_all(&fakebin).expect("fakebin");
    fs::create_dir_all(&target_dir).expect("target_dir");

    let ws_str = ws.display().to_string();
    let tgt_str = target_dir.display().to_string();
    write_file(&stdout_file, &capture.stdout.replace("<WORKSPACE>", &ws_str).replace("<TARGET>", &tgt_str));
    write_file(&stderr_file, &capture.stderr.replace("<WORKSPACE>", &ws_str).replace("<TARGET>", &tgt_str));

    let cargo_script = format!(
        r#"#!{bash}
set -euo pipefail
case "${{1:-}}" in
  --version)
    printf '%s\n' "$CAP_CARGO_VERSION"
    ;;
  metadata)
    if [[ " $* " == *" --offline "* ]]; then
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps
    else
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps --offline
    fi
    ;;
  check)
    if [[ -s "$CAP_STDOUT_FILE" ]]; then
      cat "$CAP_STDOUT_FILE"
    fi
    if [[ -s "$CAP_STDERR_FILE" ]]; then
      cat "$CAP_STDERR_FILE" >&2
    fi
    exit "${{CAP_EXIT_CODE:?}}"
    ;;
  *)
    echo "unexpected cargo subcommand: $*" >&2
    exit 97
    ;;
esac
"#,
        bash = bash.display()
    );
    write_executable(&fakebin.join("cargo"), &cargo_script);
    write_executable(&driver, &format!("#!{}\nexit 0\n", bash.display()));
    write_file(&lint_lib, "fake liboctet\n");
    (fakebin, driver, lint_lib)
}

fn run_cargo_octet_with_fake_cargo(tmp: &Path, ws: &Path, capture: &CaptureFixture, args: &[&str]) -> Output {
    let (fakebin, driver, lint_lib) = setup_fake_cargo_capture(tmp, ws, capture);
    let path = format!("{}:{}", fakebin.display(), env::var("PATH").expect("PATH"));
    let mut cmd = Command::new(cargo_octet_bin());
    for name in OCTET_RUNNER_ENV {
        cmd.env_remove(name);
    }
    cmd.args(args)
        .current_dir(ws)
        .env("PATH", path)
        .env("TS_REAL_CARGO", env::var_os("CARGO").expect("CARGO"))
        .env("CAP_CARGO_VERSION", &capture.cargo_version)
        .env("CAP_STDOUT_FILE", tmp.join("capture.stdout"))
        .env("CAP_STDERR_FILE", tmp.join("capture.stderr"))
        .env("CAP_EXIT_CODE", capture.exit_code.to_string())
        .env("OCTET_TOOLCHAIN", "nightly-2026-03-21-x86_64-unknown-linux-gnu")
        .env("OCTET_DRIVER_PATH", driver)
        .env("OCTET_LINT_LIB", lint_lib)
        .env("CARGO_TARGET_DIR", tmp.join("target"))
        .env("XDG_CACHE_HOME", tmp.join("xdg-cache"));
    cmd.output().expect("run cargo-octet")
}

fn read_json(path: &Path) -> serde_json::Value {
    let text = fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn set_fixture_unused_dep_level(ws: &Path, level: &str) {
    for rel in ["Cargo.toml", "app/Cargo.toml"] {
        let path = ws.join(rel);
        let text = fs::read_to_string(&path).expect("read manifest");
        fs::write(&path, text.replace("unused_dependencies = \"warn\"", &format!("unused_dependencies = \"{level}\"")))
            .expect("write manifest");
    }
}

#[test]
fn replay_captured_1_100_warn_workspace_across_human_json_sarif() {
    let capture = load_capture("cargo-1.100.0-warn-workspace");
    let tmp = TempDir::new("cargo-native-warn");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);

    let human_dir = tmp.path().join("artifacts-human");
    let json_dir = tmp.path().join("artifacts-json");
    let sarif_dir = tmp.path().join("artifacts-sarif");

    let human = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &[
        "check",
        "--workspace",
        "--artifact-dir",
        human_dir.to_str().expect("human dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(human.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&human.stderr));

    let json_out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        json_dir.to_str().expect("json dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(json_out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&json_out.stderr));

    let sarif_out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &[
        "check",
        "--workspace",
        "--output-format",
        "sarif",
        "--artifact-dir",
        sarif_dir.to_str().expect("sarif dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(sarif_out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&sarif_out.stderr));

    let status = read_json(&json_dir.join("status.json"));
    assert_eq!(status["status"], "warning-only");
    assert_eq!(status["total_findings"], 2);
    assert_eq!(status["warning_findings"], 2);
    assert_eq!(status["error_findings"], 0);

    let summary = fs::read_to_string(json_dir.join("summary.txt")).expect("read summary");
    assert!(summary.contains("Cargo version: cargo 1.100.0"), "{summary}");
    assert!(summary.contains("Cargo lints: partial ("), "{summary}");
    assert!(summary.contains("[dev-dependencies] excluded by cargo::unused_dependencies"), "{summary}");
    assert!(summary.contains("Cargo lint identity: b3:"), "{summary}");
    let strip_profile_hash = |text: String| -> Vec<String> {
        text.lines().filter(|line| !line.starts_with("Profile hash: ")).map(ToOwned::to_owned).collect()
    };
    assert_eq!(
        strip_profile_hash(summary.clone()),
        strip_profile_hash(fs::read_to_string(human_dir.join("summary.txt")).expect("human summary"))
    );
    assert_eq!(
        strip_profile_hash(summary),
        strip_profile_hash(fs::read_to_string(sarif_dir.join("summary.txt")).expect("sarif summary"))
    );

    let jsonl = fs::read_to_string(json_dir.join("results.jsonl")).expect("read results.jsonl");
    let findings: Vec<serde_json::Value> =
        jsonl.lines().map(|line| serde_json::from_str(line).expect("parse finding")).collect();
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0]["lint"], "cargo::unused_dependencies");
    assert_eq!(findings[0]["group"], "tigerstyle::cargo");
    assert_eq!(findings[0]["file"], "app/Cargo.toml");
    assert_eq!(findings[0]["crate_name"], "app");
    assert_eq!(findings[0]["line"], 18);
    assert_eq!(findings[1]["file"], "other_pkg/Cargo.toml");
    assert_eq!(findings[1]["crate_name"], "other_pkg");
    assert_eq!(findings[1]["line"], 7);

    let sarif = read_json(&sarif_dir.join("results.sarif"));
    let results = sarif["runs"][0]["results"].as_array().expect("sarif results");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["ruleId"], "cargo::unused_dependencies");
    assert_eq!(results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"], "app/Cargo.toml");
    assert_eq!(results[1]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"], "other_pkg/Cargo.toml");
    let rules = sarif["runs"][0]["tool"]["driver"]["rules"].as_array().expect("sarif rules");
    assert_eq!(rules[0]["id"], "cargo::unused_dependencies");
    assert_eq!(rules[0]["properties"]["group"], "tigerstyle::cargo");
}

#[test]
fn replay_captured_1_100_allow_and_deny_outcomes() {
    let allow_cap = load_capture("cargo-1.100.0-allow-workspace");
    let allow_tmp = TempDir::new("cargo-native-allow");
    let allow_ws = allow_tmp.path().join("ws");
    copy_fixture_workspace(&allow_ws);
    set_fixture_unused_dep_level(&allow_ws, "allow");
    let allow_dir = allow_tmp.path().join("artifacts");
    let allow_out = run_cargo_octet_with_fake_cargo(allow_tmp.path(), &allow_ws, &allow_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        allow_dir.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(allow_out.status.code(), Some(0));
    let allow_status = read_json(&allow_dir.join("status.json"));
    assert_eq!(allow_status["status"], "clean");
    assert_eq!(allow_status["total_findings"], 0);
    let allow_summary = fs::read_to_string(allow_dir.join("summary.txt")).expect("allow summary");
    assert!(allow_summary.contains("Cargo lints: partial ("), "{allow_summary}");
    assert!(allow_summary.contains("set to `allow`"), "{allow_summary}");

    let deny_cap = load_capture("cargo-1.100.0-deny-workspace");
    let deny_tmp = TempDir::new("cargo-native-deny");
    let deny_ws = deny_tmp.path().join("ws");
    copy_fixture_workspace(&deny_ws);
    set_fixture_unused_dep_level(&deny_ws, "deny");
    let deny_dir = deny_tmp.path().join("artifacts");
    let deny_out = run_cargo_octet_with_fake_cargo(deny_tmp.path(), &deny_ws, &deny_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "sarif",
        "--artifact-dir",
        deny_dir.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(
        deny_out.status.code(),
        Some(1),
        "deny must exit 1 (LintFailure), stderr={}",
        String::from_utf8_lossy(&deny_out.stderr)
    );
    let deny_status = read_json(&deny_dir.join("status.json"));
    assert_eq!(deny_status["status"], "lint-failure");
    assert_eq!(deny_status["error_findings"], 2);
    assert_eq!(deny_status["cargo_process_exit"]["code"], 101);
    let deny_sarif = read_json(&deny_dir.join("results.sarif"));
    assert_eq!(deny_sarif["runs"][0]["invocations"][0]["executionSuccessful"], true);
    assert_eq!(deny_sarif["runs"][0]["invocations"][0]["exitCode"], 1);
}

#[test]
fn replay_captured_1_100_partial_selection_all_features_and_warm_runs() {
    for (scenario, scope_args, cargo_args, expected_findings, expected_coverage) in [
        ("cargo-1.100.0-warn-lib", vec!["-p", "app"], vec!["--offline", "--lib"], 0, "Cargo lints: partial ("),
        (
            "cargo-1.100.0-warn-bin",
            vec!["-p", "app"],
            vec!["--offline", "--bin", "app-bin"],
            0,
            "Cargo lints: partial (",
        ),
        ("cargo-1.100.0-warn-pkg-app", vec!["-p", "app"], vec!["--offline"], 1, "Cargo lints: partial ("),
        (
            "cargo-1.100.0-warn-all-features",
            vec!["--workspace"],
            vec!["--offline", "--all-features"],
            2,
            "Cargo lints: checked (",
        ),
        ("cargo-1.100.0-warn-warm", vec!["--workspace"], vec!["--offline"], 2, "Cargo lints: partial ("),
    ] {
        let capture = load_capture(scenario);
        let tmp = TempDir::new(scenario);
        let ws = tmp.path().join("ws");
        copy_fixture_workspace(&ws);
        let art = tmp.path().join("artifacts");
        let mut full_args = vec!["check"];
        full_args.extend(scope_args);
        full_args.extend([
            "--output-format",
            "json",
            "--artifact-dir",
            art.to_str().expect("dir"),
            "--",
        ]);
        full_args.extend(cargo_args);
        let out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &full_args);
        assert_eq!(out.status.code(), Some(0), "{scenario} failed: {}", String::from_utf8_lossy(&out.stderr));
        let status = read_json(&art.join("status.json"));
        assert_eq!(status["total_findings"], expected_findings, "{scenario}");
        let summary = fs::read_to_string(art.join("summary.txt")).expect("summary");
        assert!(summary.contains(expected_coverage), "{scenario}: {summary}");
    }
}

#[test]
fn replay_captured_1_100_compile_error_and_transport_failure_stay_integration_failure() {
    let err_cap = load_capture("cargo-1.100.0-compile-error");
    let err_tmp = TempDir::new("cargo-native-compile-err");
    let err_ws = err_tmp.path().join("ws");
    copy_fixture_workspace(&err_ws);
    let err_dir = err_tmp.path().join("artifacts");
    let err_out = run_cargo_octet_with_fake_cargo(err_tmp.path(), &err_ws, &err_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        err_dir.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(err_out.status.code(), Some(2));
    let err_status = read_json(&err_dir.join("status.json"));
    assert_eq!(err_status["status"], "integration-failure");
    let err_summary = fs::read_to_string(err_dir.join("summary.txt")).expect("summary");
    assert!(err_summary.contains("Cargo lints: error ("), "{err_summary}");

    let transport_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 101,
        stdout: String::new(),
        stderr: "error: failed to parse manifest at app/Cargo.toml\n".to_owned(),
    };
    let t_tmp = TempDir::new("cargo-native-transport");
    let t_ws = t_tmp.path().join("ws");
    copy_fixture_workspace(&t_ws);
    let t_dir = t_tmp.path().join("artifacts");
    let t_out = run_cargo_octet_with_fake_cargo(t_tmp.path(), &t_ws, &transport_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        t_dir.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(t_out.status.code(), Some(2));
    let t_summary = fs::read_to_string(t_dir.join("summary.txt")).expect("summary");
    assert!(t_summary.contains("Cargo lints: error ("), "{t_summary}");
    assert!(t_summary.contains("Cargo stderr excerpt: error: failed to parse manifest"), "{t_summary}");
}

#[test]
fn replay_1_100_boundary_identity_tracks_policy_lockfile_and_selection() {
    let capture = load_capture("cargo-1.100.0-warn-workspace");
    let run_id = |mutate_policy: bool, mutate_lock: bool, args: &[&str]| -> (String, String) {
        let tmp = TempDir::new("cargo-native-id");
        let ws = tmp.path().join("ws");
        copy_fixture_workspace(&ws);
        if mutate_policy {
            set_fixture_unused_dep_level(&ws, "deny");
        }
        if mutate_lock {
            let lock_path = ws.join("Cargo.lock");
            let mut text = fs::read_to_string(&lock_path).expect("lock");
            text.push_str("# boundary-probe\n");
            fs::write(&lock_path, text).expect("write lock");
        }
        let art = tmp.path().join("artifacts");
        let mut full = args.to_vec();
        full.extend([
            "--output-format",
            "json",
            "--artifact-dir",
            art.to_str().expect("dir"),
            "--",
            "--offline",
        ]);
        let _ = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &full);
        let summary = fs::read_to_string(art.join("summary.txt")).expect("summary");
        let status = read_json(&art.join("status.json"));
        let lint_id = summary
            .lines()
            .find_map(|line| line.strip_prefix("Cargo lint identity: "))
            .expect("Cargo lint identity")
            .to_owned();
        let config_hash = status["metadata"]["config_hash"].as_str().expect("config_hash").to_owned();
        (lint_id, config_hash)
    };

    let (base_id, base_cfg) = run_id(false, false, &["check", "--workspace"]);
    let (pol_id, pol_cfg) = run_id(true, false, &["check", "--workspace"]);
    let (lock_id, _) = run_id(false, true, &["check", "--workspace"]);
    let (sel_id, sel_cfg) = run_id(false, false, &["check", "-p", "app"]);

    assert_ne!(base_id, pol_id);
    assert_ne!(base_cfg, pol_cfg);
    assert_ne!(base_id, lock_id);
    assert_ne!(base_id, sel_id);
    assert_ne!(base_cfg, sel_cfg);
}

#[test]
fn real_cargo_check_reports_unsupported_boundary_below_1_100_or_passes_on_1_100() {
    let bash = bash_path().expect("bash");
    let tmp = TempDir::new("cargo-native-real");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);

    let driver = tmp.path().join("bin/passthrough-rustc");
    let lint_lib = tmp.path().join("lib/liboctet.so");
    write_executable(&driver, &format!("#!{}\nexec \"$@\"\n", bash.display()));
    write_file(&lint_lib, "dummy\n");

    let cargo = env::var("CARGO").expect("CARGO");
    let cargo_ver_out = Command::new(&cargo).arg("--version").output().expect("cargo --version");
    let cargo_ver_str = String::from_utf8_lossy(&cargo_ver_out.stdout);
    let is_1_100 = cargo_ver_str
        .trim()
        .strip_prefix("cargo ")
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|ver| ver.split('.').nth(1))
        .and_then(|minor| minor.split(['-', '+']).next())
        .and_then(|minor| minor.parse::<u64>().ok())
        .is_some_and(|minor| minor >= 100);

    let art = tmp.path().join("artifacts");
    let mut cmd = Command::new(cargo_octet_bin());
    for name in OCTET_RUNNER_ENV {
        cmd.env_remove(name);
    }
    let out = cmd
        .args([
            "check",
            "--workspace",
            "--output-format",
            "json",
            "--artifact-dir",
            art.to_str().expect("dir"),
            "--",
            "--offline",
        ])
        .current_dir(&ws)
        .env("OCTET_DRIVER_PATH", driver)
        .env("OCTET_LINT_LIB", lint_lib)
        .env("CARGO_TARGET_DIR", tmp.path().join("target"))
        .env("XDG_CACHE_HOME", tmp.path().join("xdg-cache"))
        .output()
        .expect("run real cargo-octet check");

    let summary = fs::read_to_string(art.join("summary.txt")).expect("summary");
    let status = read_json(&art.join("status.json"));
    if is_1_100 {
        assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(status["status"], "warning-only");
        assert_eq!(status["total_findings"], 2);
        assert!(summary.contains("Cargo lints: partial ("), "{summary}");
    } else {
        assert_eq!(out.status.code(), Some(2), "Cargo < 1.100 with [lints.cargo] must fail closed: {summary}");
        assert_eq!(status["status"], "integration-failure");
        assert!(summary.contains("Cargo lints: unsupported-cargo-version"), "{summary}");
        assert!(summary.contains("Cargo stderr excerpt:"), "{summary}");
    }
}
