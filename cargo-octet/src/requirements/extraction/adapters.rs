//! Typed native-evidence adapters and cross-clause subject/theorem invariant checks (#18).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PreservationReceipt {
    schema_version: String,
    status: String,
    has_timeout: bool,
    has_checker_error: bool,
    original_blake3: String,
    extracted_core_blake3: String,
    driver_blake3: String,
    specification_blake3: String,
    dependency_closure_blake3: String,
    target_triple: String,
    toolchain_blake3: String,
    observation_contract: super::ObservationContract,
    assumptions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CorrespondenceReceipt {
    schema_version: String,
    status: String,
    original_blake3: String,
    extracted_core_blake3: String,
    backend_core_blake3: String,
    checked_steps: Vec<String>,
    unresolved_boundaries: Vec<String>,
    assumptions: Vec<String>,
}

pub(super) fn verify_registered(
    profile: &super::BoundaryProfile,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    verify_subject_files(profile, source_root, diagnostics);
    verify_semantic_effect_adapter(profile, contract, evidence, source_root, diagnostics);
    verify_preservation_adapter(profile, contract, evidence, source_root, diagnostics);
    verify_correspondence_adapter(profile, contract, evidence, source_root, diagnostics);
    let ctx = super::optional::VerifyCtx {
        profile,
        contract,
        evidence,
        source_root,
    };
    if let Some(clause_id) = profile.clauses.optional_normal_form_clause.as_deref() {
        super::optional::verify_normal_form_adapter(&ctx, clause_id, diagnostics);
    }
    if let Some(clause_id) = profile.clauses.effect_independence_clause_id.as_deref() {
        super::optional::verify_effect_independence_adapter(&ctx, clause_id, diagnostics);
    }
}

fn verify_subject_files(
    profile: &super::BoundaryProfile,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let subjects = [
        ("original_implementation", &profile.subjects.original_implementation),
        ("extracted_core", &profile.subjects.extracted_core),
        ("effect_driver", &profile.subjects.effect_driver),
        ("specification", &profile.subjects.specification),
    ];
    for (label, subject) in subjects {
        let path = super::super::resolve_path(source_root, &subject.locator);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let actual = super::super::hash_bytes(&bytes);
                if actual != subject.blake3 {
                    diagnostics.push(format!(
                        "effect-extraction subject `{label}` (`{}`) digest mismatch: expected {}, got {actual}",
                        subject.locator, subject.blake3
                    ));
                }
            }
            Err(error) => diagnostics
                .push(format!("effect-extraction subject `{label}` (`{}`) is unreadable: {error}", subject.locator)),
        }
    }
}

pub(super) fn find_empirical_artifact_path(
    clause_id: &str,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let clause = contract
        .clauses
        .iter()
        .find(|c| c.clause_id == clause_id)
        .ok_or_else(|| format!("missing clause `{clause_id}`"))?;
    let check = evidence
        .empirical_checks
        .iter()
        .find(|check| clause.allowed_evidence_ids.contains(&check.evidence_id))
        .ok_or_else(|| format!("clause `{clause_id}` has no matching empirical check evidence"))?;
    let locator = check
        .artifact
        .as_deref()
        .ok_or_else(|| format!("evidence `{}` has no artifact path", check.evidence_id))?;
    Ok(super::super::resolve_path(source_root, locator))
}

fn verify_semantic_effect_adapter(
    profile: &super::BoundaryProfile,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let path =
        match find_empirical_artifact_path(&profile.clauses.core_effects_clause_id, contract, evidence, source_root) {
            Ok(path) => path,
            Err(error) => {
                diagnostics.push(error);
                return;
            }
        };
    let receipt: crate::semantic_effect::model::MirClosureReceipt = match super::super::read_json_bounded(&path) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse semantic effect receipt {}: {error}", path.display()));
            return;
        }
    };
    diagnostics.extend(crate::semantic_effect::verify_receipt(&receipt));
    if receipt.status != crate::effect_closure::AdmissionStatus::Admitted {
        diagnostics.push("semantic effect receipt status is not admitted".to_owned());
    }
    if !receipt.coverage.is_complete {
        diagnostics.push("semantic effect receipt coverage is incomplete".to_owned());
    }
    if receipt.observed_effects.contains(&crate::semantic_effect::model::MirOperation::Unknown) {
        diagnostics.push("semantic effect receipt contains unresolved unknown effects".to_owned());
    }
    if receipt.profile_id != profile.subjects.semantic_profile_id {
        diagnostics.push(format!(
            "semantic effect receipt profile `{}` does not match `{}`",
            receipt.profile_id, profile.subjects.semantic_profile_id
        ));
    }
    if receipt.subjects.source_blake3 != profile.subjects.extracted_core.blake3 {
        diagnostics.push("semantic effect receipt core source identity does not match extracted_core".to_owned());
    }
    if receipt.subjects.cargo_lock_blake3 != profile.subjects.dependency_closure_blake3 {
        diagnostics.push("semantic effect receipt dependency closure does not match contract".to_owned());
    }
    if receipt.subjects.toolchain_blake3 != profile.subjects.toolchain_blake3 {
        diagnostics.push("semantic effect receipt toolchain identity does not match contract".to_owned());
    }
}

fn verify_preservation_adapter(
    profile: &super::BoundaryProfile,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let ctx = super::optional::VerifyCtx {
        profile,
        contract,
        evidence,
        source_root,
    };
    let Some(path) = super::optional::find_proof_artifact_path(
        &ctx,
        &profile.clauses.behavior_preservation_clause_id,
        "preservation-proof.json",
        diagnostics,
    ) else {
        return;
    };
    let receipt: PreservationReceipt = match super::super::read_json_bounded(&path) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse preservation proof receipt {}: {error}", path.display()));
            return;
        }
    };
    if receipt.schema_version != "octet-extraction-preservation-proof/v1" {
        diagnostics.push("unsupported preservation proof receipt schema".to_owned());
    }
    if receipt.status != "passed" || receipt.has_timeout || receipt.has_checker_error {
        diagnostics.push("behavior preservation proof did not cleanly pass".to_owned());
    }
    if receipt.original_blake3 != profile.subjects.original_implementation.blake3
        || receipt.extracted_core_blake3 != profile.subjects.extracted_core.blake3
        || receipt.driver_blake3 != profile.subjects.effect_driver.blake3
        || receipt.specification_blake3 != profile.subjects.specification.blake3
        || receipt.dependency_closure_blake3 != profile.subjects.dependency_closure_blake3
        || receipt.target_triple != profile.subjects.target_triple
        || receipt.toolchain_blake3 != profile.subjects.toolchain_blake3
    {
        diagnostics.push("preservation proof subject/environment identity mismatch".to_owned());
    }
    if receipt.observation_contract != profile.observation_contract {
        diagnostics.push("preservation proof observation contract differs from admitted contract".to_owned());
    }
    for assumption in &receipt.assumptions {
        if !profile.approved_assumptions.contains(assumption) {
            diagnostics.push(format!("preservation proof uses unapproved assumption `{assumption}`"));
        }
    }
}

fn verify_correspondence_adapter(
    profile: &super::BoundaryProfile,
    contract: &super::super::contract::Definition,
    evidence: &super::super::evidence::Bundle,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let path = match find_empirical_artifact_path(
        &profile.clauses.correspondence_clause_id,
        contract,
        evidence,
        source_root,
    ) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(error);
            return;
        }
    };
    let receipt: CorrespondenceReceipt = match super::super::read_json_bounded(&path) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse correspondence receipt {}: {error}", path.display()));
            return;
        }
    };
    if receipt.schema_version != "octet-extraction-correspondence/v1" || receipt.status != "passed" {
        diagnostics.push("source/model correspondence receipt is invalid or did not pass".to_owned());
    }
    if receipt.original_blake3 != profile.subjects.original_implementation.blake3
        || receipt.extracted_core_blake3 != profile.subjects.extracted_core.blake3
        || receipt.backend_core_blake3 != profile.subjects.extracted_core.blake3
    {
        diagnostics.push("source/model correspondence core or original identity mismatch".to_owned());
    }
    if receipt.checked_steps.is_empty() || !receipt.unresolved_boundaries.is_empty() {
        diagnostics.push("source/model correspondence has missing steps or unresolved boundaries".to_owned());
    }
    for assumption in &receipt.assumptions {
        if !profile.approved_assumptions.contains(assumption) {
            diagnostics.push(format!("correspondence receipt uses unapproved assumption `{assumption}`"));
        }
    }
}
