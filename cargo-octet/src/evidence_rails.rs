use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::EvidenceRailArgs;
use crate::cli::OutputFormat;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;

pub const RECEIPT_SCHEMA_VERSION: &str = "octet-evidence-rail-receipt/v1";
pub const RECEIPT_ARTIFACT_NAME: &str = "evidence-rail-receipt.json";
pub const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
pub const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";

const EXIT_SUCCESS: i32 = 0;
const EXIT_RAIL_FAILURE: i32 = 1;
const EXIT_INTEGRATION_FAILURE: i32 = 2;
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const MODE_IMPORT: &str = "import";
const MODE_RUN: &str = "run";
const LOCKED_REUSED: &str = "reused";
const LOCKED_UPDATED_PRIOR: &str = "updated_by_explicit_prior_step";
const LOCKED_UNAVAILABLE: &str = "unavailable";
const SUMMARY_REVIEW_ACTION: &str = "review optional evidence rail summary and receipt before using this artifact";
const EMPTY_LOG_IDENTITY: &str = "b3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adf687e5e01f59efb8f0b5a2";
const DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY: &str = "miri-unsafe-boundary";
const DYNAMIC_PROFILE_MIRI_SHORT: &str = "miri";
const DYNAMIC_COMMAND_MIRI_TEST: &str = "cargo miri test";
const DYNAMIC_PROFILE_LOOM_CONCURRENCY: &str = "loom-concurrency";
const DYNAMIC_PROFILE_LOOM_SHORT: &str = "loom";
const DYNAMIC_COMMAND_LOOM_TEST: &str = "cargo test --cfg loom";
const SUPPORTED_EVIDENCE_RAILS: &str = "dependency-audit, clippy-co-reporting, dependency-trust, api-stability, dynamic-evidence, release-supply-chain, mutation-testing, or coverage-baseline";
const COVERAGE_PERCENT_MIN: f64 = 0.0;
const COVERAGE_PERCENT_MAX: f64 = 100.0;
const SCHEMA_VERSION_FIELD: &str = "schema_version";
const DEPENDENCY_AUDIT_SCHEMA_VERSION: &str = "cargo-deny-diagnostics/v1";
const COVERAGE_BASELINE_SCHEMA_VERSION: &str = "cargo-llvm-cov-summary/v1";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EvidenceRailKind {
    DependencyAudit,
    ClippyCoReporting,
    DependencyTrust,
    ApiStability,
    DynamicEvidence,
    ReleaseSupplyChain,
    MutationTesting,
    CoverageBaseline,
}

impl EvidenceRailKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "dependency-audit" | "cargo-deny" => Ok(Self::DependencyAudit),
            "clippy-co-reporting" | "clippy" | "external-lint" => Ok(Self::ClippyCoReporting),
            "dependency-trust" | "cargo-vet" => Ok(Self::DependencyTrust),
            "api-stability" | "semver" | "cargo-semver-checks" => Ok(Self::ApiStability),
            "dynamic-evidence" => Ok(Self::DynamicEvidence),
            "release-supply-chain" => Ok(Self::ReleaseSupplyChain),
            "mutation-testing" | "cargo-mutants" | "mutants" => Ok(Self::MutationTesting),
            "coverage-baseline" | "coverage" | "cargo-llvm-cov" => Ok(Self::CoverageBaseline),
            other => Err(format!("unsupported evidence rail `{other}`; expected {SUPPORTED_EVIDENCE_RAILS}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DependencyAudit => "dependency-audit",
            Self::ClippyCoReporting => "clippy-co-reporting",
            Self::DependencyTrust => "dependency-trust",
            Self::ApiStability => "api-stability",
            Self::DynamicEvidence => "dynamic-evidence",
            Self::ReleaseSupplyChain => "release-supply-chain",
            Self::MutationTesting => "mutation-testing",
            Self::CoverageBaseline => "coverage-baseline",
        }
    }

    pub fn checker_id(self) -> &'static str {
        match self {
            Self::DependencyAudit => crate::evidence::CHECKER_CARGO_DENY,
            Self::ClippyCoReporting => crate::evidence::CHECKER_CLIPPY,
            Self::DependencyTrust => crate::evidence::CHECKER_CARGO_VET,
            Self::ApiStability => crate::evidence::CHECKER_CARGO_SEMVER_CHECKS,
            Self::DynamicEvidence => crate::evidence::CHECKER_OCTET_DYNAMIC_EVIDENCE,
            Self::ReleaseSupplyChain => crate::evidence::CHECKER_OCTET_RELEASE_SUPPLY_CHAIN,
            Self::MutationTesting => crate::evidence::CHECKER_CARGO_MUTANTS,
            Self::CoverageBaseline => crate::evidence::CHECKER_CARGO_LLVM_COV,
        }
    }

    pub fn source_id(self) -> &'static str {
        match self {
            Self::DependencyAudit => "octet.policy.dependency-audit",
            Self::ClippyCoReporting => "octet.external_lint.clippy-co-reporting",
            Self::DependencyTrust => "octet.dependency_trust.cargo-vet",
            Self::ApiStability => "octet.api_stability.semver-rail",
            Self::DynamicEvidence => "octet.dynamic_evidence.profile-runner",
            Self::ReleaseSupplyChain => "octet.release_supply_chain.local-receipt",
            Self::MutationTesting => "octet.mutation_testing.cargo-mutants",
            Self::CoverageBaseline => "octet.coverage_baseline.cargo-llvm-cov",
        }
    }

    pub fn evidence_tier(self) -> EvidenceTier {
        match self {
            Self::ClippyCoReporting => EvidenceTier::StaticLint,
            Self::DynamicEvidence | Self::MutationTesting | Self::CoverageBaseline => EvidenceTier::TranscriptAssertion,
            _ => EvidenceTier::PolicyCheck,
        }
    }

    fn checked_rule_families(self) -> Vec<String> {
        match self {
            Self::DependencyAudit => string_vec(&["advisories", "licenses", "bans", "sources", "duplicates"]),
            Self::ClippyCoReporting => string_vec(&["clippy-diagnostics", "source-severity", "octet-gate-outcome"]),
            Self::DependencyTrust => string_vec(&["audits", "imports", "exemptions", "unaudited-gaps"]),
            Self::ApiStability => string_vec(&["public-api", "baseline", "feature-set", "target-triple"]),
            Self::DynamicEvidence => string_vec(&["profile-selection", "allowed-command", "acceptance-signal"]),
            Self::ReleaseSupplyChain => string_vec(&[
                "auditable-binary-metadata",
                "sbom",
                "vulnerability-scan",
                "attestation-compatibility",
            ]),
            Self::MutationTesting => string_vec(&[
                "mutant-generation",
                "caught-mutants",
                "survived-mutants",
                "timeout-mutants",
                "test-suite-kill-signal",
            ]),
            Self::CoverageBaseline => string_vec(&[
                "coverage-summary",
                "accepted-baseline",
                "regression-detection",
                "threshold-failures",
            ]),
        }
    }

    fn non_claims(self) -> Vec<String> {
        match self {
            Self::DependencyAudit => string_vec(&[
                "license policy results are not legal advice",
                "advisory database state does not prove dependencies are vulnerability-free",
                "dependency audit evidence is optional and not part of starter cargo octet check",
            ]),
            Self::ClippyCoReporting => string_vec(&[
                "Clippy findings are complementary external hygiene evidence, not Octet source lint replacements",
                "source severity is preserved separately from Octet gate outcome",
            ]),
            Self::DependencyTrust => string_vec(&[
                "cargo-vet audit coverage does not prove dependencies are benign or vulnerability-free",
                "dependency trust evidence remains separate from dependency audit and advisory evidence",
            ]),
            Self::ApiStability => string_vec(&[
                "API stability evidence covers selected public API checks only",
                "compatible semver output is not a whole-program behavior compatibility proof",
            ]),
            Self::DynamicEvidence => string_vec(&[
                "dynamic evidence profiles are opt-in and not required by default cargo octet check",
                "Miri, Loom, profiling, or model-checking evidence does not prove whole-program correctness",
            ]),
            Self::ReleaseSupplyChain => string_vec(&[
                "local release receipts are SLSA/in-toto compatible shapes, not hosted-service attestations",
                "SBOM and vulnerability scan evidence does not prove the binary is vulnerability-free",
                "auditable dependency metadata does not prove source correctness",
            ]),
            Self::MutationTesting => string_vec(&[
                "mutation testing evidence samples the test suite sensitivity and does not prove the tests are complete",
                "survived or timed-out mutants require review before promotion rather than formal-proof treatment",
                "cargo-mutants evidence is optional and not part of starter cargo octet check",
            ]),
            Self::CoverageBaseline => string_vec(&[
                "coverage evidence measures executed code regions and does not prove behavioral correctness",
                "coverage baseline evidence is optional and not part of starter cargo octet check",
                "passing coverage thresholds does not replace review of uncovered critical paths",
            ]),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRailFinding {
    pub family: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRailReceipt {
    pub schema_version: String,
    pub rail_id: String,
    pub checker_id: String,
    pub mode: String,
    pub command_argv: Vec<String>,
    pub workspace_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile_identity: Option<String>,
    pub checked_rule_families: Vec<String>,
    pub status: String,
    pub finding_counts: BTreeMap<String, u64>,
    pub bounded_log_identity: String,
    pub imported_input_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_database_update: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_exit_code: Option<i32>,
    pub non_claims: Vec<String>,
    pub diagnostics: Vec<String>,
    pub findings: Vec<EvidenceRailFinding>,
    pub parent_artifacts: Vec<String>,
}

pub struct EvaluationInput<'a> {
    pub rail: EvidenceRailKind,
    pub mode: &'a str,
    pub import_text: &'a str,
    pub workspace_scope: &'a str,
    pub config_path: Option<&'a str>,
    pub baseline_identity: Option<String>,
    pub lockfile_identity: Option<String>,
    pub locked: bool,
    pub advisory_updated_by_prior_step: bool,
    pub profile: Option<&'a str>,
    pub requested_command: Option<&'a str>,
    pub parent_artifacts: Vec<String>,
    pub tool_exit_code: Option<i32>,
}

struct RailParseOutput {
    findings: Vec<EvidenceRailFinding>,
    finding_counts: BTreeMap<String, u64>,
    diagnostics: Vec<String>,
    status: String,
    profile: Option<String>,
}

pub fn run(args: &EvidenceRailArgs) -> Result<i32, String> {
    if args.output_format == OutputFormat::Sarif {
        return Err("evidence collect does not support SARIF output; use human or json".to_owned());
    }

    let workspace_root = args.workspace_root.as_deref().unwrap_or_else(|| Path::new("."));
    let workspace_scope = workspace_root.display().to_string();
    let config_path = args.config.as_ref().map(|path| path.display().to_string());
    let baseline_identity = args.baseline.as_ref().map(|path| path_identity(path)).transpose()?;
    let lockfile_identity = path_identity_if_present(&workspace_root.join("Cargo.lock"))?;
    let (mode, import_text, tool_exit_code) = evidence_input_text(args)?;
    let receipt = evaluate(EvaluationInput {
        rail: args.rail,
        mode,
        import_text: &import_text,
        workspace_scope: &workspace_scope,
        config_path: config_path.as_deref(),
        baseline_identity,
        lockfile_identity,
        locked: args.locked,
        advisory_updated_by_prior_step: args.update_advisory_database,
        profile: args.profile.as_deref(),
        requested_command: args.command.as_deref(),
        parent_artifacts: args.parent_artifacts.clone(),
        tool_exit_code,
    });

    let receipt_json = render_receipt_json(&receipt)?;
    let summary = render_summary_for_command(&receipt, &args.command_line);
    let provenance_jsonl = render_provenance_jsonl(&receipt, &receipt_json, &summary)?;
    write_artifacts(&args.artifact_dir, &receipt_json, &summary, &provenance_jsonl)?;
    render_cli_output(&receipt, args.output_format)?;

    Ok(exit_code_for_receipt(&receipt))
}

pub fn evaluate(input: EvaluationInput<'_>) -> EvidenceRailReceipt {
    let mut parsed = parse_rail_input(&input);
    parsed.diagnostics.sort();
    parsed.diagnostics.dedup();
    let status = if parsed.diagnostics.is_empty() {
        parsed.status
    } else {
        STATUS_FAILED.to_owned()
    };

    EvidenceRailReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        rail_id: input.rail.as_str().to_owned(),
        checker_id: input.rail.checker_id().to_owned(),
        mode: input.mode.to_owned(),
        command_argv: command_argv(
            input.rail,
            input.config_path,
            input.baseline_identity.as_deref(),
            input.requested_command,
        ),
        workspace_scope: input.workspace_scope.to_owned(),
        config_path: input.config_path.map(ToOwned::to_owned),
        baseline_identity: input.baseline_identity,
        lockfile_identity: input.lockfile_identity,
        checked_rule_families: input.rail.checked_rule_families(),
        status,
        finding_counts: parsed.finding_counts,
        bounded_log_identity: if input.import_text.is_empty() {
            EMPTY_LOG_IDENTITY.to_owned()
        } else {
            prefixed_identity(input.import_text.as_bytes())
        },
        imported_input_identity: prefixed_identity(input.import_text.as_bytes()),
        advisory_database_update: advisory_database_update(
            input.rail,
            input.locked,
            input.advisory_updated_by_prior_step,
        ),
        profile: parsed.profile.or_else(|| input.profile.map(ToOwned::to_owned)),
        tool_exit_code: input.tool_exit_code,
        non_claims: input.rail.non_claims(),
        diagnostics: parsed.diagnostics,
        findings: parsed.findings,
        parent_artifacts: input.parent_artifacts,
    }
}

pub fn render_receipt_json(receipt: &EvidenceRailReceipt) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(receipt).map_err(|error| format!("serialize evidence receipt: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_summary(receipt: &EvidenceRailReceipt) -> String {
    let mut summary = String::new();
    summary.push_str("--- octet evidence rail summary ---\n");
    summary.push_str(&format!("Rail: {}\n", receipt.rail_id));
    summary.push_str(&format!("Checker: {}\n", receipt.checker_id));
    summary.push_str(&format!("Status: {}\n", receipt.status));
    summary.push_str(&format!("Mode: {}\n", receipt.mode));
    summary.push_str(&format!("Workspace scope: {}\n", receipt.workspace_scope));
    summary.push_str(&format!("Input identity: {}\n", receipt.imported_input_identity));
    summary.push_str("Finding counts:\n");
    for (family, count) in &receipt.finding_counts {
        summary.push_str(&format!("  {family}: {count}\n"));
    }
    summary.push_str("Non-claims:\n");
    for non_claim in &receipt.non_claims {
        summary.push_str(&format!("  - {non_claim}\n"));
    }
    if !receipt.diagnostics.is_empty() {
        summary.push_str("Diagnostics:\n");
        for diagnostic in &receipt.diagnostics {
            summary.push_str(&format!("  - {diagnostic}\n"));
        }
    }
    summary.push_str("---\n");
    summary
}

fn render_summary_for_command(receipt: &EvidenceRailReceipt, command_line: &str) -> String {
    let mut summary = render_summary(receipt);
    if command_line.trim().is_empty() {
        return summary;
    }
    if let Some(prefix) = summary.strip_suffix("---\n") {
        summary = format!("{prefix}Octet command: {command_line}\n---\n");
    }
    summary
}

pub fn render_provenance_jsonl(
    receipt: &EvidenceRailReceipt,
    receipt_json: &str,
    summary: &str,
) -> Result<String, String> {
    let records = vec![
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: source_id_for_receipt(receipt),
            checker_id: receipt.checker_id.clone(),
            evidence_tier: receipt_tier(receipt),
            artifact_locator: RECEIPT_ARTIFACT_NAME.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(receipt_json.as_bytes()),
            checked_scope: receipt.workspace_scope.clone(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: receipt.parent_artifacts.clone(),
        },
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: "octet.runner.artifact-bundle".to_owned(),
            checker_id: crate::evidence::CHECKER_CARGO_OCTET.to_owned(),
            evidence_tier: EvidenceTier::ManualReviewRequired,
            artifact_locator: SUMMARY_ARTIFACT_NAME.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(summary.as_bytes()),
            checked_scope: receipt.workspace_scope.clone(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: Some(SUMMARY_REVIEW_ACTION.to_owned()),
            parent_artifacts: vec![RECEIPT_ARTIFACT_NAME.to_owned()],
        },
    ];
    let mut rendered = String::new();
    for record in records {
        rendered.push_str(&serde_json::to_string(&record).map_err(|error| format!("serialize provenance: {error}"))?);
        rendered.push('\n');
    }
    Ok(rendered)
}

pub fn validate_receipt(receipt: &EvidenceRailReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported evidence rail schema_version `{}`; expected `{RECEIPT_SCHEMA_VERSION}`",
            receipt.schema_version
        ));
    }
    let rail = match EvidenceRailKind::parse(&receipt.rail_id) {
        Ok(rail) => rail,
        Err(error) => {
            diagnostics.push(error);
            return diagnostics;
        }
    };
    if receipt.checker_id != rail.checker_id() {
        diagnostics.push(format!(
            "evidence rail `{}` must use checker_id `{}`, got `{}`",
            receipt.rail_id,
            rail.checker_id(),
            receipt.checker_id
        ));
    }
    if receipt.workspace_scope.trim().is_empty() {
        diagnostics.push("evidence rail receipt missing workspace_scope".to_owned());
    }
    if receipt.imported_input_identity.trim().is_empty() || receipt.bounded_log_identity.trim().is_empty() {
        diagnostics.push("evidence rail receipt missing BLAKE3 input identity".to_owned());
    }
    if receipt.non_claims.is_empty() {
        diagnostics.push("evidence rail receipt must carry at least one explicit non-claim".to_owned());
    }
    diagnostics
}

pub fn expected_bundle_files() -> Vec<String> {
    vec![
        PROVENANCE_ARTIFACT_NAME.to_owned(),
        RECEIPT_ARTIFACT_NAME.to_owned(),
        SUMMARY_ARTIFACT_NAME.to_owned(),
    ]
}

pub fn receipt_tier_from_rail_id(rail_id: &str) -> EvidenceTier {
    EvidenceRailKind::parse(rail_id).map_or(EvidenceTier::PolicyCheck, EvidenceRailKind::evidence_tier)
}

fn parse_rail_input(input: &EvaluationInput<'_>) -> RailParseOutput {
    match input.rail {
        EvidenceRailKind::DependencyAudit => parse_dependency_audit(input),
        EvidenceRailKind::ClippyCoReporting => parse_clippy(input),
        EvidenceRailKind::DependencyTrust => parse_dependency_trust(input),
        EvidenceRailKind::ApiStability => parse_api_stability(input),
        EvidenceRailKind::DynamicEvidence => parse_dynamic_evidence(input),
        EvidenceRailKind::ReleaseSupplyChain => parse_release_supply_chain(input),
        EvidenceRailKind::MutationTesting => parse_mutation_testing(input),
        EvidenceRailKind::CoverageBaseline => parse_coverage_baseline(input),
    }
}

fn parse_dependency_audit(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    if input.config_path.is_none_or(str::is_empty) {
        diagnostics
            .push("dependency-audit rail requires an explicit deny.toml config path or template path".to_owned());
    }
    if input.locked && input.advisory_updated_by_prior_step && input.mode == MODE_RUN {
        diagnostics
            .push("locked dependency-audit evidence must not update advisory databases during the run".to_owned());
    }
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let mut findings = Vec::new();
    if let Some(value) = value {
        validate_dependency_audit_report_shape(&value, &mut diagnostics);
        for item in array_field(&value, "diagnostics") {
            let family = string_field(item, "category").unwrap_or("unknown").to_owned();
            let severity = string_field(item, "severity").unwrap_or("warning").to_owned();
            let message = string_field(item, "message").unwrap_or("cargo-deny finding").to_owned();
            findings.push(EvidenceRailFinding {
                family,
                severity,
                message,
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_clippy(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    for (line_offset, line) in input.import_text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_number = line_offset + 1;
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(format!("malformed Clippy JSON line {line_number}: {error}"));
                continue;
            }
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-message") {
            continue;
        }
        let message = &value["message"];
        let Some(code) = message.get("code").and_then(|code| code.get("code")).and_then(serde_json::Value::as_str)
        else {
            diagnostics.push(format!("Clippy JSON line {line_number} missing message.code.code"));
            continue;
        };
        if !code.starts_with("clippy::") {
            continue;
        }
        let Some(text) = message.get("message").and_then(serde_json::Value::as_str) else {
            diagnostics.push(format!("Clippy JSON line {line_number} missing message.message"));
            continue;
        };
        let primary = primary_span(message);
        findings.push(EvidenceRailFinding {
            family: code.trim_start_matches("clippy::").to_owned(),
            severity: clippy_gate_severity(message.get("level").and_then(serde_json::Value::as_str)),
            message: text.to_owned(),
            source_severity: message.get("level").and_then(serde_json::Value::as_str).map(ToOwned::to_owned),
            file: primary
                .and_then(|span| span.get("file_name"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned),
            line: primary.and_then(|span| span.get("line_start")).and_then(serde_json::Value::as_u64),
            column: primary.and_then(|span| span.get("column_start")).and_then(serde_json::Value::as_u64),
        });
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_dependency_trust(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        for import in array_field(&value, "imports") {
            if string_field(import, "name").is_none_or(str::is_empty)
                || string_field(import, "source").is_none_or(str::is_empty)
            {
                diagnostics.push("dependency-trust import entries must expose name and source identity".to_owned());
            }
        }
        for exemption in array_field(&value, "exemptions") {
            let crate_name = string_field(exemption, "crate").unwrap_or("unknown crate");
            let reason = string_field(exemption, "reason").unwrap_or("missing reason");
            findings.push(EvidenceRailFinding {
                family: "exemption".to_owned(),
                severity: "manual_review".to_owned(),
                message: format!("cargo-vet exemption visible for {crate_name}: {reason}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
        for gap in array_field(&value, "unaudited") {
            let crate_name = string_field(gap, "crate").unwrap_or("unknown crate");
            findings.push(EvidenceRailFinding {
                family: "unaudited-gap".to_owned(),
                severity: "error".to_owned(),
                message: format!("cargo-vet unaudited dependency gap for {crate_name}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.iter().any(|finding| finding.family == "unaudited-gap") {
        STATUS_FAILED.to_owned()
    } else if findings.iter().any(|finding| finding.family == "exemption") {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    } else {
        STATUS_PASSED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_api_stability(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    if input.baseline_identity.is_none() {
        diagnostics.push("api-stability rail requires an accepted baseline path or identity".to_owned());
    }
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        for breakage in array_field(&value, "breakages") {
            let item = string_field(breakage, "item").unwrap_or("public item");
            let kind = string_field(breakage, "kind").unwrap_or("api breakage");
            findings.push(EvidenceRailFinding {
                family: kind.to_owned(),
                severity: "error".to_owned(),
                message: format!("public API compatibility breakage: {item}"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_dynamic_evidence(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    let mut findings = Vec::new();
    let mut profile = input.profile.map(ToOwned::to_owned);
    if let Some(value) = value {
        let selected_profile =
            input.profile.or_else(|| string_field(&value, "profile")).unwrap_or("default").to_owned();
        profile = Some(selected_profile.clone());
        let known_profiles = string_array_field(&value, "known_profiles");
        if !known_profiles.is_empty() && !known_profiles.iter().any(|candidate| candidate == &selected_profile) {
            diagnostics.push(format!("unknown dynamic evidence profile `{selected_profile}`"));
        }
        let command = input.requested_command.or_else(|| string_field(&value, "command")).unwrap_or_default();
        let mut allowed_commands = string_array_field(&value, "allowed_commands");
        if allowed_commands.is_empty()
            && let Some(allowed_command) = allowed_dynamic_command(&selected_profile)
        {
            allowed_commands.push(allowed_command.to_owned());
        }
        if !command.is_empty()
            && !allowed_commands.is_empty()
            && !allowed_commands.iter().any(|allowed| allowed == command)
        {
            diagnostics
                .push(format!("dynamic evidence command `{command}` is not allowed by profile `{selected_profile}`"));
        }
        if value.get("tool_available").and_then(serde_json::Value::as_bool) == Some(false) {
            diagnostics.push(format!("dynamic evidence tool for profile `{selected_profile}` is unavailable"));
        }
        let expected_signal = string_field(&value, "expected_signal").unwrap_or("passed");
        let observed_signal = string_field(&value, "acceptance_signal").unwrap_or("missing");
        if expected_signal != observed_signal {
            diagnostics.push(format!(
                "dynamic evidence acceptance signal mismatch for `{selected_profile}`; expected `{expected_signal}`, got `{observed_signal}`"
            ));
        }
        if value.get("finding_count").and_then(serde_json::Value::as_u64).unwrap_or_default() > 0 {
            findings.push(EvidenceRailFinding {
                family: "dynamic-finding".to_owned(),
                severity: "error".to_owned(),
                message: format!("dynamic evidence profile `{selected_profile}` reported findings"),
                source_severity: None,
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile,
    }
}

fn parse_release_supply_chain(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        let metadata_present = value
            .get("auditable_binary")
            .and_then(|binary| binary.get("dependency_metadata"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !metadata_present {
            diagnostics.push("release supply-chain rail requires auditable Rust dependency metadata".to_owned());
        }
        for vulnerability in array_field(&value, "vulnerabilities") {
            let id = string_field(vulnerability, "id").unwrap_or("unknown-vulnerability");
            let severity = string_field(vulnerability, "severity").unwrap_or("warning");
            findings.push(EvidenceRailFinding {
                family: "vulnerability".to_owned(),
                severity: severity.to_owned(),
                message: format!("vulnerability scan finding {id}"),
                source_severity: Some(severity.to_owned()),
                file: None,
                line: None,
                column: None,
            });
        }
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_FAILED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_mutation_testing(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        push_mutant_findings(
            &value,
            "survived_mutants",
            "survived-mutant",
            "manual_review",
            "survived mutant",
            &mut findings,
        );
        push_mutant_findings(&value, "survivors", "survived-mutant", "manual_review", "survived mutant", &mut findings);
        push_mutant_findings(
            &value,
            "timeout_mutants",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
        push_mutant_findings(&value, "timeouts", "timeout-mutant", "manual_review", "timed-out mutant", &mut findings);
        push_outcome_mutant_findings(&value, &mut findings);
        push_aggregate_mutation_finding(
            &value,
            "survived",
            "survived-mutant",
            "manual_review",
            "survived mutant",
            &mut findings,
        );
        push_aggregate_mutation_finding(
            &value,
            "timeout",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
        push_aggregate_mutation_finding(
            &value,
            "timed_out",
            "timeout-mutant",
            "manual_review",
            "timed-out mutant",
            &mut findings,
        );
    }
    let finding_counts = counts_by_family(&findings);
    let status = if findings.is_empty() {
        STATUS_PASSED.to_owned()
    } else {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    };
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn parse_coverage_baseline(input: &EvaluationInput<'_>) -> RailParseOutput {
    let mut diagnostics = Vec::new();
    if input.baseline_identity.is_none() {
        diagnostics.push("coverage-baseline rail requires an accepted baseline path or identity".to_owned());
    }
    let mut findings = Vec::new();
    let value = parse_json_value(input.import_text, &mut diagnostics);
    if let Some(value) = value {
        validate_coverage_report_shape(&value, &mut diagnostics);
        validate_coverage_summary(&value, &mut diagnostics);
        push_coverage_findings(&value, "regressions", "coverage-regression", "coverage regression", &mut findings);
        push_coverage_findings(
            &value,
            "threshold_failures",
            "coverage-threshold",
            "coverage threshold failure",
            &mut findings,
        );
        push_aggregate_coverage_finding(
            &value,
            "regression_count",
            "coverage-regression",
            "coverage regression",
            &mut findings,
        );
        push_aggregate_coverage_finding(
            &value,
            "threshold_failure_count",
            "coverage-threshold",
            "coverage threshold failure",
            &mut findings,
        );
    }
    let finding_counts = counts_by_family(&findings);
    let status = status_from_error_findings(&findings);
    RailParseOutput {
        findings,
        finding_counts,
        diagnostics,
        status,
        profile: None,
    }
}

fn evidence_input_text(args: &EvidenceRailArgs) -> Result<(&'static str, String, Option<i32>), String> {
    match (&args.input, args.run) {
        (Some(input), false) => {
            let text = fs::read_to_string(input)
                .map_err(|error| format!("read evidence input {}: {error}", input.display()))?;
            Ok((MODE_IMPORT, text, None))
        }
        (None, true) => run_tool(args).map(|output| (MODE_RUN, output.stdout_stderr, Some(output.exit_code))),
        (Some(_), true) => Err("evidence collect accepts either `--input PATH` or `--run`, not both".to_owned()),
        (None, false) => Err("evidence collect requires `--input PATH` or `--run`".to_owned()),
    }
}

struct ToolOutput {
    stdout_stderr: String,
    exit_code: i32,
}

fn run_tool(args: &EvidenceRailArgs) -> Result<ToolOutput, String> {
    let argv = run_mode_command_argv(args)?;
    let (program, rest) = argv.split_first().ok_or_else(|| "evidence rail command argv is empty".to_owned())?;
    let mut command = Command::new(program);
    command.args(rest).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(workspace_root) = args.workspace_root.as_deref() {
        command.current_dir(workspace_root);
    }
    let output = command
        .output()
        .map_err(|error| format!("run evidence rail command `{}`: {error}", argv.join(" ")))?;
    let mut stdout_stderr = String::new();
    stdout_stderr.push_str(&String::from_utf8_lossy(&output.stdout));
    stdout_stderr.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(ToolOutput {
        stdout_stderr,
        exit_code: output.status.code().unwrap_or(EXIT_INTEGRATION_FAILURE),
    })
}

fn run_mode_command_argv(args: &EvidenceRailArgs) -> Result<Vec<String>, String> {
    let config_path = args.config.as_ref().map(|path| path.display().to_string());
    let baseline_identity = args.baseline.as_ref().map(|path| path.display().to_string());
    if args.rail == EvidenceRailKind::DynamicEvidence {
        return dynamic_run_command_argv(args.profile.as_deref(), args.command.as_deref());
    }
    Ok(command_argv(
        args.rail,
        config_path.as_deref(),
        baseline_identity.as_deref(),
        args.command.as_deref(),
    ))
}

fn command_argv(
    rail: EvidenceRailKind,
    config_path: Option<&str>,
    baseline_identity: Option<&str>,
    requested_command: Option<&str>,
) -> Vec<String> {
    match rail {
        EvidenceRailKind::DependencyAudit => {
            let mut argv = string_vec(&["cargo", "deny", "check"]);
            if let Some(config_path) = config_path {
                argv.push("--config".to_owned());
                argv.push(config_path.to_owned());
            }
            argv
        }
        EvidenceRailKind::ClippyCoReporting => string_vec(&["cargo", "clippy", "--message-format=json"]),
        EvidenceRailKind::DependencyTrust => string_vec(&["cargo", "vet", "--output-format=json"]),
        EvidenceRailKind::ApiStability => {
            let mut argv = string_vec(&["cargo", "semver-checks", "check-release"]);
            if let Some(baseline) = baseline_identity {
                argv.push("--baseline-root".to_owned());
                argv.push(baseline.to_owned());
            }
            argv
        }
        EvidenceRailKind::DynamicEvidence => requested_command.map(split_command).unwrap_or_default(),
        EvidenceRailKind::ReleaseSupplyChain => string_vec(&["cargo", "auditable", "build"]),
        EvidenceRailKind::MutationTesting => string_vec(&["cargo", "mutants", "--json"]),
        EvidenceRailKind::CoverageBaseline => string_vec(&["cargo", "llvm-cov", "--json", "--summary-only"]),
    }
}

fn dynamic_run_command_argv(profile: Option<&str>, requested_command: Option<&str>) -> Result<Vec<String>, String> {
    let profile = profile.ok_or_else(|| "dynamic evidence run requires `--profile NAME`".to_owned())?;
    let requested_command =
        requested_command.ok_or_else(|| "dynamic evidence run requires `--command COMMAND`".to_owned())?;
    let allowed_command = allowed_dynamic_command(profile).ok_or_else(|| {
        format!(
            "unknown dynamic evidence profile `{profile}`; expected `{DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY}` or `{DYNAMIC_PROFILE_LOOM_CONCURRENCY}`"
        )
    })?;
    if requested_command != allowed_command {
        return Err(format!(
            "dynamic evidence command `{requested_command}` is not allowed by profile `{profile}`; expected `{allowed_command}`"
        ));
    }
    Ok(split_command(allowed_command))
}

fn allowed_dynamic_command(profile: &str) -> Option<&'static str> {
    match profile {
        DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY | DYNAMIC_PROFILE_MIRI_SHORT => Some(DYNAMIC_COMMAND_MIRI_TEST),
        DYNAMIC_PROFILE_LOOM_CONCURRENCY | DYNAMIC_PROFILE_LOOM_SHORT => Some(DYNAMIC_COMMAND_LOOM_TEST),
        _ => None,
    }
}

fn split_command(command: &str) -> Vec<String> {
    command.split_whitespace().map(ToOwned::to_owned).collect()
}

fn render_cli_output(receipt: &EvidenceRailReceipt, output_format: OutputFormat) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet evidence rail rail={} checker={} status={} artifact={}",
                receipt.rail_id, receipt.checker_id, receipt.status, RECEIPT_ARTIFACT_NAME
            );
            for diagnostic in &receipt.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
        }
        OutputFormat::Json => println!("{}", render_receipt_json(receipt)?.trim_end()),
        OutputFormat::Sarif => unreachable!("SARIF rejected before rendering"),
    }
    Ok(())
}

fn write_artifacts(
    artifact_dir: &Path,
    receipt_json: &str,
    summary: &str,
    provenance_jsonl: &str,
) -> Result<(), String> {
    fs::create_dir_all(artifact_dir).map_err(|error| format!("create {}: {error}", artifact_dir.display()))?;
    remove_stale_artifacts(artifact_dir)?;
    fs::write(artifact_dir.join(RECEIPT_ARTIFACT_NAME), receipt_json)
        .map_err(|error| format!("write {}: {error}", artifact_dir.join(RECEIPT_ARTIFACT_NAME).display()))?;
    fs::write(artifact_dir.join(SUMMARY_ARTIFACT_NAME), summary)
        .map_err(|error| format!("write {}: {error}", artifact_dir.join(SUMMARY_ARTIFACT_NAME).display()))?;
    fs::write(artifact_dir.join(PROVENANCE_ARTIFACT_NAME), provenance_jsonl)
        .map_err(|error| format!("write {}: {error}", artifact_dir.join(PROVENANCE_ARTIFACT_NAME).display()))?;
    Ok(())
}

fn remove_stale_artifacts(artifact_dir: &Path) -> Result<(), String> {
    let expected = expected_bundle_files();
    for entry in fs::read_dir(artifact_dir).map_err(|error| format!("read {}: {error}", artifact_dir.display()))? {
        let entry = entry.map_err(|error| format!("read {} entry: {error}", artifact_dir.display()))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if expected.iter().any(|expected| expected == &file_name) {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|error| format!("remove {}: {error}", path.display()))?;
        } else {
            fs::remove_file(&path).map_err(|error| format!("remove {}: {error}", path.display()))?;
        }
    }
    Ok(())
}

fn exit_code_for_receipt(receipt: &EvidenceRailReceipt) -> i32 {
    if !receipt.diagnostics.is_empty() {
        EXIT_INTEGRATION_FAILURE
    } else if receipt.status == STATUS_PASSED {
        EXIT_SUCCESS
    } else {
        EXIT_RAIL_FAILURE
    }
}

fn source_id_for_receipt(receipt: &EvidenceRailReceipt) -> String {
    EvidenceRailKind::parse(&receipt.rail_id)
        .map(EvidenceRailKind::source_id)
        .unwrap_or("octet.policy.unknown")
        .to_owned()
}

fn receipt_tier(receipt: &EvidenceRailReceipt) -> EvidenceTier {
    receipt_tier_from_rail_id(&receipt.rail_id)
}

fn advisory_database_update(
    rail: EvidenceRailKind,
    locked: bool,
    advisory_updated_by_prior_step: bool,
) -> Option<String> {
    if rail != EvidenceRailKind::DependencyAudit {
        return None;
    }
    if advisory_updated_by_prior_step {
        Some(LOCKED_UPDATED_PRIOR.to_owned())
    } else if locked {
        Some(LOCKED_REUSED.to_owned())
    } else {
        Some(LOCKED_UNAVAILABLE.to_owned())
    }
}

fn validate_dependency_audit_report_shape(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    validate_optional_schema_version(value, DEPENDENCY_AUDIT_SCHEMA_VERSION, "dependency-audit", diagnostics);
    validate_array_records(
        value,
        "diagnostics",
        "dependency-audit diagnostic",
        &["category", "severity", "message"],
        diagnostics,
    );
}

fn validate_coverage_report_shape(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    validate_optional_schema_version(value, COVERAGE_BASELINE_SCHEMA_VERSION, "coverage-baseline", diagnostics);
    let Some(summary) = value.get("summary") else {
        diagnostics.push("coverage-baseline external schema requires `summary` object".to_owned());
        return;
    };
    let Some(summary_object) = summary.as_object() else {
        diagnostics.push("coverage-baseline external schema field `summary` must be an object".to_owned());
        return;
    };
    for field in ["line_percent", "branch_percent", "function_percent"] {
        if let Some(percent) = summary_object.get(field)
            && !percent.is_number()
        {
            diagnostics.push(format!("coverage-baseline external schema field `summary.{field}` must be a number"));
        }
    }
    validate_array_records(value, "regressions", "coverage regression", &["metric"], diagnostics);
    validate_array_records(value, "threshold_failures", "coverage threshold failure", &["metric"], diagnostics);
}

fn validate_optional_schema_version(
    value: &serde_json::Value,
    expected: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(schema_version) = value.get(SCHEMA_VERSION_FIELD) else {
        return;
    };
    match schema_version.as_str() {
        Some(actual) if actual == expected => {}
        Some(actual) => diagnostics
            .push(format!("{label} external schema version `{actual}` is unsupported; expected `{expected}`")),
        None => diagnostics.push(format!("{label} external schema field `{SCHEMA_VERSION_FIELD}` must be a string")),
    }
}

fn validate_array_records(
    value: &serde_json::Value,
    field: &str,
    label: &str,
    required_strings: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let Some(items) = value.get(field) else {
        return;
    };
    let Some(items) = items.as_array() else {
        diagnostics.push(format!("{label} external schema field `{field}` must be an array"));
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let Some(record) = item.as_object() else {
            diagnostics.push(format!("{label} external schema item `{field}[{index}]` must be an object"));
            continue;
        };
        for required in required_strings {
            match record.get(*required).and_then(serde_json::Value::as_str) {
                Some(value) if !value.trim().is_empty() => {}
                _ => diagnostics.push(format!(
                    "{label} external schema item `{field}[{index}]` missing non-empty string `{required}`"
                )),
            }
        }
    }
}

fn parse_json_value(text: &str, diagnostics: &mut Vec<String>) -> Option<serde_json::Value> {
    if text.trim().is_empty() {
        return Some(serde_json::json!({}));
    }
    match serde_json::from_str(text) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed evidence JSON: {error}"));
            None
        }
    }
}

fn array_field<'a>(value: &'a serde_json::Value, field: &str) -> Vec<&'a serde_json::Value> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map_or_else(Vec::new, |items| items.iter().collect())
}

fn string_array_field(value: &serde_json::Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(|items| items.iter().filter_map(serde_json::Value::as_str).map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

fn string_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(serde_json::Value::as_str)
}

fn u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    value.get(field).and_then(serde_json::Value::as_u64)
}

fn f64_field(value: &serde_json::Value, field: &str) -> Option<f64> {
    value.get(field).and_then(serde_json::Value::as_f64)
}

fn summary_u64_field(value: &serde_json::Value, field: &str) -> Option<u64> {
    value
        .get("summary")
        .and_then(|summary| u64_field(summary, field))
        .or_else(|| u64_field(value, field))
}

fn push_mutant_findings(
    value: &serde_json::Value,
    field: &str,
    family: &str,
    severity: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    for mutant in array_field(value, field) {
        findings.push(mutant_finding(mutant, family, severity, label));
    }
}

fn push_outcome_mutant_findings(value: &serde_json::Value, findings: &mut Vec<EvidenceRailFinding>) {
    for outcome in array_field(value, "outcomes") {
        let status = string_field(outcome, "status").or_else(|| string_field(outcome, "outcome"));
        match status {
            Some("survived" | "missed") => {
                findings.push(mutant_finding(outcome, "survived-mutant", "manual_review", "survived mutant"))
            }
            Some("timeout" | "timed-out" | "timed_out") => {
                findings.push(mutant_finding(outcome, "timeout-mutant", "manual_review", "timed-out mutant"))
            }
            _ => {}
        }
    }
}

fn push_aggregate_mutation_finding(
    value: &serde_json::Value,
    count_field: &str,
    family: &str,
    severity: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    if findings.iter().any(|finding| finding.family == family) {
        return;
    }
    let Some(count) = summary_u64_field(value, count_field) else {
        return;
    };
    if count == 0 {
        return;
    }
    findings.push(EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: format!("{label} count: {count}"),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    });
}

fn mutant_finding(mutant: &serde_json::Value, family: &str, severity: &str, label: &str) -> EvidenceRailFinding {
    let name = string_field(mutant, "name")
        .or_else(|| string_field(mutant, "id"))
        .or_else(|| string_field(mutant, "description"))
        .unwrap_or("unnamed mutant");
    EvidenceRailFinding {
        family: family.to_owned(),
        severity: severity.to_owned(),
        message: format!("{label}: {name}"),
        source_severity: None,
        file: string_field(mutant, "file").map(ToOwned::to_owned),
        line: u64_field(mutant, "line"),
        column: u64_field(mutant, "column"),
    }
}

fn validate_coverage_summary(value: &serde_json::Value, diagnostics: &mut Vec<String>) {
    let summary = value.get("summary").unwrap_or(value);
    let percent_fields = [
        ("line_percent", "line coverage"),
        ("branch_percent", "branch coverage"),
        ("function_percent", "function coverage"),
    ];
    let mut saw_percentage = false;
    for (field, label) in percent_fields {
        let Some(percent) = f64_field(summary, field) else {
            continue;
        };
        saw_percentage = true;
        if !(COVERAGE_PERCENT_MIN..=COVERAGE_PERCENT_MAX).contains(&percent) {
            diagnostics.push(format!(
                "coverage-baseline rail {label} percentage `{percent}` must be between {COVERAGE_PERCENT_MIN} and {COVERAGE_PERCENT_MAX}"
            ));
        }
    }
    if !saw_percentage {
        diagnostics.push("coverage-baseline rail requires line, branch, or function percentage summary".to_owned());
    }
}

fn push_coverage_findings(
    value: &serde_json::Value,
    field: &str,
    family: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    for item in array_field(value, field) {
        let metric = string_field(item, "metric").unwrap_or("coverage metric");
        let message = string_field(item, "message").map_or_else(|| format!("{label}: {metric}"), ToOwned::to_owned);
        findings.push(EvidenceRailFinding {
            family: family.to_owned(),
            severity: "error".to_owned(),
            message,
            source_severity: None,
            file: None,
            line: None,
            column: None,
        });
    }
}

fn push_aggregate_coverage_finding(
    value: &serde_json::Value,
    count_field: &str,
    family: &str,
    label: &str,
    findings: &mut Vec<EvidenceRailFinding>,
) {
    if findings.iter().any(|finding| finding.family == family) {
        return;
    }
    let Some(count) = u64_field(value, count_field) else {
        return;
    };
    if count == 0 {
        return;
    }
    findings.push(EvidenceRailFinding {
        family: family.to_owned(),
        severity: "error".to_owned(),
        message: format!("{label} count: {count}"),
        source_severity: None,
        file: None,
        line: None,
        column: None,
    });
}

fn primary_span(message: &serde_json::Value) -> Option<&serde_json::Value> {
    message.get("spans").and_then(serde_json::Value::as_array).and_then(|spans| {
        spans.iter().find(|span| span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true))
    })
}

fn clippy_gate_severity(source_level: Option<&str>) -> String {
    match source_level {
        Some("error") => "error".to_owned(),
        Some(level) => format!("source-{level}"),
        None => "source-warning".to_owned(),
    }
}

fn counts_by_family(findings: &[EvidenceRailFinding]) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.family.clone()).or_insert(0) += 1;
    }
    counts
}

fn status_from_error_findings(findings: &[EvidenceRailFinding]) -> String {
    if findings.iter().any(|finding| finding.severity == "error") {
        STATUS_FAILED.to_owned()
    } else {
        STATUS_PASSED.to_owned()
    }
}

fn path_identity(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {} for BLAKE3 identity: {error}", path.display()))?;
    Ok(prefixed_identity(&bytes))
}

fn path_identity_if_present(path: &Path) -> Result<Option<String>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(prefixed_identity(&bytes))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("read {} for BLAKE3 identity: {error}", path.display())),
    }
}

fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", crate::evidence::artifact_blake3_hex(bytes))
}

fn string_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKSPACE_SCOPE: &str = "fixture-workspace";
    const DENY_CONFIG: &str = "deny.toml";
    const BASELINE_IDENTITY: &str = "b3:baseline";
    const CLIPPY_JSON: &str = r#"{"reason":"compiler-message","message":{"code":{"code":"clippy::unwrap_used"},"level":"warning","message":"used unwrap","spans":[{"is_primary":true,"file_name":"src/lib.rs","line_start":10,"column_start":5}]}}
"#;

    #[test]
    fn dependency_audit_clean_fixture_passes_with_locked_boundary() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some(DENY_CONFIG),
            baseline_identity: None,
            lockfile_identity: Some("b3:lock".to_owned()),
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_PASSED);
        assert_eq!(receipt.checker_id, crate::evidence::CHECKER_CARGO_DENY);
        assert_eq!(receipt.advisory_database_update.as_deref(), Some(LOCKED_REUSED));
        assert!(receipt.command_argv.iter().any(|arg| arg == DENY_CONFIG));
    }

    #[test]
    fn dependency_audit_external_schema_rejects_malformed_report_shape() {
        let receipt = dependency_audit_receipt(
            r#"{"schema_version":"cargo-deny-diagnostics/v0","diagnostics":[{"category":"licenses","severity":7}]}"#,
            Some(DENY_CONFIG),
        );

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(
            receipt
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `severity`"))
        );
        assert!(
            receipt
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `message`"))
        );
    }

    #[test]
    fn dependency_audit_rejects_missing_config_and_preserves_denied_category() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[{"category":"licenses","severity":"error","message":"license denied"}]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("deny.toml")));
        assert_eq!(receipt.findings[0].family, "licenses");
        assert_eq!(receipt.finding_counts.get("licenses"), Some(&1));
    }

    #[test]
    fn clippy_import_preserves_source_severity_without_static_lint_claim() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text: CLIPPY_JSON,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let receipt_json = render_receipt_json(&receipt).expect("receipt renders");
        let summary = render_summary(&receipt);
        let provenance = render_provenance_jsonl(&receipt, &receipt_json, &summary).expect("provenance renders");

        assert_eq!(receipt.status, STATUS_PASSED);
        assert_eq!(receipt.findings[0].source_severity.as_deref(), Some("warning"));
        assert_eq!(receipt.findings[0].severity, "source-warning");
        assert!(receipt.non_claims.iter().any(|claim| claim.contains("not Octet source lint replacements")));
        assert!(provenance.contains("\"evidence_tier\":\"static_lint\""));
    }

    #[test]
    fn clippy_import_rejects_malformed_json_and_missing_required_fields() {
        let receipt = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text: "not-json\n{\"reason\":\"compiler-message\",\"message\":{}}\n",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });

        assert_eq!(receipt.status, STATUS_FAILED);
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed Clippy JSON")));
        assert!(receipt.diagnostics.iter().any(|diagnostic| diagnostic.contains("message.code.code")));
    }

    #[test]
    fn dependency_trust_records_coverage_exemptions_gaps_and_import_identity() {
        let covered = dependency_trust_receipt(
            r#"{"imports":[{"name":"google","source":"https://example.invalid/audits.toml"}],"exemptions":[],"unaudited":[]}"#,
        );
        assert_eq!(covered.status, STATUS_PASSED);

        let exempted = dependency_trust_receipt(
            r#"{"imports":[{"name":"local","source":"audits.toml"}],"exemptions":[{"crate":"serde","reason":"bootstrap"}],"unaudited":[]}"#,
        );
        assert_eq!(exempted.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(exempted.finding_counts.get("exemption"), Some(&1));

        let gap = dependency_trust_receipt(
            r#"{"imports":[{"name":"missing-source"}],"exemptions":[],"unaudited":[{"crate":"bytes"}]}"#,
        );
        assert_eq!(gap.status, STATUS_FAILED);
        assert!(gap.diagnostics.iter().any(|diagnostic| diagnostic.contains("name and source identity")));
        assert_eq!(gap.finding_counts.get("unaudited-gap"), Some(&1));
    }

    #[test]
    fn dependency_audit_and_trust_remain_separate_entries() {
        let audit = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text: r#"{"diagnostics":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: Some(DENY_CONFIG),
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        let trust = dependency_trust_receipt(r#"{"imports":[],"exemptions":[],"unaudited":[]}"#);

        assert_ne!(audit.rail_id, trust.rail_id);
        assert_ne!(audit.checker_id, trust.checker_id);
        assert!(trust.non_claims.iter().any(|claim| claim.contains("separate from dependency audit")));
    }

    #[test]
    fn api_stability_requires_baseline_and_reports_removed_public_item() {
        let compatible = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ApiStability,
            mode: MODE_IMPORT,
            import_text: r#"{"breakages":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        assert_eq!(compatible.status, STATUS_PASSED);

        let removed = evaluate(EvaluationInput {
            baseline_identity: None,
            import_text: r#"{"breakages":[{"kind":"removed-public-item","item":"pub fn old"}]}"#,
            ..api_stability_input()
        });
        assert_eq!(removed.status, STATUS_FAILED);
        assert!(removed.diagnostics.iter().any(|diagnostic| diagnostic.contains("accepted baseline")));
        assert_eq!(removed.finding_counts.get("removed-public-item"), Some(&1));
    }

    #[test]
    fn dynamic_evidence_accepts_configured_profile_and_rejects_profile_errors() {
        let valid = evaluate(EvaluationInput {
            rail: EvidenceRailKind::DynamicEvidence,
            mode: MODE_IMPORT,
            import_text: r#"{"known_profiles":["miri"],"profile":"miri","allowed_commands":["cargo miri test"],"command":"cargo miri test","tool_available":true,"expected_signal":"passed","acceptance_signal":"passed","finding_count":0}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("miri"),
            requested_command: Some("cargo miri test"),
            parent_artifacts: Vec::new(),
            tool_exit_code: Some(EXIT_SUCCESS),
        });
        assert_eq!(valid.status, STATUS_PASSED);
        assert!(valid.non_claims.iter().any(|claim| claim.contains("not required by default")));

        let invalid = evaluate(EvaluationInput {
            import_text: r#"{"known_profiles":["loom"],"profile":"miri","allowed_commands":["cargo loom test"],"command":"cargo miri test","tool_available":false,"expected_signal":"passed","acceptance_signal":"failed","finding_count":1}"#,
            ..dynamic_input()
        });
        assert_eq!(invalid.status, STATUS_FAILED);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown dynamic evidence profile")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("not allowed")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("unavailable")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("acceptance signal mismatch")));
    }

    #[test]
    fn release_supply_chain_records_auditable_metadata_sbom_vulnerabilities_and_parents() {
        let parent = "artifacts/octet/status.json".to_owned();
        let clean = evaluate(EvaluationInput {
            rail: EvidenceRailKind::ReleaseSupplyChain,
            mode: MODE_IMPORT,
            import_text: r#"{"auditable_binary":{"dependency_metadata":true},"sbom":{"packages":2},"vulnerabilities":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: vec![parent.clone()],
            tool_exit_code: None,
        });
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.parent_artifacts, vec![parent]);

        let vulnerable = evaluate(EvaluationInput {
            import_text: r#"{"auditable_binary":{"dependency_metadata":false},"vulnerabilities":[{"id":"RUSTSEC-0000-0000","severity":"critical"}]}"#,
            ..release_input()
        });
        assert_eq!(vulnerable.status, STATUS_FAILED);
        assert!(
            vulnerable
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("auditable Rust dependency metadata"))
        );
        assert_eq!(vulnerable.finding_counts.get("vulnerability"), Some(&1));
        assert!(vulnerable.non_claims.iter().any(|claim| claim.contains("not hosted-service attestations")));
    }

    #[test]
    fn mutation_testing_records_clean_and_survived_mutants() {
        const SURVIVED_MUTANT_LINE: u64 = 42;

        let clean = mutation_receipt(r#"{"summary":{"total":2,"caught":2,"survived":0,"timeout":0},"outcomes":[]}"#);
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.checker_id, crate::evidence::CHECKER_CARGO_MUTANTS);
        assert!(clean.non_claims.iter().any(|claim| claim.contains("does not prove the tests are complete")));

        let survived = mutation_receipt(
            r#"{"summary":{"total":2,"caught":1,"survived":1,"timeout":0},"survived_mutants":[{"name":"replace + with -","file":"src/lib.rs","line":42}]}"#,
        );
        assert_eq!(survived.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(survived.finding_counts.get("survived-mutant"), Some(&1));
        assert_eq!(survived.findings[0].line, Some(SURVIVED_MUTANT_LINE));
    }

    #[test]
    fn coverage_baseline_external_schema_rejects_bad_summary_shape() {
        let bad_version = coverage_receipt(
            r#"{"schema_version":"cargo-llvm-cov-summary/v0","summary":{"line_percent":"many"},"regressions":[{}],"threshold_failures":[]}"#,
            Some(BASELINE_IDENTITY.to_owned()),
        );

        assert_eq!(bad_version.status, STATUS_FAILED);
        assert!(bad_version.diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported")));
        assert!(bad_version.diagnostics.iter().any(|diagnostic| diagnostic.contains("summary.line_percent")));
        assert!(
            bad_version
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-empty string `metric`"))
        );
    }

    #[test]
    fn coverage_baseline_requires_baseline_and_reports_regression() {
        let clean = coverage_receipt(
            r#"{"summary":{"line_percent":92.5,"branch_percent":81.0,"function_percent":100.0},"regressions":[],"threshold_failures":[]}"#,
            Some(BASELINE_IDENTITY.to_owned()),
        );
        assert_eq!(clean.status, STATUS_PASSED);
        assert_eq!(clean.checker_id, crate::evidence::CHECKER_CARGO_LLVM_COV);
        assert_eq!(clean.baseline_identity.as_deref(), Some(BASELINE_IDENTITY));
        assert!(clean.non_claims.iter().any(|claim| claim.contains("does not prove behavioral correctness")));

        let regressed = coverage_receipt(
            r#"{"summary":{"line_percent":88.0},"regressions":[{"metric":"line_percent","message":"line coverage fell below accepted baseline"}]}"#,
            None,
        );
        assert_eq!(regressed.status, STATUS_FAILED);
        assert!(regressed.diagnostics.iter().any(|diagnostic| diagnostic.contains("accepted baseline")));
        assert_eq!(regressed.finding_counts.get("coverage-regression"), Some(&1));
    }

    #[test]
    fn checked_in_fixture_matrix_covers_positive_and_negative_rails() {
        let clean_deny = include_str!("../tests/fixtures/evidence-rails/dependency-audit-clean.json");
        let denied = include_str!("../tests/fixtures/evidence-rails/dependency-audit-denied.json");
        let clippy = include_str!("../tests/fixtures/evidence-rails/clippy-warning.jsonl");
        let trust = include_str!("../tests/fixtures/evidence-rails/dependency-trust-covered.json");
        let semver_removed = include_str!("../tests/fixtures/evidence-rails/api-stability-removed.json");
        let dynamic_mismatch = include_str!("../tests/fixtures/evidence-rails/dynamic-profile-mismatch.json");
        let release_clean = include_str!("../tests/fixtures/evidence-rails/release-supply-chain-clean.json");
        let mutation_clean = include_str!("../tests/fixtures/evidence-rails/mutation-testing-clean.json");
        let coverage_regression = include_str!("../tests/fixtures/evidence-rails/coverage-baseline-regression.json");

        assert_eq!(dependency_audit_receipt(clean_deny, Some(DENY_CONFIG)).status, STATUS_PASSED);
        assert_eq!(dependency_audit_receipt(denied, Some(DENY_CONFIG)).finding_counts.get("licenses"), Some(&1));
        assert_eq!(clippy_receipt(clippy).findings[0].family, "unwrap_used");
        assert_eq!(dependency_trust_receipt(trust).status, STATUS_PASSED);
        assert_eq!(api_removed_receipt(semver_removed).finding_counts.get("removed-public-item"), Some(&1));
        assert!(!dynamic_mismatch_receipt(dynamic_mismatch).diagnostics.is_empty());
        assert_eq!(release_receipt(release_clean).status, STATUS_PASSED);
        assert_eq!(mutation_receipt(mutation_clean).status, STATUS_PASSED);
        assert_eq!(coverage_receipt(coverage_regression, Some(BASELINE_IDENTITY.to_owned())).status, STATUS_FAILED);
    }

    #[test]
    fn run_mode_command_argv_covers_selected_external_tools() {
        assert_eq!(command_argv(EvidenceRailKind::DependencyAudit, Some(DENY_CONFIG), None, None), vec![
            "cargo",
            "deny",
            "check",
            "--config",
            DENY_CONFIG
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ClippyCoReporting, None, None, None), vec![
            "cargo",
            "clippy",
            "--message-format=json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::DependencyTrust, None, None, None), vec![
            "cargo",
            "vet",
            "--output-format=json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ApiStability, None, Some("baseline"), None), vec![
            "cargo",
            "semver-checks",
            "check-release",
            "--baseline-root",
            "baseline"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::DynamicEvidence, None, None, Some("cargo miri test")), vec![
            "cargo", "miri", "test"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::ReleaseSupplyChain, None, None, None), vec![
            "cargo",
            "auditable",
            "build"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::MutationTesting, None, None, None), vec![
            "cargo", "mutants", "--json"
        ]);
        assert_eq!(command_argv(EvidenceRailKind::CoverageBaseline, None, None, None), vec![
            "cargo",
            "llvm-cov",
            "--json",
            "--summary-only"
        ]);
    }

    #[test]
    fn dynamic_run_mode_rejects_unconfigured_commands_before_spawn() {
        let args = crate::cli::EvidenceRailArgs {
            rail: EvidenceRailKind::DynamicEvidence,
            input: None,
            artifact_dir: std::path::PathBuf::from("artifacts/octet-evidence"),
            workspace_root: None,
            config: None,
            baseline: None,
            profile: Some(DYNAMIC_PROFILE_MIRI_UNSAFE_BOUNDARY.to_owned()),
            command: Some("cargo test".to_owned()),
            parent_artifacts: Vec::new(),
            output_format: OutputFormat::Human,
            locked: false,
            update_advisory_database: false,
            run: true,
            command_line: "cargo octet evidence collect --rail dynamic-evidence --run".to_owned(),
        };

        let error = run_mode_command_argv(&args).expect_err("disallowed dynamic command is rejected before spawn");
        assert!(error.contains("not allowed by profile"));

        let allowed = crate::cli::EvidenceRailArgs {
            command: Some(DYNAMIC_COMMAND_MIRI_TEST.to_owned()),
            ..args
        };
        assert_eq!(run_mode_command_argv(&allowed).expect("allowed dynamic command"), vec!["cargo", "miri", "test"]);
    }

    #[test]
    fn receipt_validation_rejects_schema_checker_and_non_claim_drift() {
        let mut receipt = dependency_trust_receipt(r#"{"imports":[],"exemptions":[],"unaudited":[]}"#);
        receipt.schema_version = "old".to_owned();
        receipt.checker_id = "wrong".to_owned();
        receipt.non_claims.clear();

        let diagnostics = validate_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("schema_version")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("checker_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
    }

    fn dependency_audit_receipt(import_text: &str, config_path: Option<&str>) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyAudit,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path,
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn clippy_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::ClippyCoReporting,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn dependency_trust_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            rail: EvidenceRailKind::DependencyTrust,
            mode: MODE_IMPORT,
            import_text,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        })
    }

    fn api_removed_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..api_stability_input()
        })
    }

    fn dynamic_mismatch_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..dynamic_input()
        })
    }

    fn release_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..release_input()
        })
    }

    fn mutation_receipt(import_text: &str) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            ..mutation_input()
        })
    }

    fn coverage_receipt(import_text: &str, baseline_identity: Option<String>) -> EvidenceRailReceipt {
        evaluate(EvaluationInput {
            import_text,
            baseline_identity,
            ..coverage_input()
        })
    }

    fn api_stability_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::ApiStability,
            mode: MODE_IMPORT,
            import_text: r#"{"breakages":[]}"#,
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn dynamic_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::DynamicEvidence,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("miri"),
            requested_command: Some("cargo miri test"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn release_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::ReleaseSupplyChain,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn mutation_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::MutationTesting,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }

    fn coverage_input() -> EvaluationInput<'static> {
        EvaluationInput {
            rail: EvidenceRailKind::CoverageBaseline,
            mode: MODE_IMPORT,
            import_text: "{}",
            workspace_scope: WORKSPACE_SCOPE,
            config_path: None,
            baseline_identity: Some(BASELINE_IDENTITY.to_owned()),
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        }
    }
}
