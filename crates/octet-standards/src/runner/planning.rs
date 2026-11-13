//! Policy ownership and applicable checker scope selection.

const LOCAL_CHECKER_ID: &str = "octet-standards";
pub(super) const HUMAN_CHECKER_ID: &str = "human-process";
const SUPPRESSION_CHECKER: &str = "suppression";
const ARCHITECTURE_CHECKER: &str = "architecture";
const CAPABILITY_MANIFEST_CHECKER: &str = "capability-manifest";
const RESOURCE_CHECKER: &str = "resource";
const PROOF_RAIL_CHECKER: &str = "proof-rails";
const SUPPRESSION_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
const BOUNDED_QUEUE_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
const SUPPRESSION_SCOPE: &str = "suppression-markers";
const RESOURCE_SCOPE: &str = "embedded-bounded-primitives";
const CAPABILITY_MANIFEST_SCOPE: &str = "capability-manifest";
const PROOF_RAIL_SCOPE: &str = "proof-rails";

/// Coverage plan for one selected policy.
pub(super) enum CoveragePlan {
    /// A local checker evaluates the policy.
    Local(LocalPlan),
    /// The registry assigns the local checker ID, but no local checker implements the policy.
    Unsupported,
    /// An external checker owns the policy and this run does not execute it.
    External,
    /// A human review process owns the policy.
    Human,
}

/// Local checker execution metadata for one policy.
pub(super) struct LocalPlan {
    /// Local checker name.
    pub(super) checker: &'static str,
    /// Applicable scope label, or `None` when no source scope applies.
    pub(super) scope: Option<&'static str>,
    /// Explanation used when `scope` is `None`.
    pub(super) unavailable_reason: String,
    /// Checker or mechanism identity for the coverage claim.
    pub(super) evidence_identity: String,
}

pub(super) fn coverage_plan(
    registry: &crate::model::PolicyRegistry,
    sources: &[crate::source::SourceFile],
    policy: &crate::model::PolicyEntry,
) -> CoveragePlan {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return CoveragePlan::Human;
    }
    if let Some(local) = local_plan(registry, sources, policy) {
        return CoveragePlan::Local(local);
    }
    if policy.checker_id == LOCAL_CHECKER_ID {
        return CoveragePlan::Unsupported;
    }
    CoveragePlan::External
}

fn local_plan(
    registry: &crate::model::PolicyRegistry,
    sources: &[crate::source::SourceFile],
    policy: &crate::model::PolicyEntry,
) -> Option<LocalPlan> {
    if policy.policy_id == SUPPRESSION_POLICY_ID {
        return Some(LocalPlan {
            checker: SUPPRESSION_CHECKER,
            scope: (!sources.is_empty()).then_some(SUPPRESSION_SCOPE),
            unavailable_reason: no_source_scope_reason(SUPPRESSION_CHECKER, sources.len()),
            evidence_identity: format!("checker={SUPPRESSION_CHECKER}"),
        });
    }
    if policy.policy_id == BOUNDED_QUEUE_POLICY_ID {
        return Some(LocalPlan {
            checker: RESOURCE_CHECKER,
            scope: (!sources.is_empty()).then_some(RESOURCE_SCOPE),
            unavailable_reason: no_source_scope_reason(RESOURCE_CHECKER, sources.len()),
            evidence_identity: format!("checker={RESOURCE_CHECKER}; mechanism=bounded-primitive"),
        });
    }
    if crate::architecture::ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str()) {
        if crate::capability_manifest::checker_applies(policy) {
            return Some(LocalPlan {
                checker: CAPABILITY_MANIFEST_CHECKER,
                scope: (!sources.is_empty()).then_some(CAPABILITY_MANIFEST_SCOPE),
                unavailable_reason: no_source_scope_reason(CAPABILITY_MANIFEST_CHECKER, sources.len()),
                evidence_identity: format!("checker={CAPABILITY_MANIFEST_CHECKER}; mechanism=capability-api"),
            });
        }
        return Some(LocalPlan {
            checker: ARCHITECTURE_CHECKER,
            scope: crate::architecture::applicable_scope(registry, sources, policy),
            unavailable_reason: format!(
                "no applicable architecture source scope among {} supplied source file(s)",
                sources.len()
            ),
            evidence_identity: architecture_evidence_identity(policy),
        });
    }
    if crate::proof::policy_is_verified(policy) {
        return Some(LocalPlan {
            checker: PROOF_RAIL_CHECKER,
            scope: crate::proof::rail_checker_applies(policy, sources).then_some(PROOF_RAIL_SCOPE),
            unavailable_reason: format!("no declared proof rail matches the {} supplied source file(s)", sources.len()),
            evidence_identity: format!("checker={PROOF_RAIL_CHECKER}; rails={}", policy.proof_rails.len()),
        });
    }
    None
}

fn no_source_scope_reason(checker: &str, source_count: usize) -> String {
    format!("no applicable {checker} source scope among {source_count} supplied source file(s)")
}

fn architecture_evidence_identity(policy: &crate::model::PolicyEntry) -> String {
    if policy.mechanisms.is_empty() {
        return format!("checker={ARCHITECTURE_CHECKER}");
    }
    let mechanisms = policy.mechanisms.iter().map(|mechanism| mechanism_kind_label(mechanism.kind)).collect::<Vec<_>>();
    format!("checker={ARCHITECTURE_CHECKER}; mechanism={}", mechanisms.join("+"))
}

fn mechanism_kind_label(kind: crate::model::MechanismKind) -> &'static str {
    match kind {
        crate::model::MechanismKind::CrateBoundary => "crate-boundary",
        crate::model::MechanismKind::CapabilityApi => "capability-api",
        crate::model::MechanismKind::BoundedPrimitive => "bounded-primitive",
        crate::model::MechanismKind::TypeState => "type-state",
        crate::model::MechanismKind::Port => "port",
    }
}

pub(super) fn declared_checker(policy: &crate::model::PolicyEntry) -> String {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return HUMAN_CHECKER_ID.to_owned();
    }
    if policy.policy_id == BOUNDED_QUEUE_POLICY_ID {
        return RESOURCE_CHECKER.to_owned();
    }
    if crate::architecture::ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str())
        && crate::capability_manifest::checker_applies(policy)
    {
        return CAPABILITY_MANIFEST_CHECKER.to_owned();
    }
    if crate::architecture::ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str()) {
        return ARCHITECTURE_CHECKER.to_owned();
    }
    if crate::proof::policy_is_verified(policy) {
        return PROOF_RAIL_CHECKER.to_owned();
    }
    policy.checker_id.clone()
}

pub(super) fn policy_ownership(policy: &crate::model::PolicyEntry) -> crate::report::PolicyExecutionOwnership {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return crate::report::PolicyExecutionOwnership::HumanProcess;
    }
    // A registry checker ID can lag the local runner: the resource checker evaluates
    // `bounded-queues` and the proof-rail checker evaluates verified policies with rails even when
    // the registry still names an external owner. Actual local execution wins over the declaration.
    if policy.checker_id == LOCAL_CHECKER_ID
        || policy.policy_id == BOUNDED_QUEUE_POLICY_ID
        || crate::proof::policy_is_verified(policy)
    {
        return crate::report::PolicyExecutionOwnership::Local;
    }
    crate::report::PolicyExecutionOwnership::External
}

pub(super) fn plan_ownership(plan: &CoveragePlan) -> crate::report::PolicyExecutionOwnership {
    match plan {
        CoveragePlan::Local(_) | CoveragePlan::Unsupported => crate::report::PolicyExecutionOwnership::Local,
        CoveragePlan::External => crate::report::PolicyExecutionOwnership::External,
        CoveragePlan::Human => crate::report::PolicyExecutionOwnership::HumanProcess,
    }
}
