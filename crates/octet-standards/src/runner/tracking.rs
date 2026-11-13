//! Execution records derived from checker outcomes.

const UNSELECTED_SCOPE: &str = "unselected";
const UNSUPPORTED_SCOPE: &str = "not-implemented";
const EXTERNAL_SCOPE: &str = "external-checker";
const HUMAN_SCOPE: &str = "human-review";
const NO_SCOPE: &str = "none";

pub(super) fn build_policy_executions(
    input: &super::CheckInput,
    suppression_diagnostics: &[crate::diagnostic::Diagnostic],
    checker_diagnostics: &[crate::diagnostic::Diagnostic],
    suppressed_diagnostics: &[crate::diagnostic::Diagnostic],
) -> Vec<crate::report::PolicyExecution> {
    input
        .registry
        .policies
        .iter()
        .map(|policy| {
            if !input.selection.includes(&policy.pillar) {
                return unselected_execution(input, policy);
            }
            selected_execution(input, policy, suppression_diagnostics, checker_diagnostics, suppressed_diagnostics)
        })
        .collect()
}

fn unselected_execution(
    input: &super::CheckInput,
    policy: &crate::model::PolicyEntry,
) -> crate::report::PolicyExecution {
    let checker = super::planning::declared_checker(policy);
    crate::report::PolicyExecution {
        policy_id: policy.policy_id.clone(),
        policy_revision: input.registry.schema_version.clone(),
        checker: checker.clone(),
        checker_owner: policy.checker_owner.clone(),
        selected_scope: selected_scope(input, policy, UNSELECTED_SCOPE),
        status: crate::report::PolicyExecutionStatus::NotApplicable,
        coverage_explanation: format!("policy family `{}` was not selected for this run", policy.pillar),
        linked_diagnostics: Vec::new(),
        evidence_identity: format!("checker={checker}"),
        ownership: super::planning::policy_ownership(policy),
        required: false,
    }
}

fn selected_execution(
    input: &super::CheckInput,
    policy: &crate::model::PolicyEntry,
    suppression_diagnostics: &[crate::diagnostic::Diagnostic],
    checker_diagnostics: &[crate::diagnostic::Diagnostic],
    suppressed_diagnostics: &[crate::diagnostic::Diagnostic],
) -> crate::report::PolicyExecution {
    let plan = super::planning::coverage_plan(&input.registry, &input.sources, policy);
    let ownership = super::planning::plan_ownership(&plan);
    let linked_diagnostics =
        linked_diagnostic_identities(policy, suppression_diagnostics, checker_diagnostics, suppressed_diagnostics);
    let suppressed_count =
        suppressed_diagnostics.iter().filter(|diagnostic| diagnostic.policy_id == policy.policy_id).count();
    let (checker, status, coverage_explanation, evidence_identity, scope) = match plan {
        super::planning::CoveragePlan::Local(local) => {
            local_execution(input, policy, &local, &linked_diagnostics, suppressed_count)
        }
        super::planning::CoveragePlan::Unsupported => (
            policy.checker_id.clone(),
            crate::report::PolicyExecutionStatus::Unsupported,
            format!(
                "registry assigns checker `{}`, but no implemented local checker evaluates this policy",
                policy.checker_id
            ),
            format!("checker={}; implementation=none", policy.checker_id),
            UNSUPPORTED_SCOPE,
        ),
        super::planning::CoveragePlan::External => (
            policy.checker_id.clone(),
            crate::report::PolicyExecutionStatus::NotRun,
            format!(
                "externally owned checker `{}` is not executed by octet-standards; run it separately",
                policy.checker_id
            ),
            format!("checker={}", policy.checker_id),
            EXTERNAL_SCOPE,
        ),
        super::planning::CoveragePlan::Human => (
            super::planning::HUMAN_CHECKER_ID.to_owned(),
            crate::report::PolicyExecutionStatus::NotRun,
            "human review process required; this is not a completed local check".to_owned(),
            format!("checker={}", super::planning::HUMAN_CHECKER_ID),
            HUMAN_SCOPE,
        ),
    };
    crate::report::PolicyExecution {
        policy_id: policy.policy_id.clone(),
        policy_revision: input.registry.schema_version.clone(),
        checker,
        checker_owner: policy.checker_owner.clone(),
        selected_scope: selected_scope(input, policy, scope),
        status,
        coverage_explanation,
        linked_diagnostics,
        evidence_identity,
        ownership,
        required: super::policy_required(policy, &input.selection),
    }
}

fn local_execution(
    input: &super::CheckInput,
    policy: &crate::model::PolicyEntry,
    local: &super::planning::LocalPlan,
    linked_diagnostics: &[String],
    suppressed_count: usize,
) -> (String, crate::report::PolicyExecutionStatus, String, String, &'static str) {
    let checker = local.checker.to_owned();
    let evidence_identity = local.evidence_identity.clone();
    let Some(scope) = local.scope else {
        return (
            checker,
            crate::report::PolicyExecutionStatus::NotApplicable,
            local.unavailable_reason.clone(),
            evidence_identity,
            NO_SCOPE,
        );
    };
    if let Some(failure) = checker_failure(input, local.checker, &policy.policy_id) {
        return (
            checker,
            crate::report::PolicyExecutionStatus::ToolError,
            format!(
                "{} checker did not complete: {} ({})",
                local.checker,
                failure_kind_label(failure.kind),
                failure.message
            ),
            evidence_identity,
            scope,
        );
    }
    if linked_diagnostics.is_empty() {
        (
            checker,
            crate::report::PolicyExecutionStatus::Passed,
            format!(
                "{} checker ran on {scope} scope over {} supplied source file(s) and emitted no diagnostic",
                local.checker,
                input.sources.len()
            ),
            evidence_identity,
            scope,
        )
    } else {
        (
            checker,
            crate::report::PolicyExecutionStatus::Failed,
            format!(
                "{} checker ran on {scope} scope and emitted {} diagnostic(s) ({} suppressed by active suppressions)",
                local.checker,
                linked_diagnostics.len(),
                suppressed_count
            ),
            evidence_identity,
            scope,
        )
    }
}

fn checker_failure<'a>(
    input: &'a super::CheckInput,
    checker: &str,
    policy_id: &str,
) -> Option<&'a super::CheckerFailure> {
    input.checker_failures.iter().find(|failure| {
        failure.checker == checker
            && (failure.policy_ids.is_empty() || failure.policy_ids.iter().any(|id| id == policy_id))
    })
}

fn failure_kind_label(kind: super::CheckerFailureKind) -> &'static str {
    match kind {
        super::CheckerFailureKind::ExecutionError => "execution-error",
        super::CheckerFailureKind::Timeout => "timeout",
        super::CheckerFailureKind::ParseFailure => "parse-failure",
    }
}

fn linked_diagnostic_identities(
    policy: &crate::model::PolicyEntry,
    suppression_diagnostics: &[crate::diagnostic::Diagnostic],
    checker_diagnostics: &[crate::diagnostic::Diagnostic],
    suppressed_diagnostics: &[crate::diagnostic::Diagnostic],
) -> Vec<String> {
    suppression_diagnostics
        .iter()
        .chain(checker_diagnostics)
        .chain(suppressed_diagnostics)
        .filter(|diagnostic| diagnostic.policy_id == policy.policy_id)
        .map(diagnostic_identity)
        .collect()
}

fn diagnostic_identity(diagnostic: &crate::diagnostic::Diagnostic) -> String {
    diagnostic
        .line
        .map_or_else(|| format!("{}:?", diagnostic.path), |line| format!("{}:{}", diagnostic.path, line))
}

fn selected_scope(input: &super::CheckInput, policy: &crate::model::PolicyEntry, scope: &str) -> String {
    let family = if input.selection.includes_all() {
        "all"
    } else {
        policy.pillar.as_str()
    };
    format!("family={family}; sources={}; scope={scope}", input.sources.len())
}
