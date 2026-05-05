//! Check report and stable renderers.

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::model::EnforcementMode;
use crate::model::Severity;
use crate::suppression::Suppression;

const EMPTY_REPORT_MARKDOWN: &str = "# Tiger Style Standards Report\n\nNo findings.\n";
const REPORT_HEADING: &str = "# Tiger Style Standards Report";
const FINDINGS_HEADING: &str = "## Findings";
const SUPPRESSIONS_HEADING: &str = "## Active Suppressions";
const NO_SUPPRESSIONS_LINE: &str = "No active suppressions.";

/// Stable report from a Tiger Style standards checker run.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckReport {
    /// Diagnostics emitted by the run.
    pub diagnostics: Vec<Diagnostic>,
    /// Active suppressions accepted by the run.
    pub active_suppressions: Vec<Suppression>,
    /// Evaluated policy IDs.
    pub evaluated_policy_ids: Vec<String>,
    /// Enforcement-mode metadata for evaluated policies.
    #[serde(default)]
    pub policy_evaluations: Vec<PolicyEvaluation>,
}

/// Enforcement metadata emitted in machine-readable reports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyEvaluation {
    /// Policy ID.
    pub policy_id: String,
    /// Current enforcement mode.
    pub current_enforcement_mode: EnforcementMode,
    /// Target enforcement mode.
    pub target_enforcement_mode: EnforcementMode,
    /// Mechanism or fallback classification for failures.
    pub failure_type: String,
}

impl CheckReport {
    /// Returns true when this report should fail default CI mode.
    #[must_use]
    pub fn has_blocking_diagnostics(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_blocking)
    }

    /// Returns true when this report only has advisory or informational findings.
    #[must_use]
    pub fn is_advisory_only(&self) -> bool {
        !self.diagnostics.is_empty() && !self.has_blocking_diagnostics()
    }
}

/// Serializes a report as deterministic pretty JSON.
///
/// # Errors
///
/// Returns a serde error if the report cannot be serialized.
pub fn render_json(report: &CheckReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Renders a report as deterministic markdown.
#[must_use]
pub fn render_markdown(report: &CheckReport) -> String {
    if report.diagnostics.is_empty() && report.active_suppressions.is_empty() {
        return EMPTY_REPORT_MARKDOWN.to_owned();
    }

    let mut markdown = String::new();
    append_heading(&mut markdown);
    append_diagnostics(&mut markdown, &report.diagnostics);
    append_suppressions(&mut markdown, &report.active_suppressions);
    markdown
}

fn append_heading(markdown: &mut String) {
    markdown.push_str(REPORT_HEADING);
    markdown.push_str("\n\n");
}

fn append_diagnostics(markdown: &mut String, diagnostics: &[Diagnostic]) {
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

fn append_diagnostic(markdown: &mut String, diagnostic: &Diagnostic) {
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

fn append_suppressions(markdown: &mut String, suppressions: &[Suppression]) {
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

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Note => "note",
    }
}
