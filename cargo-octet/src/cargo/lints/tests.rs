use std::io::Read;

use super::CoverageStatus;
use super::ToolchainVersion;
use super::ansi::strip_escape_sequences;
use super::parse::extract_stderr_report;
use super::parse::extract_stderr_report_with_flags;
use super::policy::AssessmentInput;
use super::policy::assess_coverage;
use super::policy::scan_workspace;
use super::stream::read_bounded_stderr;

const CAPTURE_BOUND_BYTES: usize = 256 * 1_024;

#[test]
fn cargo_version_parses_stable_and_nightly_and_checks_1_100_boundary() {
    let v196 = ToolchainVersion::parse("cargo 1.96.0-nightly (cbb9bb8bd 2026-03-13)").expect("1.96 parses");
    assert!(!v196.is_1_100_or_newer());

    let v199 = ToolchainVersion::parse("cargo 1.99.1 (deadbeef 2026-06-01)").expect("1.99 parses");
    assert!(!v199.is_1_100_or_newer());

    let v1100_nightly = ToolchainVersion::parse("cargo 1.100.0-nightly (495c385d0 2026-09-16)").expect("1.100 nightly");
    assert!(v1100_nightly.is_1_100_or_newer());

    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("1.100 stable");
    assert!(v1100.is_1_100_or_newer());
    assert!(ToolchainVersion::parse("rustc 1.100.0").is_none());
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
    let parsed_warn = extract_stderr_report(warn_stderr, false);
    assert_eq!(parsed_warn.diagnostics.len(), 1);
    assert_eq!(parsed_warn.diagnostics[0].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed_warn.diagnostics[0].severity, "warning");
    assert_eq!(parsed_warn.diagnostics[0].raw_path, "app/Cargo.toml");
    assert_eq!(parsed_warn.diagnostics[0].line, 18);
    assert_eq!(parsed_warn.diagnostics[0].column, 1);
    assert_eq!(parsed_warn.diagnostics[0].snippet.as_deref(), Some("unused_dep = { path = \"../unused_dep\" }"));
    assert!(!parsed_warn.errors.has_unrecognized_error);

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
    let parsed_deny = extract_stderr_report(deny_stderr, false);
    assert_eq!(parsed_deny.diagnostics.len(), 1);
    assert_eq!(parsed_deny.diagnostics[0].severity, "error");
    assert_eq!(parsed_deny.diagnostics[0].raw_path, "other_pkg/Cargo.toml");
    assert!(parsed_deny.errors.has_manifest_finalize_error);
    assert!(!parsed_deny.errors.has_unrecognized_error);
}

#[test]
fn strips_ansi_csi_osc_and_preserves_utf8_boundaries() {
    assert!(matches!(strip_escape_sequences("plain text"), std::borrow::Cow::Borrowed("plain text")));
    assert_eq!(strip_escape_sequences("\x1b[1m\x1b[33mwarning\x1b[0m\x1b[1m: hi"), "warning: hi");
    assert_eq!(
        strip_escape_sequences("\x1b]8;;https://example.com\x07link\x1b]8;;\x07 and \x1b]0;title\x1b\\ok"),
        "link and ok"
    );
    assert_eq!(strip_escape_sequences("\x1b[1m🦀\x1b[0m"), "🦀");
    assert_eq!(strip_escape_sequences("\x1b[🦀"), "🦀");

    let colored_warn_deny = "\
\x1b[1m\x1b[92m    Checking\x1b[0m app v0.1.0 (/ws/app)\n\
\x1b[1m\x1b[33mwarning\x1b[0m\x1b[1m: unused dependency `dep_a`\x1b[0m\n\
  \x1b[1m\x1b[94m--> \x1b[0mapp/Cargo.toml:10:1\n\
   \x1b[1m\x1b[94m|\x1b[0m\n\
\x1b[1m\x1b[94m10\x1b[0m \x1b[1m\x1b[94m|\x1b[0m dep_a = \"0.1\"\n\
   \x1b[1m\x1b[94m|\x1b[0m \x1b[1m\x1b[33m^^^^^^^^^^^^^\x1b[0m\n\
   \x1b[1m\x1b[94m|\x1b[0m\n\
   \x1b[1m\x1b[94m= \x1b[0m\x1b[1mnote\x1b[0m: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n\
\x1b[1m\x1b[96mhelp\x1b[0m: consider removing the dependency on `dep_a`\n\
\x1b[1m\x1b[33mwarning\x1b[0m\x1b[1m: \x1b[0m`app` (manifest) generated 1 warning\n";
    let parsed = extract_stderr_report(colored_warn_deny, false);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed.diagnostics[0].raw_path, "app/Cargo.toml");
    assert_eq!(parsed.diagnostics[0].line, 10);
    assert!(!parsed.raw_excerpt.contains('\x1b'));

    let colored_196 = "\
\x1b[1m\x1b[33mwarning\x1b[0m: Cargo.toml: unused manifest key `lints.cargo` (may be supported in a future version)\n\
consider passing `-Zcargo-lints` to enable this feature.\n";
    let parsed_196 = extract_stderr_report(colored_196, false);
    assert!(parsed_196.has_unused_manifest_key_lints_cargo);
    assert!(!parsed_196.raw_excerpt.contains('\x1b'));
}

#[test]
fn parses_multi_dep_unknown_lints_reversed_help_and_cooccurring_errors() {
    let multi_warn = "\
warning: unused dependency `dep_a`\n\
 --> app/Cargo.toml:5:1\n\
  |\n\
5 | dep_a = \"1\"\n\
  |\n\
  = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n\
help: remove `dep_a`\n\
warning: unused dependency `dep_b`\n\
 --> app/Cargo.toml:6:1\n\
  |\n\
6 | dep_b = \"1\"\n\
help: remove `dep_b`\n\
warning: unused dependency `dep_c`\n\
 --> app/Cargo.toml:7:1\n\
help: remove `dep_c`\n\
warning: `app` (manifest) generated 3 warnings\n\
warning: generated code has unused dependency `dep_d`\n\
 --> other/Cargo.toml:4:1\n\
  = note: `cargo::unused_dependencies` is set to `warn` in `[lints]`\n\
warning: missing `[lints]` to inherit `[workspace.lints]`\n\
 --> other/Cargo.toml:1:1\n\
  = note: `cargo::missing_lints_inheritance` is set to `warn` by default\n\
help: to inherit `workspace.lints` please add `[lints] workspace = true`\n\
help: to clarify your intent please add `[lints]`\n\
warning: unknown lint: `bogus_one`\n\
 --> other/Cargo.toml:8:1\n\
  = note: `cargo::unknown_lints` is set to `warn` by default\n\
warning: unknown lint: `bogus_two`\n\
 --> other/Cargo.toml:9:1\n\
warning: `other` (manifest) generated 4 warnings\n";
    let parsed = extract_stderr_report(multi_warn, false);
    assert_eq!(parsed.diagnostics.len(), 7);
    assert_eq!(parsed.diagnostics[1].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed.diagnostics[2].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed.diagnostics[3].lint_name, "cargo::unused_dependencies");
    assert_eq!(parsed.diagnostics[4].lint_name, "cargo::missing_lints_inheritance");
    assert_eq!(parsed.diagnostics[5].lint_name, "cargo::unknown_lints");
    assert_eq!(parsed.diagnostics[6].lint_name, "cargo::unknown_lints");

    let reversed_and_mixed_err = "\
error: unused dependency `dep_a`\n\
 --> app/Cargo.toml:5:1\n\
help: remove `dep_a`\n\
  = note: `cargo::unused_dependencies` is set to `deny` in `[lints]`\n\
error: unused dependency `dep_b`\n\
 --> app/Cargo.toml:6:1\n\
error: could not finalize `app` (manifest) due to 2 previous errors\n\
error: failed to parse manifest at other/Cargo.toml\n";
    let parsed_err = extract_stderr_report(reversed_and_mixed_err, false);
    assert_eq!(parsed_err.diagnostics.len(), 2);
    assert!(parsed_err.errors.has_manifest_finalize_error);
    assert!(parsed_err.errors.has_unrecognized_error);
}

#[test]
fn stream_bounded_stderr_handles_invalid_utf8_truncation_and_split_codepoints() {
    struct EofTrackingReader<'a> {
        data: &'a [u8],
        pos: usize,
        hit_eof: &'a mut bool,
    }

    impl Read for EofTrackingReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.pos >= self.data.len() {
                *self.hit_eof = true;
                return Ok(0);
            }
            let n = (self.data.len() - self.pos).min(buf.len());
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Ok(n)
        }
    }

    let mut hit_eof = false;
    let non_utf8 = b"prefix\xff\xfe\nsuffix\n";
    let captured = read_bounded_stderr(
        EofTrackingReader {
            data: non_utf8,
            pos: 0,
            hit_eof: &mut hit_eof,
        },
        false,
    )
    .expect("non-utf8 drains");
    assert!(hit_eof);
    assert!(captured.has_invalid_utf8);
    assert!(!captured.is_truncated);
    assert!(captured.text.contains("prefix"));
    assert!(captured.text.contains("suffix"));

    let exact = vec![b'a'; CAPTURE_BOUND_BYTES];
    let exact_cap = read_bounded_stderr(std::io::Cursor::new(exact), false).expect("exact bound");
    assert!(!exact_cap.is_truncated);
    assert!(!exact_cap.has_invalid_utf8);

    let mut multi_line = "FIRST_LINE\n".repeat(CAPTURE_BOUND_BYTES / 11 + 100);
    multi_line.push_str("TAIL_MARKER\n");
    let over_cap = read_bounded_stderr(std::io::Cursor::new(multi_line.as_bytes()), false).expect("over bound");
    assert!(over_cap.is_truncated);
    assert!(!over_cap.text.contains("TAIL_MARKER"));
    assert_eq!(over_cap.text.len(), CAPTURE_BOUND_BYTES);

    let newline_free = vec![b'x'; CAPTURE_BOUND_BYTES + 10_000];
    let nf_cap = read_bounded_stderr(std::io::Cursor::new(newline_free), false).expect("newline free");
    assert!(nf_cap.is_truncated);
    assert_eq!(nf_cap.text.len(), CAPTURE_BOUND_BYTES);

    let mut split_utf8 = vec![b'a'; CAPTURE_BOUND_BYTES - 2];
    split_utf8.extend_from_slice("🦀".as_bytes());
    split_utf8.extend_from_slice(b"trailing");
    let split_cap = read_bounded_stderr(std::io::Cursor::new(split_utf8), false).expect("split utf8");
    assert!(split_cap.is_truncated);
    assert!(!split_cap.has_invalid_utf8);
    assert!(!split_cap.text.contains('\u{FFFD}'));
}

#[test]
fn parses_span_without_line_col_and_1_96_boundary_signals() {
    let inheritance_stderr = "\
warning: missing `[lints]` to inherit `[workspace.lints]`\n\
 --> unused_dep/Cargo.toml\n\
  = note: `cargo::missing_lints_inheritance` is set to `warn` by default\n";
    let parsed = extract_stderr_report(inheritance_stderr, false);
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].lint_name, "cargo::missing_lints_inheritance");
    assert_eq!(parsed.diagnostics[0].raw_path, "unused_dep/Cargo.toml");
    assert_eq!(parsed.diagnostics[0].line, 1);
    assert_eq!(parsed.diagnostics[0].column, 1);

    let legacy_stderr = "\
warning: Cargo.toml: unused manifest key `lints.cargo` (may be supported in a future version)\n\
consider passing `-Zcargo-lints` to enable this feature.\n";
    let legacy_parsed = extract_stderr_report(legacy_stderr, false);
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
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[target.'cfg(unix)'.dependencies]\nopt = { path = \"../other\", optional = true }\n\n[lints]\nworkspace = true\n",
    )
    .expect("app manifest");
    std::fs::write(
        dir.path().join("other/Cargo.toml"),
        "[package]\nname = \"other\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"allow\"\n",
    )
    .expect("other manifest");

    let settings = scan_workspace(dir.path(), &[std::path::PathBuf::from("app"), std::path::PathBuf::from("other")]);
    assert!(settings.has_any_cargo_lints);

    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let clean_stderr = extract_stderr_report("", false);

    let full = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(full.status, CoverageStatus::Checked);
    assert!(full.fail_closed_message.is_none());
    assert!(full.to_structured_status().is_some());

    let partial_target_opt = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &[],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(partial_target_opt.status, CoverageStatus::Partial);
    assert!(partial_target_opt.boundary_reason.contains("optional feature"));

    let partial_post_dash_exclude = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["--all-features".to_owned(), "--exclude".to_owned(), "other".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(partial_post_dash_exclude.status, CoverageStatus::Partial);

    let partial_allow_pkg = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &[],
        check_args: &["-p".to_owned(), "other".to_owned(), "--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(partial_allow_pkg.status, CoverageStatus::Partial);
    assert!(partial_allow_pkg.boundary_reason.contains("set to `allow`"));
}

#[test]
fn assesses_unknown_and_unsupported_cargo_versions() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"warn\"\n",
    )
    .expect("manifest");
    let settings = scan_workspace(dir.path(), &[std::path::PathBuf::new()]);
    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let v196 = ToolchainVersion::parse("cargo 1.96.0-nightly (cbb9bb8bd 2026-03-13)").expect("v196");
    let clean_stderr = extract_stderr_report("", false);

    let full = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &[],
        check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });

    let unknown_version = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: None,
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(unknown_version.status, CoverageStatus::Error);
    assert!(unknown_version.fail_closed_message.is_some());

    let unsupported = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v196),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(unsupported.status, CoverageStatus::UnsupportedCargoVersion);
    assert!(unsupported.fail_closed_message.is_some());
    assert_ne!(full.boundary_identity, unsupported.boundary_identity);
}

#[test]
fn assesses_unselected_vs_selected_truncation_zero_inheritance_and_error_cooccurrence() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\n\n[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n",
    )
    .expect("root manifest");
    std::fs::create_dir_all(dir.path().join("app")).expect("app dir");
    std::fs::write(dir.path().join("app/Cargo.toml"), "[package]\nname = \"app\"\nversion = \"0.1.0\"\n")
        .expect("app without inheritance");

    let zero_inherit_settings = scan_workspace(dir.path(), &[std::path::PathBuf::from("app")]);
    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let clean = extract_stderr_report("", false);
    let zero_inherit = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &zero_inherit_settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["--all-features".to_owned()],
        parsed_stderr: &clean,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(zero_inherit.status, CoverageStatus::Partial);
    assert!(zero_inherit.boundary_reason.contains("inherits `[workspace.lints.cargo]`"));

    let unselected_dir = tempfile::tempdir().expect("unselected");
    std::fs::write(unselected_dir.path().join("Cargo.toml"), "[package]\nname = \"plain\"\nversion = \"0.1.0\"\n")
        .expect("plain manifest");
    let unselected_settings = scan_workspace(unselected_dir.path(), &[]);
    let truncated_stderr = extract_stderr_report_with_flags("big output", super::CaptureIntegrity {
        is_truncated: true,
        has_invalid_utf8: true,
    });
    let unselected = assess_coverage(&AssessmentInput {
        workspace_root: unselected_dir.path(),
        toolchain_version: None,
        settings: &unselected_settings,
        scope_args: &[],
        check_args: &[],
        parsed_stderr: &truncated_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(unselected.status, CoverageStatus::Unselected);
    assert!(unselected.fail_closed_message.is_none());
    assert!(unselected.to_structured_status().is_none());

    let selected_truncated = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &zero_inherit_settings,
        scope_args: &[],
        check_args: &[],
        parsed_stderr: &truncated_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(selected_truncated.status, CoverageStatus::Error);
    assert!(selected_truncated.fail_closed_message.is_some());

    let octet_deny_stderr =
        extract_stderr_report("error: could not compile `app` (lib) due to 1 previous error\n", false);
    let octet_deny = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &zero_inherit_settings,
        scope_args: &[],
        check_args: &[],
        parsed_stderr: &octet_deny_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: true,
        is_command_success: false,
        diagnostic_path_error: None,
    });
    assert!(octet_deny.fail_closed_message.is_none());
}

#[test]
fn package_selection_handles_compact_and_equals_flags_in_check_args() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"other\"]\n\n[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n",
    )
    .expect("root manifest");
    std::fs::create_dir_all(dir.path().join("app")).expect("app dir");
    std::fs::create_dir_all(dir.path().join("other")).expect("other dir");
    for name in ["app", "other"] {
        std::fs::write(
            dir.path().join(name).join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = true\n"),
        )
        .expect("member manifest");
    }

    let settings = scan_workspace(dir.path(), &[std::path::PathBuf::from("app"), std::path::PathBuf::from("other")]);
    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let clean_stderr = extract_stderr_report("", false);

    let compact_partial = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &[],
        check_args: &["-papp".to_owned(), "--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(compact_partial.status, CoverageStatus::Partial);
    assert!(compact_partial.boundary_reason.contains("package selection excludes unselected workspace members"));
    assert!(compact_partial.exclusions.iter().any(|item| item == "unselected workspace packages excluded"));

    let compact_with_workspace = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &["-papp".to_owned(), "--all-features".to_owned()],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(compact_with_workspace.status, CoverageStatus::Checked);

    for (scope_args, check_args) in [
        (vec![], vec!["-p=app".to_owned(), "--all-features".to_owned()]),
        (vec![], vec!["--package=app".to_owned(), "--all-features".to_owned()]),
        (vec!["--workspace".to_owned()], vec!["--exclude=other".to_owned(), "--all-features".to_owned()]),
    ] {
        let partial = assess_coverage(&AssessmentInput {
            workspace_root: dir.path(),
            toolchain_version: Some(&v1100),
            settings: &settings,
            scope_args: &scope_args,
            check_args: &check_args,
            parsed_stderr: &clean_stderr,
            has_rustc_compile_errors: false,
            has_octet_error_findings: false,
            is_command_success: true,
            diagnostic_path_error: None,
        });
        assert_eq!(partial.status, CoverageStatus::Partial);
        assert!(partial.exclusions.iter().any(|item| item == "unselected workspace packages excluded"));
    }
}

#[test]
fn scan_workspace_scopes_to_member_dirs_including_non_virtual_root() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"root_pkg\"\nversion = \"0.1.0\"\n\n[workspace]\nmembers = [\"app\"]\nexclude = [\"dep\"]\n\n[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n\n[lints]\nworkspace = true\n",
    )
    .expect("root manifest");
    std::fs::create_dir_all(dir.path().join("app")).expect("app dir");
    std::fs::write(
        dir.path().join("app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = true\n",
    )
    .expect("app manifest");
    std::fs::create_dir_all(dir.path().join("dep")).expect("dep dir");
    std::fs::write(
        dir.path().join("dep/Cargo.toml"),
        "[package]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[dependencies]\nopt_leaf = { path = \"../opt_leaf\", optional = true }\n\n[lints.cargo]\nunused_dependencies = \"allow\"\n",
    )
    .expect("dep manifest");

    let settings = scan_workspace(dir.path(), &[std::path::PathBuf::new(), std::path::PathBuf::from("app")]);
    assert_eq!(settings.packages.len(), 2);
    assert_eq!(settings.packages[0].package_name, "root_pkg");
    assert_eq!(settings.packages[0].manifest_path, "Cargo.toml");
    assert_eq!(settings.packages[1].package_name, "app");
    assert_eq!(settings.packages[1].manifest_path, "app/Cargo.toml");

    let v1100 = ToolchainVersion::parse("cargo 1.100.0 (3acc79c0c 2026-11-05)").expect("v1100");
    let clean_stderr = extract_stderr_report("", false);
    let checked = assess_coverage(&AssessmentInput {
        workspace_root: dir.path(),
        toolchain_version: Some(&v1100),
        settings: &settings,
        scope_args: &["--workspace".to_owned()],
        check_args: &[],
        parsed_stderr: &clean_stderr,
        has_rustc_compile_errors: false,
        has_octet_error_findings: false,
        is_command_success: true,
        diagnostic_path_error: None,
    });
    assert_eq!(checked.status, CoverageStatus::Checked);

    let root_only = scan_workspace(dir.path(), &[std::path::PathBuf::new()]);
    assert_eq!(root_only.packages.len(), 1);
    assert_eq!(root_only.packages[0].package_name, "root_pkg");
    for (scope_args, check_args) in [
        (vec!["--workspace".to_owned()], vec![]),
        (vec![], vec!["-p".to_owned(), "root_pkg".to_owned()]),
    ] {
        let assessment = assess_coverage(&AssessmentInput {
            workspace_root: dir.path(),
            toolchain_version: Some(&v1100),
            settings: &root_only,
            scope_args: &scope_args,
            check_args: &check_args,
            parsed_stderr: &clean_stderr,
            has_rustc_compile_errors: false,
            has_octet_error_findings: false,
            is_command_success: true,
            diagnostic_path_error: None,
        });
        assert_eq!(assessment.status, CoverageStatus::Checked);
    }
}
