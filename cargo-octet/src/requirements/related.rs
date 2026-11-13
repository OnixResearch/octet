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
        || clause
            .allowed_evidence_ids
            .iter()
            .any(|id| record.stable_id == *id || record.record_id == *id || record.summary.contains(id))
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
    let is_tool_current = bindings
        .tool_identity_hashes
        .iter()
        .all(|expected| record.artifact_hashes.contains(expected) || record.source_ids.contains(expected));
    let is_artifact_current = bindings.artifact_hashes.iter().all(|expected| record.artifact_hashes.contains(expected));
    let is_tier_current = bindings.evidence_tier.as_ref().is_none_or(|tier| &record.evidence_tier == tier);
    let is_scope_current =
        bindings.checked_scope.is_empty() || bindings.checked_scope.iter().any(|scope| record.summary.contains(scope));
    is_source_current
        && is_policy_current
        && is_tool_current
        && is_artifact_current
        && is_tier_current
        && is_scope_current
}

pub(super) fn record_has_failure(record: &valence_core::evidence_codebase_index::EvidenceIndexRecord) -> bool {
    let text = format!("{} {}", record.summary, record.caveats.join(" ")).to_ascii_lowercase();
    text.contains("status=failed")
        || text.contains("status: failed")
        || text.contains("status=fail")
        || text.contains("status: fail")
        || text.contains("rejected")
        || text.contains("invalid")
        || text.contains("missing required")
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
