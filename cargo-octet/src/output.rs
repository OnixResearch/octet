use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write as _;

use serde::Serialize;

use crate::baseline::BaselineEvaluation;
use crate::determinism::RunMetadata;

const FINDING_SCHEMA_VERSION: u32 = 1;
const EXIT_SUCCESS: i32 = 0;
const EXIT_LINT_FAILURE: i32 = 1;
const EXIT_INTEGRATION_FAILURE: i32 = 2;
const SUMMARY_HEADER: &str = "--- octet summary ---";
const SUMMARY_FOOTER: &str = "---";
const SARIF_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const CARGO_PROCESS_SUCCESS: &str = "success";
const CARGO_PROCESS_EXIT_CODE: &str = "exit-code";
const CARGO_PROCESS_TERMINATED: &str = "terminated";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";

pub struct LintMeta {
    pub group: &'static str,
    pub principle: &'static str,
    pub guidance: &'static str,
    pub effect_family: &'static str,
}

pub const UNKNOWN_LINT_META: LintMeta = LintMeta {
    group: "unknown",
    principle: "unknown",
    guidance: "Review this diagnostic manually; Octet has no registered Tiger Style metadata for it.",
    effect_family: "unknown",
};

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub version: u32,
    pub index: String,
    pub stable_id: String,
    pub semantic_id: String,
    pub identity_status: String,
    pub normalized_snippet_hash: String,
    pub config_hash: String,
    pub cache_status: String,
    pub lint: String,
    pub group: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
    pub applicability: Option<String>,
    pub principle: String,
    pub effect_family: String,
    pub crate_name: String,
}

impl Finding {
    pub fn is_autofixable(&self) -> bool {
        self.applicability.as_deref() == Some("MachineApplicable")
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }

    pub fn severity_is_error(&self) -> bool {
        self.severity == ERROR_SEVERITY
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingCounts {
    pub total_findings: usize,
    pub warning_findings: usize,
    pub error_findings: usize,
    pub autofixable_findings: usize,
}

impl FindingCounts {
    pub fn from_findings(findings: &[Finding]) -> Self {
        let warning_findings = findings.iter().filter(|finding| !finding.severity_is_error()).count();
        let error_findings = findings.iter().filter(|finding| finding.severity_is_error()).count();
        let autofixable_findings = findings.iter().filter(|finding| finding.is_autofixable()).count();

        Self {
            total_findings: findings.len(),
            warning_findings,
            error_findings,
            autofixable_findings,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Clean,
    WarningOnly,
    LintFailure,
    IntegrationFailure,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::WarningOnly => "warning-only",
            Self::LintFailure => "lint-failure",
            Self::IntegrationFailure => "integration-failure",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Clean | Self::WarningOnly => EXIT_SUCCESS,
            Self::LintFailure => EXIT_LINT_FAILURE,
            Self::IntegrationFailure => EXIT_INTEGRATION_FAILURE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CargoProcessExit {
    pub classification: &'static str,
    pub code: Option<i32>,
}

impl CargoProcessExit {
    pub fn success() -> Self {
        Self {
            classification: CARGO_PROCESS_SUCCESS,
            code: Some(EXIT_SUCCESS),
        }
    }

    pub fn exit_code(code: i32) -> Self {
        if code == EXIT_SUCCESS {
            return Self::success();
        }

        Self {
            classification: CARGO_PROCESS_EXIT_CODE,
            code: Some(code),
        }
    }

    pub fn terminated() -> Self {
        Self {
            classification: CARGO_PROCESS_TERMINATED,
            code: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.classification == CARGO_PROCESS_SUCCESS
    }

    pub fn is_terminated(&self) -> bool {
        self.classification == CARGO_PROCESS_TERMINATED
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub status: RunStatus,
    pub exit_code: i32,
    pub counts: FindingCounts,
    pub cargo_process_exit: CargoProcessExit,
}

impl RunOutcome {
    pub fn classify(findings: &[Finding], cargo_process_exit: CargoProcessExit) -> Self {
        let counts = FindingCounts::from_findings(findings);
        let status = classify_status(&counts, &cargo_process_exit);

        Self {
            status,
            exit_code: status.exit_code(),
            counts,
            cargo_process_exit,
        }
    }

    pub fn force_integration_failure(&self) -> Self {
        Self {
            status: RunStatus::IntegrationFailure,
            exit_code: RunStatus::IntegrationFailure.exit_code(),
            counts: self.counts.clone(),
            cargo_process_exit: self.cargo_process_exit.clone(),
        }
    }

    pub fn force_lint_failure(&self) -> Self {
        Self {
            status: RunStatus::LintFailure,
            exit_code: RunStatus::LintFailure.exit_code(),
            counts: self.counts.clone(),
            cargo_process_exit: self.cargo_process_exit.clone(),
        }
    }
}

#[derive(Serialize)]
struct StatusArtifact<'a> {
    status: RunStatus,
    exit_code: i32,
    output_format: &'a str,
    metadata: &'a RunMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline: Option<&'a BaselineEvaluation>,
    total_findings: usize,
    warning_findings: usize,
    error_findings: usize,
    autofixable_findings: usize,
    cargo_process_exit: &'a CargoProcessExit,
}

fn classify_status(counts: &FindingCounts, cargo_process_exit: &CargoProcessExit) -> RunStatus {
    if !cargo_process_exit.is_success() {
        return RunStatus::IntegrationFailure;
    }

    if counts.error_findings > 0 {
        return RunStatus::LintFailure;
    }

    if counts.total_findings > 0 {
        return RunStatus::WarningOnly;
    }

    RunStatus::Clean
}

pub fn lint_meta_map() -> HashMap<&'static str, LintMeta> {
    let entries: &[(&str, &str, &str, &str)] = &[
        (
            "compound_assertion",
            "tigerstyle::safety",
            "Assertion Quality",
            "split compound assertions so the failing clause is obvious",
        ),
        (
            "function_length",
            "tigerstyle::structure",
            "Function Length",
            "split long functions into named helpers at invariant or phase boundaries",
        ),
        (
            "acronym_style",
            "tigerstyle::naming",
            "Acronym Capitalization",
            "capitalize acronyms like ordinary words in type and variant names",
        ),
        (
            "numeric_units",
            "tigerstyle::naming",
            "Unit Suffixes",
            "put the unit in the name of numeric quantities",
        ),
        (
            "unbounded_loop",
            "tigerstyle::safety",
            "Bounded Control Flow",
            "bound long-running loops with an exit condition, timeout, or bounded input",
        ),
        (
            "compound_condition",
            "tigerstyle::structure",
            "Condition Decomposition",
            "break multi-part conditions into named predicates or nested guards",
        ),
        (
            "verified_purity",
            "tigerstyle::purity",
            "Verified Module Purity",
            "keep verified modules free of runtime-only side effects and shell concerns",
        ),
        (
            "unjustified_allow",
            "tigerstyle::structure",
            "Documented Suppression",
            "every lint suppression needs a local reason tied to the code shape",
        ),
        (
            "nested_conditionals",
            "tigerstyle::structure",
            "Simple Control Flow",
            "flatten control flow with guards, early returns, or helper predicates",
        ),
        (
            "assertion_density",
            "tigerstyle::safety",
            "Assertion Density",
            "add enough assertions to make local invariants visible near the logic they defend",
        ),
        (
            "multi_lock_ordering",
            "tigerstyle::concurrency",
            "Lock Ordering",
            "document and preserve one lock acquisition order across the codebase",
        ),
        (
            "explicit_defaults",
            "tigerstyle::safety",
            "Explicit Defaults",
            "construct foreign-crate values explicitly instead of relying on `Default`",
        ),
        (
            "unbounded_channel",
            "tigerstyle::safety",
            "Bounded Queues",
            "use bounded channels with explicit capacity instead of unbounded queues",
        ),
        (
            "unbounded_collection_growth",
            "tigerstyle::safety",
            "Bounded Resource Growth",
            "reserve collection capacity or add an explicit local length bound before growing it in a loop",
        ),
        (
            "sentinel_fallback",
            "tigerstyle::safety",
            "safety",
            "branch explicitly, propagate the missing/error state, or emit tagged fallback text instead of <int>::MAX/<int>::MIN.",
        ),
        (
            "usize_in_public_api",
            "tigerstyle::structure",
            "Cross-Platform APIs",
            "use fixed-width integers at API boundaries instead of `usize` or `isize`",
        ),
        (
            "too_many_parameters",
            "tigerstyle::structure",
            "Simple Interfaces",
            "group many parameters into a named options struct instead of a long positional signature",
        ),
        (
            "negated_predicate",
            "tigerstyle::naming",
            "Positive Predicates",
            "prefer positive predicate names over negated forms like `is_not_ready` or `no_capacity`",
        ),
        (
            "bool_naming",
            "tigerstyle::naming",
            "Positive Predicates",
            "name booleans as predicates so call sites read like questions",
        ),
        (
            "ignored_result",
            "tigerstyle::safety",
            "Error Handling",
            "handle fallible results explicitly with `?`, `match`, or an intentional logged drop",
        ),
        (
            "unchecked_narrowing",
            "tigerstyle::safety",
            "Safe Narrowing",
            "narrow integers with checked conversions instead of `as`",
        ),
        (
            "unchecked_division",
            "tigerstyle::safety",
            "Explicit Division",
            "guard divisors before division so zero is impossible",
        ),
        (
            "contradictory_time",
            "tigerstyle::purity",
            "Functional Core",
            "choose one source of time for a code path and thread it through consistently",
        ),
        (
            "ambient_clock",
            "tigerstyle::purity",
            "Functional Core",
            "pass time into core logic or isolate clock reads in a shell-boundary helper with a narrow allow reason",
        ),
        (
            "catch_all_on_enum",
            "tigerstyle::safety",
            "Exhaustive Matching",
            "match enum variants explicitly so new variants cannot slip through `_`",
        ),
        (
            "platform_dependent_cast",
            "tigerstyle::structure",
            "Explicit Types",
            "use explicit-width types at boundaries instead of `usize` or `isize` conversions",
        ),
        (
            "float_for_currency",
            "tigerstyle::naming",
            "Integer Currency",
            "store money as scaled integers, not floats",
        ),
        (
            "ambiguous_params",
            "tigerstyle::structure",
            "Explicit Interfaces",
            "replace same-type parameter clusters with named fields or an options struct",
        ),
        (
            "raw_arithmetic_overflow",
            "tigerstyle::safety",
            "Overflow Protection",
            "use checked or saturating arithmetic when overflow would change behavior",
        ),
        (
            "no_unwrap",
            "tigerstyle::safety",
            "Error Handling",
            "propagate or handle absence explicitly instead of panicking with `unwrap` or `expect`",
        ),
        (
            "no_panic",
            "tigerstyle::safety",
            "Fail Fast Correctly",
            "return errors or prove exhaustiveness instead of panicking in production code",
        ),
        (
            "no_todo",
            "tigerstyle::safety",
            "Fail Fast Correctly",
            "replace temporary stubs with implemented code or an explicit error path before release",
        ),
        (
            "unjustified_no_todo_allow",
            "tigerstyle::safety",
            "Documented Todo Escape",
            "reserve `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` for temporary `todo!()` stubs with an adjacent `//` justification comment",
        ),
        (
            "no_recursion",
            "tigerstyle::safety",
            "Simple Control Flow",
            "prefer explicit iteration or work queues over self-recursive control flow",
        ),
    ];

    entries
        .iter()
        .map(|(name, group, principle, guidance)| {
            (*name, LintMeta {
                group,
                principle,
                guidance,
                effect_family: effect_family_for_group(group),
            })
        })
        .collect()
}

fn effect_family_for_group(group: &str) -> &'static str {
    match group {
        "tigerstyle::purity" => "time",
        "tigerstyle::concurrency" => "concurrency",
        "tigerstyle::safety" => "failure",
        "tigerstyle::structure" => "resource",
        "tigerstyle::naming" => "unknown",
        _ => "unknown",
    }
}

pub fn finding_version() -> u32 {
    FINDING_SCHEMA_VERSION
}

pub fn render_json_lines(findings: &[Finding]) -> Result<String, String> {
    let mut rendered = String::new();

    for finding in findings {
        let line = serde_json::to_string(finding).map_err(|error| format!("write JSON finding: {error}"))?;
        rendered.push_str(&line);
        rendered.push('\n');
    }

    Ok(rendered)
}

pub fn render_sarif(
    findings: &[Finding],
    meta_map: &HashMap<&str, LintMeta>,
    outcome: &RunOutcome,
    command_line: &str,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    let mut seen_rules: Vec<&str> = Vec::new();
    for finding in findings {
        if !seen_rules.contains(&finding.lint.as_str()) {
            seen_rules.push(&finding.lint);
        }
    }

    let rules: Vec<serde_json::Value> = seen_rules
        .iter()
        .map(|lint_name| {
            let guidance = meta_map.get(lint_name).map(|meta| meta.guidance).unwrap_or("unknown");
            let group = meta_map.get(lint_name).map(|meta| meta.group).unwrap_or("tigerstyle::unknown");
            serde_json::json!({
                "id": lint_name,
                "shortDescription": { "text": guidance },
                "properties": { "group": group, "effectFamily": meta_map.get(lint_name).map(|meta| meta.effect_family).unwrap_or("unknown") }
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "ruleId": finding.lint,
                "level": finding.severity,
                "message": { "text": finding.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": finding.file },
                        "region": {
                            "startLine": finding.line,
                            "startColumn": finding.column
                        }
                    }
                }],
                "properties": { "findingIndex": finding.index, "stableId": finding.stable_id, "semanticId": finding.semantic_id, "effectFamily": finding.effect_family, "cacheStatus": finding.cache_status }
            })
        })
        .collect();

    let mut properties = serde_json::json!({
        "status": outcome.status.as_str(),
        "exitCode": outcome.exit_code,
        "totalFindings": outcome.counts.total_findings,
        "warningFindings": outcome.counts.warning_findings,
        "errorFindings": outcome.counts.error_findings,
        "autofixableFindings": outcome.counts.autofixable_findings,
        "cargoProcessExitClassification": outcome.cargo_process_exit.classification,
        "cargoProcessExitCode": outcome.cargo_process_exit.code,
        "metadata": metadata,
    });
    if let Some(baseline) = baseline {
        properties["baseline"] = serde_json::json!(baseline);
    }

    let sarif = serde_json::json!({
        "$schema": SARIF_SCHEMA_URL,
        "version": SARIF_VERSION,
        "runs": [{
            "tool": {
                "driver": {
                    "name": "cargo-octet",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules
                }
            },
            "invocations": [{
                "commandLine": command_line,
                "executionSuccessful": outcome.exit_code == EXIT_SUCCESS,
                "exitCode": outcome.exit_code
            }],
            "properties": properties,
            "results": results
        }]
    });

    let mut rendered = serde_json::to_string_pretty(&sarif).map_err(|error| format!("write SARIF: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_status_json(
    outcome: &RunOutcome,
    output_format: &str,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    let artifact = StatusArtifact {
        status: outcome.status,
        exit_code: outcome.exit_code,
        output_format,
        metadata,
        baseline,
        total_findings: outcome.counts.total_findings,
        warning_findings: outcome.counts.warning_findings,
        error_findings: outcome.counts.error_findings,
        autofixable_findings: outcome.counts.autofixable_findings,
        cargo_process_exit: &outcome.cargo_process_exit,
    };

    let mut rendered =
        serde_json::to_string_pretty(&artifact).map_err(|error| format!("write status JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_summary(
    findings: &[Finding],
    outcome: &RunOutcome,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    let mut summary = String::new();

    writeln!(&mut summary, "{SUMMARY_HEADER}").map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Status: {}", outcome.status.as_str()).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Findings: {}", outcome.counts.total_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Warnings: {}", outcome.counts.warning_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Errors: {}", outcome.counts.error_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Autofixable: {}", outcome.counts.autofixable_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Tool: {} {}", metadata.tool_name, metadata.tool_version)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Toolchain: {}", metadata.toolchain).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Config hash: {}", metadata.config_hash)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Profile hash: {}", metadata.profile_hash)
        .map_err(|error| format!("write summary: {error}"))?;

    if let Some(baseline) = baseline {
        writeln!(&mut summary, "Baseline hash: {}", baseline.baseline_hash)
            .map_err(|error| format!("write summary: {error}"))?;
        writeln!(
            &mut summary,
            "Baseline: {} accepted, {} current accepted, {} new, {} stale",
            baseline.accepted_total, baseline.accepted_current, baseline.new_total, baseline.stale_total
        )
        .map_err(|error| format!("write summary: {error}"))?;
    }

    if findings.is_empty() {
        writeln!(&mut summary, "{SUMMARY_FOOTER}").map_err(|error| format!("write summary: {error}"))?;
        return Ok(summary);
    }

    let mut by_crate: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_crate.entry(&finding.crate_name).or_default() += 1;
    }
    let mut by_crate_sorted: Vec<_> = by_crate.into_iter().collect();
    by_crate_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_crate_name = by_crate_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By crate:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_crate_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_crate_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let mut by_effect: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_effect.entry(&finding.effect_family).or_default() += 1;
    }
    let mut by_effect_sorted: Vec<_> = by_effect.into_iter().collect();
    by_effect_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_effect_name = by_effect_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By effect:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_effect_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_effect_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let mut by_lint: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_lint.entry(&finding.lint).or_default() += 1;
    }
    let mut by_lint_sorted: Vec<_> = by_lint.into_iter().collect();
    by_lint_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_lint_name = by_lint_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By lint:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_lint_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_lint_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let max_index_width = findings.iter().map(|finding| finding.index.len()).max().unwrap_or(0);
    let max_lint_width = findings.iter().map(|finding| finding.lint.len()).max().unwrap_or(0);
    let max_crate_width = findings.iter().map(|finding| finding.crate_name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Index:").map_err(|error| format!("write summary: {error}"))?;
    for finding in findings {
        writeln!(
            &mut summary,
            "  {:<index_width$}  {:<lint_width$} {:<crate_width$}  {}",
            finding.index,
            finding.lint,
            finding.crate_name,
            finding.location(),
            index_width = max_index_width,
            lint_width = max_lint_width,
            crate_width = max_crate_width,
        )
        .map_err(|error| format!("write summary: {error}"))?;
    }

    writeln!(&mut summary, "{SUMMARY_FOOTER}").map_err(|error| format!("write summary: {error}"))?;
    Ok(summary)
}
