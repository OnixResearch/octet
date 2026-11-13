pub(in crate::requirements) struct Context<'a> {
    pub(in crate::requirements) contract: &'a super::super::contract::Definition,
    pub(in crate::requirements) evidence: &'a super::super::evidence::Bundle,
    pub(in crate::requirements) source_root: &'a std::path::Path,
    pub(in crate::requirements) assumptions: &'a std::collections::BTreeMap<String, super::decision::AssumptionOutcome>,
}

pub(in crate::requirements) fn evaluate(
    evaluator: &Context<'_>,
    clause: &super::super::contract::RequirementClause,
    diagnostics: &mut Vec<String>,
) -> super::decision::ClauseOutcome {
    let source_root = evaluator.source_root;
    let mut clause_diagnostics = Vec::new();
    if clause.source_bindings.is_empty() && clause.disposition != super::super::DISPOSITION_EXCLUDED {
        clause_diagnostics.push("source scope is empty; no evidence can establish coverage".to_owned());
    }
    validate_sources(clause, source_root, &mut clause_diagnostics);
    let (status, evidence_ids) = evaluate_disposition(evaluator, clause, &mut clause_diagnostics);
    if clause.required && status != super::super::STATUS_PASSED && status != super::super::STATUS_ASSUMPTION_ACCEPTED {
        clause_diagnostics.push(format!("required clause `{}` did not pass", clause.clause_id));
    }
    clause_diagnostics.sort();
    clause_diagnostics.dedup();
    if !clause_diagnostics.is_empty() && status == super::super::STATUS_PASSED {
        // A passing status with diagnostics is contradictory; fail closed.
        let status = super::super::STATUS_FAILED;
        return super::decision::ClauseOutcome {
            clause_id: clause.clause_id.clone(),
            required: clause.required,
            disposition: clause.disposition.clone(),
            status: status.to_owned(),
            evidence_ids,
            source_bindings: clause.source_bindings.clone(),
            diagnostics: clause_diagnostics,
        };
    }
    if status == super::super::STATUS_FAILED && clause_diagnostics.is_empty() {
        clause_diagnostics.push("clause failed without a diagnostic".to_owned());
    }
    if clause.required && status == super::super::STATUS_EXCLUDED {
        clause_diagnostics.push("required clause cannot be satisfied by an exclusion".to_owned());
    }
    diagnostics
        .extend(clause_diagnostics.iter().map(|diagnostic| format!("clause `{}`: {diagnostic}", clause.clause_id)));
    super::decision::ClauseOutcome {
        clause_id: clause.clause_id.clone(),
        required: clause.required,
        disposition: clause.disposition.clone(),
        status,
        evidence_ids,
        source_bindings: clause.source_bindings.clone(),
        diagnostics: clause_diagnostics,
    }
}

pub(in crate::requirements) fn empirical(
    evidence: &super::super::evidence::Bundle,
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    let evidence_ids = evidence
        .empirical_checks
        .iter()
        .filter(|check| clause.allowed_evidence_ids.contains(&check.evidence_id))
        .filter_map(|check| {
            let check_diagnostics = validate_empirical_check(check, source_root);
            if check_diagnostics.is_empty() {
                Some(check.evidence_id.clone())
            } else {
                diagnostics.extend(check_diagnostics);
                None
            }
        })
        .collect::<Vec<_>>();
    if evidence_ids.is_empty() && diagnostics.is_empty() {
        diagnostics.push(format!(
            "clause `{}` has no retained empirical evidence among allowed evidence ids",
            clause.clause_id
        ));
    }
    let status = if evidence_ids.is_empty() {
        super::super::STATUS_MISSING
    } else if clause.required_evidence_classes.iter().any(|class| class == "formal_proof") {
        diagnostics.push(format!(
            "empirical evidence cannot satisfy the formal-proof requirement for clause `{}`",
            clause.clause_id
        ));
        super::super::STATUS_FAILED
    } else {
        super::super::STATUS_PASSED
    };
    (status.to_owned(), evidence_ids)
}

pub(in crate::requirements) fn evaluate_disposition(
    evaluator: &Context<'_>,
    clause: &super::super::contract::RequirementClause,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    match clause.disposition.as_str() {
        super::super::DISPOSITION_EXCLUDED => {
            let matching =
                evaluator.contract.exclusions.iter().find(|exclusion| exclusion.clause_id == clause.clause_id);
            if matching.is_none() {
                diagnostics.push("excluded clause has no exclusion record".to_owned());
            }
            (super::super::STATUS_EXCLUDED.to_owned(), Vec::new())
        }
        super::super::DISPOSITION_UNRESOLVED => {
            diagnostics.push("required clause remains unresolved".to_owned());
            (super::super::STATUS_UNRESOLVED.to_owned(), Vec::new())
        }
        super::super::DISPOSITION_FORMAL_PROOF => {
            super::formal::evaluate(evaluator.evidence, clause, evaluator.source_root, diagnostics)
        }
        super::super::DISPOSITION_EMPIRICAL_CHECK => {
            empirical(evaluator.evidence, clause, evaluator.source_root, diagnostics)
        }
        super::super::DISPOSITION_ACCEPTED_ASSUMPTION => match &clause.assumption_id {
            None => {
                diagnostics.push("assumption clause does not name an assumption".to_owned());
                (super::super::STATUS_FAILED.to_owned(), Vec::new())
            }
            Some(assumption_id) => match evaluator.assumptions.get(assumption_id) {
                Some(assumption) if assumption.status == super::super::STATUS_ASSUMPTION_ACCEPTED => {
                    (super::super::STATUS_ASSUMPTION_ACCEPTED.to_owned(), assumption.supporting_evidence_ids.clone())
                }
                Some(assumption) => {
                    diagnostics.push(format!("assumption `{assumption_id}` is not accepted: {}", assumption.status));
                    (super::super::STATUS_FAILED.to_owned(), assumption.supporting_evidence_ids.clone())
                }
                None => {
                    diagnostics.push(format!("assumption `{assumption_id}` has no decision"));
                    (super::super::STATUS_FAILED.to_owned(), Vec::new())
                }
            },
        },
        _ => {
            diagnostics.push(format!("unsupported disposition `{}`", clause.disposition));
            (super::super::STATUS_UNSUPPORTED.to_owned(), Vec::new())
        }
    }
}

pub(in crate::requirements) fn validate_empirical_check(
    check: &super::super::evidence::EmpiricalCheck,
    source_root: &std::path::Path,
) -> Vec<String> {
    let mut check_diagnostics = Vec::new();
    if check.evidence_class != super::super::DISPOSITION_EMPIRICAL_CHECK {
        check_diagnostics.push(format!(
            "evidence `{}` class `{}` is not an empirical check",
            check.evidence_id, check.evidence_class
        ));
    }
    if check.status != super::super::CHECK_STATUS_PASSED {
        check_diagnostics.push(format!("empirical evidence `{}` status is `{}`", check.evidence_id, check.status));
    }
    if check.command_argv.is_empty() {
        check_diagnostics.push(format!("empirical evidence `{}` has no command argv", check.evidence_id));
    }
    if check.checked_scope.is_empty() {
        check_diagnostics.push(format!("empirical evidence `{}` has no checked scope", check.evidence_id));
    }
    validate_empirical_sources(check, source_root, &mut check_diagnostics);
    if let (Some(artifact), Some(expected)) = (&check.artifact, &check.artifact_blake3) {
        match std::fs::read(super::super::resolve_path(source_root, artifact)) {
            Ok(bytes) => {
                let actual = super::super::hash_bytes(&bytes);
                if &actual != expected {
                    check_diagnostics
                        .push(format!("empirical evidence `{}` artifact `{artifact}` changed", check.evidence_id));
                }
            }
            Err(error) => check_diagnostics.push(format!(
                "empirical evidence `{}` artifact `{artifact}` is unreadable: {error}",
                check.evidence_id
            )),
        }
    } else {
        check_diagnostics.push(format!("empirical evidence `{}` has no retained artifact", check.evidence_id));
    }
    check_diagnostics
}

pub(in crate::requirements) fn validate_sources(
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    clause_diagnostics: &mut Vec<String>,
) {
    for binding in &clause.source_bindings {
        let path = super::super::resolve_path(source_root, &binding.locator);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let actual = super::super::hash_bytes(&bytes);
                if actual != binding.blake3 {
                    clause_diagnostics.push(format!(
                        "source `{}` changed: expected {}, actual {actual}",
                        binding.locator, binding.blake3
                    ));
                }
            }
            Err(error) => clause_diagnostics.push(format!("source `{}` is unreadable: {error}", binding.locator)),
        }
    }
}

fn validate_empirical_sources(
    check: &super::super::evidence::EmpiricalCheck,
    source_root: &std::path::Path,
    check_diagnostics: &mut Vec<String>,
) {
    for binding in &check.source_bindings {
        match std::fs::read(super::super::resolve_path(source_root, &binding.locator)) {
            Ok(bytes) => {
                let actual = super::super::hash_bytes(&bytes);
                if actual != binding.blake3 {
                    check_diagnostics.push(format!(
                        "empirical evidence `{}` source `{}` changed",
                        check.evidence_id, binding.locator
                    ));
                }
            }
            Err(error) => check_diagnostics.push(format!(
                "empirical evidence `{}` source `{}` is unreadable: {error}",
                check.evidence_id, binding.locator
            )),
        }
    }
}
