use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::MigrateArgs;
use crate::cli::MigrateMode;
use crate::cli::OutputFormat;
use crate::determinism;

const MIGRATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct MigrationManifest {
    schema_version: u32,
    profile: String,
    tool_name: String,
    tool_version: String,
    config_hash: String,
    crate_graph_hash: String,
    accepted_findings: Vec<String>,
    fixed_findings: Vec<String>,
    deferred_findings: Vec<String>,
    manual_review_required: Vec<String>,
    unresolved_findings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct MigrationStatus<'a> {
    schema_version: u32,
    profile: &'a str,
    accepted: usize,
    fixed: usize,
    deferred: usize,
    manual_review_required: usize,
    unresolved: usize,
}

pub fn run(args: MigrateArgs) -> Result<i32, String> {
    match args.mode {
        MigrateMode::Start => start(args),
        MigrateMode::Status => status(args),
        MigrateMode::ApplySafe => apply_safe(args),
    }
}

fn start(args: MigrateArgs) -> Result<i32, String> {
    let workspace = std::env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let profile = args.profile.unwrap_or_else(|| "starter".to_owned());
    let manifest = MigrationManifest {
        schema_version: MIGRATION_SCHEMA_VERSION,
        profile,
        tool_name: "cargo-octet".to_owned(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        config_hash: determinism::full_hash(&format!(
            "Cargo.toml={}\ndylint.toml={}\n",
            file_hash_text(&workspace.join("Cargo.toml")),
            file_hash_text(&workspace.join("dylint.toml")),
        )),
        crate_graph_hash: determinism::crate_graph_hash(&workspace),
        accepted_findings: Vec::new(),
        fixed_findings: Vec::new(),
        deferred_findings: Vec::new(),
        manual_review_required: Vec::new(),
        unresolved_findings: Vec::new(),
    };
    write_manifest(&args.manifest, &manifest)?;
    print_manifest(&manifest, args.output_format)
}

fn status(args: MigrateArgs) -> Result<i32, String> {
    let manifest = read_manifest(&args.manifest)?;
    print_status(&manifest, args.output_format)
}

fn apply_safe(args: MigrateArgs) -> Result<i32, String> {
    let mut manifest = read_manifest(&args.manifest)?;
    // Octet only records safe-apply state here. Actual code edits are delegated to
    // existing machine-applicable cargo fix rails so manual-review findings cannot be touched.
    manifest.deferred_findings.sort();
    manifest.manual_review_required.sort();
    write_manifest(&args.manifest, &manifest)?;
    print_status(&manifest, args.output_format)
}

fn read_manifest(path: &Path) -> Result<MigrationManifest, String> {
    let contents = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let manifest: MigrationManifest =
        serde_json::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))?;
    if manifest.schema_version != MIGRATION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported migration schema_version {}; expected {MIGRATION_SCHEMA_VERSION}",
            manifest.schema_version
        ));
    }
    Ok(manifest)
}

fn write_manifest(path: &Path, manifest: &MigrationManifest) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut rendered =
        serde_json::to_string_pretty(manifest).map_err(|error| format!("serialize migration: {error}"))?;
    rendered.push('\n');
    fs::write(path, rendered).map_err(|error| format!("write {}: {error}", path.display()))
}

fn print_manifest(manifest: &MigrationManifest, output_format: OutputFormat) -> Result<i32, String> {
    match output_format {
        OutputFormat::Human => println!(
            "octet migration profile={} unresolved={} manual_review_required={}",
            manifest.profile,
            manifest.unresolved_findings.len(),
            manifest.manual_review_required.len(),
        ),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(manifest).map_err(|error| error.to_string())?)
        }
        OutputFormat::Sarif => return Err("migrate does not support SARIF output; use human or json".to_owned()),
    }
    Ok(0)
}

fn print_status(manifest: &MigrationManifest, output_format: OutputFormat) -> Result<i32, String> {
    let status = MigrationStatus {
        schema_version: MIGRATION_SCHEMA_VERSION,
        profile: &manifest.profile,
        accepted: manifest.accepted_findings.len(),
        fixed: manifest.fixed_findings.len(),
        deferred: manifest.deferred_findings.len(),
        manual_review_required: manifest.manual_review_required.len(),
        unresolved: manifest.unresolved_findings.len(),
    };
    match output_format {
        OutputFormat::Human => println!(
            "octet migration status profile={} accepted={} fixed={} deferred={} manual_review_required={} unresolved={}",
            status.profile,
            status.accepted,
            status.fixed,
            status.deferred,
            status.manual_review_required,
            status.unresolved,
        ),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("migrate does not support SARIF output; use human or json".to_owned()),
    }
    Ok(0)
}

fn file_hash_text(path: &Path) -> String {
    fs::read(path)
        .map(|bytes| determinism::full_hash(std::str::from_utf8(&bytes).unwrap_or("<binary>")))
        .unwrap_or_else(|_| "missing".to_owned())
}
