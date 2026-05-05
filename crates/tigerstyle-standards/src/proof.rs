//! Verified proof-rail checks.

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::EnforcementMode;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::ProofRail;
use crate::source::SourceFile;

const VERIFIED_PROOF_REMEDIATION: &str =
    "rerun the configured proof command and save a passing proof receipt under the declared proof path";

/// Checks artifact-backed proof-rail evidence for verified-mode policies.
#[must_use]
pub fn check_verified_proof_rails(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    if !selection.includes("verification") {
        return Vec::new();
    }

    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| check_policy_proof_rails(policy, sources))
        .collect()
}

fn policy_is_verified(policy: &PolicyEntry) -> bool {
    policy.current_enforcement_mode == EnforcementMode::Verified
        || policy.target_enforcement_mode == EnforcementMode::Verified
}

fn check_policy_proof_rails(policy: &PolicyEntry, sources: &[SourceFile]) -> Vec<Diagnostic> {
    if policy.proof_rails.is_empty() {
        return vec![proof_diagnostic(
            policy,
            "policy",
            None,
            "verified policy is missing proof rail metadata".to_owned(),
        )];
    }

    policy.proof_rails.iter().flat_map(|rail| check_proof_rail(policy, rail, sources)).collect()
}

fn check_proof_rail(policy: &PolicyEntry, rail: &ProofRail, sources: &[SourceFile]) -> Vec<Diagnostic> {
    if !rail_is_relevant(rail, sources) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    if !patterns_match_any_source(&rail.source_paths, sources) {
        diagnostics.push(proof_diagnostic(
            policy,
            first_path_or_policy(&rail.source_paths),
            None,
            format!(
                "verified proof rail `{}` has no checked source matching [{}]",
                rail.backend,
                rail.source_paths.join(", ")
            ),
        ));
    }

    let proof_sources = matching_sources(&rail.proof_paths, sources);
    if proof_sources.is_empty() {
        diagnostics.push(proof_diagnostic(
            policy,
            first_path_or_policy(&rail.proof_paths),
            None,
            format!(
                "verified proof rail `{}` has no checked proof receipt matching [{}]",
                rail.backend,
                rail.proof_paths.join(", ")
            ),
        ));
    } else if !proof_sources.iter().any(|source| source.text.contains(&rail.pass_signal)) {
        diagnostics.push(proof_diagnostic(
            policy,
            &proof_sources[0].path,
            None,
            format!("verified proof rail `{}` is missing pass signal `{}`", rail.backend, rail.pass_signal),
        ));
    }

    diagnostics
}

fn rail_is_relevant(rail: &ProofRail, sources: &[SourceFile]) -> bool {
    patterns_match_any_source(&rail.source_paths, sources) || patterns_match_any_source(&rail.proof_paths, sources)
}

fn patterns_match_any_source(patterns: &[String], sources: &[SourceFile]) -> bool {
    !patterns.is_empty() && !matching_sources(patterns, sources).is_empty()
}

fn matching_sources<'a>(patterns: &[String], sources: &'a [SourceFile]) -> Vec<&'a SourceFile> {
    sources
        .iter()
        .filter(|source| patterns.iter().any(|pattern| source.path.contains(pattern)))
        .collect()
}

fn first_path_or_policy(paths: &[String]) -> &str {
    paths.first().map_or("policy", String::as_str)
}

fn proof_diagnostic(policy: &PolicyEntry, path: &str, line: Option<u32>, message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: path.to_owned(),
        line,
        severity: policy.default_severity,
        message,
        remediation: VERIFIED_PROOF_REMEDIATION.to_owned(),
    })
}
