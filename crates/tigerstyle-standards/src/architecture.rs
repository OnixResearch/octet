//! Architecture and FCIS boundary checks.

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::source::SourceFile;

const ARCHITECTURE_FAMILY: &str = "architecture";
const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID: &str = "tigerstyle.architecture.external-domain-leakage";
const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
const EFFECT_CAPABILITY_POLICY_ID: &str = "tigerstyle.architecture.effect-capability-boundaries";
const TYPE_STATE_POLICY_ID: &str = "tigerstyle.idioms.nominal-type-boundaries";
const FORBIDDEN_RUNTIME_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const STD_SHELL_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const CORE_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-core/src/lib.rs";
const CORE_MANIFEST_PATH_SUFFIX: &str = "crates/tigerstyle-core/Cargo.toml";
const STD_MANIFEST_PATH_SUFFIX: &str = "crates/tigerstyle-std/Cargo.toml";
const CORE_RUST_PATH_SEGMENT: &str = "crates/tigerstyle-core/src/";
const RUST_SOURCE_EXTENSION: &str = ".rs";
const STD_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-std/src/lib.rs";
const ROOT_LIB_PATH_SUFFIX: &str = "src/lib.rs";
const ROOT_MAIN_PATH_SUFFIX: &str = "src/main.rs";
const NO_STD_ATTRIBUTE: &str = "#![no_std]";
const CORE_DEPENDENCIES_SECTION: &str = "[dependencies]";
const DEV_DEPENDENCIES_SECTION: &str = "[dev-dependencies]";
const CORE_FORBIDDEN_DEPENDENCIES: &[&str] = &["tokio", "tracing", "tracing-subscriber", "clap", "parking_lot", "url", "tigerstyle-std"];
const CORE_CRATE_DEPENDENCY: &str = "tigerstyle-core";
const AMBIENT_CAPABILITY_PATTERNS: &[&str] = &["std::env::", "std::fs::", "std::net::", "SystemTime::now(", "File::open("];
const RAW_TYPE_STATE_PATTERNS: &[&str] = &["send_raw(", "raw_send(", "type ConnectionState = bool", "pub fn open_raw"];
const EXTERNAL_DOMAIN_TYPE_PATTERNS: &[&str] = &[
    "axum::",
    "hyper::",
    "prost::",
    "reqwest::",
    "serde_json::",
    "sqlx::",
    "tonic::",
];
const PUBLIC_API_CHUNK_TERMINATORS: &[char] = &['{', ';'];
const PUBLIC_API_ITEM_KEYWORDS: &[&str] = &[
    "async fn ",
    "const ",
    "enum ",
    "fn ",
    "mod ",
    "static ",
    "struct ",
    "trait ",
    "type ",
    "use ",
];
const PUBLIC_SCOPED_VISIBILITY_END: char = ')';
const PUBLIC_SCOPED_VISIBILITY_PREFIX: &str = "pub(";
const PUBLIC_UNSCOPED_VISIBILITY_PREFIX: &str = "pub ";
const ROOT_LIB_MAX_LINES: usize = 40;
const ROOT_MAIN_MAX_LINES: usize = 80;
const ROOT_PUBLIC_API_ITEM_BUDGET: usize = 8;
const FIRST_PUBLIC_API_OVER_BUDGET_INDEX: usize = ROOT_PUBLIC_API_ITEM_BUDGET;
const FIRST_LINE: u32 = 1;
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const PUBLIC_VISIBILITY_PREFIXES: &[&str] = &["pub ", "pub("];
const EXTERNAL_LEAK_REMEDIATION: &str =
    "wrap the external type behind a domain type or trait adapter and translate at the boundary";

#[derive(Clone, Debug, Eq, PartialEq)]
struct PublicApiChunk {
    line: u32,
    text: String,
}

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
    check_shell_depends_on_core(registry, sources, &mut diagnostics);
    check_core_capability_constructors(registry, sources, &mut diagnostics);
    check_raw_type_state_operations(registry, sources, &mut diagnostics);
    check_external_domain_leakage(registry, sources, &mut diagnostics);
    check_root_thinness(registry, sources, &mut diagnostics);
    check_public_api_budget(registry, sources, &mut diagnostics);
    diagnostics
}

fn check_core_no_std(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    for source in sources_by_suffix(sources, CORE_LIB_PATH_SUFFIX) {
        if source.text.contains(NO_STD_ATTRIBUTE) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(FIRST_LINE),
            "tigerstyle-core is missing #![no_std]",
            "add #![no_std] to the core crate root or move std-dependent code to tigerstyle-std",
        ));
    }
}

fn check_core_forbidden_dependencies(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FORBIDDEN_RUNTIME_POLICY_ID) else {
        return;
    };
    for source in sources_by_suffix(sources, CORE_MANIFEST_PATH_SUFFIX) {
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
}

fn check_std_shell_owns_std(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(STD_SHELL_POLICY_ID) else {
        return;
    };
    for source in sources_by_suffix(sources, STD_LIB_PATH_SUFFIX) {
        if !source.text.contains(NO_STD_ATTRIBUTE) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(FIRST_LINE),
            "tigerstyle-std must not declare #![no_std]",
            "keep std/runtime shell code in tigerstyle-std and no_std code in tigerstyle-core",
        ));
    }
}


fn check_shell_depends_on_core(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    for source in sources_by_suffix(sources, STD_MANIFEST_PATH_SUFFIX) {
        let dependency_section = dependency_section(&source.text);
        if dependency_section_contains(dependency_section, CORE_CRATE_DEPENDENCY) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            None,
            "tigerstyle-std must depend on tigerstyle-core",
            "make shell crates depend inward on the core crate and keep the reverse dependency forbidden",
        ));
    }
}

fn check_core_capability_constructors(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(EFFECT_CAPABILITY_POLICY_ID) else {
        return;
    };
    for source in core_rust_sources(sources) {
        for (line_index, line) in source.text.lines().enumerate() {
            let Some(pattern) = AMBIENT_CAPABILITY_PATTERNS.iter().copied().find(|pattern| line.contains(pattern)) else {
                continue;
            };
            diagnostics.push(policy_diagnostic(
                policy,
                source,
                Some(line_number_from_index(line_index)),
                &format!("core performs ambient authority operation `{pattern}` instead of accepting an injected capability"),
                "move ambient authority constructors to the shell and inject data or narrow capability traits into core",
            ));
        }
    }
}

fn check_raw_type_state_operations(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(TYPE_STATE_POLICY_ID) else {
        return;
    };
    for source in sources {
        for (line_index, line) in source.text.lines().enumerate() {
            let Some(pattern) = RAW_TYPE_STATE_PATTERNS.iter().copied().find(|pattern| line.contains(pattern)) else {
                continue;
            };
            diagnostics.push(policy_diagnostic(
                policy,
                source,
                Some(line_number_from_index(line_index)),
                &format!("raw protocol operation `{pattern}` bypasses type-state surface"),
                "model protocol states with nominal wrappers and transition methods instead of raw operation shortcuts",
            ));
        }
    }
}

fn check_external_domain_leakage(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID) else {
        return;
    };
    for source in core_rust_sources(sources) {
        append_external_domain_leakage_diagnostics(policy, source, diagnostics);
    }
}

fn append_external_domain_leakage_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for chunk in public_api_chunks(&source.text) {
        let Some(pattern) = external_domain_pattern(&chunk.text) else {
            continue;
        };
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(chunk.line),
            &format!("core public API leaks external domain type pattern `{pattern}`"),
            EXTERNAL_LEAK_REMEDIATION,
        ));
    }
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
    for source in root_sources_by_suffix(sources, path_suffix) {
        let line_count = source.text.lines().count();
        if line_count <= max_lines {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            None,
            &format!("root file has {line_count} lines, exceeding thin-shell limit {max_lines}"),
            "move non-trivial logic from the root crate into tigerstyle-core or tigerstyle-std",
        ));
    }
}

fn check_public_api_budget(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(PUBLIC_API_BUDGET_POLICY_ID) else {
        return;
    };
    for source in root_sources_by_suffix(sources, ROOT_LIB_PATH_SUFFIX) {
        let public_item_lines = public_api_item_lines(&source.text);
        if public_item_lines.len() <= ROOT_PUBLIC_API_ITEM_BUDGET {
            continue;
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
}

fn public_api_item_lines(text: &str) -> Vec<u32> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| is_public_api_item_line(line).then_some(line_number_from_index(line_index)))
        .collect()
}

fn public_api_chunks(text: &str) -> Vec<PublicApiChunk> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        let line = lines[line_index];
        if !is_public_api_chunk_start(line) {
            line_index = line_index.saturating_add(FIRST_LINE_NUMBER_OFFSET);
            continue;
        }

        let start_line = line_number_from_index(line_index);
        let (chunk_text, next_line_index) = collect_public_api_chunk(&lines, line_index);
        chunks.push(PublicApiChunk {
            line: start_line,
            text: chunk_text,
        });
        line_index = next_line_index;
    }

    chunks
}

fn collect_public_api_chunk(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut chunk = lines[start_index].trim().to_owned();
    let mut cursor = start_index;
    while !public_api_chunk_is_complete(&chunk) && cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET) < lines.len() {
        cursor = cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET);
        chunk.push(' ');
        chunk.push_str(lines[cursor].trim());
    }
    (chunk, cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET))
}

fn public_api_chunk_is_complete(chunk: &str) -> bool {
    chunk.chars().any(|character| PUBLIC_API_CHUNK_TERMINATORS.contains(&character))
}

fn is_public_api_item_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    PUBLIC_VISIBILITY_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn is_public_api_chunk_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(after_visibility) = text_after_public_visibility(trimmed) else {
        return false;
    };
    PUBLIC_API_ITEM_KEYWORDS.iter().any(|keyword| after_visibility.starts_with(keyword))
}

fn text_after_public_visibility(trimmed: &str) -> Option<&str> {
    if let Some(after_pub) = trimmed.strip_prefix(PUBLIC_UNSCOPED_VISIBILITY_PREFIX) {
        return Some(after_pub);
    }
    if !trimmed.starts_with(PUBLIC_SCOPED_VISIBILITY_PREFIX) {
        return None;
    }
    let (_, after_scope) = trimmed.split_once(PUBLIC_SCOPED_VISIBILITY_END)?;
    Some(after_scope.trim_start())
}

fn external_domain_pattern(text: &str) -> Option<&'static str> {
    EXTERNAL_DOMAIN_TYPE_PATTERNS.iter().copied().find(|pattern| text.contains(pattern))
}

fn sources_by_suffix<'a>(sources: &'a [SourceFile], suffix: &'a str) -> impl Iterator<Item = &'a SourceFile> {
    sources.iter().filter(move |source| source.path.ends_with(suffix))
}

fn root_sources_by_suffix<'a>(sources: &'a [SourceFile], suffix: &'a str) -> impl Iterator<Item = &'a SourceFile> {
    sources_by_suffix(sources, suffix).filter(|source| !source.path.contains("/crates/"))
}

fn core_rust_sources(sources: &[SourceFile]) -> impl Iterator<Item = &SourceFile> {
    sources
        .iter()
        .filter(|source| source.path.contains(CORE_RUST_PATH_SEGMENT) && source.path.ends_with(RUST_SOURCE_EXTENSION))
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
