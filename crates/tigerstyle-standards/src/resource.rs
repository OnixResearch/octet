//! Resource-bound checks for embedded Tiger Style policies.

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::source::SourceFile;

const EMBEDDED_FAMILY: &str = "embedded";
const BOUNDED_QUEUE_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const UNBOUNDED_QUEUE_PATTERNS: &[&str] = &[
    "unbounded_channel(",
    "mpsc::unbounded_channel(",
    "crossbeam_channel::unbounded(",
    "std::sync::mpsc::channel",
];

/// Runs resource-bound policy checks over source files.
#[must_use]
pub fn check_resources(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    if !selection.includes(EMBEDDED_FAMILY) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    check_unbounded_queue_patterns(registry, sources, &mut diagnostics);
    diagnostics
}

fn check_unbounded_queue_patterns(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(BOUNDED_QUEUE_POLICY_ID) else {
        return;
    };
    for source in sources {
        append_source_unbounded_queue_diagnostics(policy, source, diagnostics);
    }
}

fn append_source_unbounded_queue_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (line_index, line) in source.text.lines().enumerate() {
        let Some(pattern) = matching_unbounded_queue_pattern(line) else {
            continue;
        };
        diagnostics.push(policy_diagnostic(policy, source, line_index, pattern));
    }
}

fn matching_unbounded_queue_pattern(line: &str) -> Option<&'static str> {
    UNBOUNDED_QUEUE_PATTERNS.iter().copied().find(|pattern| line.contains(pattern))
}

fn policy_diagnostic(policy: &PolicyEntry, source: &SourceFile, line_index: usize, pattern: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line_number_from_index(line_index)),
        severity: policy.default_severity,
        message: format!("unbounded queue/channel pattern `{pattern}` is not allowed"),
        remediation: "use a bounded queue/channel with explicit capacity and backpressure behavior".to_owned(),
    })
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}
