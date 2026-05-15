//! Suppression parsing and validation.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::Severity;
use crate::source::SourceFile;

const SUPPRESSION_MARKER: &str = "tigerstyle-allow:";
const POLICY_FIELD: &str = "policy";
const SOURCE_FIELD: &str = "source";
const OWNER_FIELD: &str = "owner";
const RATIONALE_FIELD: &str = "rationale";
const REVIEW_FIELD: &str = "review";
const SAFER_DEFAULT_FIELD: &str = "safer_default";
const SCOPE_FIELD: &str = "scope";
const LINE_SCOPE: &str = "line";
const NEXT_LINE_SCOPE: &str = "next-line";
const UNKNOWN_POLICY_ID: &str = "tigerstyle.suppression.unknown-policy";
const SUPPRESSION_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
const MISSING_FIELD_REMEDIATION: &str = "add required suppression metadata or remove the suppression";
const UNKNOWN_POLICY_REMEDIATION: &str = "cite a policy ID present in the Tiger Style policy registry";
const BLANKET_SCOPE_REMEDIATION: &str = "narrow the suppression to a single line or next line";
const KEY_VALUE_SEPARATOR: char = '=';
const FIELD_SEPARATOR: char = ';';
const FIRST_LINE_NUMBER_OFFSET: usize = 1;

/// Parsed suppression metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Suppression {
    /// Policy ID being suppressed.
    pub policy_id: String,
    /// Tiger Style source section or anchor.
    pub source_section: Option<String>,
    /// Suppression owner.
    pub owner: Option<String>,
    /// Suppression rationale.
    pub rationale: Option<String>,
    /// Review trigger.
    pub review_trigger: Option<String>,
    /// Safer default being bypassed.
    pub safer_default: Option<String>,
    /// Suppression scope.
    pub scope: String,
    /// Source path where the suppression appears.
    pub path: String,
    /// One-based line number where the suppression appears.
    pub line: u32,
}

/// Result of validating suppressions for one source set.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SuppressionReport {
    /// Valid active suppressions.
    pub active_suppressions: Vec<Suppression>,
    /// Suppression diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses and validates all suppressions in selected rule families.
#[must_use]
pub fn validate_suppressions(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> SuppressionReport {
    let mut report = SuppressionReport::default();
    for source in sources {
        validate_source_suppressions(registry, source, selection, &mut report);
    }
    report
}

fn validate_source_suppressions(
    registry: &PolicyRegistry,
    source: &SourceFile,
    selection: &FamilySelection,
    report: &mut SuppressionReport,
) {
    for (line_index, line_text) in source.text.lines().enumerate() {
        let Some(marker_index) = line_text.find(SUPPRESSION_MARKER) else {
            continue;
        };
        let line = line_number_from_index(line_index);
        let raw_metadata = &line_text[marker_index + SUPPRESSION_MARKER.len()..];
        validate_suppression_line(registry, source, line, raw_metadata, selection, report);
    }
}

fn validate_suppression_line(
    registry: &PolicyRegistry,
    source: &SourceFile,
    line: u32,
    raw_metadata: &str,
    selection: &FamilySelection,
    report: &mut SuppressionReport,
) {
    let fields = parse_metadata_fields(raw_metadata);
    let policy_id = field_value(&fields, POLICY_FIELD).unwrap_or(UNKNOWN_POLICY_ID);
    let Some(policy) = registry.policy_by_id(policy_id) else {
        report.diagnostics.push(unknown_policy_diagnostic(source, line, policy_id));
        return;
    };
    if !selection.includes(&policy.pillar) {
        return;
    }

    let suppression = suppression_from_fields(source, line, policy_id, &fields);
    append_metadata_diagnostics(policy, source, line, &suppression, &mut report.diagnostics);
    append_scope_diagnostics(policy, source, line, &suppression, &mut report.diagnostics);
    if !has_suppression_diagnostic(&report.diagnostics, &source.path, line) {
        report.active_suppressions.push(suppression);
    }
}

fn parse_metadata_fields(raw_metadata: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for raw_field in raw_metadata.split(FIELD_SEPARATOR) {
        let trimmed_field = raw_field.trim();
        let Some((key, value)) = trimmed_field.split_once(KEY_VALUE_SEPARATOR) else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        fields.insert(key.to_owned(), value.to_owned());
    }
    fields
}

fn suppression_from_fields(
    source: &SourceFile,
    line: u32,
    policy_id: &str,
    fields: &BTreeMap<String, String>,
) -> Suppression {
    Suppression {
        policy_id: policy_id.to_owned(),
        source_section: field_value(fields, SOURCE_FIELD).map(ToOwned::to_owned),
        owner: field_value(fields, OWNER_FIELD).map(ToOwned::to_owned),
        rationale: field_value(fields, RATIONALE_FIELD).map(ToOwned::to_owned),
        review_trigger: field_value(fields, REVIEW_FIELD).map(ToOwned::to_owned),
        safer_default: field_value(fields, SAFER_DEFAULT_FIELD).map(ToOwned::to_owned),
        scope: field_value(fields, SCOPE_FIELD).unwrap_or(LINE_SCOPE).to_owned(),
        path: source.path.clone(),
        line,
    }
}

fn field_value<'a>(fields: &'a BTreeMap<String, String>, field: &str) -> Option<&'a str> {
    fields.get(field).map(String::as_str)
}

fn append_metadata_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: u32,
    suppression: &Suppression,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let requirements = &policy.suppression_requirements;
    if requirements.requires_source_section && suppression.source_section.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, SOURCE_FIELD));
    }
    if requirements.requires_owner && suppression.owner.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, OWNER_FIELD));
    }
    if requirements.requires_rationale && suppression.rationale.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, RATIONALE_FIELD));
    }
    if requirements.requires_review_trigger && suppression.review_trigger.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, REVIEW_FIELD));
    }
    if requirements.requires_safer_default && suppression.safer_default.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, SAFER_DEFAULT_FIELD));
    }
}

fn append_scope_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: u32,
    suppression: &Suppression,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if suppression.scope == LINE_SCOPE || suppression.scope == NEXT_LINE_SCOPE {
        return;
    }
    diagnostics.push(Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("blanket suppression scope `{}` is not allowed", suppression.scope),
        remediation: BLANKET_SCOPE_REMEDIATION.to_owned(),
    }));
}

fn missing_field_diagnostic(policy: &PolicyEntry, source: &SourceFile, line: u32, field: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("suppression is missing required `{field}` metadata"),
        remediation: MISSING_FIELD_REMEDIATION.to_owned(),
    })
}

fn unknown_policy_diagnostic(source: &SourceFile, line: u32, policy_id: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: SUPPRESSION_POLICY_ID.to_owned(),
        source_section: String::from("docs/standards/source/devtools.adoc#enforcing-not-advising"),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("suppression references unknown policy `{policy_id}`"),
        remediation: UNKNOWN_POLICY_REMEDIATION.to_owned(),
    })
}

fn has_suppression_diagnostic(diagnostics: &[Diagnostic], path: &str, line: u32) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.path == path && diagnostic.line == Some(line) && diagnostic.is_blocking())
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}
