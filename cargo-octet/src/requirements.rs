//! Reviewed requirement-to-evidence contracts and required-evidence freshness queries.
//!
//! The contract is human-authored Nickel exported to deterministic JSON. Evaluation binds each
//! clause to an explicit evidence disposition, retained artifacts, source identities, and, where
//! required, attributable negative controls. The emitted decision is a Rust-owned DTO that can be
//! recomputed from the same contract and evidence without an AI model or hosted provider.
//!
//! Freshness queries consume the same contract as the expected-obligation inventory. They are
//! read-only: they never execute providers and never infer causal dependency from a shared label.

mod contract;
mod evaluation;
mod evidence;
mod freshness;
mod output;
mod related;

#[cfg(test)]
mod tests;

use serde::Deserialize;

const CONTRACT_SCHEMA_VERSION: &str = "octet-requirement-contract/v1";
const EVIDENCE_SCHEMA_VERSION: &str = "octet-requirement-evidence/v1";
const DECISION_SCHEMA_VERSION: &str = "octet-requirement-decision/v1";
const FRESHNESS_SCHEMA_VERSION: &str = "octet-requirement-freshness/v1";
const PROOF_CARDS_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION: &str = "octet-formal-proof-envelope/v1";
const NEGATIVE_CHECK_SCHEMA_VERSION_V2: &str = "octet-proof-negative-check/v2";
const NEGATIVE_STATUS_DEMONSTRATED: &str = "property_demonstrated";
const REQUIRED_NON_CLAIM: &str = "evaluates supplied reviewed contracts and evidence only; does not authenticate reviewers or producers, prove verifier soundness, or establish whole-program correctness";
const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RELATED_RECORDS: usize = 12;
const MAX_FRESHNESS_DEPTH: usize = 3;
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;
const DISPOSITION_FORMAL_PROOF: &str = "formal_proof";
const DISPOSITION_EMPIRICAL_CHECK: &str = "empirical_check";
const DISPOSITION_ACCEPTED_ASSUMPTION: &str = "accepted_assumption";
const DISPOSITION_UNRESOLVED: &str = "unresolved";
const DISPOSITION_EXCLUDED: &str = "excluded";
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MISSING: &str = "missing";
const STATUS_STALE_BINDING: &str = "stale_binding";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_INCOMPLETE: &str = "incomplete";
const STATUS_EXCLUDED: &str = "excluded";
const STATUS_UNRESOLVED: &str = "unresolved";
const STATUS_ASSUMPTION_ACCEPTED: &str = "assumption_accepted";
const STATUS_CURRENT: &str = "current";
const CHECK_STATUS_PASSED: &str = "passed";
const ASSUMPTION_STATUS_ACCEPTED: &str = "accepted";
const BLAKE3_HEX_LENGTH: usize = 64;

/// Runs the requirements command.
pub(crate) fn run(args: &crate::cli::RequirementsArgs) -> Result<i32, String> {
    match &args.mode {
        crate::cli::RequirementsMode::Check(check) => run_check(check),
        crate::cli::RequirementsMode::Verify(verify) => run_verify(verify),
        crate::cli::RequirementsMode::Freshness(freshness) => run_freshness(freshness),
    }
}

fn run_check(args: &crate::cli::RequirementsCheckArgs) -> Result<i32, String> {
    if let Some(output) = &args.output
        && (output == &args.contract || output == &args.evidence)
    {
        return Err("requirements check output must not overwrite the contract or evidence input".to_owned());
    }
    let contract_bytes = read_bounded(&args.contract)?;
    let evidence_bytes = read_bounded(&args.evidence)?;
    let contract: contract::Definition = parse_json(&contract_bytes, &args.contract)?;
    let evidence: evidence::Bundle = parse_json(&evidence_bytes, &args.evidence)?;
    let decision = evaluation::decision::evaluate_contract(
        &contract,
        &evidence,
        &contract_bytes,
        &evidence_bytes,
        &args.source_root,
    )?;
    output::write_decision(&decision, args.output.as_deref(), args.output_format)?;
    Ok(exit_for_aggregate(&decision.aggregate_result))
}

fn run_verify(args: &crate::cli::RequirementsVerifyArgs) -> Result<i32, String> {
    let contract_bytes = read_bounded(&args.contract)?;
    let evidence_bytes = read_bounded(&args.evidence)?;
    let contract: contract::Definition = parse_json(&contract_bytes, &args.contract)?;
    let evidence: evidence::Bundle = parse_json(&evidence_bytes, &args.evidence)?;
    let recorded_bytes = read_bounded(&args.decision)?;
    let recorded: evaluation::decision::Outcome = parse_json(&recorded_bytes, &args.decision)?;
    let recomputed = evaluation::decision::evaluate_contract(
        &contract,
        &evidence,
        &contract_bytes,
        &evidence_bytes,
        &args.source_root,
    )?;
    let mut diagnostics = Vec::new();
    evaluation::verification::compare_decisions(&recorded, &recomputed, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let status = if diagnostics.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    };
    let report = evaluation::verification::ValidationReport {
        schema_version: DECISION_SCHEMA_VERSION.to_owned(),
        status: status.to_owned(),
        contract_id: recomputed.contract_id.clone(),
        recorded_decision_blake3: recorded.decision_blake3.clone(),
        recomputed_decision_blake3: recomputed.decision_blake3.clone(),
        aggregate_result: recomputed.aggregate_result.clone(),
        diagnostics,
        non_claims: non_claims(&contract),
    };
    match args.output_format {
        crate::cli::OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize decision verification: {error}"))?
        ),
        crate::cli::OutputFormat::Human | crate::cli::OutputFormat::Sarif => {
            println!(
                "octet requirements verify contract={} status={} aggregate={} recorded={} recomputed={} diagnostics={}",
                report.contract_id,
                report.status,
                report.aggregate_result,
                report.recorded_decision_blake3,
                report.recomputed_decision_blake3,
                report.diagnostics.len()
            );
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &report.non_claims {
                println!("non-claim: {non_claim}");
            }
        }
    }
    Ok(exit_for_aggregate(status))
}

fn run_freshness(args: &crate::cli::RequirementsFreshnessArgs) -> Result<i32, String> {
    let contract_bytes = read_bounded(&args.contract)?;
    let contract: contract::Definition = parse_json(&contract_bytes, &args.contract)?;
    let index_bytes = read_bounded(&args.index)?;
    let index: crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex =
        parse_json(&index_bytes, &args.index)?;
    let report = freshness::evaluate(&contract, &index, &contract_bytes);
    match args.output_format {
        crate::cli::OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| format!("serialize freshness report: {error}"))?
        ),
        crate::cli::OutputFormat::Human | crate::cli::OutputFormat::Sarif => output::print_freshness_human(&report),
    }
    Ok(exit_for_freshness(&report.aggregate_result))
}

fn exit_for_aggregate(result: &str) -> i32 {
    if result == STATUS_PASSED || result == STATUS_EXCLUDED {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    }
}

fn exit_for_freshness(result: &str) -> i32 {
    if result == STATUS_CURRENT {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    }
}

fn default_true() -> bool {
    true
}

fn default_dot() -> String {
    ".".to_owned()
}

fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8], path: &std::path::Path) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("parse {} as JSON: {error}", path.display()))
}

fn read_bounded(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|error| format!("read metadata {}: {error}", path.display()))?;
    if metadata.len() > MAX_ARTIFACT_BYTES {
        return Err(format!("{} exceeds the maximum retained artifact size", path.display()));
    }
    std::fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn read_json_bounded<T: for<'de> Deserialize<'de>>(path: &std::path::Path) -> Result<T, String> {
    let bytes = read_bounded(path)?;
    parse_json(&bytes, path)
}

fn non_claims(contract: &contract::Definition) -> Vec<String> {
    let mut claims = contract.non_claims.clone();
    if !claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        claims.push(REQUIRED_NON_CLAIM.to_owned());
    }
    claims.sort();
    claims.dedup();
    claims
}

fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn resolve_path(root: &std::path::Path, locator: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(locator);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
