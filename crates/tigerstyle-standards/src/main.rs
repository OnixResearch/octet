#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! CLI shell for `tigerstyle-standards`.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use tigerstyle_standards::artifacts::CHECK_CONFIG_TOML_FILE;
use tigerstyle_standards::artifacts::POLICIES_JSON_FILE;
use tigerstyle_standards::artifacts::SEED_MANIFESTS_JSON_FILE;
use tigerstyle_standards::artifacts::render_check_config_toml;
use tigerstyle_standards::artifacts::seed_manifest_artifact;
use tigerstyle_standards::fixture::FixtureManifest;
use tigerstyle_standards::fixture::validate_hard_policy_fixture_coverage;
use tigerstyle_standards::model::FamilySelection;
use tigerstyle_standards::model::PolicyRegistry;
use tigerstyle_standards::report::CheckReport;
use tigerstyle_standards::report::render_json;
use tigerstyle_standards::report::render_markdown;
use tigerstyle_standards::runner::CheckInput;
use tigerstyle_standards::runner::run_checks;
use tigerstyle_standards::source::SourceFile;

const EXIT_SUCCESS: u8 = 0;
const EXIT_FAILURE: u8 = 1;

/// Tiger Style policy checker.
#[derive(Debug, Parser)]
#[command(name = "tigerstyle-standards", about = "Run Tiger Style policy checks")]
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
    let registry = load_registry_for_check(&args)?;
    let mut sources = load_sources(&args.source)?;
    let selection = FamilySelection::from_families(args.family);
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
    let registry = load_json::<PolicyRegistry>(&args.registry)?;
    fs::create_dir_all(&args.out_dir).map_err(|source| CliError::CreateDir {
        path: args.out_dir.clone(),
        source,
    })?;
    write_json(&args.out_dir.join(POLICIES_JSON_FILE), &registry)?;
    write_text(&args.out_dir.join(CHECK_CONFIG_TOML_FILE), &render_check_config_toml(&registry))?;
    write_json(&args.out_dir.join(SEED_MANIFESTS_JSON_FILE), &seed_manifest_artifact(&registry))?;
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

fn load_json<T>(path: &Path) -> Result<T, CliError>
where T: serde::de::DeserializeOwned {
    let text = fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn load_registry_for_check(args: &CheckArgs) -> Result<PolicyRegistry, CliError> {
    match (&args.registry, &args.policy_artifacts_dir) {
        (Some(registry), None) => load_json::<PolicyRegistry>(registry),
        (None, Some(directory)) => load_json::<PolicyRegistry>(&directory.join(POLICIES_JSON_FILE)),
        (Some(_), Some(_)) => Err(CliError::Config {
            message: "pass either --registry or --policy-artifacts-dir, not both".to_owned(),
        }),
        (None, None) => Err(CliError::Config {
            message: "pass --registry or --policy-artifacts-dir".to_owned(),
        }),
    }
}

fn write_json<T>(path: &Path, value: &T) -> Result<(), CliError>
where T: serde::Serialize {
    let text = serde_json::to_string_pretty(value).map_err(|source| CliError::Serialize { source })?;
    write_text(path, &text)
}

fn write_text(path: &Path, text: &str) -> Result<(), CliError> {
    let mut text = text.to_owned();
    text.push('\n');
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

fn policy_is_verified(policy: &tigerstyle_standards::model::PolicyEntry) -> bool {
    policy.current_enforcement_mode == tigerstyle_standards::model::EnforcementMode::Verified
        || policy.target_enforcement_mode == tigerstyle_standards::model::EnforcementMode::Verified
}

fn proof_rail_is_relevant(rail: &tigerstyle_standards::model::ProofRail, sources: &[SourceFile]) -> bool {
    rail.source_paths
        .iter()
        .chain(&rail.proof_paths)
        .any(|pattern| sources.iter().any(|source| source.path.contains(pattern)))
}

fn execute_proof_rail(
    rail: &tigerstyle_standards::model::ProofRail,
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

fn first_proof_path(rail: &tigerstyle_standards::model::ProofRail) -> &str {
    rail.proof_paths.first().map_or("proof-receipt.txt", String::as_str)
}

fn proof_command_receipt(
    rail: &tigerstyle_standards::model::ProofRail,
    sources: &[SourceFile],
    working_dir: &Path,
    output: Result<std::process::Output, io::Error>,
) -> String {
    let receipt_provenance = proof_receipt_provenance(rail);
    let receipt_binding = proof_receipt_binding(rail, sources);
    match output {
        Ok(output) => format!(
            "proof-command: {}\nproof-cwd: {}\n{}{}exit-status: {}\nstdout:\n{}\nstderr:\n{}\n",
            rail.command,
            working_dir.display(),
            receipt_provenance,
            receipt_binding,
            output.status.code().map_or_else(|| "signal".to_owned(), |code| code.to_string()),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(error) => format!(
            "proof-command: {}\nproof-cwd: {}\n{}{}exit-status: spawn-error\nstderr:\n{}\n",
            rail.command,
            working_dir.display(),
            receipt_provenance,
            receipt_binding,
            error
        ),
    }
}

fn proof_receipt_provenance(rail: &tigerstyle_standards::model::ProofRail) -> String {
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

fn proof_receipt_binding(rail: &tigerstyle_standards::model::ProofRail, sources: &[SourceFile]) -> String {
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
