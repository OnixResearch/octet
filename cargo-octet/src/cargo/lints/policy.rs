const UNKNOWN_CARGO_VERSION: &str = "unknown";
const CARGO_LINT_UNUSED_DEPENDENCIES: &str = "unused_dependencies";
const DEFAULT_CARGO_TARGET: &str = "host";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PackageLintPolicy {
    pub(crate) manifest_path: String,
    pub(crate) package_name: String,
    pub(crate) unused_dependencies_level: Option<String>,
    pub(crate) has_cargo_lints: bool,
    pub(crate) has_optional_deps: bool,
    pub(crate) has_target_deps: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CargoLintPolicy {
    pub(crate) workspace_unused_dependencies: Option<String>,
    pub(crate) has_workspace_cargo_lints: bool,
    pub(crate) has_any_cargo_lints: bool,
    pub(crate) packages: Vec<PackageLintPolicy>,
}

pub(crate) struct AssessmentInput<'a> {
    pub(crate) workspace_root: &'a std::path::Path,
    pub(crate) cargo_version: Option<&'a super::CargoVersion>,
    pub(crate) policy: &'a CargoLintPolicy,
    pub(crate) scope_args: &'a [String],
    pub(crate) cargo_check_args: &'a [String],
    pub(crate) parsed_stderr: &'a super::parse::ParsedCargoStderr,
    pub(crate) has_rustc_compile_errors: bool,
    pub(crate) is_cargo_success: bool,
}

pub(crate) fn scan_workspace_policy(
    workspace_root: &std::path::Path,
    package_dirs: &std::collections::BTreeMap<String, std::path::PathBuf>,
) -> CargoLintPolicy {
    let root_table = read_toml_table(&workspace_root.join("Cargo.toml"));
    let workspace_cargo_table = root_table
        .as_ref()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_table)
        .and_then(|workspace| cargo_lints_table(workspace.get("lints")));
    let workspace_unused =
        workspace_cargo_table.and_then(|table| extract_lint_level(table, CARGO_LINT_UNUSED_DEPENDENCIES));
    let has_workspace_cargo_lints = workspace_cargo_table.is_some_and(|table| !table.is_empty());

    let mut rel_dirs: Vec<std::path::PathBuf> = package_dirs.values().cloned().collect();
    rel_dirs.sort();
    rel_dirs.dedup();
    if rel_dirs.is_empty() && workspace_root.join("Cargo.toml").is_file() {
        rel_dirs.push(std::path::PathBuf::new());
    }
    let packages = rel_dirs
        .iter()
        .filter_map(|rel_dir| parse_package_policy(workspace_root, rel_dir, workspace_unused.as_deref()))
        .collect::<Vec<_>>();
    let has_any_cargo_lints = has_workspace_cargo_lints || packages.iter().any(|pkg| pkg.has_cargo_lints);

    CargoLintPolicy {
        workspace_unused_dependencies: workspace_unused,
        has_workspace_cargo_lints,
        has_any_cargo_lints,
        packages,
    }
}

pub(crate) fn assess_cargo_lints(input: &AssessmentInput<'_>) -> super::CargoLintAssessment {
    let cargo_version_raw = input.cargo_version.map_or_else(|| UNKNOWN_CARGO_VERSION.to_owned(), |ver| ver.raw.clone());
    let boundary_identity = compute_boundary_identity(input, &cargo_version_raw);
    let target_filter = find_target_narrowing_arg(input.scope_args, input.cargo_check_args);
    let package_filter = has_package_narrowing(input.scope_args, input.policy);
    let has_all_features = has_flag(input.scope_args, input.cargo_check_args, "--all-features");
    let scope_summary = format_scope_summary(input.scope_args, input.cargo_check_args);
    let exclusions = collect_exclusions(input.policy, target_filter, package_filter, has_all_features);

    if input.parsed_stderr.is_truncated {
        return build_assessment(
            super::CargoLintCoverageStatus::Error,
            &cargo_version_raw,
            boundary_identity,
            &scope_summary,
            exclusions,
            "Cargo stderr exceeded capture bound; native-lint output may be incomplete".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some("Cargo stderr exceeded capture bound while scanning native Cargo lints".to_owned()),
        );
    }
    if input.has_rustc_compile_errors || input.parsed_stderr.error_kind == super::parse::StderrErrorKind::CompileError {
        return build_assessment(
            super::CargoLintCoverageStatus::Error,
            &cargo_version_raw,
            boundary_identity,
            &scope_summary,
            exclusions,
            "rustc compile errors invalidate Cargo unused_dependencies analysis".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        );
    }
    let has_below_1_100 = input.cargo_version.is_some_and(|ver| !ver.is_cargo_1_100_or_newer());
    let has_unsupported_signal =
        input.parsed_stderr.has_unused_manifest_key_lints_cargo || input.parsed_stderr.has_unknown_cargo_lint;
    if (has_below_1_100 && input.policy.has_any_cargo_lints) || has_unsupported_signal {
        let reason = format!(
            "Cargo `{cargo_version_raw}` does not support stabilized `[lints.cargo]` (`cargo >= 1.100.0` required)"
        );
        return build_assessment(
            super::CargoLintCoverageStatus::UnsupportedCargoVersion,
            &cargo_version_raw,
            boundary_identity,
            &scope_summary,
            exclusions,
            reason.clone(),
            input.parsed_stderr.raw_excerpt.clone(),
            Some(reason),
        );
    }
    if !input.is_cargo_success && input.parsed_stderr.error_kind == super::parse::StderrErrorKind::UnrecognizedError {
        return build_assessment(
            super::CargoLintCoverageStatus::Error,
            &cargo_version_raw,
            boundary_identity,
            &scope_summary,
            exclusions,
            "Cargo exited non-zero with unrecognized error output".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        );
    }
    if !input.policy.has_any_cargo_lints && input.parsed_stderr.diagnostics.is_empty() {
        return build_assessment(
            super::CargoLintCoverageStatus::Unselected,
            &cargo_version_raw,
            boundary_identity,
            &scope_summary,
            Vec::new(),
            "no `[lints.cargo]` or `[workspace.lints.cargo]` policy configured".to_owned(),
            String::new(),
            None,
        );
    }
    classify_supported_coverage(input, &cargo_version_raw, boundary_identity, &scope_summary, exclusions)
}

#[allow(clippy::too_many_arguments)]
fn build_assessment(
    status: super::CargoLintCoverageStatus,
    cargo_version_raw: &str,
    boundary_identity: String,
    scope_summary: &str,
    exclusions: Vec<String>,
    boundary_reason: String,
    raw_excerpt: String,
    fail_closed_message: Option<String>,
) -> super::CargoLintAssessment {
    super::CargoLintAssessment {
        status,
        cargo_version_raw: cargo_version_raw.to_owned(),
        boundary_identity,
        scope_summary: scope_summary.to_owned(),
        exclusions,
        boundary_reason,
        raw_excerpt,
        fail_closed_message,
    }
}

fn classify_supported_coverage(
    input: &AssessmentInput<'_>,
    cargo_version_raw: &str,
    boundary_identity: String,
    scope_summary: &str,
    exclusions: Vec<String>,
) -> super::CargoLintAssessment {
    let target_filter = find_target_narrowing_arg(input.scope_args, input.cargo_check_args);
    let package_filter = has_package_narrowing(input.scope_args, input.policy);
    let has_all_features = has_flag(input.scope_args, input.cargo_check_args, "--all-features");
    let has_optional = input.policy.packages.iter().any(|pkg| pkg.has_optional_deps);
    let is_all_allow = is_policy_all_allow(input.policy);

    let mut partial_reasons = Vec::new();
    if let Some(flag) = target_filter {
        partial_reasons.push(format!("target filter `{flag}` skips `cargo::unused_dependencies`"));
    }
    if package_filter {
        partial_reasons.push("package selection excludes unselected workspace members".to_owned());
    }
    if has_optional && !has_all_features {
        partial_reasons.push("optional feature dependencies inactive without `--all-features`".to_owned());
    }
    if is_all_allow {
        partial_reasons.push("`cargo::unused_dependencies` is set to `allow`".to_owned());
    }
    if partial_reasons.is_empty() {
        build_assessment(
            super::CargoLintCoverageStatus::Checked,
            cargo_version_raw,
            boundary_identity,
            scope_summary,
            exclusions,
            "Cargo `>= 1.100` evaluated active `[lints.cargo]` policy across selected workspace packages".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        )
    } else {
        build_assessment(
            super::CargoLintCoverageStatus::Partial,
            cargo_version_raw,
            boundary_identity,
            scope_summary,
            exclusions,
            partial_reasons.join("; "),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        )
    }
}

fn parse_package_policy(
    workspace_root: &std::path::Path,
    rel_dir: &std::path::Path,
    workspace_unused: Option<&str>,
) -> Option<PackageLintPolicy> {
    let manifest_file = workspace_root.join(rel_dir).join("Cargo.toml");
    let table = read_toml_table(&manifest_file)?;
    let package_name = table
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|pkg| pkg.get("name"))
        .and_then(toml::Value::as_str)?
        .to_owned();
    let inherits_workspace = table
        .get("lints")
        .and_then(toml::Value::as_table)
        .and_then(|tbl| tbl.get("workspace"))
        .and_then(toml::Value::as_bool)
        == Some(true);
    let cargo_table = cargo_lints_table(table.get("lints"));
    let pkg_unused = cargo_table.and_then(|tbl| extract_lint_level(tbl, CARGO_LINT_UNUSED_DEPENDENCIES));
    let unused_dependencies_level =
        pkg_unused.or_else(|| inherits_workspace.then(|| workspace_unused.map(ToOwned::to_owned)).flatten());
    let has_cargo_lints =
        cargo_table.is_some_and(|tbl| !tbl.is_empty()) || (inherits_workspace && workspace_unused.is_some());
    let has_optional_deps = has_optional_dependencies(&table);
    let has_target_deps = table.get("target").and_then(toml::Value::as_table).is_some_and(|tbl| !tbl.is_empty());
    let manifest_path = if rel_dir.as_os_str().is_empty() {
        "Cargo.toml".to_owned()
    } else {
        format!("{}/Cargo.toml", crate::determinism::normalize_path(rel_dir))
    };

    Some(PackageLintPolicy {
        manifest_path,
        package_name,
        unused_dependencies_level,
        has_cargo_lints,
        has_optional_deps,
        has_target_deps,
    })
}

fn has_optional_dependencies(table: &toml::Table) -> bool {
    table.get("dependencies").and_then(toml::Value::as_table).is_some_and(|deps| {
        deps.values()
            .filter_map(toml::Value::as_table)
            .any(|spec| spec.get("optional").and_then(toml::Value::as_bool) == Some(true))
    })
}

fn read_toml_table(path: &std::path::Path) -> Option<toml::Table> {
    std::fs::read_to_string(path).ok()?.parse::<toml::Table>().ok()
}

pub(crate) fn cargo_lints_table(lints_value: Option<&toml::Value>) -> Option<&toml::Table> {
    lints_value?.as_table()?.get("cargo")?.as_table()
}

fn extract_lint_level(table: &toml::Table, key: &str) -> Option<String> {
    match table.get(key)? {
        toml::Value::String(level) => Some(level.clone()),
        toml::Value::Table(sub) => sub.get("level").and_then(toml::Value::as_str).map(ToOwned::to_owned),
        _ => None,
    }
}

fn is_policy_all_allow(policy: &CargoLintPolicy) -> bool {
    let configured = policy
        .packages
        .iter()
        .filter_map(|pkg| pkg.unused_dependencies_level.as_deref())
        .collect::<Vec<_>>();
    !configured.is_empty() && configured.iter().all(|level| *level == "allow")
}

fn find_target_narrowing_arg<'a>(scope_args: &'a [String], check_args: &'a [String]) -> Option<&'a str> {
    const TARGET_FLAGS: &[&str] = &[
        "--lib",
        "--bin",
        "--bins",
        "--example",
        "--examples",
        "--test",
        "--tests",
        "--bench",
        "--benches",
    ];
    for arg in scope_args.iter().chain(check_args.iter()) {
        if TARGET_FLAGS.contains(&arg.as_str())
            || arg.starts_with("--bin=")
            || arg.starts_with("--example=")
            || arg.starts_with("--test=")
            || arg.starts_with("--bench=")
        {
            return Some(arg.as_str());
        }
    }
    None
}

fn has_package_narrowing(scope_args: &[String], policy: &CargoLintPolicy) -> bool {
    let selects_pkg = scope_args.iter().any(|arg| arg == "-p" || arg == "--package" || arg.starts_with("--package="));
    selects_pkg && policy.packages.len() > 1
}

fn has_flag(scope_args: &[String], check_args: &[String], flag: &str) -> bool {
    scope_args.iter().chain(check_args.iter()).any(|arg| arg == flag)
}

fn format_scope_summary(scope_args: &[String], check_args: &[String]) -> String {
    let scope = if scope_args.is_empty() {
        "default".to_owned()
    } else {
        scope_args.join(" ")
    };
    let check = if check_args.is_empty() {
        "default-targets".to_owned()
    } else {
        check_args.join(" ")
    };
    format!("scope=[{scope}] check=[{check}]")
}

fn collect_exclusions(
    policy: &CargoLintPolicy,
    target_filter: Option<&str>,
    package_filter: bool,
    has_all_features: bool,
) -> Vec<String> {
    let mut exclusions = vec!["[dev-dependencies] excluded by cargo::unused_dependencies".to_owned()];
    if let Some(flag) = target_filter {
        exclusions.push(format!("target narrowing (`{flag}`) disables `cargo::unused_dependencies`"));
    }
    if package_filter {
        exclusions.push("unselected workspace packages excluded".to_owned());
    }
    if policy.packages.iter().any(|pkg| pkg.has_optional_deps) && !has_all_features {
        exclusions.push("inactive optional feature dependencies excluded".to_owned());
    }
    if policy.packages.iter().any(|pkg| pkg.has_target_deps) {
        exclusions.push("non-matching target-cfg dependencies excluded".to_owned());
    }
    exclusions
}

fn compute_boundary_identity(input: &AssessmentInput<'_>, cargo_version_raw: &str) -> String {
    let lock_hash = std::fs::read(input.workspace_root.join("Cargo.lock"))
        .ok()
        .map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()));
    let target_filter = target_platform_arg(input.scope_args, input.cargo_check_args);
    let payload = serde_json::json!({
        "cargo_version": cargo_version_raw,
        "policy": input.policy,
        "cargo_lock": lock_hash,
        "scope_args": input.scope_args,
        "cargo_check_args": input.cargo_check_args,
        "target_platform": target_filter.unwrap_or(DEFAULT_CARGO_TARGET),
    });
    crate::determinism::full_hash(&payload.to_string())
}

fn target_platform_arg<'a>(scope_args: &'a [String], check_args: &'a [String]) -> Option<&'a str> {
    let mut iter = scope_args.iter().chain(check_args.iter());
    while let Some(arg) = iter.next() {
        if arg == "--target" || arg == "--filter-platform" {
            return iter.next().map(String::as_str);
        }
        if let Some(val) = arg.strip_prefix("--target=") {
            return Some(val);
        }
        if let Some(val) = arg.strip_prefix("--filter-platform=") {
            return Some(val);
        }
    }
    None
}

pub(crate) fn canonical_toml_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(text) => serde_json::Value::String(text.clone()),
        toml::Value::Integer(number) => serde_json::Value::from(*number),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map_or_else(|| serde_json::Value::String(number.to_string()), serde_json::Value::Number),
        toml::Value::Boolean(flag) => serde_json::Value::Bool(*flag),
        toml::Value::Datetime(datetime) => serde_json::Value::String(datetime.to_string()),
        toml::Value::Array(items) => serde_json::Value::Array(items.iter().map(canonical_toml_json).collect()),
        toml::Value::Table(table) => {
            let mut keys = table.keys().collect::<Vec<_>>();
            keys.sort();
            serde_json::Value::Object(
                keys.into_iter().map(|key| (key.clone(), canonical_toml_json(&table[key.as_str()]))).collect(),
            )
        }
    }
}
