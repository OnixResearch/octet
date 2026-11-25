const UNKNOWN_VERSION: &str = "unknown";
const UNUSED_DEPENDENCIES_KEY: &str = "unused_dependencies";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PackageSettings {
    pub(crate) manifest_path: String,
    pub(crate) package_name: String,
    pub(crate) unused_dependencies_level: Option<String>,
    pub(crate) has_cargo_lints: bool,
    pub(crate) has_optional_deps: bool,
    pub(crate) has_target_deps: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct WorkspaceSettings {
    pub(crate) workspace_unused_dependencies: Option<String>,
    pub(crate) has_workspace_cargo_lints: bool,
    pub(crate) has_any_cargo_lints: bool,
    pub(crate) packages: Vec<PackageSettings>,
}

pub(crate) struct AssessmentInput<'a> {
    pub(crate) workspace_root: &'a std::path::Path,
    pub(crate) toolchain_version: Option<&'a super::ToolchainVersion>,
    pub(crate) settings: &'a WorkspaceSettings,
    pub(crate) scope_args: &'a [String],
    pub(crate) check_args: &'a [String],
    pub(crate) parsed_stderr: &'a super::StderrReport,
    pub(crate) has_rustc_compile_errors: bool,
    pub(crate) has_octet_error_findings: bool,
    pub(crate) is_command_success: bool,
    pub(crate) diagnostic_path_error: Option<&'a str>,
}

pub(crate) fn scan_workspace(
    workspace_root: &std::path::Path,
    package_dirs: &std::collections::BTreeMap<String, std::path::PathBuf>,
) -> WorkspaceSettings {
    let root_table = read_toml_table(&workspace_root.join("Cargo.toml"));
    let workspace_lint_table = root_table
        .as_ref()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_table)
        .and_then(|workspace| manifest_lint_table(workspace.get("lints")));
    let workspace_unused = workspace_lint_table.and_then(|table| extract_lint_level(table, UNUSED_DEPENDENCIES_KEY));
    let has_workspace_cargo_lints = workspace_lint_table.is_some_and(|table| !table.is_empty());

    let mut rel_dirs: Vec<std::path::PathBuf> = package_dirs.values().cloned().collect();
    rel_dirs.sort();
    rel_dirs.dedup();
    if rel_dirs.is_empty() && workspace_root.join("Cargo.toml").is_file() {
        rel_dirs.push(std::path::PathBuf::new());
    }
    let packages = rel_dirs
        .iter()
        .filter_map(|rel_dir| {
            read_package_settings(workspace_root, rel_dir, workspace_unused.as_deref(), has_workspace_cargo_lints)
        })
        .collect::<Vec<_>>();
    let has_any_cargo_lints = has_workspace_cargo_lints || packages.iter().any(|pkg| pkg.has_cargo_lints);

    WorkspaceSettings {
        workspace_unused_dependencies: workspace_unused,
        has_workspace_cargo_lints,
        has_any_cargo_lints,
        packages,
    }
}

pub(crate) fn assess_coverage(input: &AssessmentInput<'_>) -> super::LintAssessment {
    let version_raw = input.toolchain_version.map_or_else(|| UNKNOWN_VERSION.to_owned(), |ver| ver.raw.clone());
    let scope_identity = super::boundary::compute_scope_identity(input, &version_raw);
    let scope_summary = super::boundary::format_scope_summary(input.scope_args, input.check_args);

    let has_unsupported_signal =
        input.parsed_stderr.has_unused_manifest_key_lints_cargo || input.parsed_stderr.has_unknown_cargo_lint;
    let has_context = input.settings.has_any_cargo_lints
        || has_unsupported_signal
        || !input.parsed_stderr.diagnostics.is_empty()
        || input.diagnostic_path_error.is_some();

    if !has_context {
        let fail_closed = (!input.is_command_success && input.parsed_stderr.errors.has_unrecognized_error)
            .then(|| "Cargo exited non-zero with unrecognized error output".to_owned());
        return super::boundary::build_assessment(
            super::CoverageStatus::Unselected,
            super::boundary::AssessmentContext {
                version_raw: &version_raw,
                scope_identity,
                scope_summary,
                exclusions: Vec::new(),
            },
            "no `[lints.cargo]` or `[workspace.lints.cargo]` policy configured".to_owned(),
            String::new(),
            fail_closed,
        );
    }

    let selection = super::boundary::PackageSelection::parse(input.scope_args, input.check_args);
    let active_packages = selection.active_packages(&input.settings.packages);
    let target_filter = super::boundary::find_target_narrowing_arg(input.scope_args, input.check_args);
    let flags = super::boundary::ScopeFlags {
        has_package_filter: selection.is_narrowed(&input.settings.packages),
        has_all_features: super::boundary::has_flag(input.scope_args, input.check_args, "--all-features"),
    };
    let exclusions = super::boundary::collect_exclusions(&active_packages, target_filter, flags);
    let context = super::boundary::AssessmentContext {
        version_raw: &version_raw,
        scope_identity,
        scope_summary,
        exclusions,
    };

    if let Some(blocking) = assess_capture_and_version_blocking(input, &context) {
        return blocking;
    }
    if let Some(blocking) = assess_process_blocking(input, &context) {
        return blocking;
    }

    super::boundary::classify_supported_coverage(input, &selection, &active_packages, flags, context)
}

fn clone_context<'a>(context: &super::boundary::AssessmentContext<'a>) -> super::boundary::AssessmentContext<'a> {
    super::boundary::AssessmentContext {
        version_raw: context.version_raw,
        scope_identity: context.scope_identity.clone(),
        scope_summary: context.scope_summary.clone(),
        exclusions: context.exclusions.clone(),
    }
}

fn assess_capture_and_version_blocking(
    input: &AssessmentInput<'_>,
    context: &super::boundary::AssessmentContext<'_>,
) -> Option<super::LintAssessment> {
    if let Some(path_err) = input.diagnostic_path_error {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            path_err.to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some(path_err.to_owned()),
        ));
    }
    if input.parsed_stderr.capture.is_truncated {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "Cargo stderr exceeded capture bound; native-lint output may be incomplete".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("Cargo stderr exceeded capture bound while scanning native Cargo lints".to_owned()),
        ));
    }
    if input.parsed_stderr.capture.has_invalid_utf8 {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "Cargo stderr contained invalid UTF-8; native-lint output could not be parsed cleanly".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("Cargo stderr contained invalid UTF-8 while scanning native Cargo lints".to_owned()),
        ));
    }
    let has_below_1_100 = input.toolchain_version.is_some_and(|ver| !ver.is_1_100_or_newer());
    let has_unsupported_signal =
        input.parsed_stderr.has_unused_manifest_key_lints_cargo || input.parsed_stderr.has_unknown_cargo_lint;
    if (has_below_1_100 && input.settings.has_any_cargo_lints) || has_unsupported_signal {
        let reason = format!(
            "Cargo `{}` does not support stabilized `[lints.cargo]` (`cargo >= 1.100.0` required)",
            context.version_raw
        );
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::UnsupportedCargoVersion,
            clone_context(context),
            reason.clone(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some(reason),
        ));
    }
    None
}

fn assess_process_blocking(
    input: &AssessmentInput<'_>,
    context: &super::boundary::AssessmentContext<'_>,
) -> Option<super::LintAssessment> {
    if !input.is_command_success && input.parsed_stderr.errors.has_unrecognized_error {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "Cargo exited non-zero with unrecognized error output".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("Cargo exited non-zero with unrecognized error output".to_owned()),
        ));
    }
    if input.has_rustc_compile_errors
        || (input.parsed_stderr.errors.has_compile_error && !input.has_octet_error_findings)
    {
        let fail_closed = (!input.is_command_success && !input.has_rustc_compile_errors)
            .then(|| "Cargo exited with compile errors while scanning native Cargo lints".to_owned());
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "rustc compile errors invalidate Cargo unused_dependencies analysis".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            fail_closed,
        ));
    }
    if input.toolchain_version.is_none() {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "unable to verify Cargo version (`cargo >= 1.100.0` required for `[lints.cargo]`)".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("unable to verify Cargo version while `[lints.cargo]` is configured".to_owned()),
        ));
    }
    if !input.is_command_success
        && !input.parsed_stderr.errors.has_manifest_finalize_error
        && !input.has_octet_error_findings
    {
        return Some(super::boundary::build_assessment(
            super::CoverageStatus::Error,
            clone_context(context),
            "Cargo exited non-zero before completing native Cargo lint analysis".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("Cargo exited non-zero before completing native Cargo lint analysis".to_owned()),
        ));
    }
    None
}

fn read_package_settings(
    workspace_root: &std::path::Path,
    rel_dir: &std::path::Path,
    workspace_unused: Option<&str>,
    has_workspace_cargo_lints: bool,
) -> Option<PackageSettings> {
    let manifest_file = workspace_root.join(rel_dir).join("Cargo.toml");
    let table = read_toml_table(&manifest_file)?;
    let package_name = table
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|pkg| pkg.get("name"))
        .and_then(toml::Value::as_str)?
        .to_owned();
    let has_workspace_inheritance = super::has_workspace_lint_inheritance(&table);
    let lint_table = manifest_lint_table(table.get("lints"));
    let pkg_unused = lint_table.and_then(|tbl| extract_lint_level(tbl, UNUSED_DEPENDENCIES_KEY));
    let unused_dependencies_level =
        pkg_unused.or_else(|| has_workspace_inheritance.then(|| workspace_unused.map(ToOwned::to_owned)).flatten());
    let has_cargo_lints =
        lint_table.is_some_and(|tbl| !tbl.is_empty()) || (has_workspace_inheritance && has_workspace_cargo_lints);
    let has_optional_deps = super::boundary::has_optional_dependencies(&table);
    let has_target_deps = table.get("target").and_then(toml::Value::as_table).is_some_and(|tbl| !tbl.is_empty());
    let manifest_path = if rel_dir.as_os_str().is_empty() {
        "Cargo.toml".to_owned()
    } else {
        format!("{}/Cargo.toml", crate::determinism::normalize_path(rel_dir))
    };

    Some(PackageSettings {
        manifest_path,
        package_name,
        unused_dependencies_level,
        has_cargo_lints,
        has_optional_deps,
        has_target_deps,
    })
}

fn read_toml_table(path: &std::path::Path) -> Option<toml::Table> {
    std::fs::read_to_string(path).ok()?.parse::<toml::Table>().ok()
}

pub(crate) fn manifest_lint_table(lints_value: Option<&toml::Value>) -> Option<&toml::Table> {
    lints_value?.as_table()?.get("cargo")?.as_table()
}

fn extract_lint_level(table: &toml::Table, key: &str) -> Option<String> {
    match table.get(key)? {
        toml::Value::String(level) => Some(level.clone()),
        toml::Value::Table(sub) => sub.get("level").and_then(toml::Value::as_str).map(ToOwned::to_owned),
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_)
        | toml::Value::Array(_) => None,
    }
}
