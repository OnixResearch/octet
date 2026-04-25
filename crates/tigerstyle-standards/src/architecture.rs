//! Architecture and FCIS boundary checks.

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::source::SourceFile;

const ARCHITECTURE_FAMILY: &str = "architecture";
const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
const FORBIDDEN_RUNTIME_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const STD_SHELL_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const CORE_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-core/src/lib.rs";
const CORE_MANIFEST_PATH_SUFFIX: &str = "crates/tigerstyle-core/Cargo.toml";
const STD_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-std/src/lib.rs";
const ROOT_LIB_PATH_SUFFIX: &str = "src/lib.rs";
const ROOT_MAIN_PATH_SUFFIX: &str = "src/main.rs";
const NO_STD_ATTRIBUTE: &str = "#![no_std]";
const CORE_DEPENDENCIES_SECTION: &str = "[dependencies]";
const DEV_DEPENDENCIES_SECTION: &str = "[dev-dependencies]";
const CORE_FORBIDDEN_DEPENDENCIES: &[&str] = &["tokio", "tracing", "tracing-subscriber", "clap", "parking_lot", "url"];
const ROOT_LIB_MAX_LINES: usize = 40;
const ROOT_MAIN_MAX_LINES: usize = 80;
const ROOT_PUBLIC_API_ITEM_BUDGET: usize = 8;
const FIRST_PUBLIC_API_OVER_BUDGET_INDEX: usize = ROOT_PUBLIC_API_ITEM_BUDGET;
const FIRST_LINE: u32 = 1;
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const PUBLIC_VISIBILITY_PREFIXES: &[&str] = &["pub ", "pub("];

/// Runs architecture policy checks over source files.
#[must_use]
pub fn check_architecture(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    if !selection.includes(ARCHITECTURE_FAMILY) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    check_core_no_std(registry, sources, &mut diagnostics);
    check_core_forbidden_dependencies(registry, sources, &mut diagnostics);
    check_std_shell_owns_std(registry, sources, &mut diagnostics);
    check_root_thinness(registry, sources, &mut diagnostics);
    check_public_api_budget(registry, sources, &mut diagnostics);
    diagnostics
}

fn check_core_no_std(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    let Some(source) = source_by_suffix(sources, CORE_LIB_PATH_SUFFIX) else {
        return;
    };
    if source.text.contains(NO_STD_ATTRIBUTE) {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        source,
        Some(FIRST_LINE),
        "tigerstyle-core is missing #![no_std]",
        "add #![no_std] to the core crate root or move std-dependent code to tigerstyle-std",
    ));
}

fn check_core_forbidden_dependencies(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FORBIDDEN_RUNTIME_POLICY_ID) else {
        return;
    };
    let Some(source) = source_by_suffix(sources, CORE_MANIFEST_PATH_SUFFIX) else {
        return;
    };
    let dependency_section = dependency_section(&source.text);
    for forbidden_dependency in CORE_FORBIDDEN_DEPENDENCIES {
        if dependency_section_contains(dependency_section, forbidden_dependency) {
            diagnostics.push(policy_diagnostic(
                policy,
                source,
                line_for_text(&source.text, forbidden_dependency),
                &format!("tigerstyle-core depends on runtime crate `{forbidden_dependency}`"),
                "move runtime, async, logging, CLI, or I/O dependencies to tigerstyle-std",
            ));
        }
    }
}

fn check_std_shell_owns_std(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(STD_SHELL_POLICY_ID) else {
        return;
    };
    let Some(source) = source_by_suffix(sources, STD_LIB_PATH_SUFFIX) else {
        return;
    };
    if !source.text.contains(NO_STD_ATTRIBUTE) {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        source,
        Some(FIRST_LINE),
        "tigerstyle-std must not declare #![no_std]",
        "keep std/runtime shell code in tigerstyle-std and no_std code in tigerstyle-core",
    ));
}

fn check_root_thinness(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(ROOT_THINNESS_POLICY_ID) else {
        return;
    };
    check_root_file_thinness(policy, sources, ROOT_LIB_PATH_SUFFIX, ROOT_LIB_MAX_LINES, diagnostics);
    check_root_file_thinness(policy, sources, ROOT_MAIN_PATH_SUFFIX, ROOT_MAIN_MAX_LINES, diagnostics);
}

fn check_root_file_thinness(
    policy: &PolicyEntry,
    sources: &[SourceFile],
    path_suffix: &str,
    max_lines: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(source) = root_source_by_suffix(sources, path_suffix) else {
        return;
    };
    let line_count = source.text.lines().count();
    if line_count <= max_lines {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        source,
        None,
        &format!("root file has {line_count} lines, exceeding thin-shell limit {max_lines}"),
        "move non-trivial logic from the root crate into tigerstyle-core or tigerstyle-std",
    ));
}

fn check_public_api_budget(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(PUBLIC_API_BUDGET_POLICY_ID) else {
        return;
    };
    let Some(source) = root_source_by_suffix(sources, ROOT_LIB_PATH_SUFFIX) else {
        return;
    };
    let public_item_lines = public_api_item_lines(&source.text);
    if public_item_lines.len() <= ROOT_PUBLIC_API_ITEM_BUDGET {
        return;
    }
    assert!(public_item_lines.len() > FIRST_PUBLIC_API_OVER_BUDGET_INDEX);
    diagnostics.push(policy_diagnostic(
        policy,
        source,
        Some(public_item_lines[FIRST_PUBLIC_API_OVER_BUDGET_INDEX]),
        &format!(
            "root public API has {} visible items, exceeding advisory budget {}",
            public_item_lines.len(),
            ROOT_PUBLIC_API_ITEM_BUDGET
        ),
        "move public surface into domain crates or document the expansion in API review",
    ));
}

fn public_api_item_lines(text: &str) -> Vec<u32> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| is_public_api_item_line(line).then_some(line_number_from_index(line_index)))
        .collect()
}

fn is_public_api_item_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    PUBLIC_VISIBILITY_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn source_by_suffix<'a>(sources: &'a [SourceFile], suffix: &str) -> Option<&'a SourceFile> {
    sources.iter().find(|source| source.path.ends_with(suffix))
}

fn root_source_by_suffix<'a>(sources: &'a [SourceFile], suffix: &str) -> Option<&'a SourceFile> {
    sources.iter().find(|source| source.path.ends_with(suffix) && !source.path.contains("/crates/"))
}

fn dependency_section(manifest: &str) -> &str {
    let Some((_, after_dependencies)) = manifest.split_once(CORE_DEPENDENCIES_SECTION) else {
        return "";
    };
    match after_dependencies.split_once(DEV_DEPENDENCIES_SECTION) {
        Some((dependencies, _)) => dependencies,
        None => after_dependencies,
    }
}

fn dependency_section_contains(dependency_section: &str, dependency_name: &str) -> bool {
    dependency_section
        .lines()
        .map(str::trim_start)
        .any(|line| line_has_dependency_name(line, dependency_name))
}

fn line_has_dependency_name(line: &str, dependency_name: &str) -> bool {
    if !line.starts_with(dependency_name) {
        return false;
    }
    matches!(line[dependency_name.len()..].chars().next(), Some(' ' | '='))
}

fn line_for_text(text: &str, needle: &str) -> Option<u32> {
    for (line_index, line) in text.lines().enumerate() {
        if line.contains(needle) {
            return Some(line_number_from_index(line_index));
        }
    }
    None
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}

fn policy_diagnostic(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: Option<u32>,
    message: &str,
    remediation: &str,
) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line,
        severity: policy.default_severity,
        message: message.to_owned(),
        remediation: remediation.to_owned(),
    })
}
