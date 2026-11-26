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

fn index_completeness(
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
) -> crate::evidence_multihop::SnapshotCompleteness {
    let has_index_errors = index.issues.iter().any(|issue| {
        issue.severity == crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity::Error
    });
    let has_index_caveats = index.issues.iter().any(|issue| {
        issue.severity == crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity::Caveat
    });
    if index.index_hash.is_empty() {
        crate::evidence_multihop::SnapshotCompleteness::Undeclared
    } else if has_index_errors || has_index_caveats {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredPartial {
            reason: "evidence index carries issues or caveats".to_owned(),
        }
    } else {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete
    }
}

fn verify_live_reviewed_specification(
    contract: &super::contract::Definition,
    source_root: Option<&std::path::Path>,
    diagnostics: &mut Vec<String>,
) -> bool {
    let Some(root) = source_root else {
        return false;
    };
    let spec_path = super::resolve_path(root, &contract.reviewed_specification.path);
    match crate::envelope::read_regular_non_symlink_file(&spec_path) {
        Ok(bytes) => {
            let actual = super::hash_bytes(&bytes);
            if actual != contract.reviewed_specification.blake3 {
                diagnostics.push(format!(
                    "reviewed specification `{}` changed: expected {}, actual {actual}",
                    contract.reviewed_specification.path, contract.reviewed_specification.blake3
                ));
            }
            false
        }
        Err(error) => {
            diagnostics.push(format!(
                "reviewed specification `{}` is missing, symlinked, or unreadable: {error}",
                contract.reviewed_specification.path
            ));
            true
        }
    }
}

pub(super) fn evaluate(
    contract: &super::contract::Definition,
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    contract_bytes: &[u8],
    source_root: Option<&std::path::Path>,
) -> InventoryReport {
    let mut diagnostics = super::contract::validation::validate(contract);
    let is_live_spec_incomplete = verify_live_reviewed_specification(contract, source_root, &mut diagnostics);
    let contract_blake3 = super::hash_bytes(contract_bytes);
    let completeness = index_completeness(index);
    let snapshot = crate::evidence_multihop::EvidenceSnapshot::from_index_declared(index, completeness.clone());
    let clauses = contract
        .clauses
        .iter()
        .map(|clause| evaluate_clause(index, &snapshot, clause, source_root, &mut diagnostics))
        .collect::<Vec<_>>();
    let mut required_clauses = clauses.iter().filter(|clause| clause.required).peekable();
    let aggregate_result = if required_clauses.peek().is_none() {
        diagnostics.push("required inventory is empty".to_owned());
        super::STATUS_FAILED
    } else if is_live_spec_incomplete
        || required_clauses.clone().any(|clause| clause.status == super::STATUS_INCOMPLETE)
    {
        super::STATUS_INCOMPLETE
    } else if !diagnostics.is_empty() || required_clauses.clone().any(|clause| clause.status != super::STATUS_CURRENT) {
        super::STATUS_FAILED
    } else {
        super::STATUS_CURRENT
    }
    .to_owned();
    let completeness_label = match &completeness {
        crate::evidence_multihop::SnapshotCompleteness::DeclaredComplete => {
            if source_root.is_some() && aggregate_result == super::STATUS_INCOMPLETE {
                "live_incomplete"
            } else if source_root.is_some() {
                "live_verified"
            } else {
                "declared_complete"
            }
        }
        crate::evidence_multihop::SnapshotCompleteness::DeclaredPartial { .. } => "declared_partial",
        crate::evidence_multihop::SnapshotCompleteness::Undeclared => "undeclared",
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
    source_root: Option<&std::path::Path>,
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
    if let Some(root) = source_root
        && let Some(live_status) =
            super::evaluation::source::verify_clause_live_sources(clause, root, &mut clause_diagnostics)
        && status != super::STATUS_FAILED
        && (status == super::STATUS_CURRENT
            || live_status == super::STATUS_INCOMPLETE
            || live_status == super::STATUS_UNSUPPORTED)
    {
        status = live_status;
    }
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

fn is_tier_overclaim_only(
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    candidates: &[&valence_core::evidence_codebase_index::EvidenceIndexRecord],
    matched: &[String],
    clause: &super::contract::RequirementClause,
) -> bool {
    (clause.disposition != super::DISPOSITION_FORMAL_PROOF && super::contract::validation::targets_formal_proof(clause))
        || (candidates.iter().any(|record| super::related::record_is_tier_overclaim(record, clause))
            && matched.iter().all(|id| {
                index
                    .records
                    .iter()
                    .any(|r| &r.record_id == id && super::related::record_is_tier_overclaim(r, clause))
            }))
}

pub(super) fn classify(
    index: &crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex,
    snapshot: &crate::evidence_multihop::EvidenceSnapshot,
    clause: &super::contract::RequirementClause,
    records: (&[&valence_core::evidence_codebase_index::EvidenceIndexRecord], &[String]),
    clause_diagnostics: &mut Vec<String>,
) -> &'static str {
    let (candidates, matched) = records;
    if clause.disposition == super::DISPOSITION_UNRESOLVED {
        clause_diagnostics.push("required clause remains unresolved".to_owned());
        return super::STATUS_UNRESOLVED;
    }
    if is_tier_overclaim_only(index, candidates, matched, clause) {
        clause_diagnostics.push(format!(
            "clause `{}` cannot promote reviewed assumption or policy-check evidence to formal_proof",
            clause.clause_id
        ));
        return super::STATUS_UNSUPPORTED;
    }
    if candidates.iter().any(|record| super::related::record_is_legacy_lockfile(record, clause)) {
        clause_diagnostics.push(format!(
            "clause `{}` rejects historical lockfile-only summary under `implementation-bound-v1`",
            clause.clause_id
        ));
        return super::STATUS_UNSUPPORTED;
    }
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
