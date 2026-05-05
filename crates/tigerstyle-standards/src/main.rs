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
    /// Validate hard-policy fixture coverage from a manifest.
    FixtureCoverage(FixtureCoverageArgs),
}

/// Source-check arguments.
#[derive(Debug, Parser)]
struct CheckArgs {
    /// JSON registry exported from `standards/policies/registry.ncl`.
    #[arg(long)]
    registry: PathBuf,

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

    #[error("failed to serialize report as JSON: {source}")]
    Serialize { source: serde_json::Error },
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
        Command::FixtureCoverage(args) => run_fixture_coverage_command(args),
    }
}

fn run_check_command(args: CheckArgs) -> Result<bool, CliError> {
    let registry = load_json::<PolicyRegistry>(&args.registry)?;
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
