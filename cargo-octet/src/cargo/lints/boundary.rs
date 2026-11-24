const DEFAULT_TARGET_PLATFORM: &str = "host";
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScopeFlags {
    pub(crate) has_package_filter: bool,
    pub(crate) has_all_features: bool,
}

pub(crate) struct AssessmentContext<'a> {
    pub(crate) version_raw: &'a str,
    pub(crate) scope_identity: String,
    pub(crate) scope_summary: String,
    pub(crate) exclusions: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PackageSelection<'a> {
    pub(crate) selected: Vec<&'a str>,
    pub(crate) excluded: Vec<&'a str>,
    pub(crate) has_workspace: bool,
}

enum PendingSelectionArg {
    Package,
    Exclude,
}

impl<'a> PackageSelection<'a> {
    pub(crate) fn parse(scope_args: &'a [String], check_args: &'a [String]) -> Self {
        let capacity = scope_args.len().saturating_add(check_args.len());
        let mut selected = Vec::with_capacity(capacity);
        let mut excluded = Vec::with_capacity(capacity);
        let mut has_workspace = false;
        let mut pending = None;
        for arg in scope_args.iter().chain(check_args.iter()) {
            match pending.take() {
                Some(PendingSelectionArg::Package) => {
                    selected.push(strip_package_version(arg));
                    continue;
                }
                Some(PendingSelectionArg::Exclude) => {
                    excluded.push(strip_package_version(arg));
                    continue;
                }
                None => {}
            }
            if arg == "--workspace" || arg == "--all" {
                has_workspace = true;
            } else if arg == "-p" || arg == "--package" {
                pending = Some(PendingSelectionArg::Package);
            } else if let Some(spec) = arg.strip_prefix("-p=").or_else(|| arg.strip_prefix("--package=")) {
                selected.push(strip_package_version(spec));
            } else if arg == "--exclude" {
                pending = Some(PendingSelectionArg::Exclude);
            } else if let Some(spec) = arg.strip_prefix("--exclude=") {
                excluded.push(strip_package_version(spec));
            }
        }
        Self {
            selected,
            excluded,
            has_workspace,
        }
    }

    pub(crate) fn active_packages<'p>(
        &self,
        packages: &'p [super::policy::PackageSettings],
    ) -> Vec<&'p super::policy::PackageSettings> {
        let is_selected_subset = !self.has_workspace && !self.selected.is_empty();
        packages
            .iter()
            .filter(|pkg| {
                let name = pkg.package_name.as_str();
                let is_in_selection = !is_selected_subset || self.selected.contains(&name);
                let is_excluded = self.excluded.contains(&name);
                is_in_selection && !is_excluded
            })
            .collect()
    }

    pub(crate) fn is_narrowed(&self, packages: &[super::policy::PackageSettings]) -> bool {
        packages.len() > 1 && self.active_packages(packages).len() < packages.len()
    }
}

fn strip_package_version(spec: &str) -> &str {
    spec.split('@').next().unwrap_or(spec)
}

pub(crate) fn find_target_narrowing_arg<'a>(scope_args: &'a [String], check_args: &'a [String]) -> Option<&'a str> {
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

pub(crate) fn has_flag(scope_args: &[String], check_args: &[String], flag: &str) -> bool {
    scope_args.iter().chain(check_args.iter()).any(|arg| arg == flag)
}

pub(crate) fn format_scope_summary(scope_args: &[String], check_args: &[String]) -> String {
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

pub(crate) fn collect_exclusions(
    active_packages: &[&super::policy::PackageSettings],
    target_filter: Option<&str>,
    flags: ScopeFlags,
) -> Vec<String> {
    let mut exclusions = vec!["[dev-dependencies] excluded by cargo::unused_dependencies".to_owned()];
    if let Some(flag) = target_filter {
        exclusions.push(format!("target narrowing (`{flag}`) disables `cargo::unused_dependencies`"));
    }
    if flags.has_package_filter {
        exclusions.push("unselected workspace packages excluded".to_owned());
    }
    if active_packages.iter().any(|pkg| pkg.has_optional_deps) && !flags.has_all_features {
        exclusions.push("inactive optional feature dependencies excluded".to_owned());
    }
    if active_packages.iter().any(|pkg| pkg.has_target_deps) {
        exclusions.push("non-matching target-cfg dependencies excluded".to_owned());
    }
    exclusions
}

pub(crate) fn compute_scope_identity(input: &super::policy::AssessmentInput<'_>, version_raw: &str) -> String {
    let lock_hash = std::fs::read(input.workspace_root.join("Cargo.lock"))
        .ok()
        .map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()));
    let target_filter = target_platform_arg(input.scope_args, input.check_args);
    let payload = serde_json::json!({
        "cargo_version": version_raw,
        "policy": input.settings,
        "cargo_lock": lock_hash,
        "scope_args": input.scope_args,
        "cargo_check_args": input.check_args,
        "target_platform": target_filter.unwrap_or(DEFAULT_TARGET_PLATFORM),
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

pub(crate) fn build_assessment(
    status: super::CoverageStatus,
    context: AssessmentContext<'_>,
    boundary_reason: String,
    raw_excerpt: String,
    fail_closed_message: Option<String>,
) -> super::LintAssessment {
    super::LintAssessment {
        status,
        cargo_version_raw: context.version_raw.to_owned(),
        boundary_identity: context.scope_identity,
        scope_summary: context.scope_summary,
        exclusions: context.exclusions,
        boundary_reason,
        raw_excerpt,
        fail_closed_message,
    }
}

pub(crate) fn classify_supported_coverage(
    input: &super::policy::AssessmentInput<'_>,
    selection: &PackageSelection<'_>,
    active_packages: &[&super::policy::PackageSettings],
    flags: ScopeFlags,
    context: AssessmentContext<'_>,
) -> super::LintAssessment {
    let target_filter = find_target_narrowing_arg(input.scope_args, input.check_args);
    let has_package_filter = selection.is_narrowed(&input.settings.packages);
    let has_optional = active_packages.iter().any(|pkg| pkg.has_optional_deps);
    let is_all_allow = are_all_unused_deps_allowed(active_packages);
    let has_uninherited_workspace_lints =
        input.settings.has_workspace_cargo_lints && !active_packages.iter().any(|pkg| pkg.has_cargo_lints);

    let mut partial_reasons = Vec::new();
    if let Some(flag) = target_filter {
        partial_reasons.push(format!("target filter `{flag}` skips `cargo::unused_dependencies`"));
    }
    if has_package_filter {
        partial_reasons.push("package selection excludes unselected workspace members".to_owned());
    }
    if has_uninherited_workspace_lints {
        partial_reasons.push("no selected workspace package inherits `[workspace.lints.cargo]`".to_owned());
    }
    if has_optional && !flags.has_all_features {
        partial_reasons.push("optional feature dependencies inactive without `--all-features`".to_owned());
    }
    if is_all_allow {
        partial_reasons.push("`cargo::unused_dependencies` is set to `allow`".to_owned());
    }
    if partial_reasons.is_empty() {
        build_assessment(
            super::CoverageStatus::Checked,
            context,
            "Cargo `>= 1.100` evaluated active `[lints.cargo]` policy across selected workspace packages".to_owned(),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        )
    } else {
        build_assessment(
            super::CoverageStatus::Partial,
            context,
            partial_reasons.join("; "),
            input.parsed_stderr.raw_excerpt.clone(),
            None,
        )
    }
}

fn are_all_unused_deps_allowed(packages: &[&super::policy::PackageSettings]) -> bool {
    let configured = packages.iter().filter_map(|pkg| pkg.unused_dependencies_level.as_deref()).collect::<Vec<_>>();
    !configured.is_empty() && configured.iter().all(|level| *level == "allow")
}

pub(crate) fn has_optional_dependencies(table: &toml::Table) -> bool {
    if table_has_optional_deps(table.get("dependencies")) {
        return true;
    }
    table.get("target").and_then(toml::Value::as_table).is_some_and(|targets| {
        targets
            .values()
            .filter_map(toml::Value::as_table)
            .any(|target_table| table_has_optional_deps(target_table.get("dependencies")))
    })
}

fn table_has_optional_deps(deps_value: Option<&toml::Value>) -> bool {
    deps_value.and_then(toml::Value::as_table).is_some_and(|deps| {
        deps.values()
            .filter_map(toml::Value::as_table)
            .any(|spec| spec.get("optional").and_then(toml::Value::as_bool) == Some(true))
    })
}
