use serde::Serialize;

/// Freshness report over one evidence index.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct InventoryReport {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) contract_blake3: String,
    pub(crate) index_hash: String,
    pub(crate) index_schema_version: String,
    pub(crate) snapshot_id: String,
    pub(crate) completeness: String,
    pub(crate) aggregate_result: String,
    pub(crate) clauses: Vec<ClauseInventory>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

/// One clause freshness outcome.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ClauseInventory {
    pub(crate) clause_id: String,
    pub(crate) required: bool,
    pub(crate) status: String,
    pub(crate) expected_bindings: super::contract::RequirementBindings,
    pub(crate) matched_record_ids: Vec<String>,
    pub(crate) actual_bindings: Vec<super::related::ActualBindings>,
    pub(crate) related_records: Vec<super::related::Neighbor>,
    pub(crate) diagnostics: Vec<String>,
}

pub(super) fn evaluate(
    contract: &super::contract::Definition,
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    contract_bytes: &[u8],
) -> InventoryReport {
    let mut diagnostics = super::contract::validation::validate(contract);
    let contract_blake3 = super::hash_bytes(contract_bytes);
    let has_index_errors = index.issues.iter().any(|issue| {
        issue.severity == crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity::Error
    });
    let has_index_caveats = index.issues.iter().any(|issue| {
        issue.severity == crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity::Caveat
    });
    let completeness = if index.index_hash.is_empty() {
        crate::evidence_multihop::SnapshotCompleteness::Undeclared
    } else if has_index_errors || has_index_caveats {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredPartial {
            reason: "evidence index carries issues or caveats".to_owned(),
        }
    } else {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete
    };
    let completeness_label = match &completeness {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete => "declared_complete",
        crate::evidence_multihop::SnapshotCompleteness::DeclaredPartial { .. } => "declared_partial",
        crate::evidence_multihop::SnapshotCompleteness::Undeclared => "undeclared",
    }
    .to_owned();
    let snapshot = crate::evidence_multihop::EvidenceSnapshot::from_index_declared(index, completeness);
    let clauses = contract
        .clauses
        .iter()
        .map(|clause| evaluate_clause(index, &snapshot, clause, &mut diagnostics))
        .collect::<Vec<_>>();
    let mut required_clauses = clauses.iter().filter(|clause| clause.required).peekable();
    let aggregate_result = if required_clauses.peek().is_none() {
        diagnostics.push("required inventory is empty".to_owned());
        super::STATUS_FAILED
    } else if required_clauses.clone().any(|clause| clause.status != super::STATUS_CURRENT) {
        if required_clauses.clone().any(|clause| clause.status == super::STATUS_INCOMPLETE) {
            super::STATUS_INCOMPLETE
        } else {
            super::STATUS_FAILED
        }
    } else {
        super::STATUS_CURRENT
    }
    .to_owned();
    diagnostics.sort();
    diagnostics.dedup();
    InventoryReport {
        schema_version: super::FRESHNESS_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3,
        index_hash: index.index_hash.clone(),
        index_schema_version: index.schema_version.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        completeness: completeness_label,
        aggregate_result,
        clauses,
        diagnostics,
        non_claims: super::non_claims(contract),
    }
}

pub(super) fn evaluate_clause(
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    snapshot: &crate::evidence_multihop::EvidenceSnapshot,
    clause: &super::contract::RequirementClause,
    diagnostics: &mut Vec<String>,
) -> ClauseInventory {
    if clause.disposition == super::DISPOSITION_EXCLUDED {
        return ClauseInventory {
            clause_id: clause.clause_id.clone(),
            required: clause.required,
            status: super::STATUS_EXCLUDED.to_owned(),
            expected_bindings: clause.required_bindings.clone(),
            matched_record_ids: Vec::new(),
            actual_bindings: Vec::new(),
            related_records: Vec::new(),
            diagnostics: vec!["clause is explicitly excluded from this contract".to_owned()],
        };
    }
    let related = super::related::traverse(snapshot, &clause.clause_id);
    if related.is_truncated {
        diagnostics.push(format!(
            "clause `{}` related-record traversal was truncated; coverage is incomplete",
            clause.clause_id
        ));
    }
    let candidates = index
        .records
        .iter()
        .filter(|record| super::related::record_matches_clause(record, clause))
        .collect::<Vec<_>>();
    let mut actual = candidates.iter().map(|record| super::related::actual_bindings(record)).collect::<Vec<_>>();
    actual.sort_by(|left, right| left.record_id.cmp(&right.record_id));
    let matched = candidates
        .iter()
        .filter(|record| super::related::record_satisfies_bindings(record, clause))
        .map(|record| record.record_id.clone())
        .collect::<Vec<_>>();
    let mut clause_diagnostics = Vec::new();
    let mut status = classify(index, snapshot, clause, (&candidates, &matched), &mut clause_diagnostics);
    if related.is_truncated
        || matches!(snapshot.completeness, crate::evidence_multihop::SnapshotCompleteness::Undeclared)
    {
        status = super::STATUS_INCOMPLETE;
        clause_diagnostics
            .push("freshness coverage is incomplete because the snapshot or traversal is partial".to_owned());
    }
    diagnostics
        .extend(clause_diagnostics.iter().map(|diagnostic| format!("clause `{}`: {diagnostic}", clause.clause_id)));
    ClauseInventory {
        clause_id: clause.clause_id.clone(),
        required: clause.required,
        status: status.to_owned(),
        expected_bindings: clause.required_bindings.clone(),
        matched_record_ids: matched,
        actual_bindings: actual,
        related_records: related.records,
        diagnostics: clause_diagnostics,
    }
}

pub(super) fn classify(
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    snapshot: &crate::evidence_multihop::EvidenceSnapshot,
    clause: &super::contract::RequirementClause,
    records: (&[&valence_core::evidence_codebase_index::EvidenceIndexRecord], &[String]),
    clause_diagnostics: &mut Vec<String>,
) -> &'static str {
    let (candidates, matched) = records;
    if matched.is_empty() {
        if candidates.is_empty() {
            if matches!(snapshot.completeness, crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete) {
                clause_diagnostics.push(format!(
                    "clause `{}` has no indexed evidence and the inventory is declared complete",
                    clause.clause_id
                ));
                super::STATUS_MISSING
            } else {
                clause_diagnostics.push(format!(
                    "clause `{}` has no indexed evidence and the inventory is incomplete",
                    clause.clause_id
                ));
                super::STATUS_INCOMPLETE
            }
        } else {
            clause_diagnostics.push(format!(
                "clause `{}` has evidence records but none satisfy the declared source/policy/tool bindings",
                clause.clause_id
            ));
            super::STATUS_STALE_BINDING
        }
    } else if matched.iter().any(|record_id| {
        index
            .records
            .iter()
            .any(|record| record.record_id == *record_id && super::related::record_has_failure(record))
    }) {
        clause_diagnostics.push(format!("clause `{}` has a failed indexed evidence record", clause.clause_id));
        super::STATUS_FAILED
    } else if matched.iter().any(|record_id| {
        index.records.iter().any(|record| {
            record.record_id == *record_id
                && (record.caveats.iter().any(|caveat| caveat.contains("unsupported"))
                    || record.artifact_family.contains("unsupported"))
        })
    }) {
        clause_diagnostics.push(format!("clause `{}` has unsupported indexed evidence", clause.clause_id));
        super::STATUS_UNSUPPORTED
    } else {
        super::STATUS_CURRENT
    }
}
