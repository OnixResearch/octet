//! Verified proof-rail checks.

use std::fmt::Write as _;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::EnforcementMode;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::ProofRail;
use crate::model::ReceiptProvenance;
use crate::model::ReceiptVerifier;
use crate::source::SourceFile;

const RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE: &str = "blake3-command-and-source";
const VERUS_BACKEND: &str = "verus";
const VERIFIED_PATH_COMPONENT: &str = "/verified/";
const VERIFIED_PATH_PREFIX: &str = "verified/";
const FORMAL_PROOF_CLAIM_MARKER: &str = "octet-formal-proof-claim:";
const VERIFIED_MODULE_COVERAGE_POLICY_ID: &str = "octet.verus_proof_rails.verified_module_coverage";
const VERIFIED_MODULE_COVERAGE_SOURCE: &str = "cairn/specs/verus-proof-rails/spec.md#verified-module-proof-coverage";
const VERIFIED_PROOF_REMEDIATION: &str =
    "rerun the configured proof command and save a passing proof receipt under the declared proof path";
const VERIFIED_MODULE_REMEDIATION: &str =
    "declare a matching Verus proof rail and persist a validated receipt, or record manual_review_required provenance";

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

    let mut diagnostics = registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| check_policy_proof_rails(policy, sources))
        .collect::<Vec<_>>();
    diagnostics.extend(check_verified_module_coverage(registry, sources, selection));
    diagnostics
}

/// Validates one persisted proof receipt against a declared rail and in-memory source set.
#[must_use]
pub fn validate_proof_receipt(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !proof_source.text.contains(&rail.pass_signal) {
        diagnostics.push(format!("proof receipt field `pass_signal` missing pass signal `{}`", rail.pass_signal));
    }
    if receipt_binding_enabled(rail) && !receipt_binding_matches(rail, sources, proof_source) {
        diagnostics.push(format!(
            "mismatched receipt binding `{}` in proof receipt field `receipt_binding`",
            rail.receipt_binding
        ));
    }
    if receipt_provenance_enabled(rail) && !receipt_provenance_matches(rail, proof_source) {
        diagnostics.push(
            "mismatched receipt provenance in proof receipt field `receipt_provenance` required lines".to_owned(),
        );
    }
    if receipt_verifier_enabled(rail) && !receipt_verifier_matches(rail, sources, proof_source) {
        diagnostics.push("mismatched receipt verifier metadata in proof receipt field `verifier_identity`".to_owned());
    }
    diagnostics
}

/// Renders deterministic receipt metadata lines that the pure proof checker validates.
#[must_use]
pub fn proof_receipt_metadata(rail: &ProofRail, sources: &[SourceFile]) -> String {
    let mut lines = expected_receipt_provenance_lines(rail);
    lines.extend(expected_receipt_verifier_lines(rail, sources));
    if receipt_binding_enabled(rail) {
        lines.extend(expected_receipt_binding_lines(rail, sources));
    }
    receipt_lines(&lines)
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
        return diagnostics;
    }

    if proof_sources
        .iter()
        .any(|proof_source| validate_proof_receipt(rail, sources, proof_source).is_empty())
    {
        return diagnostics;
    }

    let first_failure = validate_proof_receipt(rail, sources, proof_sources[0]);
    diagnostics.push(proof_diagnostic(
        policy,
        &proof_sources[0].path,
        None,
        format!("verified proof rail `{}` failed receipt validation: {}", rail.backend, first_failure.join("; ")),
    ));
    diagnostics
}

fn check_verified_module_coverage(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    sources
        .iter()
        .filter(|source| source_has_verified_claim(source))
        .filter(|source| !matching_verus_rail_exists(registry, selection, source))
        .map(verified_module_coverage_diagnostic)
        .collect()
}

fn source_has_verified_claim(source: &SourceFile) -> bool {
    let normalized_path = source.path.replace('\\', "/");
    normalized_path.starts_with(VERIFIED_PATH_PREFIX)
        || normalized_path.contains(VERIFIED_PATH_COMPONENT)
        || source.text.contains(FORMAL_PROOF_CLAIM_MARKER)
}

fn matching_verus_rail_exists(registry: &PolicyRegistry, selection: &FamilySelection, source: &SourceFile) -> bool {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| policy.proof_rails.iter())
        .filter(|rail| rail.backend == VERUS_BACKEND)
        .any(|rail| rail.source_paths.iter().any(|pattern| source.path.contains(pattern)))
}

fn verified_module_coverage_diagnostic(source: &SourceFile) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: VERIFIED_MODULE_COVERAGE_POLICY_ID.to_owned(),
        source_section: VERIFIED_MODULE_COVERAGE_SOURCE.to_owned(),
        path: source.path.clone(),
        line: None,
        severity: crate::model::Severity::Error,
        message: "manual_review_required: verified module or formal-proof claim has no matching Verus proof rail"
            .to_owned(),
        remediation: VERIFIED_MODULE_REMEDIATION.to_owned(),
    })
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

fn receipt_binding_enabled(rail: &ProofRail) -> bool {
    rail.receipt_binding == RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE
}

fn receipt_binding_matches(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> bool {
    expected_receipt_binding_lines(rail, sources)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn receipt_provenance_enabled(rail: &ProofRail) -> bool {
    let provenance = &rail.receipt_provenance;
    !provenance.schema_version.is_empty()
        || !provenance.runner.is_empty()
        || !provenance.environment.is_empty()
        || provenance.requires_backend
        || provenance.requires_trust_boundary
}

fn receipt_provenance_matches(rail: &ProofRail, proof_source: &SourceFile) -> bool {
    expected_receipt_provenance_lines(rail)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn receipt_verifier_enabled(rail: &ProofRail) -> bool {
    let verifier = &rail.receipt_verifier;
    !verifier.verifier_id.is_empty()
        || !verifier.verifier_digest.is_empty()
        || !verifier.verifier_binary_path.is_empty()
        || !verifier.verifier_binary_digest.is_empty()
        || !verifier.verifier_build_closure.is_empty()
        || !verifier.verifier_build_closure_digest.is_empty()
        || !verifier.verifier_build_closure_paths.is_empty()
        || !verifier.verifier_nix_derivation.is_empty()
        || !verifier.verifier_nix_derivation_digest.is_empty()
        || !verifier.verifier_nix_closure.is_empty()
        || !verifier.verifier_nix_closure_digest.is_empty()
        || !verifier.verifier_nix_closure_paths.is_empty()
        || !verifier.verifier_nix_closure_graph_digest.is_empty()
        || !verifier.verifier_nix_closure_graph_paths.is_empty()
        || !verifier.toolchain_id.is_empty()
        || !verifier.toolchain_digest.is_empty()
}

fn receipt_verifier_matches(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> bool {
    expected_receipt_verifier_lines(rail, sources)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn expected_receipt_verifier_lines(rail: &ProofRail, sources: &[SourceFile]) -> Vec<String> {
    let ReceiptVerifier {
        verifier_id,
        verifier_digest,
        verifier_binary_path,
        verifier_binary_digest,
        verifier_build_closure,
        verifier_build_closure_digest,
        verifier_build_closure_paths,
        verifier_nix_derivation,
        verifier_nix_derivation_digest,
        verifier_nix_closure,
        verifier_nix_closure_digest,
        verifier_nix_closure_paths,
        verifier_nix_closure_graph_digest,
        verifier_nix_closure_graph_paths,
        toolchain_id,
        toolchain_digest,
    } = &rail.receipt_verifier;
    let mut lines = Vec::new();
    if !verifier_id.is_empty() {
        lines.push(format!("proof-verifier: {verifier_id}"));
    }
    if !verifier_digest.is_empty() {
        lines.push(format!("proof-verifier-blake3: {verifier_digest}"));
    }
    if !verifier_binary_path.is_empty() {
        lines.push(format!("proof-verifier-binary: {verifier_binary_path}"));
    }
    if !verifier_binary_digest.is_empty() {
        lines.push(format!("proof-verifier-binary-blake3: {verifier_binary_digest}"));
    }
    if !verifier_build_closure.is_empty() {
        lines.push(format!("proof-verifier-build-closure: {verifier_build_closure}"));
    }
    if !verifier_build_closure_digest.is_empty() {
        let digest = build_closure_digest(verifier_build_closure_paths, sources)
            .unwrap_or_else(|| verifier_build_closure_digest.clone());
        lines.push(format!("proof-verifier-build-closure-blake3: {digest}"));
    }
    if !verifier_nix_derivation.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation: {verifier_nix_derivation}"));
    }
    if !verifier_nix_derivation_digest.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation-blake3: {verifier_nix_derivation_digest}"));
    }
    if !verifier_nix_closure.is_empty() {
        lines.push(format!("proof-verifier-nix-closure: {verifier_nix_closure}"));
    }
    if !verifier_nix_closure_digest.is_empty() {
        let digest = build_closure_digest(verifier_nix_closure_paths, sources)
            .unwrap_or_else(|| verifier_nix_closure_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-blake3: {digest}"));
    }
    if !verifier_nix_closure_graph_digest.is_empty() {
        let digest = nix_closure_graph_digest(verifier_nix_closure_graph_paths, sources)
            .unwrap_or_else(|| verifier_nix_closure_graph_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-graph-blake3: {digest}"));
    }
    if !toolchain_id.is_empty() {
        lines.push(format!("proof-toolchain: {toolchain_id}"));
    }
    if !toolchain_digest.is_empty() {
        lines.push(format!("proof-toolchain-blake3: {toolchain_digest}"));
    }
    lines
}

fn build_closure_digest(paths: &[String], sources: &[SourceFile]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    let mut matched_sources = sources
        .iter()
        .filter(|source| paths.iter().any(|pattern| source.path.contains(pattern)))
        .collect::<Vec<_>>();
    if matched_sources.is_empty() {
        return None;
    }
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    let mut manifest = String::new();
    for source in matched_sources {
        writeln!(manifest, "{}:{}", source.path, blake3_hex(&source.text)).expect("writing to string should not fail");
    }
    Some(blake3_hex(&manifest))
}

fn nix_closure_graph_digest(paths: &[String], sources: &[SourceFile]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    let mut matched_sources = sources
        .iter()
        .filter(|source| paths.iter().any(|pattern| source.path.contains(pattern)))
        .collect::<Vec<_>>();
    if matched_sources.is_empty() {
        return None;
    }
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    let mut canonical_graphs = String::new();
    for source in matched_sources {
        let canonical = canonical_nix_closure_graph(&source.text)?;
        canonical_graphs.push_str(&canonical);
    }
    Some(blake3_hex(&canonical_graphs))
}

fn canonical_nix_closure_graph(text: &str) -> Option<String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut records = Vec::new();
    while index < lines.len() {
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= lines.len() {
            break;
        }
        let path = lines[index].trim().to_owned();
        index += 1;
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= lines.len() {
            return None;
        }
        let reference_count = lines[index].trim().parse::<usize>().ok()?;
        index += 1;
        let mut references = Vec::with_capacity(reference_count);
        for _ in 0..reference_count {
            while index < lines.len() && lines[index].trim().is_empty() {
                index += 1;
            }
            if index >= lines.len() {
                return None;
            }
            references.push(lines[index].trim().to_owned());
            index += 1;
        }
        references.sort();
        records.push((path, references));
    }
    if records.is_empty() {
        return None;
    }
    records.sort_by(|left, right| left.0.cmp(&right.0));
    let mut canonical = String::new();
    for (path, references) in records {
        writeln!(canonical, "{path}").expect("writing to string should not fail");
        writeln!(canonical, "{}", references.len()).expect("writing to string should not fail");
        for reference in references {
            writeln!(canonical, "{reference}").expect("writing to string should not fail");
        }
    }
    Some(canonical)
}

fn expected_receipt_provenance_lines(rail: &ProofRail) -> Vec<String> {
    let ReceiptProvenance {
        schema_version,
        runner,
        environment,
        requires_backend,
        requires_trust_boundary,
    } = &rail.receipt_provenance;
    let mut lines = Vec::new();
    if !schema_version.is_empty() {
        lines.push(format!("proof-receipt-schema: {schema_version}"));
    }
    if *requires_backend {
        lines.push(format!("proof-backend: {}", rail.backend));
    }
    if !runner.is_empty() {
        lines.push(format!("proof-runner: {runner}"));
    }
    if !environment.is_empty() {
        lines.push(format!("proof-environment: {environment}"));
    }
    if *requires_trust_boundary {
        lines.extend(rail.trust_boundaries.iter().map(|boundary| format!("trust-boundary: {boundary}")));
    }
    lines
}

fn expected_receipt_binding_lines(rail: &ProofRail, sources: &[SourceFile]) -> Vec<String> {
    let mut lines = vec![format!("proof-command-blake3: {}", blake3_hex(&rail.command))];
    let mut matched_sources = matching_sources(&rail.source_paths, sources);
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    lines.extend(
        matched_sources
            .into_iter()
            .map(|source| format!("proof-source-blake3: {}:{}", source.path, blake3_hex(&source.text))),
    );
    lines
}

fn receipt_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn blake3_hex(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
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
