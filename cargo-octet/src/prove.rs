use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use octet_standards::artifacts::POLICIES_JSON_FILE;
use octet_standards::model::EnforcementMode;
use octet_standards::model::FamilySelection;
use octet_standards::model::PolicyEntry;
use octet_standards::model::PolicyRegistry;
use octet_standards::model::ProofRail;
use octet_standards::proof::proof_receipt_metadata;
use octet_standards::proof::validate_proof_receipt;
use octet_standards::source::SourceFile;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::ProveArgs;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;

const REPORT_SCHEMA_VERSION: u32 = 1;
const EXIT_PROOF_VALID: i32 = 0;
const EXIT_PROOF_INVALID: i32 = 1;
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const PROOF_ARTIFACT_DIR_NAME: &str = "proofs";
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_SCHEMA_VERSION: &str = "octet-prove-status/v1";
const BUNDLE_KIND_PROVE: &str = "prove";
const PROVE_PROFILE_NAME: &str = "prove-verification";
const PROOF_SOURCE_ID: &str = "octet.standards.formal-proof";
const RUNNER_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const CHECKER_OCTET_STANDARDS: &str = "octet-standards";
const CHECKER_CARGO_OCTET: &str = "cargo-octet";
const MANUAL_ACTION: &str = "review cargo octet prove command, status, and scoped proof receipt before release";
const DEFAULT_PROOF_RECEIPT_NAME: &str = "proof-receipt.txt";
const SAFE_SEPARATOR: char = '-';
const CARGO_MANIFEST_FILE: &str = "Cargo.toml";

#[derive(Debug, Serialize)]
struct ProveReport {
    schema_version: u32,
    status: &'static str,
    artifact_dir: String,
    proofs: Vec<ProofReportRow>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ProofReportRow {
    policy_id: String,
    rail_index: usize,
    backend: String,
    checked_scope: String,
    receipt: String,
    status: &'static str,
    diagnostics: Vec<String>,
}

#[derive(Debug, Serialize)]
struct StatusArtifact {
    schema_version: &'static str,
    bundle_kind: &'static str,
    status: &'static str,
    output_format: &'static str,
    metadata: StatusMetadata,
    proofs: Vec<ProofReportRow>,
}

#[derive(Debug, Serialize)]
struct StatusMetadata {
    profile_name: &'static str,
}

struct SelectedRail<'a> {
    policy: &'a PolicyEntry,
    rail_index: usize,
    rail: &'a ProofRail,
}

struct ExecutedProof {
    row: ProofReportRow,
    receipt_text: String,
    formal_provenance: Option<ProvenanceRecord>,
}

struct ReceiptOutput {
    text: String,
    command_succeeded: bool,
    command_diagnostic: Option<String>,
}

pub fn run(args: ProveArgs) -> Result<i32, String> {
    let registry = load_registry(&args)?;
    let sources = load_sources(&args.source)?;
    let selection = prove_selection(&args.family);
    let selected_rails = selected_rails(&registry, &sources, &selection);
    let mut diagnostics = Vec::new();
    if selected_rails.is_empty() {
        diagnostics.push("no selected proof rails matched the provided sources".to_owned());
    }
    let working_dir = proof_command_working_dir(&args.source);
    let executions = selected_rails
        .into_iter()
        .map(|selected| execute_proof(selected, &sources, &working_dir))
        .collect::<Vec<_>>();
    diagnostics.extend(executions.iter().flat_map(|execution| execution.row.diagnostics.clone()));

    let report = ProveReport {
        schema_version: REPORT_SCHEMA_VERSION,
        status: if diagnostics.is_empty() {
            STATUS_PASSED
        } else {
            STATUS_FAILED
        },
        artifact_dir: args.artifact_dir.display().to_string(),
        proofs: executions.iter().map(|execution| execution.row.clone()).collect(),
        diagnostics,
    };
    write_artifacts(&args, &report, &executions)?;
    print_report(args.output_format, &report)?;

    Ok(if report.diagnostics.is_empty() {
        EXIT_PROOF_VALID
    } else {
        EXIT_PROOF_INVALID
    })
}

fn load_registry(args: &ProveArgs) -> Result<PolicyRegistry, String> {
    let registry_path = if let Some(registry) = &args.registry {
        registry.clone()
    } else {
        args.policy_artifacts_dir
            .as_ref()
            .expect("parser requires a registry source")
            .join(POLICIES_JSON_FILE)
    };
    let text =
        fs::read_to_string(&registry_path).map_err(|error| format!("read {}: {error}", registry_path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("parse {} as policy registry JSON: {error}", registry_path.display()))
}

fn load_sources(paths: &[PathBuf]) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    for path in paths {
        append_sources(path, &mut sources)?;
    }
    Ok(sources)
}

fn append_sources(path: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    if path.is_dir() {
        append_directory_sources(path, sources)
    } else {
        append_file_source(path, sources)
    }
}

fn append_directory_sources(directory: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("read source directory {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(|error| format!("read source directory {} entry: {error}", directory.display()))?;
    entries.sort_by_key(std::fs::DirEntry::path);
    for entry in entries {
        append_sources(&entry.path(), sources)?;
    }
    Ok(())
}

fn append_file_source(path: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read source {}: {error}", path.display()))?;
    sources.push(SourceFile::new(path.display().to_string(), text));
    Ok(())
}

fn prove_selection(families: &[String]) -> FamilySelection {
    if families.is_empty() {
        FamilySelection::from_families(["verification"])
    } else {
        FamilySelection::from_families(families.iter().cloned())
    }
}

fn selected_rails<'a>(
    registry: &'a PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<SelectedRail<'a>> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| {
            policy.proof_rails.iter().enumerate().filter(|(_, rail)| rail_matches_sources(rail, sources)).map(
                move |(rail_index, rail)| SelectedRail {
                    policy,
                    rail_index,
                    rail,
                },
            )
        })
        .collect()
}

fn policy_is_verified(policy: &PolicyEntry) -> bool {
    policy.current_enforcement_mode == EnforcementMode::Verified
        || policy.target_enforcement_mode == EnforcementMode::Verified
}

fn rail_matches_sources(rail: &ProofRail, sources: &[SourceFile]) -> bool {
    rail.source_paths.iter().any(|pattern| sources.iter().any(|source| source.path.contains(pattern)))
}

fn execute_proof(selected: SelectedRail<'_>, sources: &[SourceFile], working_dir: &Path) -> ExecutedProof {
    let receipt_locator = proof_artifact_locator(selected.policy, selected.rail_index);
    let checked_scope = checked_scope(selected.policy, selected.rail_index);
    let output = run_proof_command(selected.rail, sources, working_dir);
    let receipt_source_path = first_proof_path(selected.rail).to_owned();
    let proof_source = SourceFile::new(receipt_source_path, output.text.clone());
    let mut diagnostics = validate_proof_receipt(selected.rail, sources, &proof_source);
    if let Some(command_diagnostic) = output.command_diagnostic {
        diagnostics.push(command_diagnostic);
    }
    if !output.command_succeeded {
        diagnostics.push(format!("proof command `{}` exited without success", selected.rail.command));
    }
    let verifier_identity = selected.rail.receipt_verifier.verifier_id.trim().to_owned();
    if verifier_identity.is_empty() {
        diagnostics.push("proof rail is missing verifier identity required for formal_proof provenance".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();

    let status = if diagnostics.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    };
    let formal_provenance = diagnostics
        .is_empty()
        .then(|| proof_provenance_record(&receipt_locator, output.text.as_bytes(), &checked_scope, &verifier_identity));
    ExecutedProof {
        row: ProofReportRow {
            policy_id: selected.policy.policy_id.clone(),
            rail_index: selected.rail_index,
            backend: selected.rail.backend.clone(),
            checked_scope,
            receipt: receipt_locator,
            status,
            diagnostics,
        },
        receipt_text: output.text,
        formal_provenance,
    }
}

fn proof_command_working_dir(source_paths: &[PathBuf]) -> PathBuf {
    source_paths.first().map_or_else(
        || PathBuf::from("."),
        |path| {
            let start = if path.is_dir() {
                path.as_path()
            } else {
                path.parent().unwrap_or_else(|| Path::new("."))
            };
            nearest_manifest_directory(start).unwrap_or_else(|| start.to_path_buf())
        },
    )
}

fn nearest_manifest_directory(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(directory) = current {
        if directory.join(CARGO_MANIFEST_FILE).is_file() {
            return Some(non_empty_directory(directory));
        }
        current = directory.parent();
    }
    None
}

fn non_empty_directory(directory: &Path) -> PathBuf {
    if directory.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        directory.to_path_buf()
    }
}

fn run_proof_command(rail: &ProofRail, sources: &[SourceFile], working_dir: &Path) -> ReceiptOutput {
    let output = Command::new("sh").arg("-c").arg(&rail.command).current_dir(working_dir).output();
    proof_receipt_output(rail, sources, working_dir, output)
}

fn proof_receipt_output(
    rail: &ProofRail,
    sources: &[SourceFile],
    working_dir: &Path,
    output: Result<Output, io::Error>,
) -> ReceiptOutput {
    let metadata = proof_receipt_metadata(rail, sources);
    match output {
        Ok(output) => ReceiptOutput {
            text: format!(
                "proof-command: {}\nproof-cwd: {}\n{}exit-status: {}\nstdout:\n{}\nstderr:\n{}\n",
                rail.command,
                working_dir.display(),
                metadata,
                output.status.code().map_or_else(|| "signal".to_owned(), |code| code.to_string()),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            command_succeeded: output.status.success(),
            command_diagnostic: None,
        },
        Err(error) => ReceiptOutput {
            text: format!(
                "proof-command: {}\nproof-cwd: {}\n{}exit-status: spawn-error\nstderr:\n{}\n",
                rail.command,
                working_dir.display(),
                metadata,
                error
            ),
            command_succeeded: false,
            command_diagnostic: Some(format!("spawn proof command `{}`: {error}", rail.command)),
        },
    }
}

fn first_proof_path(rail: &ProofRail) -> &str {
    rail.proof_paths.first().map_or(DEFAULT_PROOF_RECEIPT_NAME, String::as_str)
}

fn checked_scope(policy: &PolicyEntry, rail_index: usize) -> String {
    format!("formal-proof:{}#rail-{rail_index}", policy.policy_id)
}

fn proof_artifact_locator(policy: &PolicyEntry, rail_index: usize) -> String {
    format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.receipt", safe_artifact_stem(&policy.policy_id))
}

fn safe_artifact_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                SAFE_SEPARATOR
            }
        })
        .collect()
}

fn proof_provenance_record(
    artifact_locator: &str,
    contents: &[u8],
    checked_scope: &str,
    verifier_identity: &str,
) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: PROOF_SOURCE_ID.to_owned(),
        checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
        evidence_tier: EvidenceTier::FormalProof,
        artifact_locator: artifact_locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: Some(verifier_identity.to_owned()),
        proof_artifact_identity: Some(artifact_locator.to_owned()),
        human_action: None,
        parent_artifacts: vec![COMMAND_ARTIFACT_NAME.to_owned(), STATUS_ARTIFACT_NAME.to_owned()],
    }
}

fn write_artifacts(args: &ProveArgs, report: &ProveReport, executions: &[ExecutedProof]) -> Result<(), String> {
    fs::create_dir_all(&args.artifact_dir)
        .map_err(|error| format!("create artifact directory {}: {error}", args.artifact_dir.display()))?;
    fs::create_dir_all(args.artifact_dir.join(PROOF_ARTIFACT_DIR_NAME)).map_err(|error| {
        format!(
            "create proof artifact directory {}: {error}",
            args.artifact_dir.join(PROOF_ARTIFACT_DIR_NAME).display()
        )
    })?;

    let command_text = format!("{}\n", args.command_line);
    let status_text = status_text(args.output_format, report)?;
    let summary_text = summary_text(report);
    let provenance_text = provenance_text(&command_text, &status_text, &summary_text, executions)?;

    write_text(&args.artifact_dir.join(COMMAND_ARTIFACT_NAME), &command_text)?;
    write_text(&args.artifact_dir.join(STATUS_ARTIFACT_NAME), &status_text)?;
    write_text(&args.artifact_dir.join(SUMMARY_ARTIFACT_NAME), &summary_text)?;
    write_text(&args.artifact_dir.join(PROVENANCE_ARTIFACT_NAME), &provenance_text)?;
    for execution in executions {
        write_text(&args.artifact_dir.join(&execution.row.receipt), &execution.receipt_text)?;
    }
    Ok(())
}

fn status_text(output_format: OutputFormat, report: &ProveReport) -> Result<String, String> {
    let artifact = StatusArtifact {
        schema_version: STATUS_SCHEMA_VERSION,
        bundle_kind: BUNDLE_KIND_PROVE,
        status: report.status,
        output_format: output_format.as_str(),
        metadata: StatusMetadata {
            profile_name: PROVE_PROFILE_NAME,
        },
        proofs: report.proofs.clone(),
    };
    json_text(&artifact)
}

fn summary_text(report: &ProveReport) -> String {
    let mut text = format!(
        "octet prove status={} proofs={} diagnostics={}\n",
        report.status,
        report.proofs.len(),
        report.diagnostics.len()
    );
    for proof in &report.proofs {
        text.push_str(&format!(
            "proof policy_id={} rail_index={} backend={} status={} receipt={} checked_scope={}\n",
            proof.policy_id, proof.rail_index, proof.backend, proof.status, proof.receipt, proof.checked_scope
        ));
    }
    for diagnostic in &report.diagnostics {
        text.push_str(&format!("diagnostic: {diagnostic}\n"));
    }
    text
}

fn provenance_text(
    command_text: &str,
    status_text: &str,
    summary_text: &str,
    executions: &[ExecutedProof],
) -> Result<String, String> {
    let mut records = vec![
        runner_provenance_record(COMMAND_ARTIFACT_NAME, command_text.as_bytes()),
        runner_provenance_record(STATUS_ARTIFACT_NAME, status_text.as_bytes()),
        runner_provenance_record(SUMMARY_ARTIFACT_NAME, summary_text.as_bytes()),
    ];
    records.extend(executions.iter().filter_map(|execution| execution.formal_provenance.clone()));
    let mut text = String::new();
    for record in records {
        text.push_str(
            &serde_json::to_string(&record).map_err(|error| format!("serialize provenance record: {error}"))?,
        );
        text.push('\n');
    }
    Ok(text)
}

fn runner_provenance_record(artifact_locator: &str, contents: &[u8]) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: RUNNER_SOURCE_ID.to_owned(),
        checker_id: CHECKER_CARGO_OCTET.to_owned(),
        evidence_tier: EvidenceTier::ManualReviewRequired,
        artifact_locator: artifact_locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents),
        checked_scope: format!("profile:{PROVE_PROFILE_NAME}"),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(MANUAL_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    }
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn json_text<T: Serialize>(value: &T) -> Result<String, String> {
    let text = serde_json::to_string_pretty(value).map_err(|error| format!("serialize JSON: {error}"))?;
    Ok(format!("{text}\n"))
}

fn print_report(output_format: OutputFormat, report: &ProveReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove artifact_dir={} status={} proofs={} diagnostics={}",
                report.artifact_dir,
                report.status,
                report.proofs.len(),
                report.diagnostics.len()
            );
            for proof in &report.proofs {
                println!(
                    "proof policy_id={} rail_index={} backend={} status={} receipt={} checked_scope={}",
                    proof.policy_id, proof.rail_index, proof.backend, proof.status, proof.receipt, proof.checked_scope
                );
            }
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).map_err(|error| format!("serialize report: {error}"))?);
            Ok(())
        }
        OutputFormat::Sarif => Err("prove does not support SARIF output; use human or json".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use octet_standards::model::Classification;
    use octet_standards::model::EnforcementLevel;
    use octet_standards::model::FixtureCoverage;
    use octet_standards::model::FixtureStatus;
    use octet_standards::model::ReceiptProvenance;
    use octet_standards::model::ReceiptVerifier;
    use octet_standards::model::Severity;
    use octet_standards::model::SuppressionRequirements;

    use super::*;
    use crate::artifact::ArtifactBundleInput;
    use crate::artifact::BundleFile;
    use crate::artifact::verify_bundle;

    const POLICY_ID: &str = "octet.verus_proof_rails.fixture";
    const SOURCE_RELATIVE_PATH: &str = "src/verified/addition.rs";
    const PASS_SIGNAL: &str = "proof-result: pass fixture-verus";
    const VERIFIER_ID: &str = "octet-verus@fixture-v1";
    const TRUST_BOUNDARY: &str = "fixture verifies the proof plumbing only";

    #[test]
    fn prove_writes_valid_formal_proof_artifact_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL);
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove fixture".to_owned(),
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_VALID);
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "valid", "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.artifacts.iter().any(|artifact| artifact.evidence_tier == "formal_proof"));
    }

    #[test]
    fn prove_fails_without_pass_signal_and_does_not_emit_formal_proof() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        let registry = write_fixture_registry(temp.path(), "proof-result: pass never-printed");
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(!provenance.contains("formal_proof"));
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "invalid");
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("unexpected artifact")));
    }

    fn write_fixture_source(root: &Path) -> PathBuf {
        let path = root.join(SOURCE_RELATIVE_PATH);
        fs::create_dir_all(path.parent().expect("source parent")).expect("create source parent");
        fs::write(&path, "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n").expect("write source");
        path
    }

    fn write_fixture_registry(root: &Path, pass_signal: &str) -> PathBuf {
        let path = root.join("registry.json");
        let registry = fixture_registry(root, pass_signal);
        fs::write(&path, serde_json::to_string_pretty(&registry).expect("registry json")).expect("write registry");
        path
    }

    fn fixture_registry(root: &Path, pass_signal: &str) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec![CHECKER_OCTET_STANDARDS.to_owned()],
            policies: vec![fixture_policy(root, pass_signal)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn fixture_policy(root: &Path, pass_signal: &str) -> PolicyEntry {
        PolicyEntry {
            policy_id: POLICY_ID.to_owned(),
            pillar: "verification".to_owned(),
            source_path: "docs/standards/source/verification.adoc".to_owned(),
            source_heading: "Verus proof rails".to_owned(),
            source_anchor: "docs/standards/source/verification.adoc#verus-proof-rails".to_owned(),
            summary: "fixture policy".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            escalation_criteria: "fixture".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: octet_standards::model::EvidenceCommand::default(),
            proof_rails: vec![fixture_rail(root, pass_signal)],
            checker_owner: "verification".to_owned(),
            checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: SuppressionRequirements {
                requires_policy_id: true,
                requires_source_section: true,
                requires_owner: true,
                requires_rationale: true,
                requires_review_trigger: true,
                requires_safer_default: true,
            },
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Covered,
                compliant: vec![SOURCE_RELATIVE_PATH.to_owned()],
                noncompliant: vec!["missing-pass-signal".to_owned()],
            },
            verification_evidence_requirements: vec!["fixture".to_owned()],
            classification_rationale: "fixture".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "fixture".to_owned(),
        }
    }

    fn fixture_rail(root: &Path, pass_signal: &str) -> ProofRail {
        ProofRail {
            backend: "verus".to_owned(),
            command: format!("printf '{}\\n'", PASS_SIGNAL),
            source_paths: vec![root.join(SOURCE_RELATIVE_PATH).display().to_string()],
            proof_paths: vec!["proofs/fixture.receipt".to_owned()],
            pass_signal: pass_signal.to_owned(),
            trust_boundaries: vec![TRUST_BOUNDARY.to_owned()],
            receipt_binding: "blake3-command-and-source".to_owned(),
            receipt_provenance: ReceiptProvenance {
                schema_version: "octet-verus-proof-receipt/v1".to_owned(),
                runner: CHECKER_OCTET_STANDARDS.to_owned(),
                environment: "cargo-octet-prove-fixture".to_owned(),
                requires_backend: true,
                requires_trust_boundary: true,
            },
            receipt_verifier: ReceiptVerifier {
                verifier_id: VERIFIER_ID.to_owned(),
                verifier_digest: "fixture-digest".to_owned(),
                ..ReceiptVerifier::default()
            },
        }
    }

    fn bundle_input(artifact_dir: &Path) -> ArtifactBundleInput {
        ArtifactBundleInput {
            artifact_dir: artifact_dir.display().to_string(),
            files: collect_bundle_files(artifact_dir, artifact_dir),
        }
    }

    fn collect_bundle_files(root: &Path, directory: &Path) -> Vec<BundleFile> {
        let mut entries = fs::read_dir(directory)
            .expect("read artifact directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read artifact entries");
        entries.sort_by_key(std::fs::DirEntry::path);
        let mut files = Vec::new();
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_bundle_files(root, &path));
                continue;
            }
            let relative = path.strip_prefix(root).expect("relative artifact").display().to_string();
            files.push(BundleFile {
                name: relative,
                contents: fs::read(&path).expect("read artifact"),
            });
        }
        files
    }
}
