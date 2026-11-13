pub(in crate::requirements) fn validate(contract: &super::Definition) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if contract.schema_version != super::super::CONTRACT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "requirement contract field `schema_version` expected `{CONTRACT_SCHEMA_VERSION}` but got `{}`",
            contract.schema_version,
            CONTRACT_SCHEMA_VERSION = super::super::CONTRACT_SCHEMA_VERSION
        ));
    }
    push_required(&mut diagnostics, "contract_id", &contract.contract_id);
    push_required(&mut diagnostics, "reviewed_specification.path", &contract.reviewed_specification.path);
    if !is_blake3_hex(&contract.reviewed_specification.blake3) {
        diagnostics.push("reviewed_specification.blake3 must be a lowercase BLAKE3 hex digest".to_owned());
    }
    let mut clause_ids = std::collections::BTreeSet::new();
    for clause in &contract.clauses {
        validate_clause(clause, &mut clause_ids, &mut diagnostics);
    }
    if clause_ids.is_empty() {
        diagnostics.push("requirement contract must declare at least one clause".to_owned());
    }
    if !contract.clauses.iter().any(|clause| clause.required) {
        diagnostics.push("requirement contract has an empty required inventory".to_owned());
    }
    validate_relationships(contract, &clause_ids, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(super) fn push_required(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("missing required requirement-contract field `{field}`"));
    }
}

pub(super) fn is_blake3_hex(value: &str) -> bool {
    value.len() == super::super::BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_clause<'a>(
    clause: &'a super::RequirementClause,
    clause_ids: &mut std::collections::BTreeSet<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if !clause_ids.insert(clause.clause_id.as_str()) {
        diagnostics.push(format!("duplicate requirement clause id `{}`", clause.clause_id));
    }
    push_required(diagnostics, "clause_id", &clause.clause_id);
    push_required(diagnostics, "statement", &clause.statement);
    if !matches!(
        clause.disposition.as_str(),
        super::super::DISPOSITION_FORMAL_PROOF
            | super::super::DISPOSITION_EMPIRICAL_CHECK
            | super::super::DISPOSITION_ACCEPTED_ASSUMPTION
            | super::super::DISPOSITION_UNRESOLVED
            | super::super::DISPOSITION_EXCLUDED
    ) {
        diagnostics.push(format!(
            "requirement clause `{}` has unsupported disposition `{}`",
            clause.clause_id, clause.disposition
        ));
    }
    if clause.disposition == super::super::DISPOSITION_EXCLUDED {
        if clause.exclusion_reason.as_deref().is_none_or(str::is_empty) {
            diagnostics
                .push(format!("excluded requirement clause `{}` must declare exclusion_reason", clause.clause_id));
        }
    } else if clause.checked_scope.is_empty() {
        diagnostics.push(format!("requirement clause `{}` must declare checked_scope", clause.clause_id));
    }
    if clause.disposition == super::super::DISPOSITION_FORMAL_PROOF
        && clause.expected_evidence_tier != super::super::DISPOSITION_FORMAL_PROOF
    {
        diagnostics
            .push(format!("formal-proof clause `{}` must require evidence tier `formal_proof`", clause.clause_id));
    }
    if clause.disposition == super::super::DISPOSITION_ACCEPTED_ASSUMPTION && clause.assumption_id.is_none() {
        diagnostics.push(format!("assumption clause `{}` must name an assumption_id", clause.clause_id));
    }
    if clause.source_bindings.is_empty() && clause.disposition != super::super::DISPOSITION_EXCLUDED {
        diagnostics.push(format!(
            "requirement clause `{}` has an empty source scope; an unexamined scope cannot establish coverage",
            clause.clause_id
        ));
    }
    if clause.allowed_evidence_ids.is_empty() && clause.disposition != super::super::DISPOSITION_EXCLUDED {
        diagnostics.push(format!("requirement clause `{}` must name allowed_evidence_ids", clause.clause_id));
    }
    for binding in &clause.source_bindings {
        if !is_blake3_hex(&binding.blake3) {
            diagnostics.push(format!(
                "requirement clause `{}` source binding `{}` must carry a BLAKE3 digest",
                clause.clause_id, binding.locator
            ));
        }
    }
}

pub(super) fn validate_relationships(
    contract: &super::Definition,
    clause_ids: &std::collections::BTreeSet<&str>,
    diagnostics: &mut Vec<String>,
) {
    let assumption_ids = contract
        .assumptions
        .iter()
        .map(|assumption| assumption.assumption_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for assumption in &contract.assumptions {
        push_required(diagnostics, "assumption_id", &assumption.assumption_id);
        push_required(diagnostics, "assumption.identity", &assumption.identity);
        push_required(diagnostics, "assumption.trusted_boundary_identity", &assumption.trusted_boundary_identity);
        for clause_id in &assumption.affected_clause_ids {
            if !clause_ids.contains(clause_id.as_str()) {
                diagnostics.push(format!(
                    "assumption `{}` names unknown affected clause `{clause_id}`",
                    assumption.assumption_id
                ));
            }
        }
        if assumption.required_empirical_evidence.is_empty() {
            diagnostics.push(format!(
                "assumption `{}` must require at least one empirical evidence id",
                assumption.assumption_id
            ));
        }
    }
    for clause in &contract.clauses {
        if clause.disposition == super::super::DISPOSITION_ACCEPTED_ASSUMPTION
            && let Some(assumption_id) = &clause.assumption_id
            && !assumption_ids.contains(assumption_id.as_str())
        {
            diagnostics
                .push(format!("requirement clause `{}` names unknown assumption `{assumption_id}`", clause.clause_id));
        }
    }
    let exclusion_clause_ids = contract
        .exclusions
        .iter()
        .map(|exclusion| exclusion.clause_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for exclusion in &contract.exclusions {
        push_required(diagnostics, "exclusion_id", &exclusion.exclusion_id);
        push_required(diagnostics, "exclusion.reason", &exclusion.reason);
        if !clause_ids.contains(exclusion.clause_id.as_str()) {
            diagnostics
                .push(format!("exclusion `{}` names unknown clause `{}`", exclusion.exclusion_id, exclusion.clause_id));
        }
    }
    for clause in &contract.clauses {
        if clause.disposition == super::super::DISPOSITION_EXCLUDED
            && !exclusion_clause_ids.contains(clause.clause_id.as_str())
        {
            diagnostics
                .push(format!("excluded requirement clause `{}` has no matching exclusion record", clause.clause_id));
        }
    }
}
