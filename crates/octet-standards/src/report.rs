//! Check report and stable renderers.

mod presentation;

pub use presentation::render_json;
pub use presentation::render_markdown;
use serde::Deserialize;
use serde::Serialize;

use crate::capability_manifest::CapabilityReport;
use crate::diagnostic::Diagnostic;
use crate::model::EnforcementMode;
use crate::suppression::Suppression;

/// Actual execution outcome for one policy checker.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyExecutionStatus {
    /// The checker ran on an applicable scope and emitted no diagnostic for the policy.
    Passed,
    /// The checker ran on an applicable scope and emitted at least one diagnostic for the policy.
    Failed,
    /// The checker was not executed for this policy.
    NotRun,
    /// No implemented checker evaluates this policy.
    Unsupported,
    /// The policy family was not selected or no applicable source scope exists.
    NotApplicable,
    /// The checker failed to execute, for example after a timeout or unparseable input.
    ToolError,
}

/// Ownership of the checker responsible for a policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyExecutionOwnership {
    /// Implemented by a local pure checker in this crate.
    Local,
    /// Owned by an external tool that this run does not execute.
    External,
    /// Requires a human review process instead of a machine check.
    HumanProcess,
}

/// Evidence that one selected policy was actually executed by a named checker.
///
/// Unlike [`CheckReport::evaluated_policy_ids`], this record is derived from checker execution
/// rather than registry selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyExecution {
    /// Stable policy ID.
    pub policy_id: String,
    /// Registry revision (`schema_version`) that supplied the policy definition.
    ///
    /// The registry schema version is the revision available on the policy entry; it changes when
    /// the exported policy definitions change.
    pub policy_revision: String,
    /// Checker that evaluated or owns the policy.
    pub checker: String,
    /// Declared checker owner for the policy.
    pub checker_owner: String,
    /// Deterministic description of the selected family and source scope.
    pub selected_scope: String,
    /// Actual execution outcome.
    pub status: PolicyExecutionStatus,
    /// Why the status was assigned.
    pub coverage_explanation: String,
    /// Identities of diagnostics the checker emitted for this policy.
    pub linked_diagnostics: Vec<String>,
    /// Mechanism or checker identity that produced the coverage claim.
    pub evidence_identity: String,
    /// Whether the checker is local, external, or human-process owned.
    pub ownership: PolicyExecutionOwnership,
    /// True when the policy was selected and carries a hard enforcement level.
    pub required: bool,
}

/// Stable report from a Tiger Style standards checker run.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckReport {
    /// Diagnostics emitted by the run.
    pub diagnostics: Vec<Diagnostic>,
    /// Active suppressions accepted by the run.
    pub active_suppressions: Vec<Suppression>,
    /// Checker diagnostics an active suppression covers; they do not block.
    #[serde(default)]
    pub suppressed_diagnostics: Vec<Diagnostic>,
    /// Selected policy IDs from registry metadata.
    ///
    /// This list is registry selection metadata, not execution proof. Read
    /// [`Self::policy_executions`] for what a checker actually executed.
    pub evaluated_policy_ids: Vec<String>,
    /// Enforcement-mode metadata for selected policies.
    ///
    /// This list is migration metadata preserved for compatibility; it does not prove that a
    /// checker ran. Read [`Self::policy_executions`] for actual coverage.
    #[serde(default)]
    pub policy_evaluations: Vec<PolicyEvaluation>,
    /// Capability manifest reports emitted by architecture checks.
    #[serde(default)]
    pub capability_reports: Vec<CapabilityReport>,
    /// Actual execution coverage for every policy in the registry.
    #[serde(default = "Vec::new")]
    pub policy_executions: Vec<PolicyExecution>,
    /// Selected hard policy IDs whose execution coverage this report must account for.
    #[serde(default = "Vec::new")]
    pub required_policy_ids: Vec<String>,
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

    /// Returns true when every selected hard policy has an accounted-for execution outcome.
    ///
    /// Local checkers must report `passed`, `failed`, or `not_applicable`. External and
    /// human-process checkers must report `not_run`, because this run cannot complete them. A
    /// local `not_run`, `unsupported`, or `tool_error` outcome makes coverage incomplete.
    #[must_use]
    pub fn coverage_complete(&self) -> bool {
        self.required_policy_ids.iter().all(|policy_id| {
            self.policy_executions
                .iter()
                .filter(|execution| &execution.policy_id == policy_id)
                .any(PolicyExecution::is_accounted_for)
        }) && self
            .policy_executions
            .iter()
            .filter(|execution| execution.required)
            .all(PolicyExecution::is_accounted_for)
    }

    /// Returns true when strict policy coverage passes this report.
    ///
    /// Strict mode requires no blocking diagnostics, complete execution coverage for selected hard
    /// policies, and no selected hard policy that is left to an external or human-process checker
    /// with only a `not_run` outcome. Local `not_applicable` outcomes remain eligible because they
    /// carry an explicit reason.
    #[must_use]
    pub fn strict_passing(&self) -> bool {
        !self.has_blocking_diagnostics() && self.coverage_complete() && !self.required_checker_not_run()
    }

    fn required_checker_not_run(&self) -> bool {
        self.policy_executions.iter().any(|execution| {
            execution.required
                && execution.ownership != PolicyExecutionOwnership::Local
                && execution.status == PolicyExecutionStatus::NotRun
        })
    }
}

impl PolicyExecution {
    /// Returns true when this record proves a completed local checker outcome.
    #[must_use]
    pub fn is_completed_local_check(&self) -> bool {
        self.ownership == PolicyExecutionOwnership::Local
            && matches!(self.status, PolicyExecutionStatus::Passed | PolicyExecutionStatus::Failed)
    }

    /// Returns true when this execution record accounts for a selected hard policy.
    #[must_use]
    pub fn is_accounted_for(&self) -> bool {
        match self.ownership {
            PolicyExecutionOwnership::Local => matches!(
                self.status,
                PolicyExecutionStatus::Passed | PolicyExecutionStatus::Failed | PolicyExecutionStatus::NotApplicable
            ),
            PolicyExecutionOwnership::External | PolicyExecutionOwnership::HumanProcess => {
                self.status == PolicyExecutionStatus::NotRun
            }
        }
    }
}
