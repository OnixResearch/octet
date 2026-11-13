//! Deterministic human-readable and JSON output.

const EMPTY_MARKDOWN: &str = "# Tiger Style Standards Report\n\nNo findings.\n";
const HEADING: &str = "# Tiger Style Standards Report";
const FINDINGS_HEADING: &str = "## Findings";
const POLICY_EXECUTIONS_HEADING: &str = "## Policy Execution Coverage";
const SUPPRESSIONS_HEADING: &str = "## Active Suppressions";
const SUPPRESSED_FINDINGS_HEADING: &str = "## Suppressed Findings";
const CAPABILITIES_HEADING: &str = "## Capability Manifest Reports";
const NO_SUPPRESSIONS_LINE: &str = "No active suppressions.";

/// Serializes a report as deterministic pretty JSON.
///
/// # Errors
///
/// Returns a serde error if the report cannot be serialized.
pub fn render_json(report: &super::CheckReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Renders a report as deterministic markdown.
#[must_use]
pub fn render_markdown(report: &super::CheckReport) -> String {
    if report.diagnostics.is_empty()
        && report.active_suppressions.is_empty()
        && report.capability_reports.is_empty()
        && report.policy_executions.is_empty()
    {
        return EMPTY_MARKDOWN.to_owned();
    }

    let mut markdown = String::new();
    append_heading(&mut markdown);
    append_diagnostics(&mut markdown, &report.diagnostics);
    append_policy_executions(&mut markdown, report);
    append_suppressions(&mut markdown, &report.active_suppressions);
    append_suppressed_diagnostics(&mut markdown, &report.suppressed_diagnostics);
    append_capabilities(&mut markdown, &report.capability_reports);
    markdown
}

fn append_heading(markdown: &mut String) {
    markdown.push_str(HEADING);
    markdown.push_str("\n\n");
}

fn append_diagnostics(markdown: &mut String, diagnostics: &[crate::diagnostic::Diagnostic]) {
    markdown.push_str(FINDINGS_HEADING);
    markdown.push_str("\n\n");
    if diagnostics.is_empty() {
        markdown.push_str("No findings.\n\n");
        return;
    }

    for diagnostic in diagnostics {
        append_diagnostic(markdown, diagnostic);
    }
    markdown.push('\n');
}

fn append_diagnostic(markdown: &mut String, diagnostic: &crate::diagnostic::Diagnostic) {
    let line = diagnostic.line.map_or_else(|| String::from("unknown"), |line| line.to_string());
    markdown.push_str("- [");
    markdown.push_str(severity_label(diagnostic.severity));
    markdown.push_str("] `");
    markdown.push_str(&diagnostic.policy_id);
    markdown.push_str("` at `");
    markdown.push_str(&diagnostic.path);
    markdown.push(':');
    markdown.push_str(&line);
    markdown.push_str("` — ");
    markdown.push_str(&diagnostic.message);
    markdown.push('\n');
    markdown.push_str("  - Source: ");
    markdown.push_str(&diagnostic.source_section);
    markdown.push('\n');
    markdown.push_str("  - Remediation: ");
    markdown.push_str(&diagnostic.remediation);
    markdown.push('\n');
}

fn append_policy_executions(markdown: &mut String, report: &super::CheckReport) {
    if report.policy_executions.is_empty() {
        return;
    }
    markdown.push_str(POLICY_EXECUTIONS_HEADING);
    markdown.push_str("\n\nCoverage complete: ");
    markdown.push_str(if report.coverage_complete() { "yes" } else { "no" });
    markdown.push_str(".\n\n");
    for execution in &report.policy_executions {
        markdown.push_str("- `");
        markdown.push_str(&execution.policy_id);
        markdown.push_str("` status=");
        markdown.push_str(execution_status_label(execution.status));
        markdown.push_str(" checker=");
        markdown.push_str(&execution.checker);
        markdown.push_str(" owner=");
        markdown.push_str(&execution.checker_owner);
        markdown.push_str(" ownership=");
        markdown.push_str(execution_ownership_label(execution.ownership));
        markdown.push_str(" required=");
        markdown.push_str(if execution.required { "yes" } else { "no" });
        markdown.push('\n');
        markdown.push_str("  - Scope: ");
        markdown.push_str(&execution.selected_scope);
        markdown.push('\n');
        markdown.push_str("  - Explanation: ");
        markdown.push_str(&execution.coverage_explanation);
        markdown.push('\n');
        markdown.push_str("  - Evidence: ");
        markdown.push_str(&execution.evidence_identity);
        markdown.push('\n');
        markdown.push_str("  - Linked diagnostics: ");
        markdown.push_str(&execution.linked_diagnostics.len().to_string());
        if !execution.linked_diagnostics.is_empty() {
            markdown.push_str(" (");
            markdown.push_str(&execution.linked_diagnostics.join(", "));
            markdown.push(')');
        }
        markdown.push('\n');
    }
}

fn execution_status_label(status: super::PolicyExecutionStatus) -> &'static str {
    match status {
        super::PolicyExecutionStatus::Passed => "passed",
        super::PolicyExecutionStatus::Failed => "failed",
        super::PolicyExecutionStatus::NotRun => "not_run",
        super::PolicyExecutionStatus::Unsupported => "unsupported",
        super::PolicyExecutionStatus::NotApplicable => "not_applicable",
        super::PolicyExecutionStatus::ToolError => "tool_error",
    }
}

fn execution_ownership_label(ownership: super::PolicyExecutionOwnership) -> &'static str {
    match ownership {
        super::PolicyExecutionOwnership::Local => "local",
        super::PolicyExecutionOwnership::External => "external",
        super::PolicyExecutionOwnership::HumanProcess => "human_process",
    }
}

fn append_suppressions(markdown: &mut String, suppressions: &[crate::suppression::Suppression]) {
    markdown.push_str(SUPPRESSIONS_HEADING);
    markdown.push_str("\n\n");
    if suppressions.is_empty() {
        markdown.push_str(NO_SUPPRESSIONS_LINE);
        markdown.push('\n');
        return;
    }

    for suppression in suppressions {
        markdown.push_str("- `");
        markdown.push_str(&suppression.policy_id);
        markdown.push_str("` at `");
        markdown.push_str(&suppression.path);
        markdown.push(':');
        markdown.push_str(&suppression.line.to_string());
        markdown.push_str("` scope=");
        markdown.push_str(&suppression.scope);
        markdown.push('\n');
    }
}

fn append_suppressed_diagnostics(markdown: &mut String, diagnostics: &[crate::diagnostic::Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    markdown.push('\n');
    markdown.push_str(SUPPRESSED_FINDINGS_HEADING);
    markdown.push_str("\n\n");
    for diagnostic in diagnostics {
        append_diagnostic(markdown, diagnostic);
    }
}

fn append_capabilities(markdown: &mut String, reports: &[crate::capability_manifest::CapabilityReport]) {
    if reports.is_empty() {
        return;
    }
    markdown.push('\n');
    markdown.push_str(CAPABILITIES_HEADING);
    markdown.push_str("\n\n");
    for report in reports {
        append_capability(markdown, report);
    }
}

fn append_capability(markdown: &mut String, report: &crate::capability_manifest::CapabilityReport) {
    markdown.push_str("- `");
    markdown.push_str(&report.policy_id);
    markdown.push_str("` mode=");
    markdown.push_str(&report.enforcement_mode);
    markdown.push('\n');
    markdown.push_str("  - Scope classes: ");
    markdown.push_str(&report.declared_scope_classes.join(", "));
    markdown.push('\n');
    markdown.push_str("  - Capability families: ");
    markdown.push_str(&report.declared_capability_families.join(", "));
    markdown.push('\n');
    markdown.push_str("  - Observations: ");
    markdown.push_str(&report.observations.len().to_string());
    markdown.push('\n');
    for observation in &report.observations {
        append_capability_observation(markdown, observation);
    }
    markdown.push_str("  - Handler evidence: ");
    markdown.push_str(&report.handler_evidence.len().to_string());
    markdown.push('\n');
    for evidence in &report.handler_evidence {
        markdown.push_str("    - `");
        markdown.push_str(&evidence.handler);
        markdown.push_str("` artifact=`");
        markdown.push_str(&evidence.artifact);
        markdown.push_str("` replay=`");
        markdown.push_str(&evidence.replay_command);
        markdown.push_str("` blake3=");
        markdown.push_str(&evidence.blake3);
        markdown.push_str(" success=");
        markdown.push_str(&evidence.success_path_coverage.to_string());
        markdown.push_str(" failure=");
        markdown.push_str(&evidence.failure_path_coverage.to_string());
        markdown.push('\n');
    }
    markdown.push_str("  - Non-claims:\n");
    for non_claim in &report.non_claims {
        markdown.push_str("    - ");
        markdown.push_str(non_claim);
        markdown.push('\n');
    }
}

fn append_capability_observation(
    markdown: &mut String,
    observation: &crate::capability_manifest::CapabilityObservation,
) {
    let line = observation.line.map_or_else(|| String::from("unknown"), |line| line.to_string());
    markdown.push_str("    - ");
    markdown.push_str(&observation.status);
    markdown.push_str(" `");
    markdown.push_str(&observation.observation_kind);
    markdown.push_str("` ");
    markdown.push_str(&observation.capability_family);
    markdown.push_str(" at `");
    markdown.push_str(&observation.path);
    markdown.push(':');
    markdown.push_str(&line);
    markdown.push_str("` matched `");
    markdown.push_str(&observation.matched);
    markdown.push_str("` lint_ids=[");
    markdown.push_str(&observation.linked_lint_ids.join(", "));
    markdown.push_str("]\n");
}

fn severity_label(severity: crate::model::Severity) -> &'static str {
    match severity {
        crate::model::Severity::Error => "error",
        crate::model::Severity::Warning => "warning",
        crate::model::Severity::Info => "info",
        crate::model::Severity::Note => "note",
    }
}
