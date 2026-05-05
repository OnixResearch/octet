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
    let sources = load_sources(&args.source)?;
    let selection = FamilySelection::from_families(args.family);
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
