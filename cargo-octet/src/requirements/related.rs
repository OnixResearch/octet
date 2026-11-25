use serde::Serialize;

/// Binding facts observed on one related record.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ActualBindings {
    pub(crate) record_id: String,
    pub(crate) stable_id: String,
    pub(crate) evidence_tier: String,
    pub(crate) source_ids: Vec<String>,
    pub(crate) policy_ids: Vec<String>,
    pub(crate) artifact_hashes: Vec<String>,
    pub(crate) requirement_ids: Vec<String>,
}

/// One bounded related-record explanation.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Neighbor {
    pub(crate) record_id: String,
    pub(crate) stable_id: String,
    pub(crate) depth: usize,
    pub(crate) relation: String,
    pub(crate) semantics: String,
}

pub(super) fn record_matches_clause(
    record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
    clause: &super::contract::RequirementClause,
) -> bool {
    record.requirement_ids.iter().any(|id| normalized_requirement_id(id) == clause.clause_id)
        || clause.allowed_evidence_ids.iter().any(|id| record.stable_id == *id || record.record_id == *id)
}

pub(super) fn normalized_requirement_id(value: &str) -> &str {
    value.strip_prefix("r[").and_then(|value| value.strip_suffix(']')).unwrap_or(value)
}

pub(super) fn record_satisfies_bindings(
    record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
    clause: &super::contract::RequirementClause,
) -> bool {
    let bindings = &clause.required_bindings;
    let is_source_current = bindings.source_ids.iter().all(|expected| record.source_ids.contains(expected));
    let is_policy_current = bindings.policy_ids.iter().all(|expected| record.policy_ids.contains(expected));
    let is_tool_current =
        bindings.tool_identity_hashes.iter().all(|expected| record.artifact_hashes.contains(expected));
    let is_artifact_current = bindings.artifact_hashes.iter().all(|expected| record.artifact_hashes.contains(expected));
    let is_tier_current = bindings.evidence_tier.as_ref().is_none_or(|tier| &record.evidence_tier == tier);
    let is_scope_current = bindings.checked_scope.iter().all(|scope| record.source_ids.contains(scope));
    is_source_current
        && is_policy_current
        && is_tool_current
        && is_artifact_current
        && is_tier_current
        && is_scope_current
        && !has_wrong_role_substitution(record, bindings)
}

fn has_wrong_role_substitution(
    record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
    bindings: &super::contract::RequirementBindings,
) -> bool {
    let is_hash_in_wrong_role =
        bindings.tool_identity_hashes.iter().chain(bindings.artifact_hashes.iter()).any(|expected| {
            (record.source_ids.contains(expected) && !bindings.source_ids.contains(expected))
                || (record.policy_ids.contains(expected) && !bindings.policy_ids.contains(expected))
        });
    let is_source_in_wrong_role = bindings.source_ids.iter().chain(bindings.checked_scope.iter()).any(|expected| {
        (record.policy_ids.contains(expected) && !bindings.policy_ids.contains(expected))
            || (record.artifact_hashes.contains(expected)
                && !bindings.artifact_hashes.contains(expected)
                && !bindings.tool_identity_hashes.contains(expected))
    });
    let is_policy_in_wrong_role = bindings.policy_ids.iter().any(|expected| {
        (record.source_ids.contains(expected)
            && !bindings.source_ids.contains(expected)
            && !bindings.checked_scope.contains(expected))
            || (record.artifact_hashes.contains(expected)
                && !bindings.artifact_hashes.contains(expected)
                && !bindings.tool_identity_hashes.contains(expected))
    });
    is_hash_in_wrong_role || is_source_in_wrong_role || is_policy_in_wrong_role
}

pub(super) fn record_is_tier_overclaim(
    record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
    clause: &super::contract::RequirementClause,
) -> bool {
    let is_proof_required = clause.disposition == super::DISPOSITION_FORMAL_PROOF
        || super::contract::validation::targets_formal_proof(clause);
    if !is_proof_required {
        return false;
    }
    record.evidence_tier != "formal_proof"
        || record.artifact_family.contains("effect-closure")
        || record.artifact_family.contains("reviewed-assumption")
        || record.artifact_family.contains("cargo-lock-review")
}

pub(super) fn record_is_legacy_lockfile(
    record: &valence_core::evidence_codebase_index::EvidenceIndexRecord,
    clause: &super::contract::RequirementClause,
) -> bool {
    if clause.required_bindings.binding_profile.as_deref() != Some("implementation-bound-v1") {
        return false;
    }
    record.artifact_family.contains("lockfile-only")
        || record.artifact_family.contains("cargo-lock-review-v1")
        || record.caveats.iter().any(|caveat| caveat.contains("lockfile-only"))
}

pub(super) fn record_has_failure(record: &valence_core::evidence_codebase_index::EvidenceIndexRecord) -> bool {
    record.caveats.iter().any(|caveat| is_failure_marker(caveat)) || is_failure_summary(&record.summary)
}

fn is_failure_marker(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "failed"
            | "status=failed"
            | "status: failed"
            | "status:failed"
            | "status=fail"
            | "status: fail"
            | "status:fail"
            | "outcome=failed"
            | "outcome: failed"
            | "outcome:failed"
            | "outcome=fail"
            | "outcome: fail"
            | "outcome:fail"
    )
}

fn is_failure_token(token: &str) -> bool {
    let clean_token = token.trim_end_matches(['.', ';', ',', ')']);
    matches!(
        clean_token,
        "status=failed"
            | "status=fail"
            | "status:failed"
            | "status:fail"
            | "outcome=failed"
            | "outcome=fail"
            | "outcome:failed"
            | "outcome:fail"
    )
}

fn is_failure_pair(key: &str, value: &str) -> bool {
    let clean_value = value.trim_end_matches(['.', ';', ',', ')']);
    matches!((key, clean_value), ("status:" | "outcome:", "failed" | "fail"))
}

fn is_failure_summary(summary: &str) -> bool {
    let normalized = summary.trim().to_ascii_lowercase();
    if is_failure_marker(&normalized) {
        return true;
    }
    if normalized.split([';', ',']).any(|segment| is_failure_marker(segment.trim())) {
        return true;
    }
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    tokens.iter().enumerate().any(|(idx, token)| {
        let is_negated = idx.checked_sub(1).and_then(|prev| tokens.get(prev)) == Some(&"not");
        if is_negated {
            return false;
        }
        is_failure_token(token) || tokens.get(idx + 1).is_some_and(|next| is_failure_pair(token, next))
    })
}

pub(super) fn actual_bindings(record: &valence_core::evidence_codebase_index::EvidenceIndexRecord) -> ActualBindings {
    ActualBindings {
        record_id: record.record_id.clone(),
        stable_id: record.stable_id.clone(),
        evidence_tier: record.evidence_tier.clone(),
        source_ids: record.source_ids.clone(),
        policy_ids: record.policy_ids.clone(),
        artifact_hashes: record.artifact_hashes.clone(),
        requirement_ids: record.requirement_ids.clone(),
    }
}

pub(super) struct Traversal {
    pub(super) records: Vec<Neighbor>,
    pub(super) is_truncated: bool,
}

pub(super) fn traverse(snapshot: &crate::evidence_multihop::EvidenceSnapshot, clause_id: &str) -> Traversal {
    let request = crate::evidence_multihop::ReachabilityRequest::new(clause_id)
        .with_admitted_relations(std::collections::BTreeSet::from([
            crate::evidence_multihop::RelationKind::ParentLink,
            crate::evidence_multihop::RelationKind::ProvenanceSource,
            crate::evidence_multihop::RelationKind::SharedRequirement,
            crate::evidence_multihop::RelationKind::SharedPolicy,
            crate::evidence_multihop::RelationKind::ContentIdentity,
        ]))
        .with_limits(crate::evidence_multihop::QueryLimits {
            max_depth: super::MAX_FRESHNESS_DEPTH,
            max_output: super::MAX_RELATED_RECORDS,
            ..crate::evidence_multihop::QueryLimits::default()
        });
    let report = crate::evidence_multihop::evaluate_typed_reference(snapshot, &request);
    let is_unknown_start = report
        .gaps
        .iter()
        .any(|gap| matches!(gap, crate::evidence_multihop::SnapshotGap::UnknownStart { .. }));
    let mut records = report
        .reached
        .iter()
        .filter(|reached| reached.stable_id != clause_id)
        .take(super::MAX_RELATED_RECORDS)
        .map(|reached| {
            let (relation, semantics) = match &reached.reason {
                crate::evidence_multihop::ReachReason::Origin => ("origin".to_owned(), "origin".to_owned()),
                crate::evidence_multihop::ReachReason::Typed {
                    relation, semantics, ..
                } => (format!("{relation:?}").to_ascii_lowercase(), format!("{semantics:?}").to_ascii_lowercase()),
            };
            Neighbor {
                record_id: reached.record_id.clone(),
                stable_id: reached.stable_id.clone(),
                depth: reached.depth,
                relation,
                semantics,
            }
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.depth.cmp(&right.depth).then_with(|| left.stable_id.cmp(&right.stable_id)));
    let is_truncated = !is_unknown_start
        && (report.status != crate::evidence_multihop::AnalysisStatus::Complete
            || report.output_truncated
            || report.reached.len() > super::MAX_RELATED_RECORDS);
    Traversal { records, is_truncated }
}
