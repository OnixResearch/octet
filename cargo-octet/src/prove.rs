use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use octet_standards::artifacts::POLICIES_JSON_FILE;
use octet_standards::model::EnforcementMode;
use octet_standards::model::FamilySelection;
use octet_standards::model::PolicyEntry;
use octet_standards::model::PolicyRegistry;
use octet_standards::model::ProofRail;
use octet_standards::proof::PROOF_STATUS_FAILED;
use octet_standards::proof::PROOF_STATUS_PASSED;
use octet_standards::proof::VERIFIER_MODE_FIXTURE;
use octet_standards::proof::VERIFIER_MODE_PRODUCTION;
use octet_standards::proof::VerusLogArtifact;
use octet_standards::proof::VerusProofReceiptInput;
use octet_standards::proof::build_verus_proof_receipt;
use octet_standards::proof::matching_sources;
use octet_standards::proof::path_pattern_diagnostics;
use octet_standards::proof::validate_proof_receipt_with_identity;
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
const SAFE_SEPARATOR: char = '-';
const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
const DEFAULT_PROOF_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_LOG_MAX_BYTES: usize = 4_096;
const PROCESS_POLL_INTERVAL_MS: u64 = 10;
const COMMAND_STDOUT_SUFFIX: &str = "stdout";
const COMMAND_STDERR_SUFFIX: &str = "stderr";
const COMMAND_ARGV_FIELD: &str = "command_argv";
const DEFAULT_ENV_ALLOWLIST: &[&str] = &["PATH", "TMPDIR"];

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
    verifier_mode: String,
    checked_scope: String,
    receipt: String,
    status: &'static str,
    diagnostics: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RailListReport {
    schema_version: u32,
    selected_rails: Vec<RailListRow>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct RailListRow {
    policy_id: String,
    rail_index: usize,
    backend: String,
    source_paths: Vec<String>,
    proof_paths: Vec<String>,
    receipt_paths: Vec<String>,
    verifier_label: String,
    verifier_mode: String,
    checked_scope: String,
    non_claim: String,
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

struct CommandSpec {
    executable: String,
    args: Vec<String>,
    argv: Vec<String>,
}

struct CommandOutcome {
    command_succeeded: bool,
    timed_out: bool,
    stdout_text: String,
    stdout_original_bytes: usize,
    stdout_truncated: bool,
    stderr_text: String,
    stderr_original_bytes: usize,
    stderr_truncated: bool,
    command_blake3: String,
    diagnostics: Vec<String>,
}

pub fn run(args: ProveArgs) -> Result<i32, String> {
    let registry = load_registry(&args)?;
    let working_dir = proof_command_working_dir(&args.source);
    let sources = load_sources(&args.source, &working_dir)?;
    let selection = prove_selection(&args.family);
    let selected_rails = selected_rails(&registry, &sources, &selection);
    if args.list {
        let diagnostics = list_diagnostics(&selected_rails, &sources);
        print_rail_list(args.output_format, &rail_list_report(&selected_rails, diagnostics))?;
        return Ok(EXIT_PROOF_VALID);
    }

    let mut diagnostics = Vec::new();
    if selected_rails.is_empty() {
        diagnostics.push("no selected proof rails matched the provided sources".to_owned());
    }
    let executions = selected_rails
        .into_iter()
        .map(|selected| execute_proof(selected, &sources, &working_dir, args.explain))
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

fn load_sources(paths: &[PathBuf], root: &Path) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    for path in paths {
        append_sources(path, root, &mut sources)?;
    }
    Ok(sources)
}

fn append_sources(path: &Path, root: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    if path.is_dir() {
        append_directory_sources(path, root, sources)
    } else {
        append_file_source(path, root, sources)
    }
}

fn append_directory_sources(directory: &Path, root: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("read source directory {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(|error| format!("read source directory {} entry: {error}", directory.display()))?;
    entries.sort_by_key(std::fs::DirEntry::path);
    for entry in entries {
        append_sources(&entry.path(), root, sources)?;
    }
    Ok(())
}

fn append_file_source(path: &Path, root: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read source {}: {error}", path.display()))?;
    sources.push(SourceFile::new(repo_relative_display(path, root), text));
    Ok(())
}

fn repo_relative_display(path: &Path, root: &Path) -> String {
    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let absolute_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    absolute.strip_prefix(&absolute_root).map_or_else(|_| normalized_display(path), normalized_display)
}

fn normalized_display(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
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
    !matching_sources(&rail.source_paths, sources).is_empty()
}

fn execute_proof(
    selected: SelectedRail<'_>,
    sources: &[SourceFile],
    working_dir: &Path,
    explain: bool,
) -> ExecutedProof {
    let receipt_locator = proof_artifact_locator(selected.policy, selected.rail_index);
    let stdout_locator = proof_log_artifact_locator(selected.policy, selected.rail_index, COMMAND_STDOUT_SUFFIX);
    let stderr_locator = proof_log_artifact_locator(selected.policy, selected.rail_index, COMMAND_STDERR_SUFFIX);
    let checked_scope = checked_scope(selected.policy, selected.rail_index, selected.rail);
    let mut diagnostics = path_pattern_diagnostics(&selected.rail.source_paths, sources, "source_paths");
    let (proof_sources, proof_source_diagnostics) = load_proof_sources(selected.rail, working_dir);
    diagnostics.extend(proof_source_diagnostics);
    let outcome = run_proof_command(selected.rail, working_dir);
    diagnostics.extend(outcome.diagnostics.clone());
    if outcome.timed_out {
        diagnostics.push(format!("proof command `{}` timed out", command_display(selected.rail)));
    }
    if !outcome.command_succeeded {
        diagnostics.push(format!("proof command `{}` exited without success", command_display(selected.rail)));
    }
    if !outcome.stdout_text.contains(&selected.rail.pass_signal) {
        diagnostics.push(format!("proof command stdout missing pass signal `{}`", selected.rail.pass_signal));
    }

    let command_status = if diagnostics.is_empty() {
        PROOF_STATUS_PASSED
    } else {
        PROOF_STATUS_FAILED
    };
    let stdout_artifact =
        log_artifact(&stdout_locator, &outcome.stdout_text, outcome.stdout_original_bytes, outcome.stdout_truncated);
    let stderr_artifact =
        log_artifact(&stderr_locator, &outcome.stderr_text, outcome.stderr_original_bytes, outcome.stderr_truncated);
    let receipt_result = build_verus_proof_receipt(&VerusProofReceiptInput {
        policy_id: &selected.policy.policy_id,
        rail_index: selected.rail_index,
        rail: selected.rail,
        sources,
        proof_sources: &proof_sources,
        checked_scope: &checked_scope,
        status: command_status,
        command_blake3: &outcome.command_blake3,
        proof_artifact_identity: &receipt_locator,
        stdout_artifact,
        stderr_artifact,
    });
    let receipt_text = match receipt_result {
        Ok(receipt) => serde_json::to_string_pretty(&receipt).map(|text| format!("{text}\n")).unwrap_or_else(|error| {
            format!("{{\"schema_version\":\"receipt-serialization-error\",\"error\":{error:?}}}\n")
        }),
        Err(errors) => {
            diagnostics.extend(errors);
            failed_receipt_text(&selected.policy.policy_id, selected.rail_index, &diagnostics)
        }
    };
    let proof_source = SourceFile::new(receipt_locator.clone(), receipt_text.clone());
    let mut validation_sources = sources.to_vec();
    validation_sources.extend(proof_sources);
    diagnostics.extend(validate_proof_receipt_with_identity(
        selected.rail,
        Some(&selected.policy.policy_id),
        Some(selected.rail_index),
        &validation_sources,
        &proof_source,
    ));
    if explain {
        diagnostics = diagnostics.into_iter().map(|diagnostic| format!("explain: {diagnostic}")).collect();
    }
    diagnostics.sort();
    diagnostics.dedup();

    let status = if diagnostics.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    };
    let formal_provenance = diagnostics.is_empty().then(|| {
        proof_provenance_record(
            &receipt_locator,
            receipt_text.as_bytes(),
            &checked_scope,
            &selected.rail.receipt_verifier.verifier_id,
        )
    });
    ExecutedProof {
        row: ProofReportRow {
            policy_id: selected.policy.policy_id.clone(),
            rail_index: selected.rail_index,
            backend: selected.rail.backend.clone(),
            verifier_mode: verifier_mode(selected.rail).to_owned(),
            checked_scope,
            receipt: receipt_locator,
            status,
            diagnostics,
        },
        receipt_text,
        formal_provenance,
    }
}

fn load_proof_sources(rail: &ProofRail, working_dir: &Path) -> (Vec<SourceFile>, Vec<String>) {
    let mut sources = Vec::new();
    let mut diagnostics = Vec::new();
    for proof_path in &rail.proof_paths {
        if proof_path.contains('*') || proof_path.contains('?') {
            diagnostics.push(format!(
                "proof rail field `proof_paths` pattern `{proof_path}` must be passed as an explicit proof source for receipt construction"
            ));
            continue;
        }
        let path = working_dir.join(proof_path);
        match fs::read_to_string(&path) {
            Ok(text) => sources.push(SourceFile::new(proof_path.clone(), text)),
            Err(error) => diagnostics.push(format!("read proof path {}: {error}", path.display())),
        }
    }
    (sources, diagnostics)
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

fn run_proof_command(rail: &ProofRail, working_dir: &Path) -> CommandOutcome {
    let spec = match proof_command_spec(rail) {
        Ok(spec) => spec,
        Err(diagnostic) => {
            return CommandOutcome {
                command_succeeded: false,
                timed_out: false,
                stdout_text: String::new(),
                stdout_original_bytes: 0,
                stdout_truncated: false,
                stderr_text: diagnostic.clone(),
                stderr_original_bytes: diagnostic.len(),
                stderr_truncated: false,
                command_blake3: command_blake3(&rail.command_argv),
                diagnostics: vec![diagnostic],
            };
        }
    };
    let output = spawn_and_wait(&spec, rail, working_dir);
    let command_blake3 = command_blake3(&spec.argv);
    match output {
        Ok(output) => {
            let stdout_limit = configured_log_limit(rail.stdout_max_bytes);
            let stderr_limit = configured_log_limit(rail.stderr_max_bytes);
            let bounded_stdout = bounded_utf8(&output.stdout, stdout_limit);
            let bounded_stderr = bounded_utf8(&output.stderr, stderr_limit);
            CommandOutcome {
                command_succeeded: output.status.is_some_and(|status| status.success()) && !output.timed_out,
                timed_out: output.timed_out,
                stdout_text: bounded_stdout.text,
                stdout_original_bytes: bounded_stdout.original_bytes,
                stdout_truncated: bounded_stdout.truncated,
                stderr_text: bounded_stderr.text,
                stderr_original_bytes: bounded_stderr.original_bytes,
                stderr_truncated: bounded_stderr.truncated,
                command_blake3,
                diagnostics: output.diagnostics,
            }
        }
        Err(error) => CommandOutcome {
            command_succeeded: false,
            timed_out: false,
            stdout_text: String::new(),
            stdout_original_bytes: 0,
            stdout_truncated: false,
            stderr_text: error.clone(),
            stderr_original_bytes: error.len(),
            stderr_truncated: false,
            command_blake3,
            diagnostics: vec![error],
        },
    }
}

fn proof_command_spec(rail: &ProofRail) -> Result<CommandSpec, String> {
    let executable = rail.command_argv.first().cloned().ok_or_else(|| {
        format!("proof rail field `{COMMAND_ARGV_FIELD}` is required; refusing to execute shell command string")
    })?;
    let args = rail.command_argv.iter().skip(1).cloned().collect::<Vec<_>>();
    Ok(CommandSpec {
        executable,
        args,
        argv: rail.command_argv.clone(),
    })
}

struct CommandOutput {
    status: Option<ExitStatus>,
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    diagnostics: Vec<String>,
}

fn spawn_and_wait(spec: &CommandSpec, rail: &ProofRail, working_dir: &Path) -> Result<CommandOutput, String> {
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.args)
        .current_dir(working_dir)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in allowed_environment(rail) {
        command.env(name, value);
    }
    let mut child =
        command.spawn().map_err(|error| format!("spawn proof command `{}`: {error}", spec.argv.join(" ")))?;
    let timeout = Duration::from_millis(configured_timeout_ms(rail));
    let poll_interval = Duration::from_millis(PROCESS_POLL_INTERVAL_MS);
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| format!("collect proof command `{}` output: {error}", spec.argv.join(" ")))?;
                return Ok(CommandOutput {
                    status: Some(status),
                    timed_out: false,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    diagnostics: Vec::new(),
                });
            }
            Ok(None) if started.elapsed() >= timeout => {
                let mut diagnostics = Vec::new();
                if let Err(error) = child.kill() {
                    diagnostics.push(format!("kill timed-out proof command `{}`: {error}", spec.argv.join(" ")));
                }
                let output = child.wait_with_output().map_err(|error| {
                    format!("collect timed-out proof command `{}` output: {error}", spec.argv.join(" "))
                })?;
                return Ok(CommandOutput {
                    status: Some(output.status),
                    timed_out: true,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    diagnostics,
                });
            }
            Ok(None) => thread::sleep(poll_interval),
            Err(error) => return Err(format!("poll proof command `{}`: {error}", spec.argv.join(" "))),
        }
    }
}

fn allowed_environment(rail: &ProofRail) -> Vec<(String, OsString)> {
    let mut names = DEFAULT_ENV_ALLOWLIST.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    names.extend(rail.environment_allowlist.iter().cloned());
    names.sort();
    names.dedup();
    names.into_iter().filter_map(|name| std::env::var_os(&name).map(|value| (name, value))).collect()
}

struct BoundedText {
    text: String,
    original_bytes: usize,
    truncated: bool,
}

fn bounded_utf8(bytes: &[u8], max_bytes: usize) -> BoundedText {
    let captured_len = bytes.len().min(max_bytes);
    BoundedText {
        text: String::from_utf8_lossy(&bytes[..captured_len]).to_string(),
        original_bytes: bytes.len(),
        truncated: bytes.len() > captured_len,
    }
}

fn configured_timeout_ms(rail: &ProofRail) -> u64 {
    if rail.timeout_ms == 0 {
        DEFAULT_PROOF_TIMEOUT_MS
    } else {
        rail.timeout_ms
    }
}

fn configured_log_limit(limit: usize) -> usize {
    if limit == 0 { DEFAULT_LOG_MAX_BYTES } else { limit }
}

fn command_blake3(argv: &[String]) -> String {
    let text = serde_json::to_string(argv).expect("command argv should serialize");
    crate::evidence::artifact_blake3_hex(text.as_bytes())
}

fn log_artifact(locator: &str, text: &str, original_bytes: usize, truncated: bool) -> VerusLogArtifact {
    VerusLogArtifact {
        locator: locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(text.as_bytes()),
        bytes: original_bytes,
        truncated,
    }
}

fn checked_scope(policy: &PolicyEntry, rail_index: usize, rail: &ProofRail) -> String {
    if !rail.checked_scope.trim().is_empty() {
        rail.checked_scope.clone()
    } else if verifier_mode(rail) == VERIFIER_MODE_PRODUCTION {
        format!("formal-proof:production:{}#rail-{rail_index}", policy.policy_id)
    } else {
        format!("formal-proof:fixture-only:{}#rail-{rail_index}", policy.policy_id)
    }
}

fn verifier_mode(rail: &ProofRail) -> &str {
    if rail.verifier_mode.trim().is_empty() {
        VERIFIER_MODE_FIXTURE
    } else {
        rail.verifier_mode.as_str()
    }
}

fn proof_artifact_locator(policy: &PolicyEntry, rail_index: usize) -> String {
    format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.receipt", safe_artifact_stem(&policy.policy_id))
}

fn proof_log_artifact_locator(policy: &PolicyEntry, rail_index: usize, suffix: &str) -> String {
    format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.{suffix}", safe_artifact_stem(&policy.policy_id))
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

fn failed_receipt_text(policy_id: &str, rail_index: usize, diagnostics: &[String]) -> String {
    let value = serde_json::json!({
        "schema_version": "octet-verus-proof-receipt/v1",
        "policy_id": policy_id,
        "rail_index": rail_index,
        "status": PROOF_STATUS_FAILED,
        "diagnostics": diagnostics,
    });
    format!("{}\n", serde_json::to_string_pretty(&value).expect("failed receipt json"))
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
            "proof policy_id={} rail_index={} backend={} verifier_mode={} status={} receipt={} checked_scope={}\n",
            proof.policy_id,
            proof.rail_index,
            proof.backend,
            proof.verifier_mode,
            proof.status,
            proof.receipt,
            proof.checked_scope
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
                    "proof policy_id={} rail_index={} backend={} verifier_mode={} status={} receipt={} checked_scope={}",
                    proof.policy_id,
                    proof.rail_index,
                    proof.backend,
                    proof.verifier_mode,
                    proof.status,
                    proof.receipt,
                    proof.checked_scope
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

fn rail_list_report(selected_rails: &[SelectedRail<'_>], diagnostics: Vec<String>) -> RailListReport {
    RailListReport {
        schema_version: REPORT_SCHEMA_VERSION,
        selected_rails: selected_rails.iter().map(rail_list_row).collect(),
        diagnostics,
    }
}

fn rail_list_row(selected: &SelectedRail<'_>) -> RailListRow {
    RailListRow {
        policy_id: selected.policy.policy_id.clone(),
        rail_index: selected.rail_index,
        backend: selected.rail.backend.clone(),
        source_paths: selected.rail.source_paths.clone(),
        proof_paths: selected.rail.proof_paths.clone(),
        receipt_paths: if selected.rail.receipt_paths.is_empty() {
            selected.rail.proof_paths.clone()
        } else {
            selected.rail.receipt_paths.clone()
        },
        verifier_label: selected.rail.receipt_verifier.verifier_id.clone(),
        verifier_mode: verifier_mode(selected.rail).to_owned(),
        checked_scope: checked_scope(selected.policy, selected.rail_index, selected.rail),
        non_claim: "scoped proof evidence only; not whole-program correctness, certification, semantic equivalence, or full safety".to_owned(),
    }
}

fn list_diagnostics(selected_rails: &[SelectedRail<'_>], sources: &[SourceFile]) -> Vec<String> {
    selected_rails
        .iter()
        .flat_map(|selected| path_pattern_diagnostics(&selected.rail.source_paths, sources, "source_paths"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn print_rail_list(output_format: OutputFormat, report: &RailListReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove rails selected={} diagnostics={}",
                report.selected_rails.len(),
                report.diagnostics.len()
            );
            for rail in &report.selected_rails {
                println!(
                    "rail policy_id={} rail_index={} backend={} verifier={} mode={} scope={} sources={} proofs={}",
                    rail.policy_id,
                    rail.rail_index,
                    rail.backend,
                    rail.verifier_label,
                    rail.verifier_mode,
                    rail.checked_scope,
                    rail.source_paths.join(","),
                    rail.proof_paths.join(",")
                );
                println!("non-claim: {}", rail.non_claim);
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

fn command_display(rail: &ProofRail) -> String {
    if rail.command_argv.is_empty() {
        rail.command.clone()
    } else {
        rail.command_argv.join(" ")
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
    const PROOF_RELATIVE_PATH: &str = "verus/proofs/addition_bound.rs";
    const PASS_SIGNAL: &str = "proof-result: pass fixture-verus";
    const WRONG_PASS_SIGNAL: &str = "proof-result: pass never-printed";
    const VERIFIER_ID: &str = "octet-verus@fixture-v1";
    const TRUST_BOUNDARY: &str = "fixture verifies the proof plumbing only";
    const SHORT_TIMEOUT_MS: u64 = 1;
    const TEST_TIMEOUT_MS: u64 = 1_000;
    const TEST_LOG_LIMIT_BYTES: usize = 128;

    #[test]
    fn prove_writes_valid_formal_proof_artifact_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_VALID);
        let receipt_text = fs::read_to_string(
            artifact_dir
                .join(proof_artifact_locator(&fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL)), 0)),
        )
        .expect("read receipt");
        assert!(receipt_text.contains("source_bindings"));
        assert!(receipt_text.contains("proof_bindings"));
        assert!(receipt_text.contains("stdout_artifact"));
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "valid", "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.artifacts.iter().any(|artifact| artifact.evidence_tier == "formal_proof"));
    }

    #[test]
    fn prove_fails_without_pass_signal_and_does_not_emit_formal_proof() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), WRONG_PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: true,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(!provenance.contains("formal_proof"));
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains("explain: proof command stdout missing pass signal"));
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "invalid");
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("unexpected artifact")));
    }

    #[test]
    fn prove_rejects_missing_command_argv_without_shell_execution() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, Vec::new());
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir,
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
    }

    #[test]
    fn prove_reports_spawn_failure_nonzero_exit_and_timeout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());

        let spawn = run_fixture_with_command(temp.path(), &source, vec!["missing-octet-verifier".to_owned()]);
        assert!(spawn.contains("spawn proof command"));

        let nonzero = run_fixture_with_command(temp.path(), &source, vec!["false".to_owned()]);
        assert!(nonzero.contains("exited without success"));

        let timeout = run_fixture_with_timeout(temp.path(), &source, vec!["sleep".to_owned(), "1".to_owned()]);
        assert!(timeout.contains("timed out"));
    }

    #[test]
    fn prove_list_reports_selected_rail_without_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove --list fixture".to_owned(),
            list: true,
            explain: false,
        })
        .expect("prove list");

        assert_eq!(code, EXIT_PROOF_VALID);
        assert!(!artifact_dir.exists(), "list mode should not write artifact bundles");
    }

    fn run_fixture_with_command(root: &Path, source: &Path, argv: Vec<String>) -> String {
        let registry = write_fixture_registry(root, PASS_SIGNAL, argv);
        let artifact_dir =
            root.join(format!("artifacts-{}", safe_artifact_stem(&format!("{:?}", std::time::Instant::now()))));
        let code = run(ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source.to_path_buf()],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_INVALID);
        fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary")
    }

    fn run_fixture_with_timeout(root: &Path, source: &Path, argv: Vec<String>) -> String {
        let registry_path = root.join("registry-timeout.json");
        let mut registry = fixture_registry(root, PASS_SIGNAL, argv);
        registry.policies[0].proof_rails[0].timeout_ms = SHORT_TIMEOUT_MS;
        fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("registry json"))
            .expect("write registry");
        let artifact_dir = root.join("artifacts-timeout");
        let code = run(ProveArgs {
            registry: Some(registry_path),
            policy_artifacts_dir: None,
            source: vec![source.to_path_buf()],
            family: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_INVALID);
        fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary")
    }

    fn command_argv(pass_signal: &str) -> Vec<String> {
        vec!["printf".to_owned(), "%s\\n".to_owned(), pass_signal.to_owned()]
    }

    fn write_fixture_source(root: &Path) -> PathBuf {
        fs::write(root.join(CARGO_MANIFEST_FILE), "[package]\nname = \"proof-fixture\"\nversion = \"0.0.0\"\n")
            .expect("write fixture manifest");
        let path = root.join(SOURCE_RELATIVE_PATH);
        fs::create_dir_all(path.parent().expect("source parent")).expect("create source parent");
        fs::write(&path, "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n").expect("write source");
        path
    }

    fn write_fixture_proof(root: &Path) -> PathBuf {
        let path = root.join(PROOF_RELATIVE_PATH);
        fs::create_dir_all(path.parent().expect("proof parent")).expect("create proof parent");
        fs::write(&path, "verus! { proof fn addition_bound() ensures true {} }\n").expect("write proof");
        path
    }

    fn write_fixture_registry(root: &Path, pass_signal: &str, argv: Vec<String>) -> PathBuf {
        let path = root.join("registry.json");
        let registry = fixture_registry(root, pass_signal, argv);
        fs::write(&path, serde_json::to_string_pretty(&registry).expect("registry json")).expect("write registry");
        path
    }

    fn fixture_registry(root: &Path, pass_signal: &str, argv: Vec<String>) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec![CHECKER_OCTET_STANDARDS.to_owned()],
            policies: vec![fixture_policy(root, pass_signal, argv)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn fixture_policy(root: &Path, pass_signal: &str, argv: Vec<String>) -> PolicyEntry {
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
            proof_rails: vec![fixture_rail(root, pass_signal, argv)],
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

    fn fixture_rail(_root: &Path, pass_signal: &str, argv: Vec<String>) -> ProofRail {
        ProofRail {
            backend: "verus".to_owned(),
            command: argv.join(" "),
            command_argv: argv,
            source_paths: vec![SOURCE_RELATIVE_PATH.to_owned()],
            proof_paths: vec![PROOF_RELATIVE_PATH.to_owned()],
            receipt_paths: vec![proof_artifact_locator(&fixture_policy_stub(), 0)],
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
                verifier_binary_path: "octet-verus".to_owned(),
                verifier_binary_digest: "fixture-binary-digest".to_owned(),
                verifier_nix_closure: "fixture-closure".to_owned(),
                verifier_nix_closure_digest: "fixture-closure-digest".to_owned(),
                toolchain_id: "verus-fixture-wrapper/v1".to_owned(),
                toolchain_digest: "fixture-toolchain-digest".to_owned(),
                ..ReceiptVerifier::default()
            },
            verifier_mode: VERIFIER_MODE_FIXTURE.to_owned(),
            checked_scope: format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"),
            timeout_ms: TEST_TIMEOUT_MS,
            stdout_max_bytes: TEST_LOG_LIMIT_BYTES,
            stderr_max_bytes: TEST_LOG_LIMIT_BYTES,
            environment_allowlist: vec!["PATH".to_owned()],
        }
    }

    fn fixture_policy_stub() -> PolicyEntry {
        PolicyEntry {
            policy_id: POLICY_ID.to_owned(),
            pillar: "verification".to_owned(),
            source_path: "fixture".to_owned(),
            source_heading: "fixture".to_owned(),
            source_anchor: "fixture".to_owned(),
            summary: "fixture".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            escalation_criteria: "fixture".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: octet_standards::model::EvidenceCommand::default(),
            proof_rails: Vec::new(),
            checker_owner: "fixture".to_owned(),
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
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec!["fixture".to_owned()],
            classification_rationale: "fixture".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "fixture".to_owned(),
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
