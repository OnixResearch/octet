use super::CargoLintCoverageStatus;
use super::CargoVersion;
use super::parse::parse_cargo_stderr;
use super::policy::AssessmentInput;
use super::policy::assess_cargo_lints;
use super::policy::scan_workspace_policy;

#[test]
fn cargo_version_parses_stable_and_nightly_and_checks_1_100_boundary() {
    let v196 = CargoVersion::parse("cargo 1.96.0-nightly (cbb9bb8bd 2026-03-13)").expect("1.96 parses");
    assert!(!v196.is_cargo_1_100_or_newer());

    let v199 = CargoVersion::parse("cargo 1.99.1 (deadbeef 2026-06-01)").expect("1.99 parses");
    assert!(!v199.is_cargo_1_100_or_newer());

    let v1100_nightly = CargoVersion::parse("cargo 1.100.0-nightly (495c385d0 2026-09-16)").expect("1.100 nightly");
    assert!(v1100_nightly.is_cargo_1_100_or_newer());

    let v1100 = CargoVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("1.100 stable");
    assert!(v1100.is_cargo_1_100_or_newer());
    assert!(CargoVersion::parse("rustc 1.100.0").is_none());
}

#[test]
fn parses_cargo_1_100_warn_and_deny_stderr_blocks() {
    let warn_stderr = "\
    Checking app v0.1.0 (/ws/app)\n\
warning: unused dependency `unused_dep`\n\
  --> app/Cargo.toml:18:1\n\
   |\n\
18 | unused_dep = { path = \"../unused_dep\" }\n\
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\
   |\n\
   = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n\
help: consider removing the dependency on `unused_dep`\n\
warning: `app` (manifest) generated 1 warning\n";
    let parsed_warn = parse_cargo_stderr(warn_stderr, false);
    assert_eq!(parsed_warn.diagnostics.len(), 1);
    assert_eq!(parsed_warn.diagnostics[0].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed_warn.diagnostics[0].severity, "warning");
    assert_eq!(parsed_warn.diagnostics[0].raw_path, "app/Cargo.toml");
    assert_eq!(parsed_warn.diagnostics[0].line, 18);
    assert_eq!(parsed_warn.diagnostics[0].column, 1);
    assert_eq!(parsed_warn.diagnostics[0].snippet.as_deref(), Some("unused_dep = { path = \"../unused_dep\" }"));
    assert_ne!(parsed_warn.error_kind, super::parse::StderrErrorKind::UnrecognizedError);

    let deny_stderr = "\
error: unused dependency `unused_dep`\n\
 --> other_pkg/Cargo.toml:7:1\n\
  |\n\
7 | unused_dep = { path = \"../unused_dep\" }\n\
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n\
  |\n\
  = note: `cargo::unused_dependencies` is set to `deny` in `[lints]`\n\
help: consider removing the dependency on `unused_dep`\n\
error: could not finalize `other_pkg` (manifest) due to 1 previous error\n";
    let parsed_deny = parse_cargo_stderr(deny_stderr, false);
    assert_eq!(parsed_deny.diagnostics.len(), 1);
    assert_eq!(parsed_deny.diagnostics[0].severity, "error");
    assert_eq!(parsed_deny.diagnostics[0].raw_path, "other_pkg/Cargo.toml");
    assert_eq!(parsed_deny.error_kind, super::parse::StderrErrorKind::ManifestFinalizeError);
}

#[test]
fn parses_span_without_line_col_and_1_96_boundary_signals() {
    let inheritance_stderr = "\
warning: missing `[lints]` to inherit `[workspace.lints]`\n\
 --> unused_dep/Cargo.toml\n\
  = note: `cargo::missing_lints_inheritance` is set to `warn` by default\n";
    let parsed = parse_cargo_stderr(inheritance_stderr, false);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].lint_name, "cargo::missing_lints_inheritance");
    assert_eq!(parsed.diagnostics[0].raw_path, "unused_dep/Cargo.toml");
    assert_eq!(parsed.diagnostics[0].line, 1);
    assert_eq!(parsed.diagnostics[0].column, 1);

    let legacy_stderr = "\
warning: Cargo.toml: unused manifest key `lints.cargo` (may be supported in a future version)\n\
consider passing `-Zcargo-lints` to enable this feature.\n";
    let legacy_parsed = parse_cargo_stderr(legacy_stderr, false);
    assert!(legacy_parsed.has_unused_manifest_key_lints_cargo);
    assert!(legacy_parsed.diagnostics.is_empty());
}

#[test]
fn assesses_coverage_boundaries_and_deterministic_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"other\"]\n\n[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n",
    )
    .expect("root manifest");
    std::fs::write(dir.path().join("Cargo.lock"), "# lock v1\n").expect("lock");
    std::fs::create_dir_all(dir.path().join("app")).expect("app dir");
    std::fs::create_dir_all(dir.path().join("other")).expect("other dir");
    std::fs::write(
        dir.path().join("app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\nopt = { path = \"../other\", optional = true }\n\n[lints]\nworkspace = true\n",
    )
    .expect("app manifest");
    std::fs::write(
        dir.path().join("other/Cargo.toml"),
        "[package]\nname = \"other\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = true\n",
    )
    .expect("other manifest");

    let mut package_dirs = std::collections::BTreeMap::new();
    package_dirs.insert("app#0.1.0".to_owned(), std::path::PathBuf::from("app"));
    package_dirs.insert("other#0.1.0".to_owned(), std::path::PathBuf::from("other"));
    let policy = scan_workspace_policy(dir.path(), &package_dirs);
    assert!(policy.has_any_cargo_lints);

    let v1100 = CargoVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let v196 = CargoVersion::parse("cargo 1.96.0-nightly (cbb9bb8bd 2026-03-13)").expect("v196");
    let clean_stderr = parse_cargo_stderr("", false);

    let full = assess_cargo_lints(&AssessmentInput {
        workspace_root: dir.path(),
        cargo_version: Some(&v1100),
        policy: &policy,
        scope_args: &["--workspace".to_owned()],
        cargo_check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        is_cargo_success: true,
    });
    assert_eq!(full.status, CargoLintCoverageStatus::Checked);
    assert!(full.fail_closed_message.is_none());

    let partial_lib = assess_cargo_lints(&AssessmentInput {
        workspace_root: dir.path(),
        cargo_version: Some(&v1100),
        policy: &policy,
        scope_args: &["--workspace".to_owned()],
        cargo_check_args: &["--lib".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        is_cargo_success: true,
    });
    assert_eq!(partial_lib.status, CargoLintCoverageStatus::Partial);
    assert_ne!(full.boundary_identity, partial_lib.boundary_identity);

    let unsupported = assess_cargo_lints(&AssessmentInput {
        workspace_root: dir.path(),
        cargo_version: Some(&v196),
        policy: &policy,
        scope_args: &["--workspace".to_owned()],
        cargo_check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        is_cargo_success: true,
    });
    assert_eq!(unsupported.status, CargoLintCoverageStatus::UnsupportedCargoVersion);
    assert!(unsupported.fail_closed_message.is_some());
    assert_ne!(full.boundary_identity, unsupported.boundary_identity);
}
