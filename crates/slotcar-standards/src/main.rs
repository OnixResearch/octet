#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! CLI shell for `slotcar-standards`.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use slotcar_standards::artifacts::ARTIFACT_MANIFEST_JSON_FILE;
use slotcar_standards::artifacts::ArtifactManifest;
use slotcar_standards::artifacts::CHECK_CONFIG_TOML_FILE;
use slotcar_standards::artifacts::CONFIG_RECEIPT_FAMILIES_JSON_FILE;
use slotcar_standards::artifacts::POLICIES_JSON_FILE;
use slotcar_standards::artifacts::PROOF_BUNDLES_JSON_FILE;
use slotcar_standards::artifacts::ProofBundleArtifact;
use slotcar_standards::artifacts::SEED_MANIFESTS_JSON_FILE;
use slotcar_standards::artifacts::artifact_manifest;
use slotcar_standards::artifacts::config_receipt_family_artifact;
use slotcar_standards::artifacts::proof_bundle_artifact;
use slotcar_standards::artifacts::registry_has_verified_proof_rails;
use slotcar_standards::artifacts::render_check_config_toml;
use slotcar_standards::artifacts::seed_manifest_artifact;
use slotcar_standards::artifacts::validate_artifact_manifest;
use slotcar_standards::artifacts::validate_artifact_manifest_texts;
use slotcar_standards::artifacts::validate_proof_bundle_artifact;
use slotcar_standards::diagnostic::Diagnostic;
use slotcar_standards::diagnostic::DiagnosticInput;
use slotcar_standards::fixture::FixtureManifest;
use slotcar_standards::fixture::validate_hard_policy_fixture_coverage;
use slotcar_standards::model::FamilySelection;
use slotcar_standards::model::PolicyRegistry;
use slotcar_standards::model::Severity;
use slotcar_standards::receipt::AcceptanceReceipt;
use slotcar_standards::receipt::AcceptanceStatus;
use slotcar_standards::receipt::ExportReceiptInput;
use slotcar_standards::receipt::ReceiptArtifact;
use slotcar_standards::receipt::ReceiptDigest;
use slotcar_standards::receipt::ReceiptEvidence;
use slotcar_standards::receipt::ReceiptInput;
use slotcar_standards::receipt::ReceiptVerificationCommand;
use slotcar_standards::receipt::RunProfile;
use slotcar_standards::report::CheckReport;
use slotcar_standards::report::render_json;
use slotcar_standards::report::render_markdown;
use slotcar_standards::runner::CheckInput;
use slotcar_standards::runner::run_checks;
use slotcar_standards::source::SourceFile;

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1;

/// Tiger Style policy checker.
#[derive(Debug, Parser)]
#[command(name = "slotcar-standards", about = "Run Tiger Style policy checks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check source files for Tiger Style policy diagnostics.
    Check(CheckArgs),
    /// Export deterministic policy artifacts for consumer checkers.
    ExportArtifacts(ExportArtifactsArgs),
    /// Validate hard-policy fixture coverage from a manifest.
    FixtureCoverage(FixtureCoverageArgs),
}

/// Source-check arguments.
#[derive(Debug, Parser)]
struct CheckArgs {
    /// JSON registry exported from `standards/policies/registry.ncl`.
    #[arg(long)]
    registry: Option<PathBuf>,

    /// Directory containing generated Tiger Style policy artifacts.
    #[arg(long)]
    policy_artifacts_dir: Option<PathBuf>,

    /// Source files or directories to inspect.
    #[arg(long, required = true)]
    source: Vec<PathBuf>,

    /// Rule family to run. Repeat for multiple families. Omit to run all.
    #[arg(long)]
    family: Vec<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    output: OutputFormat,

    /// Execute relevant verified proof rail commands and use their output as proof receipts.
    #[arg(long)]
    execute_proof_rails: bool,
}

/// Generated-artifact export arguments.
#[derive(Debug, Parser)]
struct ExportArtifactsArgs {
    /// JSON registry exported from `standards/policies/registry.ncl`.
    #[arg(long)]
    registry: PathBuf,

    /// Output directory for generated policy artifacts.
    #[arg(long)]
    out_dir: PathBuf,

    /// JSON profile exported from a Nickel run profile.
    #[arg(long)]
    profile_json: Option<PathBuf>,

    /// Output path for a structured acceptance receipt.
    #[arg(long)]
    receipt_out: Option<PathBuf>,

    /// Verification or replay command to record in the receipt.
    #[arg(long)]
    verification_command: Vec<String>,
}

/// Fixture-coverage arguments.
#[derive(Debug, Parser)]
struct FixtureCoverageArgs {
    /// JSON registry exported from `standards/policies/registry.ncl`.
    #[arg(long)]
    registry: PathBuf,

    /// Fixture manifest JSON.
    #[arg(long)]
    manifest: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    output: OutputFormat,
}

/// CLI output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    /// Human-readable markdown emitted to stdout.
    Human,
    /// Pretty JSON emitted to stdout.
    Json,
    /// Markdown emitted to stdout.
    Markdown,
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("failed to read {path}: {source}")]
    Read { path: PathBuf, source: io::Error },

    #[error("failed to parse {path} as JSON: {source}")]
    Json { path: PathBuf, source: serde_json::Error },

    #[error("failed to create {path}: {source}")]
    CreateDir { path: PathBuf, source: io::Error },

    #[error("failed to write {path}: {source}")]
    Write { path: PathBuf, source: io::Error },

    #[error("failed to serialize report as JSON: {source}")]
    Serialize { source: serde_json::Error },

    #[error("{message}")]
    Config { message: String },
}

fn main() -> ExitCode {
    match run_cli(Cli::parse()) {
        Ok(blocking) => exit_code_for_blocking(blocking),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

fn run_cli(cli: Cli) -> Result<bool, CliError> {
    match cli.command {
        Command::Check(args) => run_check_command(args),
        Command::ExportArtifacts(args) => run_export_artifacts_command(&args),
        Command::FixtureCoverage(args) => run_fixture_coverage_command(args),
    }
}

fn run_check_command(args: CheckArgs) -> Result<bool, CliError> {
    if args.registry.is_some() && args.policy_artifacts_dir.is_some() {
        return Err(CliError::Config {
            message: "pass either --registry or --policy-artifacts-dir, not both".to_owned(),
        });
    }

    let selection = FamilySelection::from_families(args.family);
    let registry = if let Some(policy_artifacts_dir) = &args.policy_artifacts_dir {
        match load_artifact_backed_registry(policy_artifacts_dir)? {
            ArtifactBackedRegistry::Valid(registry) => registry,
            ArtifactBackedRegistry::Diagnostics(diagnostics) => {
                let report = CheckReport {
                    diagnostics,
                    active_suppressions: Vec::new(),
                    evaluated_policy_ids: Vec::new(),
                    policy_evaluations: Vec::new(),
                };
                emit_report(args.output, &report)?;
                return Ok(report.has_blocking_diagnostics());
            }
        }
    } else if let Some(registry) = &args.registry {
        load_json::<PolicyRegistry>(registry)?
    } else {
        return Err(CliError::Config {
            message: "pass --registry or --policy-artifacts-dir".to_owned(),
        });
    };
    let mut sources = load_sources(&args.source)?;
    if args.execute_proof_rails {
        let executed = execute_relevant_proof_rails(&registry, &sources, &selection, &args.source);
        sources.retain(|source| !executed.proof_path_patterns.iter().any(|pattern| source.path.contains(pattern)));
        sources.extend(executed.receipts);
    }
    let report = run_checks(&CheckInput {
        registry,
        sources,
        selection,
    });
    emit_report(args.output, &report)?;
    Ok(report.has_blocking_diagnostics())
}

fn run_export_artifacts_command(args: &ExportArtifactsArgs) -> Result<bool, CliError> {
    let started_at = timestamp_now();
    if args.receipt_out.is_some() && args.profile_json.is_none() {
        return Err(CliError::Config {
            message: "pass --profile-json when --receipt-out is set".to_owned(),
        });
    }

    let registry = load_json::<PolicyRegistry>(&args.registry)?;
    fs::create_dir_all(&args.out_dir).map_err(|source| CliError::CreateDir {
        path: args.out_dir.clone(),
        source,
    })?;

    let generated_artifacts = generated_artifact_texts(&registry)?;
    let manifest_text = json_text(&artifact_manifest(&registry, &generated_artifacts))?;
    for (file_name, text) in &generated_artifacts {
        write_text(&args.out_dir.join(file_name), text)?;
    }
    let manifest_path = args.out_dir.join(ARTIFACT_MANIFEST_JSON_FILE);
    write_text(&manifest_path, &manifest_text)?;
    if let Some(receipt_out) = &args.receipt_out {
        write_export_receipt(args, &generated_artifacts, &manifest_path, receipt_out, &started_at)?;
    }
    Ok(false)
}

fn run_fixture_coverage_command(args: FixtureCoverageArgs) -> Result<bool, CliError> {
    let FixtureCoverageArgs {
        registry,
        manifest,
        output,
    } = args;
    let registry = load_json::<PolicyRegistry>(&registry)?;
    let manifest = load_json::<FixtureManifest>(&manifest)?;
    let diagnostics = validate_hard_policy_fixture_coverage(&registry, &manifest);
    let report = CheckReport {
        diagnostics,
        active_suppressions: Vec::new(),
        evaluated_policy_ids: Vec::new(),
        policy_evaluations: Vec::new(),
    };
    emit_report(output, &report)?;
    Ok(report.has_blocking_diagnostics())
}

fn write_export_receipt(
    args: &ExportArtifactsArgs,
    generated_artifacts: &[(String, String)],
    manifest_path: &Path,
    receipt_out: &Path,
    started_at: &str,
) -> Result<(), CliError> {
    let profile_path = args.profile_json.as_ref().expect("receipt profile must be checked before writing receipt");
    let profile_text = read_text(profile_path)?;
    let profile = parse_profile(profile_path, &profile_text)?;
    let receipt = export_receipt(args, generated_artifacts, manifest_path, started_at, &profile_text, &profile);
    if let Some(parent) = receipt_out.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    write_text(receipt_out, &json_text(&receipt)?)
}

fn export_receipt(
    args: &ExportArtifactsArgs,
    generated_artifacts: &[(String, String)],
    manifest_path: &Path,
    started_at: &str,
    profile_text: &str,
    profile: &RunProfile,
) -> AcceptanceReceipt {
    let finished_at = timestamp_now();
    let config_digest = blake3_text(profile_text);
    AcceptanceReceipt::passed_export_artifacts(ExportReceiptInput {
        run_id: export_run_id(profile, started_at),
        started_at: started_at.to_owned(),
        finished_at,
        tool: "slotcar-standards".to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        git_revision: git_revision(),
        config_digest: ReceiptDigest {
            algorithm: profile.config_digest_algorithm.clone(),
            value: config_digest,
        },
        inputs: export_receipt_inputs(args, profile_text),
        artifacts: export_receipt_artifacts(args, generated_artifacts, manifest_path),
        verification_commands: export_verification_commands(args),
        verification_evidence: export_verification_evidence(manifest_path),
    })
}

fn read_text(path: &Path) -> Result<String, CliError> {
    fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn load_json<T>(path: &Path) -> Result<T, CliError>
where T: serde::de::DeserializeOwned {
    let text = read_text(path)?;
    serde_json::from_str(&text).map_err(|source| CliError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_profile(path: &Path, text: &str) -> Result<RunProfile, CliError> {
    let profile = serde_json::from_str::<RunProfile>(text).map_err(|source| CliError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    if profile.config_digest_algorithm == "blake3" {
        Ok(profile)
    } else {
        Err(CliError::Config {
            message: format!(
                "profile `{}` uses unsupported config digest algorithm `{}`",
                path.display(),
                profile.config_digest_algorithm
            ),
        })
    }
}

fn export_receipt_inputs(args: &ExportArtifactsArgs, profile_text: &str) -> Vec<ReceiptInput> {
    let profile_path = args.profile_json.as_ref().expect("receipt profile must be present");
    vec![
        ReceiptInput {
            path: profile_path.display().to_string(),
            blake3: blake3_text(profile_text),
        },
        ReceiptInput {
            path: args.registry.display().to_string(),
            blake3: blake3_file(&args.registry),
        },
    ]
}

fn export_receipt_artifacts(
    args: &ExportArtifactsArgs,
    generated_artifacts: &[(String, String)],
    manifest_path: &Path,
) -> Vec<ReceiptArtifact> {
    generated_artifacts
        .iter()
        .map(|(file_name, text)| text_artifact(args.out_dir.join(file_name), text))
        .chain(std::iter::once(file_artifact(manifest_path)))
        .collect()
}

fn export_verification_commands(args: &ExportArtifactsArgs) -> Vec<ReceiptVerificationCommand> {
    let commands = if args.verification_command.is_empty() {
        vec!["slotcar-standards export-artifacts receipt emission".to_owned()]
    } else {
        args.verification_command.clone()
    };
    commands
        .into_iter()
        .map(|command| ReceiptVerificationCommand {
            command,
            status: AcceptanceStatus::Passed,
        })
        .collect()
}

fn export_verification_evidence(manifest_path: &Path) -> Vec<ReceiptEvidence> {
    vec![ReceiptEvidence {
        path: manifest_path.display().to_string(),
        blake3: blake3_file(manifest_path),
    }]
}

enum ArtifactBytes<'a> {
    Text(&'a str),
    File(&'a Path),
}

fn text_artifact(path: PathBuf, text: &str) -> ReceiptArtifact {
    artifact_from_bytes(path, ArtifactBytes::Text(text))
}

fn file_artifact(path: &Path) -> ReceiptArtifact {
    artifact_from_bytes(path.to_path_buf(), ArtifactBytes::File(path))
}

fn artifact_from_bytes(path: PathBuf, bytes: ArtifactBytes<'_>) -> ReceiptArtifact {
    let (blake3, bytes) = match bytes {
        ArtifactBytes::Text(text) => (blake3_text(text), text.len()),
        ArtifactBytes::File(path) => {
            let content = fs::read(path).expect("receipt artifact should have just been written");
            (blake3::hash(&content).to_hex().to_string(), content.len())
        }
    };
    ReceiptArtifact {
        path: path.display().to_string(),
        blake3,
        bytes: u64::try_from(bytes).expect("artifact size should fit u64"),
    }
}

fn export_run_id(profile: &RunProfile, started_at: &str) -> String {
    format!("{}-{started_at}", profile.profile_id)
}

fn timestamp_now() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("unix:{}", duration.as_secs()),
        Err(error) => format!("unix-before-epoch:{}", error.duration().as_secs()),
    }
}

fn git_revision() -> String {
    ProcessCommand::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .and_then(|output| output.status.success().then_some(output.stdout))
        .and_then(|stdout| String::from_utf8(stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

fn blake3_text(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn blake3_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("receipt input should be readable");
    blake3::hash(&bytes).to_hex().to_string()
}

enum ArtifactBackedRegistry {
    Valid(PolicyRegistry),
    Diagnostics(Vec<Diagnostic>),
}

fn load_artifact_backed_registry(directory: &Path) -> Result<ArtifactBackedRegistry, CliError> {
    let manifest_path = directory.join(ARTIFACT_MANIFEST_JSON_FILE);
    let manifest = match load_json::<ArtifactManifest>(&manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Ok(ArtifactBackedRegistry::Diagnostics(vec![artifact_manifest_diagnostic(
                manifest_path.display().to_string(),
                format!("missing or unreadable artifact integrity manifest: {error}"),
            )]));
        }
    };
    let generated_artifacts = match load_generated_artifact_texts(directory) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            return Ok(ArtifactBackedRegistry::Diagnostics(vec![artifact_manifest_diagnostic(
                manifest_path.display().to_string(),
                error,
            )]));
        }
    };
    let manifest_diagnostics = artifact_manifest_diagnostics(
        &manifest_path.display().to_string(),
        validate_artifact_manifest_texts(&manifest, &generated_artifacts),
    );
    if !manifest_diagnostics.is_empty() {
        return Ok(ArtifactBackedRegistry::Diagnostics(manifest_diagnostics));
    }

    let policies_text = generated_artifacts
        .iter()
        .find(|(file_name, _)| file_name == POLICIES_JSON_FILE)
        .map(|(_, text)| text)
        .expect("generated artifact loader should include policies JSON");
    let registry = serde_json::from_str::<PolicyRegistry>(policies_text).map_err(|source| CliError::Json {
        path: directory.join(POLICIES_JSON_FILE),
        source,
    })?;
    let diagnostics = validate_policy_artifact_directory(&registry, &manifest, &generated_artifacts, directory);
    if diagnostics.is_empty() {
        Ok(ArtifactBackedRegistry::Valid(registry))
    } else {
        Ok(ArtifactBackedRegistry::Diagnostics(diagnostics))
    }
}

fn validate_policy_artifact_directory(
    registry: &PolicyRegistry,
    manifest: &ArtifactManifest,
    generated_artifacts: &[(String, String)],
    directory: &Path,
) -> Vec<Diagnostic> {
    let manifest_path = directory.join(ARTIFACT_MANIFEST_JSON_FILE).display().to_string();
    let mut diagnostics = artifact_manifest_diagnostics(
        &manifest_path,
        validate_artifact_manifest(registry, manifest, generated_artifacts),
    );
    if diagnostics.is_empty() {
        diagnostics.extend(validate_artifact_backed_proof_bundle(registry, directory));
    }
    diagnostics
}

fn artifact_manifest_diagnostics(path: &str, messages: Vec<String>) -> Vec<Diagnostic> {
    messages.into_iter().map(|message| artifact_manifest_diagnostic(path.to_owned(), message)).collect()
}

fn load_generated_artifact_texts(directory: &Path) -> Result<Vec<(String, String)>, String> {
    [
        POLICIES_JSON_FILE,
        CHECK_CONFIG_TOML_FILE,
        SEED_MANIFESTS_JSON_FILE,
        PROOF_BUNDLES_JSON_FILE,
        CONFIG_RECEIPT_FAMILIES_JSON_FILE,
    ]
    .into_iter()
    .map(|file_name| {
        let path = directory.join(file_name);
        fs::read_to_string(&path)
            .map(|text| (file_name.to_owned(), text))
            .map_err(|error| format!("failed to read generated artifact `{}`: {error}", path.display()))
    })
    .collect()
}

fn artifact_manifest_diagnostic(path: String, message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: "tigerstyle.generated-artifacts.integrity-manifest".to_owned(),
        source_section: "openspec/specs/standards-codification/spec.md#generated-artifact-integrity-manifest"
            .to_owned(),
        path,
        line: None,
        severity: Severity::Error,
        message,
        remediation: "regenerate policy artifacts with slotcar-standards export-artifacts and consume the complete generated directory".to_owned(),
    })
}

fn validate_artifact_backed_proof_bundle(registry: &PolicyRegistry, directory: &Path) -> Vec<Diagnostic> {
    if !registry_has_verified_proof_rails(registry) {
        return Vec::new();
    }

    let path = directory.join(PROOF_BUNDLES_JSON_FILE);
    let artifact = match load_json::<ProofBundleArtifact>(&path) {
        Ok(artifact) => artifact,
        Err(error) => {
            return vec![proof_bundle_diagnostic(
                path.display().to_string(),
                format!("missing or unreadable proof-bundle artifact: {error}"),
            )];
        }
    };

    validate_proof_bundle_artifact(registry, &artifact)
        .into_iter()
        .map(|message| proof_bundle_diagnostic(path.display().to_string(), message))
        .collect()
}

fn proof_bundle_diagnostic(path: String, message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: "tigerstyle.verification.proof-bundle-schema".to_owned(),
        source_section: "openspec/specs/standards-codification/spec.md#proof-bundle-schema-validation".to_owned(),
        path,
        line: None,
        severity: Severity::Error,
        message,
        remediation: "regenerate policy artifacts with slotcar-standards export-artifacts and keep tigerstyle-proof-bundles.json with the policy bundle".to_owned(),
    })
}

fn generated_artifact_texts(registry: &PolicyRegistry) -> Result<Vec<(String, String)>, CliError> {
    Ok(vec![
        (POLICIES_JSON_FILE.to_owned(), json_text(registry)?),
        (CHECK_CONFIG_TOML_FILE.to_owned(), text_with_trailing_newline(&render_check_config_toml(registry))),
        (SEED_MANIFESTS_JSON_FILE.to_owned(), json_text(&seed_manifest_artifact(registry))?),
        (PROOF_BUNDLES_JSON_FILE.to_owned(), json_text(&proof_bundle_artifact(registry))?),
        (CONFIG_RECEIPT_FAMILIES_JSON_FILE.to_owned(), json_text(&config_receipt_family_artifact(registry))?),
    ])
}

fn json_text<T>(value: &T) -> Result<String, CliError>
where T: serde::Serialize {
    let text = serde_json::to_string_pretty(value).map_err(|source| CliError::Serialize { source })?;
    Ok(text_with_trailing_newline(&text))
}

fn text_with_trailing_newline(text: &str) -> String {
    let mut text = text.to_owned();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

fn write_text(path: &Path, text: &str) -> Result<(), CliError> {
    fs::write(path, text).map_err(|source| CliError::Write {
        path: path.to_path_buf(),
        source,
    })
}

struct ExecutedProofRails {
    receipts: Vec<SourceFile>,
    proof_path_patterns: Vec<String>,
}

fn execute_relevant_proof_rails(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
    source_paths: &[PathBuf],
) -> ExecutedProofRails {
    let mut proof_path_patterns = Vec::new();
    let receipts = registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| policy.proof_rails.iter())
        .filter(|rail| proof_rail_is_relevant(rail, sources))
        .map(|rail| {
            proof_path_patterns.extend(rail.proof_paths.iter().cloned());
            execute_proof_rail(rail, sources, source_paths)
        })
        .collect();
    ExecutedProofRails {
        receipts,
        proof_path_patterns,
    }
}

fn policy_is_verified(policy: &slotcar_standards::model::PolicyEntry) -> bool {
    policy.current_enforcement_mode == slotcar_standards::model::EnforcementMode::Verified
        || policy.target_enforcement_mode == slotcar_standards::model::EnforcementMode::Verified
}

fn proof_rail_is_relevant(rail: &slotcar_standards::model::ProofRail, sources: &[SourceFile]) -> bool {
    rail.source_paths
        .iter()
        .chain(&rail.proof_paths)
        .any(|pattern| sources.iter().any(|source| source.path.contains(pattern)))
}

fn execute_proof_rail(
    rail: &slotcar_standards::model::ProofRail,
    sources: &[SourceFile],
    source_paths: &[PathBuf],
) -> SourceFile {
    let working_dir = proof_command_working_dir(source_paths);
    let output = ProcessCommand::new("sh").arg("-c").arg(&rail.command).current_dir(&working_dir).output();
    let receipt = proof_command_receipt(rail, sources, &working_dir, output);
    let receipt_path = working_dir.join(first_proof_path(rail));
    SourceFile::new(receipt_path.display().to_string(), receipt)
}

fn proof_command_working_dir(source_paths: &[PathBuf]) -> PathBuf {
    source_paths.first().map_or_else(
        || PathBuf::from("."),
        |path| {
            if path.is_dir() {
                path.clone()
            } else {
                path.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf)
            }
        },
    )
}

fn first_proof_path(rail: &slotcar_standards::model::ProofRail) -> &str {
    rail.proof_paths.first().map_or("proof-receipt.txt", String::as_str)
}

fn proof_command_receipt(
    rail: &slotcar_standards::model::ProofRail,
    sources: &[SourceFile],
    working_dir: &Path,
    output: Result<std::process::Output, io::Error>,
) -> String {
    let receipt_provenance = proof_receipt_provenance(rail);
    let receipt_verifier = proof_receipt_verifier(rail, sources);
    let receipt_binding = proof_receipt_binding(rail, sources);
    match output {
        Ok(output) => format!(
            "proof-command: {}\nproof-cwd: {}\n{}{}{}exit-status: {}\nstdout:\n{}\nstderr:\n{}\n",
            rail.command,
            working_dir.display(),
            receipt_provenance,
            receipt_verifier,
            receipt_binding,
            output.status.code().map_or_else(|| "signal".to_owned(), |code| code.to_string()),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(error) => format!(
            "proof-command: {}\nproof-cwd: {}\n{}{}{}exit-status: spawn-error\nstderr:\n{}\n",
            rail.command,
            working_dir.display(),
            receipt_provenance,
            receipt_verifier,
            receipt_binding,
            error
        ),
    }
}

fn proof_receipt_provenance(rail: &slotcar_standards::model::ProofRail) -> String {
    let provenance = &rail.receipt_provenance;
    let mut lines = Vec::new();
    if !provenance.schema_version.is_empty() {
        lines.push(format!("proof-receipt-schema: {}", provenance.schema_version));
    }
    if provenance.requires_backend {
        lines.push(format!("proof-backend: {}", rail.backend));
    }
    if !provenance.runner.is_empty() {
        lines.push(format!("proof-runner: {}", provenance.runner));
    }
    if !provenance.environment.is_empty() {
        lines.push(format!("proof-environment: {}", provenance.environment));
    }
    if provenance.requires_trust_boundary {
        lines.extend(rail.trust_boundaries.iter().map(|boundary| format!("trust-boundary: {boundary}")));
    }
    if lines.is_empty() {
        String::new()
    } else {
        lines.join(
            "
",
        ) + "
"
    }
}

fn proof_receipt_verifier(rail: &slotcar_standards::model::ProofRail, sources: &[SourceFile]) -> String {
    let verifier = &rail.receipt_verifier;
    let mut lines = Vec::new();
    if !verifier.verifier_id.is_empty() {
        lines.push(format!("proof-verifier: {}", verifier.verifier_id));
    }
    if !verifier.verifier_digest.is_empty() {
        lines.push(format!("proof-verifier-blake3: {}", verifier.verifier_digest));
    }
    if !verifier.verifier_binary_path.is_empty() {
        lines.push(format!("proof-verifier-binary: {}", verifier.verifier_binary_path));
    }
    if !verifier.verifier_binary_digest.is_empty() {
        lines.push(format!(
            "proof-verifier-binary-blake3: {}",
            current_exe_blake3().unwrap_or_else(|| "unavailable".to_owned())
        ));
    }
    if !verifier.verifier_build_closure.is_empty() {
        lines.push(format!("proof-verifier-build-closure: {}", verifier.verifier_build_closure));
    }
    if !verifier.verifier_build_closure_digest.is_empty() {
        let digest = build_closure_digest(&verifier.verifier_build_closure_paths, sources)
            .unwrap_or_else(|| verifier.verifier_build_closure_digest.clone());
        lines.push(format!("proof-verifier-build-closure-blake3: {digest}"));
    }
    if !verifier.verifier_nix_derivation.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation: {}", verifier.verifier_nix_derivation));
    }
    if !verifier.verifier_nix_derivation_digest.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation-blake3: {}", verifier.verifier_nix_derivation_digest));
    }
    if !verifier.verifier_nix_closure.is_empty() {
        lines.push(format!("proof-verifier-nix-closure: {}", verifier.verifier_nix_closure));
    }
    if !verifier.verifier_nix_closure_digest.is_empty() {
        let digest = build_closure_digest(&verifier.verifier_nix_closure_paths, sources)
            .unwrap_or_else(|| verifier.verifier_nix_closure_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-blake3: {digest}"));
    }
    if !verifier.verifier_nix_closure_graph_digest.is_empty() {
        let digest = nix_closure_graph_digest(&verifier.verifier_nix_closure_graph_paths, sources)
            .unwrap_or_else(|| verifier.verifier_nix_closure_graph_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-graph-blake3: {digest}"));
    }
    if !verifier.toolchain_id.is_empty() {
        lines.push(format!("proof-toolchain: {}", verifier.toolchain_id));
    }
    if !verifier.toolchain_digest.is_empty() {
        lines.push(format!("proof-toolchain-blake3: {}", verifier.toolchain_digest));
    }
    if lines.is_empty() {
        String::new()
    } else {
        lines.join(
            "
",
        ) + "
"
    }
}

fn current_exe_blake3() -> Option<String> {
    let path = env::current_exe().ok()?;
    let bytes = fs::read(path).ok()?;
    Some(blake3::hash(&bytes).to_hex().to_string())
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

fn proof_receipt_binding(rail: &slotcar_standards::model::ProofRail, sources: &[SourceFile]) -> String {
    if rail.receipt_binding != "blake3-command-and-source" {
        return String::new();
    }
    let mut lines = vec![format!("proof-command-blake3: {}", blake3_hex(&rail.command))];
    let mut matched_sources = sources
        .iter()
        .filter(|source| rail.source_paths.iter().any(|pattern| source.path.contains(pattern)))
        .collect::<Vec<_>>();
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    lines.extend(
        matched_sources
            .into_iter()
            .map(|source| format!("proof-source-blake3: {}:{}", source.path, blake3_hex(&source.text))),
    );
    lines.join("\n") + "\n"
}

fn blake3_hex(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn load_sources(paths: &[PathBuf]) -> Result<Vec<SourceFile>, CliError> {
    let mut sources = Vec::new();
    for path in paths {
        append_sources_from_path(path, &mut sources)?;
    }
    Ok(sources)
}

fn append_sources_from_path(path: &Path, sources: &mut Vec<SourceFile>) -> Result<(), CliError> {
    if path.is_dir() {
        append_sources_from_directory(path, sources)
    } else {
        append_source_file(path, sources)
    }
}

fn append_sources_from_directory(directory: &Path, sources: &mut Vec<SourceFile>) -> Result<(), CliError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|source| CliError::Read {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(|source| CliError::Read {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::path);
    for entry in entries {
        append_sources_from_path(&entry.path(), sources)?;
    }
    Ok(())
}

fn append_source_file(path: &Path, sources: &mut Vec<SourceFile>) -> Result<(), CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    sources.push(SourceFile::new(path.display().to_string(), text));
    Ok(())
}

fn emit_report(output: OutputFormat, report: &CheckReport) -> Result<(), CliError> {
    match output {
        OutputFormat::Human | OutputFormat::Markdown => {
            println!("{}", render_markdown(report));
            Ok(())
        }
        OutputFormat::Json => {
            let json = render_json(report).map_err(|source| CliError::Serialize { source })?;
            println!("{json}");
            Ok(())
        }
    }
}

fn exit_code_for_blocking(blocking: bool) -> ExitCode {
    if blocking {
        ExitCode::from(EXIT_FAILURE)
    } else {
        ExitCode::from(EXIT_SUCCESS)
    }
}
