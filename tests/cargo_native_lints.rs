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
    assert_eq!(status["cargo_lints"]["status"], "partial");
    assert!(status["cargo_lints"]["cargo_version"].as_str().is_some_and(|v| v.contains("1.100.0")));
    assert!(status["cargo_lints"]["boundary_identity"].as_str().is_some_and(|v| v.starts_with("b3:")));

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
    assert_eq!(sarif["runs"][0]["properties"]["cargoLints"]["status"], "partial");
    assert_eq!(sarif["runs"][0]["properties"]["cargoLints"], status["cargo_lints"]);
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

fn run_with_custom_cargo_script(tmp: &Path, ws: &Path, cargo_script: &str, args: &[&str]) -> Output {
    let bash = bash_path().expect("bash");
    let fakebin = tmp.join("fakebin");
    let driver = tmp.join("bin/dylint-driver");
    let lint_lib = tmp.join("lib/liboctet.so");
    fs::create_dir_all(&fakebin).expect("fakebin");
    write_executable(&fakebin.join("cargo"), cargo_script);
    write_executable(&driver, &format!("#!{}\nexit 0\n", bash.display()));
    write_file(&lint_lib, "fake liboctet\n");
    let path = format!("{}:{}", fakebin.display(), env::var("PATH").expect("PATH"));
    let mut cmd = Command::new(cargo_octet_bin());
    for name in OCTET_RUNNER_ENV {
        cmd.env_remove(name);
    }
    cmd.args(args)
        .current_dir(ws)
        .env("PATH", path)
        .env("TS_REAL_CARGO", env::var_os("CARGO").expect("CARGO"))
        .env("OCTET_TOOLCHAIN", "nightly-2026-03-21-x86_64-unknown-linux-gnu")
        .env("OCTET_DRIVER_PATH", driver)
        .env("OCTET_LINT_LIB", lint_lib)
        .env("CARGO_TARGET_DIR", tmp.join("target"))
        .env("XDG_CACHE_HOME", tmp.join("xdg-cache"));
    cmd.output().expect("run cargo-octet")
}

fn colorize_cargo_stderr(stderr: &str) -> String {
    stderr
        .replace("warning:", "\x1b[1m\x1b[33mwarning\x1b[0m\x1b[1m:")
        .replace("error:", "\x1b[1m\x1b[31merror\x1b[0m\x1b[1m:")
        .replace("--> ", "\x1b[1m\x1b[94m--> \x1b[0m")
        .replace("= note:", "\x1b[1m\x1b[94m= \x1b[0m\x1b[1mnote\x1b[0m:")
        .replace("help:", "\x1b[1m\x1b[96mhelp\x1b[0m:")
}

#[test]
fn ansi_color_always_warn_deny_and_1_96_parse_cleanly() {
    let warn_base = load_capture("cargo-1.100.0-warn-workspace");
    let warn_colored = CaptureFixture {
        stderr: colorize_cargo_stderr(&warn_base.stderr),
        ..warn_base
    };
    let tmp_w = TempDir::new("cargo-native-ansi-warn");
    let ws_w = tmp_w.path().join("ws");
    copy_fixture_workspace(&ws_w);
    let art_w = tmp_w.path().join("artifacts");
    let out_w = run_cargo_octet_with_fake_cargo(tmp_w.path(), &ws_w, &warn_colored, &[
        "check",
        "--workspace",
        "--artifact-dir",
        art_w.to_str().expect("dir"),
        "--",
        "--offline",
        "--color=always",
    ]);
    assert_eq!(out_w.status.code(), Some(0));
    let status_w = read_json(&art_w.join("status.json"));
    assert_eq!(status_w["total_findings"], 2);
    let summary_w = fs::read_to_string(art_w.join("summary.txt")).expect("summary");
    assert!(!summary_w.contains('\x1b'));

    let deny_base = load_capture("cargo-1.100.0-deny-workspace");
    let deny_colored = CaptureFixture {
        stderr: colorize_cargo_stderr(&deny_base.stderr),
        ..deny_base
    };
    let tmp_d = TempDir::new("cargo-native-ansi-deny");
    let ws_d = tmp_d.path().join("ws");
    copy_fixture_workspace(&ws_d);
    set_fixture_unused_dep_level(&ws_d, "deny");
    let art_d = tmp_d.path().join("artifacts");
    let out_d = run_cargo_octet_with_fake_cargo(tmp_d.path(), &ws_d, &deny_colored, &[
        "check",
        "--workspace",
        "--artifact-dir",
        art_d.to_str().expect("dir"),
        "--",
        "--offline",
        "--color=always",
    ]);
    assert_eq!(out_d.status.code(), Some(1));
    let status_d = read_json(&art_d.join("status.json"));
    assert_eq!(status_d["status"], "lint-failure");
    assert_eq!(status_d["error_findings"], 2);

    let legacy_base = load_capture("cargo-1.96.0-nightly-warn-workspace");
    let legacy_colored = CaptureFixture {
        stderr: colorize_cargo_stderr(&legacy_base.stderr),
        ..legacy_base
    };
    let tmp_l = TempDir::new("cargo-native-ansi-196");
    let ws_l = tmp_l.path().join("ws");
    copy_fixture_workspace(&ws_l);
    let art_l = tmp_l.path().join("artifacts");
    let out_l = run_cargo_octet_with_fake_cargo(tmp_l.path(), &ws_l, &legacy_colored, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_l.to_str().expect("dir"),
        "--",
        "--offline",
        "--color=always",
    ]);
    assert_eq!(out_l.status.code(), Some(2));
    let summary_l = fs::read_to_string(art_l.join("summary.txt")).expect("summary");
    assert!(summary_l.contains("Cargo lints: unsupported-cargo-version"), "{summary_l}");
    assert!(!summary_l.contains('\x1b'));
}

#[test]
fn multi_dep_single_package_warn_and_deny_extract_all_findings() {
    let multi_warn = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 0,
        stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
        stderr: "\
warning: unused dependency `unused_dep`\n\
  --> app/Cargo.toml:18:1\n\
   |\n\
18 | unused_dep = { path = \"../unused_dep\" }\n\
   = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n\
help: remove `unused_dep`\n\
warning: unused dependency `feature_dep`\n\
  --> app/Cargo.toml:20:1\n\
   |\n\
20 | feature_dep = { path = \"../feature_dep\" }\n\
help: remove `feature_dep`\n\
warning: `app` (manifest) generated 2 warnings\n"
            .to_owned(),
    };
    let tmp = TempDir::new("cargo-native-multi-warn");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);
    let art = tmp.path().join("artifacts");
    let out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &multi_warn, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art.to_str().expect("dir"),
        "--",
        "--offline",
        "--all-features",
    ]);
    assert_eq!(out.status.code(), Some(0));
    let status = read_json(&art.join("status.json"));
    assert_eq!(status["total_findings"], 2);
    let summary = fs::read_to_string(art.join("summary.txt")).expect("summary");
    assert!(summary.contains("Cargo lints: checked ("), "{summary}");

    let multi_deny = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 101,
        stdout: "{\"reason\":\"build-finished\",\"success\":false}\n".to_owned(),
        stderr: multi_warn
            .stderr
            .replace("warning: unused dependency", "error: unused dependency")
            .replace("set to `warn`", "set to `deny`")
            .replace(
                "warning: `app` (manifest) generated 2 warnings",
                "error: could not finalize `app` (manifest) due to 2 previous errors",
            ),
    };
    let tmp_d = TempDir::new("cargo-native-multi-deny");
    let ws_d = tmp_d.path().join("ws");
    copy_fixture_workspace(&ws_d);
    set_fixture_unused_dep_level(&ws_d, "deny");
    let art_d = tmp_d.path().join("artifacts");
    let out_d = run_cargo_octet_with_fake_cargo(tmp_d.path(), &ws_d, &multi_deny, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_d.to_str().expect("dir"),
        "--",
        "--offline",
        "--all-features",
    ]);
    assert_eq!(out_d.status.code(), Some(1));
    let status_d = read_json(&art_d.join("status.json"));
    assert_eq!(status_d["status"], "lint-failure");
    assert_eq!(status_d["error_findings"], 2);
}

#[test]
fn human_mode_large_stderr_does_not_deadlock_and_non_utf8_emits_artifacts() {
    let bash = bash_path().expect("bash");
    let tmp = TempDir::new("cargo-native-deadlock");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);
    let art = tmp.path().join("artifacts");
    let large_script = format!(
        r#"#!{bash}
set -euo pipefail
case "${{1:-}}" in
  --version) echo "cargo 1.100.0 (3acc79c0c 2026-11-05)" ;;
  metadata)
    if [[ " $* " == *" --offline "* ]]; then
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps
    else
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps --offline
    fi
    ;;
  check)
    head -c 140000 /dev/zero | tr '\0' 'a' >&2
    printf '\n' >&2
    printf '{{"reason":"build-finished","success":true}}\n'
    ;;
esac
"#,
        bash = bash.display()
    );
    let out = run_with_custom_cargo_script(tmp.path(), &ws, &large_script, &[
        "check",
        "--workspace",
        "--artifact-dir",
        art.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));

    let tmp_bad = TempDir::new("cargo-native-nonutf8");
    let ws_bad = tmp_bad.path().join("ws");
    copy_fixture_workspace(&ws_bad);
    let art_bad = tmp_bad.path().join("artifacts");
    let bad_script = format!(
        r#"#!{bash}
set -euo pipefail
case "${{1:-}}" in
  --version) echo "cargo 1.100.0 (3acc79c0c 2026-11-05)" ;;
  metadata)
    if [[ " $* " == *" --offline "* ]]; then
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps
    else
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps --offline
    fi
    ;;
  check)
    printf 'error: bad \377\376 manifest\n' >&2
    head -c 310000 /dev/zero | tr '\0' 'b' >&2
    printf '\n' >&2
    exit 101
    ;;
esac
"#,
        bash = bash.display()
    );
    let out_bad = run_with_custom_cargo_script(tmp_bad.path(), &ws_bad, &bad_script, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_bad.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_bad.status.code(), Some(2));
    let status_bad = read_json(&art_bad.join("status.json"));
    assert_eq!(status_bad["status"], "integration-failure");
    let summary_bad = fs::read_to_string(art_bad.join("summary.txt")).expect("summary");
    assert!(summary_bad.contains("Cargo stderr excerpt:"), "{summary_bad}");
}

#[test]
fn large_stderr_unselected_vs_selected_version_none_and_cooccurring_errors() {
    let bash = bash_path().expect("bash");
    let big_stderr = "note: verbose progress\n".repeat(15_000);
    assert!(big_stderr.len() > 256 * 1024);

    let tmp_un = TempDir::new("cargo-native-unselected-big");
    let ws_un = tmp_un.path().join("ws");
    write_file(
        &ws_un.join("Cargo.toml"),
        "[workspace]\n\n[workspace.metadata.octet]\ndefault_scope = [\"--workspace\"]\ncargo_check_args = []\n\n[package]\nname = \"plain\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write_file(&ws_un.join("src/lib.rs"), "pub fn ok() {}\n");
    write_file(&ws_un.join("Cargo.lock"), "# lock\n");
    let art_un = tmp_un.path().join("artifacts");
    let big_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 0,
        stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
        stderr: big_stderr.clone(),
    };
    let out_un = run_cargo_octet_with_fake_cargo(tmp_un.path(), &ws_un, &big_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "sarif",
        "--artifact-dir",
        art_un.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_un.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out_un.stderr));
    let status_un = read_json(&art_un.join("status.json"));
    assert!(status_un.get("cargo_lints").is_none());
    let sarif_un = read_json(&art_un.join("results.sarif"));
    assert!(sarif_un["runs"][0]["properties"].get("cargoLints").is_none());

    let tmp_sel = TempDir::new("cargo-native-selected-big");
    let ws_sel = tmp_sel.path().join("ws");
    copy_fixture_workspace(&ws_sel);
    let art_sel = tmp_sel.path().join("artifacts");
    let out_sel = run_cargo_octet_with_fake_cargo(tmp_sel.path(), &ws_sel, &big_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_sel.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_sel.status.code(), Some(2));
    let status_sel = read_json(&art_sel.join("status.json"));
    assert_eq!(status_sel["cargo_lints"]["status"], "error");

    let tmp_nv = TempDir::new("cargo-native-no-version");
    let ws_nv = tmp_nv.path().join("ws");
    copy_fixture_workspace(&ws_nv);
    let art_nv = tmp_nv.path().join("artifacts");
    let no_ver_script = format!(
        r#"#!{bash}
set -euo pipefail
case "${{1:-}}" in
  --version) exit 1 ;;
  metadata)
    if [[ " $* " == *" --offline "* ]]; then
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps
    else
      exec "${{TS_REAL_CARGO:?}}" "$@" --no-deps --offline
    fi
    ;;
  check) printf '{{"reason":"build-finished","success":true}}\n' ;;
esac
"#,
        bash = bash.display()
    );
    let out_nv = run_with_custom_cargo_script(tmp_nv.path(), &ws_nv, &no_ver_script, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_nv.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_nv.status.code(), Some(2));
    let status_nv = read_json(&art_nv.join("status.json"));
    assert_eq!(status_nv["cargo_lints"]["status"], "error");
}

#[test]
fn cooccurring_unrecognized_error_fails_closed_while_octet_deny_stays_lint_failure() {
    let deny_plus_unrec = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 101,
        stdout: "{\"reason\":\"build-finished\",\"success\":false}\n".to_owned(),
        stderr: format!(
            "{}\nerror: failed to parse manifest at other_pkg/Cargo.toml\n",
            load_capture("cargo-1.100.0-deny-workspace").stderr
        ),
    };
    let tmp_mix = TempDir::new("cargo-native-deny-unrec");
    let ws_mix = tmp_mix.path().join("ws");
    copy_fixture_workspace(&ws_mix);
    set_fixture_unused_dep_level(&ws_mix, "deny");
    let art_mix = tmp_mix.path().join("artifacts");
    let out_mix = run_cargo_octet_with_fake_cargo(tmp_mix.path(), &ws_mix, &deny_plus_unrec, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_mix.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_mix.status.code(), Some(2));
    assert_eq!(read_json(&art_mix.join("status.json"))["status"], "integration-failure");

    let octet_deny_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 101,
        stdout: "{\"reason\":\"compiler-message\",\"package_id\":\"path+file://<WORKSPACE>/app#0.1.0\",\"manifest_path\":\"<WORKSPACE>/app/Cargo.toml\",\"target\":{\"kind\":[\"lib\"],\"crate_types\":[\"lib\"],\"name\":\"app\",\"src_path\":\"<WORKSPACE>/app/src/lib.rs\",\"edition\":\"2024\"},\"message\":{\"rendered\":\"error: used unwrap\",\"children\":[],\"level\":\"error\",\"message\":\"used unwrap\",\"spans\":[{\"file_name\":\"app/src/lib.rs\",\"line_start\":1,\"column_start\":1,\"is_primary\":true,\"text\":[{\"text\":\"x.unwrap()\"}]}],\"code\":{\"code\":\"tigerstyle::no_unwrap\"}}}\n{\"reason\":\"build-finished\",\"success\":false}\n".to_owned(),
        stderr: "error: could not compile `app` (lib) due to 1 previous error\n".to_owned(),
    };
    let tmp_od = TempDir::new("cargo-native-octet-deny");
    let ws_od = tmp_od.path().join("ws");
    copy_fixture_workspace(&ws_od);
    let art_od = tmp_od.path().join("artifacts");
    let out_od = run_cargo_octet_with_fake_cargo(tmp_od.path(), &ws_od, &octet_deny_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_od.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_od.status.code(), Some(1));
    assert_eq!(read_json(&art_od.join("status.json"))["status"], "lint-failure");
}

#[cfg(unix)]
#[test]
fn diagnostic_path_containment_and_member_config_hash() {
    let tmp = TempDir::new("cargo-native-path-contain");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);
    write_file(&ws.join("app/secret.toml"), "secret = true\n");
    fs::create_dir_all(ws.join("app/evil")).expect("evil dir");
    std::os::unix::fs::symlink("../secret.toml", ws.join("app/evil/Cargo.toml")).expect("symlink");

    for bad_path in ["app/evil/Cargo.toml", "nonexistent_crate/Cargo.toml"] {
        let cap = CaptureFixture {
            cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
            exit_code: 0,
            stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
            stderr: format!(
                "warning: unused dependency `unused_dep`\n --> {bad_path}:1:1\n  = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n"
            ),
        };
        let art = tmp.path().join(format!("art-{}", bad_path.replace('/', "_")));
        let out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &cap, &[
            "check",
            "--workspace",
            "--output-format",
            "json",
            "--artifact-dir",
            art.to_str().expect("dir"),
            "--",
            "--offline",
        ]);
        assert_eq!(out.status.code(), Some(2), "{bad_path} must fail closed");
        let status = read_json(&art.join("status.json"));
        assert_eq!(status["status"], "integration-failure");
        assert!(art.join("summary.txt").is_file());
    }

    let dot_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 0,
        stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
        stderr: "warning: unused dependency `unused_dep`\n --> ./app/Cargo.toml:18:1\n  = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n".to_owned(),
    };
    let art_dot = tmp.path().join("art-dot");
    let out_dot = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &dot_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_dot.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_dot.status.code(), Some(0));
    let jsonl = fs::read_to_string(art_dot.join("results.jsonl")).expect("jsonl");
    let finding: serde_json::Value = serde_json::from_str(jsonl.lines().next().expect("line")).expect("json");
    assert_eq!(finding["file"], "app/Cargo.toml");
    assert_eq!(finding["crate_name"], "app");

    let hash_before = read_json(&art_dot.join("status.json"))["metadata"]["config_hash"]
        .as_str()
        .expect("config_hash")
        .to_owned();
    let app_manifest = fs::read_to_string(ws.join("app/Cargo.toml")).expect("read app");
    fs::write(ws.join("app/Cargo.toml"), app_manifest.replace("\"warn\"", "\"deny\"")).expect("mutate member");
    let art_after = tmp.path().join("art-after");
    let _ = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &dot_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_after.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    let hash_after = read_json(&art_after.join("status.json"))["metadata"]["config_hash"]
        .as_str()
        .expect("config_hash")
        .to_owned();
    assert_ne!(hash_before, hash_after);
}

#[test]
fn member_workspace_lint_inheritance_mutates_config_hash_and_cache_key() {
    let capture = load_capture("cargo-1.100.0-warn-workspace");
    let tmp = TempDir::new("cargo-native-inherit-cache");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);

    let run_cached = |art: &Path| {
        let out = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &capture, &[
            "check",
            "--workspace",
            "--cache",
            "--output-format",
            "json",
            "--artifact-dir",
            art.to_str().expect("dir"),
            "--",
            "--offline",
        ]);
        assert_eq!(out.status.code(), Some(0), "stderr={}", String::from_utf8_lossy(&out.stderr));
        let status = read_json(&art.join("status.json"));
        let jsonl = fs::read_to_string(art.join("results.jsonl")).expect("jsonl");
        let findings: Vec<serde_json::Value> =
            jsonl.lines().map(|line| serde_json::from_str(line).expect("parse finding")).collect();
        (status, findings)
    };

    let (status1, findings1) = run_cached(&tmp.path().join("art1"));
    assert_eq!(findings1.len(), 2);
    assert!(findings1.iter().all(|f| f["cache_status"] == "miss"));
    let other1 = findings1.iter().find(|f| f["crate_name"] == "other_pkg").expect("other_pkg finding");
    let cfg1 = status1["metadata"]["config_hash"].as_str().expect("cfg1").to_owned();
    let prof1 = status1["metadata"]["profile_hash"].as_str().expect("prof1").to_owned();
    let sem1 = other1["semantic_id"].as_str().expect("sem1").to_owned();
    let stab1 = other1["stable_id"].as_str().expect("stab1").to_owned();

    let (status2, findings2) = run_cached(&tmp.path().join("art2"));
    assert_eq!(status2["metadata"]["config_hash"], cfg1);
    assert_eq!(status2["metadata"]["profile_hash"], prof1);
    assert!(findings2.iter().all(|f| f["cache_status"] == "hit"));

    write_file(
        &ws.join("other_pkg/Cargo.toml"),
        "[package]\nname = \"other_pkg\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nunused_dep = { path = \"../unused_dep\" }\n",
    );

    let (status3, findings3) = run_cached(&tmp.path().join("art3"));
    let other3 = findings3.iter().find(|f| f["crate_name"] == "other_pkg").expect("other_pkg finding");
    assert_ne!(status3["metadata"]["config_hash"].as_str().expect("cfg3"), cfg1);
    assert_ne!(status3["metadata"]["profile_hash"].as_str().expect("prof3"), prof1);
    assert_ne!(other3["semantic_id"].as_str().expect("sem3"), sem1);
    assert_ne!(other3["stable_id"].as_str().expect("stab3"), stab1);
    assert!(findings3.iter().all(|f| f["cache_status"] == "miss"));

    let tmp_plain = TempDir::new("cargo-native-plain-inherit");
    let ws_plain = tmp_plain.path().join("ws");
    write_file(
        &ws_plain.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n\n[workspace.metadata.octet]\ndefault_scope = [\"--workspace\"]\ncargo_check_args = []\n\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
    );
    write_file(
        &ws_plain.join("app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    write_file(&ws_plain.join("app/src/lib.rs"), "pub fn ok() {}\n");
    write_file(&ws_plain.join("Cargo.lock"), "# lock\n");
    let plain_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 0,
        stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
        stderr: String::new(),
    };
    let art_plain1 = tmp_plain.path().join("art1");
    let out_p1 = run_cargo_octet_with_fake_cargo(tmp_plain.path(), &ws_plain, &plain_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_plain1.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_p1.status.code(), Some(0));
    let plain_cfg1 = read_json(&art_plain1.join("status.json"))["metadata"]["config_hash"]
        .as_str()
        .expect("plain_cfg1")
        .to_owned();

    write_file(
        &ws_plain.join("app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lints]\nworkspace = true\n",
    );
    let art_plain2 = tmp_plain.path().join("art2");
    let out_p2 = run_cargo_octet_with_fake_cargo(tmp_plain.path(), &ws_plain, &plain_cap, &[
        "check",
        "--workspace",
        "--output-format",
        "json",
        "--artifact-dir",
        art_plain2.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out_p2.status.code(), Some(0));
    assert_eq!(read_json(&art_plain2.join("status.json"))["metadata"]["config_hash"], plain_cfg1);
}

#[test]
fn write_baseline_does_not_create_or_overwrite_when_native_lint_fails_closed() {
    let tmp = TempDir::new("cargo-native-write-baseline");
    let ws = tmp.path().join("ws");
    copy_fixture_workspace(&ws);

    let legacy_cap = load_capture("cargo-1.96.0-nightly-warn-workspace");
    let missing_baseline = tmp.path().join("new-baseline.json");
    let art1 = tmp.path().join("art1");
    let out1 = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &legacy_cap, &[
        "check",
        "--workspace",
        "--write-baseline",
        missing_baseline.to_str().expect("baseline path"),
        "--output-format",
        "json",
        "--artifact-dir",
        art1.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out1.status.code(), Some(2));
    let status1 = read_json(&art1.join("status.json"));
    assert_eq!(status1["status"], "integration-failure");
    assert_eq!(status1["cargo_lints"]["status"], "unsupported-cargo-version");
    assert!(!missing_baseline.exists(), "fail-closed run must not create baseline");

    let existing_baseline = tmp.path().join("existing-baseline.json");
    let sentinel = "{\"sentinel\":true}\n";
    write_file(&existing_baseline, sentinel);
    let big_cap = CaptureFixture {
        cargo_version: "cargo 1.100.0 (3acc79c0c 2026-11-05)".to_owned(),
        exit_code: 0,
        stdout: "{\"reason\":\"build-finished\",\"success\":true}\n".to_owned(),
        stderr: "note: verbose progress\n".repeat(15_000),
    };
    let art2 = tmp.path().join("art2");
    let out2 = run_cargo_octet_with_fake_cargo(tmp.path(), &ws, &big_cap, &[
        "check",
        "--workspace",
        "--write-baseline",
        existing_baseline.to_str().expect("baseline path"),
        "--output-format",
        "json",
        "--artifact-dir",
        art2.to_str().expect("dir"),
        "--",
        "--offline",
    ]);
    assert_eq!(out2.status.code(), Some(2));
    let status2 = read_json(&art2.join("status.json"));
    assert_eq!(status2["status"], "integration-failure");
    assert_eq!(status2["cargo_lints"]["status"], "error");
    assert_eq!(fs::read_to_string(&existing_baseline).expect("read baseline"), sentinel);
}
