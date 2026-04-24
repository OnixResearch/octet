//! Pure checker runner.

use crate::architecture::check_architecture;
use crate::model::FamilySelection;
use crate::model::PolicyRegistry;
use crate::report::CheckReport;
use crate::resource::check_resources;
use crate::source::SourceFile;
use crate::suppression::validate_suppressions;

/// Inputs for a pure checker run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckInput {
    /// Policy registry.
    pub registry: PolicyRegistry,
    /// Source files to inspect.
    pub sources: Vec<SourceFile>,
    /// Selected rule families. Empty means every family.
    pub selection: FamilySelection,
}

/// Runs pure checker logic.
#[must_use]
pub fn run_checks(input: &CheckInput) -> CheckReport {
    let suppression_report = validate_suppressions(&input.registry, &input.sources, &input.selection);
    let mut diagnostics = suppression_report.diagnostics;
    diagnostics.extend(check_architecture(&input.registry, &input.sources, &input.selection));
    diagnostics.extend(check_resources(&input.registry, &input.sources, &input.selection));
    let evaluated_policy_ids = evaluated_policy_ids(&input.registry, &input.selection);

    CheckReport {
        diagnostics,
        active_suppressions: suppression_report.active_suppressions,
        evaluated_policy_ids,
    }
}

fn evaluated_policy_ids(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<String> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .map(|policy| policy.policy_id.clone())
        .collect()
}
