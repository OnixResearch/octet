//! Pure checker runner.

mod planning;
mod tracking;

use serde::Deserialize;
use serde::Serialize;

use crate::architecture::check_architecture;
use crate::capability_manifest::check_capability_manifest;
use crate::model::EnforcementLevel;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::proof::check_verified_proof_rails;
// r[impl octet.runner_self_check.warning_budget.reduction.standards_resource]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_report]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation]
use crate::source::SourceFile;
use crate::suppression::apply_suppressions;
use crate::suppression::validate_suppressions;

/// Classified failure from a policy checker execution.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerFailureKind {
    /// The checker process returned an execution error.
    ExecutionError,
    /// The checker exceeded its execution deadline.
    Timeout,
    /// The checker could not parse its input or output.
    ParseFailure,
}

/// A checker execution failure supplied to a run.
///
/// Failures are recorded against local checkers (`suppression`, `architecture`,
/// `capability-manifest`, `resource`, `proof-rails`). Externally owned checkers are never executed
/// by this crate, so they remain `not_run` even when a failure record is supplied.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckerFailure {
    /// Local checker name the failure belongs to.
    pub checker: String,
    /// Failure classification.
    pub kind: CheckerFailureKind,
    /// Policy IDs this failure applies to. Empty applies to every policy the checker evaluates.
    #[serde(default = "Vec::new")]
    pub policy_ids: Vec<String>,
    /// Human-readable failure detail.
    pub message: String,
}

/// Inputs for a pure checker run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckInput {
    /// Policy registry.
    pub registry: PolicyRegistry,
    /// Source files to inspect.
    pub sources: Vec<SourceFile>,
    /// Selected rule families. Empty means every family.
    pub selection: FamilySelection,
    /// Checker execution failures supplied by the caller. Empty by default.
    pub checker_failures: Vec<CheckerFailure>,
}

/// Runs pure checker logic.
///
/// Active suppressions remove the checker diagnostics they cover into
/// `suppressed_diagnostics`; suppression-validation diagnostics are never suppressed.
#[must_use]
pub fn run_checks(input: &CheckInput) -> crate::report::CheckReport {
    let suppression_report = validate_suppressions(&input.registry, &input.sources, &input.selection);
    let mut checker_diagnostics = check_architecture(&input.registry, &input.sources, &input.selection);
    let capability_manifest = check_capability_manifest(&input.registry, &input.sources, &input.selection);
    checker_diagnostics.extend(capability_manifest.diagnostics);
    checker_diagnostics.extend(crate::resource::check_resources(&input.registry, &input.sources, &input.selection));
    checker_diagnostics.extend(check_verified_proof_rails(&input.registry, &input.sources, &input.selection));
    let checker_diagnostics = apply_suppressions(checker_diagnostics, &suppression_report.active_suppressions);
    let policy_executions = tracking::build_policy_executions(
        input,
        &suppression_report.diagnostics,
        &checker_diagnostics.remaining,
        &checker_diagnostics.suppressed,
    );
    let required_policy_ids = required_policy_ids(&input.registry, &input.selection);
    let mut diagnostics = suppression_report.diagnostics;
    diagnostics.extend(checker_diagnostics.remaining);
    let evaluated_policy_ids = evaluated_policy_ids(&input.registry, &input.selection);
    let policy_evaluations = policy_evaluations(&input.registry, &input.selection);

    crate::report::CheckReport {
        diagnostics,
        active_suppressions: suppression_report.active_suppressions,
        suppressed_diagnostics: checker_diagnostics.suppressed,
        evaluated_policy_ids,
        policy_evaluations,
        capability_reports: capability_manifest.reports,
        policy_executions,
        required_policy_ids,
    }
}

fn required_policy_ids(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<String> {
    let mut policy_ids = registry
        .policies
        .iter()
        .filter(|policy| policy_required(policy, selection))
        .map(|policy| policy.policy_id.clone())
        .collect::<Vec<_>>();
    policy_ids.sort_unstable();
    policy_ids.dedup();
    policy_ids
}

fn policy_required(policy: &PolicyEntry, selection: &FamilySelection) -> bool {
    selection.includes(&policy.pillar) && policy.enforcement_level == EnforcementLevel::Hard
}

fn evaluated_policy_ids(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<String> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .map(|policy| policy.policy_id.clone())
        .collect()
}

fn policy_evaluations(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<crate::report::PolicyEvaluation> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .map(|policy| crate::report::PolicyEvaluation {
            policy_id: policy.policy_id.clone(),
            current_enforcement_mode: policy.current_enforcement_mode,
            target_enforcement_mode: policy.target_enforcement_mode,
            failure_type: policy_failure_type(policy),
        })
        .collect()
}

fn policy_failure_type(policy: &crate::model::PolicyEntry) -> String {
    policy.mechanisms.first().map_or_else(
        || match policy.target_enforcement_mode {
            crate::model::EnforcementMode::Lint => "lint".to_owned(),
            crate::model::EnforcementMode::Gate => "gate".to_owned(),
            crate::model::EnforcementMode::Structural => "structural-missing-mechanism".to_owned(),
            crate::model::EnforcementMode::Verified => "verified-proof-rail".to_owned(),
            crate::model::EnforcementMode::HumanOracle => "human-oracle".to_owned(),
        },
        |mechanism| match mechanism.kind {
            crate::model::MechanismKind::CrateBoundary => "crate-boundary".to_owned(),
            crate::model::MechanismKind::CapabilityApi => "capability-api".to_owned(),
            crate::model::MechanismKind::BoundedPrimitive => "bounded-primitive".to_owned(),
            crate::model::MechanismKind::TypeState => "type-state".to_owned(),
            crate::model::MechanismKind::Port => "port".to_owned(),
        },
    )
}
