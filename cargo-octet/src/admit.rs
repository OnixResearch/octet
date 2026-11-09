use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::artifact;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;
use crate::cli::AdmissionArgs;
use crate::cli::AdmissionMode;
use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::effect_closure;
use crate::effect_closure::AdmissionStatus;
use crate::effect_closure::ClosureInput;
use crate::effect_closure::EffectClosureReceipt;
use crate::effect_closure::FunctionAddressRecord;
use crate::effect_closure::ProviderIdentity;
use crate::effect_closure::ProviderKind;
use crate::effect_closure::SourceEnvelope;
use crate::effect_closure::WorkspacePolicy;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;
use crate::function_address::FunctionAddressCheck;
use crate::function_address::FunctionAddressMode;

const EXIT_ADMITTED: i32 = 0;
const EXIT_DENIED: i32 = 1;
const ADMISSION_STATUS_SCHEMA: &str = "octet-effect-closure-status/v1";
const ADMISSION_VERIFICATION_SCHEMA: &str = "octet-effect-closure-verification/v1";
const DEPENDENCY_GRAPH_SCHEMA: &str = "octet-effect-closure-dependency-graph/v1";
const STATUS_ADMITTED: &str = "admitted";
const STATUS_DENIED: &str = "denied";
const STATUS_VALID: &str = "valid";
const STATUS_INVALID: &str = "invalid";
const PHASE_DIRECT_CHECK: &str = "direct-check";
const PHASE_CLOSURE: &str = "effect-closure";
const CHECK_DIR: &str = "check";
const COMMAND_FILE: &str = "command.txt";
const STATUS_FILE: &str = "status.json";
const SUMMARY_FILE: &str = "summary.txt";
const PROVENANCE_FILE: &str = "provenance.jsonl";
const WORKSPACE_POLICY_ARTIFACT: &str = "workspace-policy.json";
const DEPENDENCY_GRAPH_ARTIFACT: &str = "dependency-graph.json";
const CHECK_STATUS_ARTIFACT: &str = "check/status.json";
const CHECK_PROVENANCE_ARTIFACT: &str = "check/provenance.jsonl";
const CHECK_FUNCTION_ADDRESS_ARTIFACT: &str = "check/function-addresses.json";
const FUNCTION_ADDRESS_ROOT_KIND: &str = "function-address-record-root";
const FUNCTION_ADDRESS_MODE_DENY: &str = "deny";
const CARGO_COMMAND: &str = "cargo";
const CARGO_METADATA_COMMAND: &str = "metadata";
const CARGO_METADATA_FORMAT_VERSION: &str = "1";
const TOOLCHAIN_ENV: &str = "RUSTUP_TOOLCHAIN";
const CHECKED_SCOPE_PREFIX: &str = "effect-closure:";
const PROVENANCE_SOURCE_ID: &str = "octet.runner.effect-closure";
const CHECKER_ID: &str = "cargo-octet";
const PROVIDER_SOURCE_TOOLCHAIN: &str = "rustc-toolchain";
const PROVIDER_SOURCE_WORKSPACE: &str = "workspace";
const PROVIDER_SOURCE_PATH_PREFIX: &str = "path-manifest:";
const TOOLCHAIN_PROVIDER_NAMES: &[&str] = &["alloc", "core", "rustc", "std"];
const TOP_LEVEL_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    PROVENANCE_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
];
const REQUIRED_PROVENANCE_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
    CHECK_STATUS_ARTIFACT,
    CHECK_FUNCTION_ADDRESS_ARTIFACT,
];
pub(crate) const INTEGRATED_CLOSURE_RECEIPT_ARTIFACT_NAME: &str = "effect-closure.json";
pub(crate) const INTEGRATED_CLOSURE_POLICY_ARTIFACT_NAME: &str = "effect-closure-policy.json";
pub(crate) const INTEGRATED_CLOSURE_GRAPH_ARTIFACT_NAME: &str = "effect-closure-dependency-graph.json";
pub(crate) const INTEGRATED_CLOSURE_PARENT_STATUS_ARTIFACT_NAME: &str = "effect-closure-parent-status.json";
pub(crate) const INTEGRATED_CLOSURE_PROVENANCE_SOURCE_ID: &str = "octet.runner.effect-closure";
const NON_CLAIM_INTEGRITY: &str = "Admission verification checks stored BLAKE3 identity and linkage only; it does not rerun Cargo, source inference, dependency review, or compiler analysis, and it provides no signer authenticity, authorization, or defense against coherent replacement of every trust root.";

#[derive(Clone, Debug, Deserialize)]
struct CheckStatus {
    status: String,
    exit_code: i32,
    metadata: CheckMetadata,
    error_findings: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct CheckMetadata {
    rustc_version: String,
    toolchain: String,
    profile_name: String,
    profile_hash: String,
    config_hash: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FunctionAddressReport {
    mode: String,
    source_envelopes: Vec<FunctionAddressSourceEnvelope>,
    records: Vec<FunctionAddressRecord>,
    evidence_chain_handoff: FunctionAddressHandoff,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct FunctionAddressSourceEnvelope {
    path: String,
    source_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FunctionAddressHandoff {
    canonical_roots: Vec<CanonicalRoot>,
}

#[derive(Clone, Debug, Deserialize)]
struct CanonicalRoot {
    kind: String,
    digest: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
    resolve: Option<CargoResolve>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoNode {
    id: String,
    features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct DependencyGraphArtifact {
    pub schema_version: String,
    pub target: String,
    pub toolchain: String,
    pub rustc_version: String,
    pub cargo_lock_blake3: String,
    pub providers: Vec<ProviderIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AdmissionStatusArtifact {
    schema_version: String,
    status: String,
    exit_code: i32,
    phase: String,
    profile_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_identity: Option<String>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AdmissionVerificationReport {
    schema_version: String,
    artifact_dir: String,
    status: String,
    receipt_identity: Option<String>,
    checked_artifacts: Vec<String>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

pub(crate) struct PreparedIntegratedClosure {
    policy: WorkspacePolicy,
    policy_bytes: Vec<u8>,
    pub baseline_path: PathBuf,
}

#[derive(Clone)]
pub(crate) struct IntegratedClosureArtifacts {
    pub parent: octet_standards::project_architecture_gate::EffectClosureParent,
    pub receipt_json: String,
    pub policy_json: String,
    pub dependency_graph_json: String,
    pub parent_status_json: String,
}

// r[impl octet.effect_closure.cli]
// r[impl octet.effect_closure.boundaries.default]
pub fn run(args: &AdmissionArgs) -> Result<i32, String> {
    match &args.mode {
        AdmissionMode::Run {
            profile,
            closure,
            policy,
            scope_args,
            cargo_check_args,
        } => run_admission(args, profile, *closure, policy, scope_args, cargo_check_args.as_deref()),
        AdmissionMode::Verify => run_verification(args),
    }
}

// r[impl octet.effect_closure.cli.admitted]
// r[impl octet.effect_closure.cli.check_failure]
fn run_admission(
    args: &AdmissionArgs,
    profile: &str,
    closure: bool,
    policy_path: &Path,
    scope_args: &[String],
    cargo_check_args: Option<&[String]>,
) -> Result<i32, String> {
    if profile != effect_closure::DETERMINISTIC_CORE_PROFILE || !closure {
        return Err(format!(
            "effect closure v1 requires `--profile {} --closure`",
            effect_closure::DETERMINISTIC_CORE_PROFILE
        ));
    }
    crate::engine::reject_process_dylint_toml_override()?;

    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let workspace_config = crate::config::load(&cwd)?;
    let workspace_root = workspace_config.workspace_root;
    let selected_profile = crate::strict_profile::prepare_selected_profile(Some(profile), &workspace_root)?
        .ok_or_else(|| "effect closure requires a selected strict profile".to_owned())?;
    let prepared = prepare_inputs(&workspace_root, policy_path)?;
    let artifact_dir = absolute_from_workspace(&workspace_root, &args.artifact_dir);
    let check_dir = artifact_dir.join(CHECK_DIR);
    let root = prepare_artifact_root(&artifact_dir)?;
    let command_bytes = normalized_text_bytes(&args.command_line);

    let check = CheckArgs {
        scope_args: scope_args.to_vec(),
        cargo_check_args: cargo_check_args.map(<[String]>::to_vec),
        profile: Some(profile.to_owned()),
        output_format: OutputFormat::Human,
        artifact_dir: Some(check_dir.clone()),
        baseline: None,
        write_baseline: None,
        cache: false,
        function_addresses: FunctionAddressCheck {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
            baseline: Some(prepared.baseline_path.clone()),
        },
        command_line: args.command_line.clone(),
    };
    let check_exit = crate::engine::run(&check)?;
    if check_exit != EXIT_ADMITTED {
        write_blocked_status(&root, profile, check_exit, &command_bytes)?;
        return Ok(check_exit);
    }
    verify_check_bundle(&check_dir)?;

    let check_status_bytes = read_required(&check_dir.join(STATUS_FILE))?;
    let function_address_bytes =
        read_required(&check_dir.join(crate::function_address::FUNCTION_ADDRESS_ARTIFACT_NAME))?;
    let check_status = parse_check_status(&check_status_bytes, profile)?;
    let function_report = parse_function_report(&function_address_bytes)?;
    let target = selected_target(cargo_check_args.unwrap_or_default())?;
    let (dependency_graph, dependency_graph_bytes) =
        dependency_graph(&workspace_root, cargo_check_args.unwrap_or_default(), &target, &check_status.metadata)?;
    let sources = read_selected_sources(&workspace_root, &selected_profile.core_source_scopes)?;
    validate_function_source_envelopes(&sources, &function_report)?;
    let record_root = function_address_record_root(&function_report)?;
    let catalog = effect_closure::embedded_catalog()?;

    let receipt = effect_closure::compute_closure(ClosureInput {
        profile_id: profile.to_owned(),
        profile_hash: check_status.metadata.profile_hash.clone(),
        config_hash: check_status.metadata.config_hash.clone(),
        command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
        check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
        function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
        function_address_record_root: record_root,
        workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&prepared.policy_bytes),
        cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&dependency_graph_bytes),
        cargo_lock_blake3: dependency_graph.cargo_lock_blake3.clone(),
        target,
        toolchain: check_status.metadata.toolchain.clone(),
        sources,
        function_addresses: function_report.records,
        resolved_providers: dependency_graph.providers.clone(),
        catalog,
        policy: prepared.policy,
    });

    write_closure_bundle(
        &root,
        &artifact_dir,
        &command_bytes,
        &prepared.policy_bytes,
        &dependency_graph_bytes,
        &check_status_bytes,
        &function_address_bytes,
        &receipt,
    )?;
    render_admission_output(args.output_format, &artifact_dir, &receipt)?;
    Ok(match receipt.status {
        AdmissionStatus::Admitted => EXIT_ADMITTED,
        AdmissionStatus::Denied => EXIT_DENIED,
    })
}

fn run_verification(args: &AdmissionArgs) -> Result<i32, String> {
    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let report = verify_bundle(&verification_artifact_dir(&cwd, &args.artifact_dir)?)?;
    match args.output_format {
        OutputFormat::Human => {
            eprintln!("Effect closure admission verification: {}", report.status);
            eprintln!("Artifact directory: {}", report.artifact_dir);
            if let Some(identity) = &report.receipt_identity {
                eprintln!("Receipt identity: {identity}");
            }
            for diagnostic in &report.diagnostics {
                eprintln!("- {diagnostic}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize admission verification report: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    Ok(if report.diagnostics.is_empty() {
        EXIT_ADMITTED
    } else {
        EXIT_DENIED
    })
}

/// Resolves `admit verify --artifact-dir` the same way `admit` does: an absolute path is used as
/// given and a relative path is relative to the workspace root, not the current directory.
fn verification_artifact_dir(cwd: &Path, artifact_dir: &Path) -> Result<PathBuf, String> {
    if artifact_dir.is_absolute() {
        return Ok(artifact_dir.to_path_buf());
    }
    let workspace_root = crate::config::load(cwd)?.workspace_root;
    Ok(absolute_from_workspace(&workspace_root, artifact_dir))
}

pub(crate) fn prepare_integrated_closure(
    workspace_root: &Path,
    binding: &octet_standards::project_architecture_policy::ArchitectureAdmissionBinding,
) -> Result<PreparedIntegratedClosure, String> {
    prepare_inputs(workspace_root, Path::new(&binding.effect_closure_policy))
}

fn prepare_inputs(workspace_root: &Path, policy_path: &Path) -> Result<PreparedIntegratedClosure, String> {
    let policy_path = checked_workspace_path(workspace_root, policy_path, "effect-closure policy")?;
    let policy_bytes = read_required(&policy_path)?;
    let policy = serde_json::from_slice::<WorkspacePolicy>(&policy_bytes)
        .map_err(|error| format!("parse {}: {error}", policy_path.display()))?;
    let diagnostics = effect_closure::validate_workspace_policy(&policy);
    if !diagnostics.is_empty() {
        return Err(format!("invalid effect-closure workspace policy:\n- {}", diagnostics.join("\n- ")));
    }
    let baseline_path = checked_workspace_path(
        workspace_root,
        Path::new(&policy.function_address_baseline),
        "function-address baseline",
    )?;
    if !baseline_path.is_file() {
        return Err(format!("function-address baseline does not exist: {}", baseline_path.display()));
    }
    Ok(PreparedIntegratedClosure {
        policy,
        policy_bytes,
        baseline_path,
    })
}

/// Opens the admission artifact directory and removes the previous bundle's top-level artifacts. A
/// non-empty directory that is not an existing admission bundle is refused before anything is
/// deleted, so `--artifact-dir` cannot remove unrelated files.
fn prepare_artifact_root(path: &Path) -> Result<ArtifactRoot, String> {
    let root = ArtifactRoot::open_or_create(path)?;
    if !root.read_entry_names()?.is_empty() && !is_existing_admission_bundle(&root, path)? {
        return Err(format!(
            "refusing to use artifact directory {}: it is not empty and is not an existing Octet admission bundle \
             (no admission `{STATUS_FILE}` or strict-profile `{CHECK_DIR}/` bundle); choose a new or empty directory",
            path.display()
        ));
    }
    for artifact in TOP_LEVEL_ARTIFACTS {
        root.remove_file_or_dir_all(&relative(artifact)?)?;
    }
    Ok(root)
}

/// Every completed or blocked admission run writes an admission-schema `status.json`; a run that
/// stopped after its direct check still leaves the strict-profile `check/` bundle markers.
fn is_existing_admission_bundle(root: &ArtifactRoot, path: &Path) -> Result<bool, String> {
    if root.is_file(&relative(STATUS_FILE)?)? {
        let status = serde_json::from_slice::<AdmissionStatusArtifact>(&read_required(&path.join(STATUS_FILE))?);
        if status.is_ok_and(|status| status.schema_version == ADMISSION_STATUS_SCHEMA) {
            return Ok(true);
        }
    }
    for marker in [CHECK_STATUS_ARTIFACT, CHECK_PROVENANCE_ARTIFACT] {
        if !root.is_file(&relative(marker)?)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn write_blocked_status(
    root: &ArtifactRoot,
    profile: &str,
    check_exit: i32,
    command_bytes: &[u8],
) -> Result<(), String> {
    let diagnostic =
        format!("direct strict-profile check failed with exit code {check_exit}; closure was not attempted");
    let status = AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: STATUS_DENIED.to_owned(),
        exit_code: check_exit,
        phase: PHASE_DIRECT_CHECK.to_owned(),
        profile_id: profile.to_owned(),
        receipt_identity: None,
        diagnostics: vec![diagnostic.clone()],
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    };
    let status_bytes = pretty_json_bytes(&status)?;
    let summary = format!(
        "Octet effect closure admission\nStatus: denied\nPhase: direct-check\nProfile: {profile}\nDiagnostic: {diagnostic}\n"
    );
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, summary.as_bytes())
}

fn verify_check_bundle(check_dir: &Path) -> Result<(), String> {
    let root = ArtifactRoot::open_existing(check_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| artifact::BundleFile {
            name: file.locator,
            contents: file.contents,
        })
        .collect::<Vec<_>>();
    let report = artifact::verify_bundle(artifact::ArtifactBundleInput {
        artifact_dir: check_dir.display().to_string(),
        files,
    });
    if report.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "strict-profile parent artifact verification failed:\n- {}",
            report.diagnostics.join("\n- ")
        ))
    }
}

fn parse_check_status(bytes: &[u8], expected_profile: &str) -> Result<CheckStatus, String> {
    let status =
        serde_json::from_slice::<CheckStatus>(bytes).map_err(|error| format!("parse check/status.json: {error}"))?;
    if status.exit_code != EXIT_ADMITTED
        || status.error_findings != 0
        || status.metadata.profile_name != expected_profile
    {
        return Err(format!(
            "direct check status is not eligible for closure: status={}, exit_code={}, errors={}, profile={}",
            status.status, status.exit_code, status.error_findings, status.metadata.profile_name
        ));
    }
    Ok(status)
}

fn parse_function_report(bytes: &[u8]) -> Result<FunctionAddressReport, String> {
    let report = serde_json::from_slice::<FunctionAddressReport>(bytes)
        .map_err(|error| format!("parse check/function-addresses.json: {error}"))?;
    if report.mode != FUNCTION_ADDRESS_MODE_DENY {
        return Err(format!("effect closure requires deny-mode function addresses, got `{}`", report.mode));
    }
    if report.records.is_empty() {
        return Err("effect closure requires at least one function-address record".to_owned());
    }
    Ok(report)
}

pub(crate) fn run_integrated_closure(
    workspace_root: &Path,
    binding: &octet_standards::project_architecture_policy::ArchitectureAdmissionBinding,
    selected_profile: &crate::strict_profile::SelectedStrictProfile,
    cargo_check_args: &[String],
    metadata: &crate::determinism::RunMetadata,
    command_line: &str,
    status_json: &str,
    function_address_json: &str,
    prepared: PreparedIntegratedClosure,
) -> Result<IntegratedClosureArtifacts, String> {
    if selected_profile.policy.profile_id != binding.strict_profile {
        return Err("architecture admission strict-profile binding does not match selected profile".to_owned());
    }
    let function_report = parse_function_report(function_address_json.as_bytes())?;
    let target = selected_target(cargo_check_args)?;
    let (dependency_graph, dependency_graph_bytes) =
        dependency_graph(workspace_root, cargo_check_args, &target, &CheckMetadata {
            rustc_version: metadata.rustc_version.clone(),
            toolchain: metadata.toolchain.clone(),
            profile_name: metadata.profile_name.clone(),
            profile_hash: metadata.profile_hash.clone(),
            config_hash: metadata.config_hash.clone(),
        })?;
    let sources = read_selected_sources(workspace_root, &selected_profile.core_source_scopes)?;
    validate_function_source_envelopes(&sources, &function_report)?;
    let record_root = function_address_record_root(&function_report)?;
    let command_bytes = normalized_text_bytes(command_line);
    let receipt = effect_closure::compute_closure(ClosureInput {
        profile_id: binding.strict_profile.clone(),
        profile_hash: metadata.profile_hash.clone(),
        config_hash: metadata.config_hash.clone(),
        command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
        check_status_blake3: crate::evidence::artifact_blake3_hex(status_json.as_bytes()),
        function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(function_address_json.as_bytes()),
        function_address_record_root: record_root,
        workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&prepared.policy_bytes),
        cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&dependency_graph_bytes),
        cargo_lock_blake3: dependency_graph.cargo_lock_blake3.clone(),
        target,
        toolchain: metadata.toolchain.clone(),
        sources,
        function_addresses: function_report.records,
        resolved_providers: dependency_graph.providers.clone(),
        catalog: effect_closure::embedded_catalog()?,
        policy: prepared.policy,
    });
    let diagnostics = effect_closure::verify_receipt_identity(&receipt);
    if !diagnostics.is_empty() {
        return Err(format!(
            "new integrated effect-closure receipt failed self-verification: {}",
            diagnostics.join("; ")
        ));
    }
    let parent = octet_standards::project_architecture_gate::EffectClosureParent {
        receipt_id: receipt.receipt_identity.clone(),
        admitted: receipt.status == AdmissionStatus::Admitted,
        evidence_tier: EvidenceTier::PolicyCheck.as_str().to_owned(),
        non_claims: receipt.non_claims.clone(),
    };
    Ok(IntegratedClosureArtifacts {
        parent,
        receipt_json: pretty_json_string(&receipt)?,
        policy_json: String::from_utf8(prepared.policy_bytes)
            .map_err(|error| format!("effect-closure policy is not UTF-8: {error}"))?,
        dependency_graph_json: String::from_utf8(dependency_graph_bytes)
            .map_err(|error| format!("effect-closure dependency graph is not UTF-8: {error}"))?,
        parent_status_json: status_json.to_owned(),
    })
}

fn function_address_record_root(report: &FunctionAddressReport) -> Result<String, String> {
    report
        .evidence_chain_handoff
        .canonical_roots
        .iter()
        .find(|root| root.kind == FUNCTION_ADDRESS_ROOT_KIND)
        .map(|root| root.digest.clone())
        .ok_or_else(|| "function-address receipt is missing its canonical record root".to_owned())
}

fn read_selected_sources(workspace_root: &Path, scopes: &[PathBuf]) -> Result<Vec<SourceEnvelope>, String> {
    let source_root = SourceRoot::open_existing(workspace_root)?;
    let mut selected = BTreeMap::<String, Vec<u8>>::new();
    for scope in scopes {
        let relative_scope = RootRelativePath::new(scope)?;
        if source_root.is_dir(&relative_scope)? {
            for file in source_root.read_files_recursive_from(&relative_scope)? {
                if Path::new(&file.locator).extension().is_some_and(|extension| extension == "rs") {
                    selected.insert(file.locator, file.contents);
                }
            }
        } else {
            if scope.extension().is_none_or(|extension| extension != "rs") {
                return Err(format!("selected core source file is not Rust: {}", scope.display()));
            }
            let source = source_root.read_to_string(&relative_scope)?;
            selected.insert(relative_scope.display(), source.into_bytes());
        }
    }
    if selected.is_empty() {
        return Err("selected core source scopes contain no Rust files".to_owned());
    }
    selected
        .into_iter()
        .map(|(path, bytes)| {
            let source = String::from_utf8(bytes)
                .map_err(|error| format!("selected Rust source `{path}` is not UTF-8: {error}"))?;
            let source_identity = crate::determinism::full_hash(&source);
            Ok(SourceEnvelope {
                path,
                source,
                source_identity,
            })
        })
        .collect()
}

fn validate_function_source_envelopes(
    sources: &[SourceEnvelope],
    report: &FunctionAddressReport,
) -> Result<(), String> {
    let selected = sources
        .iter()
        .map(|source| FunctionAddressSourceEnvelope {
            path: source.path.clone(),
            source_identity: source.source_identity.clone(),
        })
        .collect::<BTreeSet<_>>();
    let reported = report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
    if selected == reported {
        Ok(())
    } else {
        Err("selected source identities do not exactly match deny-mode function-address envelopes".to_owned())
    }
}

fn dependency_graph(
    workspace_root: &Path,
    cargo_check_args: &[String],
    target: &str,
    check_metadata: &CheckMetadata,
) -> Result<(DependencyGraphArtifact, Vec<u8>), String> {
    let cargo_lock = read_required(&workspace_root.join("Cargo.lock"))?;
    let cargo_lock_blake3 = crate::evidence::artifact_blake3_hex(&cargo_lock);
    let mut command = Command::new(CARGO_COMMAND);
    command
        .arg(CARGO_METADATA_COMMAND)
        .arg("--format-version")
        .arg(CARGO_METADATA_FORMAT_VERSION)
        .arg("--locked")
        .arg("--filter-platform")
        .arg(target)
        .args(metadata_cargo_args(cargo_check_args)?)
        .current_dir(workspace_root)
        .env(TOOLCHAIN_ENV, &check_metadata.toolchain);
    let output = command.output().map_err(|error| format!("run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata = serde_json::from_slice::<CargoMetadata>(&output.stdout)
        .map_err(|error| format!("parse cargo metadata JSON: {error}"))?;
    let workspace_members = metadata.workspace_members.into_iter().collect::<BTreeSet<_>>();
    let feature_map = metadata
        .resolve
        .map(|resolve| {
            resolve
                .nodes
                .into_iter()
                .map(|node| {
                    let mut features = node.features;
                    features.sort();
                    features.dedup();
                    (node.id, features)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut providers = Vec::new();
    for package in metadata.packages {
        let Some(features) = feature_map.get(&package.id).cloned() else {
            continue;
        };
        let workspace_package = workspace_members.contains(&package.id);
        let source = package_source(&package, workspace_package);
        providers.push(ProviderIdentity {
            kind: if workspace_package {
                ProviderKind::Workspace
            } else {
                ProviderKind::CargoPackage
            },
            name: package.name,
            version: package.version,
            source,
            target: target.to_owned(),
            features,
        });
    }
    for name in TOOLCHAIN_PROVIDER_NAMES {
        providers.push(ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: (*name).to_owned(),
            version: check_metadata.rustc_version.clone(),
            source: PROVIDER_SOURCE_TOOLCHAIN.to_owned(),
            target: target.to_owned(),
            features: Vec::new(),
        });
    }
    providers.sort();
    providers.dedup();
    let graph = DependencyGraphArtifact {
        schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
        target: target.to_owned(),
        toolchain: check_metadata.toolchain.clone(),
        rustc_version: check_metadata.rustc_version.clone(),
        cargo_lock_blake3,
        providers,
    };
    let bytes = pretty_json_bytes(&graph)?;
    Ok((graph, bytes))
}

fn package_source(package: &CargoPackage, workspace_package: bool) -> String {
    if workspace_package {
        return PROVIDER_SOURCE_WORKSPACE.to_owned();
    }
    package.source.clone().unwrap_or_else(|| {
        let manifest_hash = fs::read(&package.manifest_path).map_or_else(
            |_| crate::determinism::full_hash(&package.manifest_path),
            |bytes| crate::evidence::artifact_blake3_hex(&bytes),
        );
        format!("{PROVIDER_SOURCE_PATH_PREFIX}{manifest_hash}")
    })
}

/// Selects the direct Cargo arguments that change which manifest, configuration, or feature set
/// `cargo metadata` resolves, so the recorded provider graph describes the checked build.
fn metadata_cargo_args(cargo_check_args: &[String]) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut cargo_check_args = cargo_check_args.iter();
    while let Some(arg) = cargo_check_args.next() {
        if ["--manifest-path", "--config", "--features", "-F"].contains(&arg.as_str()) {
            let value = cargo_check_args.next().ok_or_else(|| format!("Cargo option `{arg}` is missing its value"))?;
            args.push(arg.clone());
            args.push(value.clone());
        } else if ["--offline", "--frozen", "--all-features", "--no-default-features"].contains(&arg.as_str())
            || arg.starts_with("--manifest-path=")
            || arg.starts_with("--config=")
            || arg.starts_with("--features=")
            || arg.starts_with("-F")
        {
            args.push(arg.clone());
        }
    }
    Ok(args)
}

fn selected_target(cargo_check_args: &[String]) -> Result<String, String> {
    let mut explicit = None;
    let mut index = 0;
    while index < cargo_check_args.len() {
        let arg = &cargo_check_args[index];
        if arg == "--target" {
            let value = cargo_check_args.get(index + 1).ok_or_else(|| "missing target after `--target`".to_owned())?;
            if explicit.replace(value.clone()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
            index += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--target=") {
            if value.is_empty() {
                return Err("missing target after `--target=`".to_owned());
            }
            if explicit.replace(value.to_owned()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
        }
        index += 1;
    }
    explicit.ok_or_else(|| {
        "effect closure v1 requires an explicit Cargo `--target TARGET` argument after `--`; ambient Cargo target configuration is not admissible evidence"
            .to_owned()
    })
}

// r[impl octet.effect_closure.evidence.visible]
fn write_closure_bundle(
    root: &ArtifactRoot,
    artifact_dir: &Path,
    command_bytes: &[u8],
    policy_bytes: &[u8],
    dependency_graph_bytes: &[u8],
    check_status_bytes: &[u8],
    function_address_bytes: &[u8],
    receipt: &EffectClosureReceipt,
) -> Result<(), String> {
    let receipt_bytes = pretty_json_bytes(receipt)?;
    let status = status_from_receipt(receipt);
    let status_bytes = pretty_json_bytes(&status)?;
    let summary_bytes = render_summary(receipt).into_bytes();
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(effect_closure::RECEIPT_FILE)?, &receipt_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, &summary_bytes)?;
    root.write(&relative(WORKSPACE_POLICY_ARTIFACT)?, policy_bytes)?;
    root.write(&relative(DEPENDENCY_GRAPH_ARTIFACT)?, dependency_graph_bytes)?;

    let mut artifacts = BTreeMap::from([
        (COMMAND_FILE.to_owned(), command_bytes.to_vec()),
        (effect_closure::RECEIPT_FILE.to_owned(), receipt_bytes),
        (STATUS_FILE.to_owned(), status_bytes),
        (SUMMARY_FILE.to_owned(), summary_bytes),
        (WORKSPACE_POLICY_ARTIFACT.to_owned(), policy_bytes.to_vec()),
        (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), dependency_graph_bytes.to_vec()),
        (CHECK_STATUS_ARTIFACT.to_owned(), check_status_bytes.to_vec()),
        (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), function_address_bytes.to_vec()),
    ]);
    let provenance = render_provenance(&artifacts, &receipt.inputs.profile_id)?;
    root.write(&relative(PROVENANCE_FILE)?, provenance.as_bytes())?;
    artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());

    let verification = verify_bundle(artifact_dir)?;
    if verification.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "new effect-closure artifact bundle failed self-verification:\n- {}",
            verification.diagnostics.join("\n- ")
        ))
    }
}

fn status_from_receipt(receipt: &EffectClosureReceipt) -> AdmissionStatusArtifact {
    AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: match receipt.status {
            AdmissionStatus::Admitted => STATUS_ADMITTED,
            AdmissionStatus::Denied => STATUS_DENIED,
        }
        .to_owned(),
        exit_code: match receipt.status {
            AdmissionStatus::Admitted => EXIT_ADMITTED,
            AdmissionStatus::Denied => EXIT_DENIED,
        },
        phase: PHASE_CLOSURE.to_owned(),
        profile_id: receipt.inputs.profile_id.clone(),
        receipt_identity: Some(receipt.receipt_identity.clone()),
        diagnostics: receipt
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect(),
        non_claims: receipt.non_claims.clone(),
    }
}

fn render_summary(receipt: &EffectClosureReceipt) -> String {
    let mut summary = String::new();
    let status = match receipt.status {
        AdmissionStatus::Admitted => STATUS_ADMITTED,
        AdmissionStatus::Denied => STATUS_DENIED,
    };
    let _ = writeln!(&mut summary, "Octet effect closure admission");
    let _ = writeln!(&mut summary, "Status: {status}");
    let _ = writeln!(&mut summary, "Claim scope: {}", receipt.claim_scope);
    let _ = writeln!(&mut summary, "Profile: {}", receipt.inputs.profile_id);
    let _ = writeln!(&mut summary, "Profile hash: {}", receipt.inputs.profile_hash);
    let _ = writeln!(&mut summary, "Config hash: {}", receipt.inputs.config_hash);
    let _ = writeln!(&mut summary, "Receipt identity: {}", receipt.receipt_identity);
    let _ = writeln!(&mut summary, "Selected sources: {}", receipt.sources.len());
    let _ = writeln!(&mut summary, "Functions: {}", receipt.nodes.len());
    let _ = writeln!(&mut summary, "Reviewed providers: {}", receipt.providers.len());
    let _ = writeln!(&mut summary, "Diagnostics: {}", receipt.diagnostics.len());
    for diagnostic in &receipt.diagnostics {
        let call_site = diagnostic.call_site_id.as_deref().unwrap_or("-");
        let _ = writeln!(&mut summary, "- {} [{}] {}", diagnostic.code, call_site, diagnostic.message);
    }
    let _ = writeln!(&mut summary, "Non-claims:");
    for non_claim in &receipt.non_claims {
        let _ = writeln!(&mut summary, "- {non_claim}");
    }
    summary
}

fn render_admission_output(
    output_format: OutputFormat,
    artifact_dir: &Path,
    receipt: &EffectClosureReceipt,
) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => eprint!("{}", render_summary(receipt)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&status_from_receipt(receipt))
                .map_err(|error| format!("serialize admission status output: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    eprintln!("Admission artifacts: {}", artifact_dir.display());
    Ok(())
}

fn render_provenance(artifacts: &BTreeMap<String, Vec<u8>>, profile_id: &str) -> Result<String, String> {
    let checked_scope = format!("{CHECKED_SCOPE_PREFIX}{profile_id}");
    let records = REQUIRED_PROVENANCE_ARTIFACTS
        .iter()
        .map(|artifact| {
            let bytes = artifacts
                .get(*artifact)
                .ok_or_else(|| format!("missing artifact `{artifact}` while rendering admission provenance"))?;
            let parent_artifacts = match *artifact {
                effect_closure::RECEIPT_FILE => vec![
                    COMMAND_FILE.to_owned(),
                    WORKSPACE_POLICY_ARTIFACT.to_owned(),
                    DEPENDENCY_GRAPH_ARTIFACT.to_owned(),
                    CHECK_STATUS_ARTIFACT.to_owned(),
                    CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(),
                ],
                STATUS_FILE | SUMMARY_FILE => vec![effect_closure::RECEIPT_FILE.to_owned()],
                _ => Vec::new(),
            };
            Ok(ProvenanceRecord {
                schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
                source_id: PROVENANCE_SOURCE_ID.to_owned(),
                checker_id: CHECKER_ID.to_owned(),
                evidence_tier: EvidenceTier::PolicyCheck,
                artifact_locator: (*artifact).to_owned(),
                blake3: crate::evidence::artifact_blake3_hex(bytes),
                checked_scope: checked_scope.clone(),
                verifier_identity: None,
                proof_artifact_identity: None,
                human_action: None,
                parent_artifacts,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut output = String::new();
    for record in records {
        output.push_str(
            &serde_json::to_string(&record).map_err(|error| format!("serialize admission provenance: {error}"))?,
        );
        output.push('\n');
    }
    Ok(output)
}

// r[impl octet.effect_closure.evidence.tamper]
fn verify_bundle(artifact_dir: &Path) -> Result<AdmissionVerificationReport, String> {
    verify_bundle_inner(artifact_dir, true)
}

// Bundle verification checks every bound artifact against one parsed receipt; splitting it would
// scatter the cross-artifact identity checks.
#[allow(clippy::too_many_lines)]
fn verify_bundle_inner(
    artifact_dir: &Path,
    verify_strict_profile_parent: bool,
) -> Result<AdmissionVerificationReport, String> {
    let root = ArtifactRoot::open_existing(artifact_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| (file.locator, file.contents))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    if verify_strict_profile_parent {
        match verify_check_bundle(&artifact_dir.join(CHECK_DIR)) {
            Ok(()) => {}
            Err(error) => diagnostics.push(error),
        }
    }
    let unsupported_schema = files
        .get(effect_closure::RECEIPT_FILE)
        .and_then(|bytes| effect_closure::unsupported_receipt_schema(bytes));
    let receipt = match unsupported_schema {
        Some(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
        None => parse_required::<EffectClosureReceipt>(&files, effect_closure::RECEIPT_FILE, &mut diagnostics),
    };
    let status = parse_required::<AdmissionStatusArtifact>(&files, STATUS_FILE, &mut diagnostics);
    let policy = parse_required::<WorkspacePolicy>(&files, WORKSPACE_POLICY_ARTIFACT, &mut diagnostics);
    let graph = parse_required::<DependencyGraphArtifact>(&files, DEPENDENCY_GRAPH_ARTIFACT, &mut diagnostics);
    let check_status = parse_required::<CheckStatus>(&files, CHECK_STATUS_ARTIFACT, &mut diagnostics);
    let function_report =
        parse_required::<FunctionAddressReport>(&files, CHECK_FUNCTION_ADDRESS_ARTIFACT, &mut diagnostics);

    if let Some(receipt) = &receipt {
        diagnostics.extend(effect_closure::verify_receipt_identity(receipt));
        verify_bound_artifact_hash(receipt.inputs.command_blake3.as_str(), &files, COMMAND_FILE, &mut diagnostics);
        verify_bound_artifact_hash(
            receipt.inputs.check_status_blake3.as_str(),
            &files,
            CHECK_STATUS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.function_address_artifact_blake3.as_str(),
            &files,
            CHECK_FUNCTION_ADDRESS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.workspace_policy_blake3.as_str(),
            &files,
            WORKSPACE_POLICY_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.cargo_graph_blake3.as_str(),
            &files,
            DEPENDENCY_GRAPH_ARTIFACT,
            &mut diagnostics,
        );
        if let Some(graph) = &graph {
            if graph.schema_version != DEPENDENCY_GRAPH_SCHEMA
                || graph.cargo_lock_blake3 != receipt.inputs.cargo_lock_blake3
                || graph.target != receipt.inputs.target
                || graph.toolchain != receipt.inputs.toolchain
            {
                diagnostics.push("dependency-graph identity does not match the effect-closure receipt".to_owned());
            }
            let providers = graph.providers.iter().collect::<BTreeSet<_>>();
            if receipt.providers.iter().any(|provider| !providers.contains(provider)) {
                diagnostics.push("receipt uses a provider absent from dependency-graph.json".to_owned());
            }
        }
        if let Some(check_status) = &check_status {
            let parent_matches = check_status.exit_code == EXIT_ADMITTED
                && check_status.error_findings == 0
                && check_status.metadata.profile_name == receipt.inputs.profile_id
                && check_status.metadata.profile_hash == receipt.inputs.profile_hash
                && check_status.metadata.config_hash == receipt.inputs.config_hash
                && check_status.metadata.toolchain == receipt.inputs.toolchain;
            if !parent_matches {
                diagnostics.push("direct check status identity does not match the effect-closure receipt".to_owned());
            }
        }
        diagnostics.extend(verify_bound_closure_inputs(receipt, function_report.as_ref(), policy.as_ref()));
        if let Some(status) = &status {
            let status_matches = status == &status_from_receipt(receipt);
            if !status_matches {
                diagnostics.push("status.json is stale or does not match effect-closure.json".to_owned());
            }
        }
        if let Some(summary_bytes) = files.get(SUMMARY_FILE) {
            if summary_bytes.as_slice() != render_summary(receipt).as_bytes() {
                diagnostics.push("summary.txt is stale or does not match effect-closure.json".to_owned());
            }
        } else {
            diagnostics.push(format!("missing admission artifact `{SUMMARY_FILE}`"));
        }
        match render_provenance(&files, &receipt.inputs.profile_id) {
            Ok(expected) => match files.get(PROVENANCE_FILE) {
                Some(actual) if actual.as_slice() == expected.as_bytes() => {}
                Some(_) => {
                    diagnostics.push("provenance.jsonl is stale or contains an artifact hash mismatch".to_owned());
                }
                None => diagnostics.push(format!("missing admission artifact `{PROVENANCE_FILE}`")),
            },
            Err(error) => diagnostics.push(error),
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    let checked_artifacts = REQUIRED_PROVENANCE_ARTIFACTS.iter().map(|name| (*name).to_owned()).collect();
    Ok(AdmissionVerificationReport {
        schema_version: ADMISSION_VERIFICATION_SCHEMA.to_owned(),
        artifact_dir: artifact_dir.display().to_string(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        }
        .to_owned(),
        receipt_identity: receipt.map(|receipt| receipt.receipt_identity),
        checked_artifacts,
        diagnostics,
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    })
}

/// Re-derives what a closure receipt asserts about its bound function-address report and workspace
/// policy: a valid policy, a deny-mode report with the receipt's record root and source envelopes,
/// and, for an `admitted` receipt, nodes, roles, and reviewed summaries that match both. `admit
/// verify` and `artifact verify` share this so an edited, re-identified bundle fails either way.
pub(crate) fn verify_bound_closure_inputs(
    receipt: &EffectClosureReceipt,
    function_report: Option<&FunctionAddressReport>,
    policy: Option<&WorkspacePolicy>,
) -> Vec<String> {
    let mut diagnostics = policy.map(effect_closure::validate_workspace_policy).unwrap_or_default();
    if let Some(function_report) = function_report {
        if function_report.mode != FUNCTION_ADDRESS_MODE_DENY {
            diagnostics.push("function-address parent is not deny mode".to_owned());
        }
        match function_address_record_root(function_report) {
            Ok(root) if root == receipt.inputs.function_address_record_root => {}
            Ok(_) => diagnostics.push("function-address record root does not match the receipt".to_owned()),
            Err(error) => diagnostics.push(error),
        }
        let receipt_sources = receipt
            .sources
            .iter()
            .map(|source| FunctionAddressSourceEnvelope {
                path: source.path.clone(),
                source_identity: source.source_identity.clone(),
            })
            .collect::<BTreeSet<_>>();
        let report_sources = function_report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
        if receipt_sources != report_sources {
            diagnostics.push("receipt sources do not match function-address source envelopes".to_owned());
        }
    }
    if let (Some(function_report), Some(policy)) = (function_report, policy) {
        diagnostics.extend(effect_closure::validate_admitted_bindings(receipt, &function_report.records, policy));
    }
    diagnostics
}

fn verify_bound_artifact_hash(
    expected: &str,
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) {
    match files.get(artifact) {
        Some(bytes) if crate::evidence::artifact_blake3_hex(bytes) == expected => {}
        Some(_) => diagnostics.push(format!("artifact `{artifact}` BLAKE3 identity does not match the receipt")),
        None => diagnostics.push(format!("missing admission artifact `{artifact}`")),
    }
}

fn parse_required<T: for<'de> Deserialize<'de>>(
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) -> Option<T> {
    let Some(bytes) = files.get(artifact) else {
        diagnostics.push(format!("missing admission artifact `{artifact}`"));
        return None;
    };
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{artifact}`: {error}"));
            None
        }
    }
}

fn checked_workspace_path(workspace_root: &Path, path: &Path, kind: &str) -> Result<PathBuf, String> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map_err(|_| format!("{kind} path must stay within workspace root: {}", path.display()))?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("{kind} path must be a non-empty workspace-relative path: {}", path.display()));
    }
    Ok(workspace_root.join(relative))
}

fn absolute_from_workspace(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn relative(path: &str) -> Result<RootRelativePath, String> {
    RootRelativePath::new(path)
}

fn read_required(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn normalized_text_bytes(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(b'\n');
    bytes
}

fn pretty_json_string<T: Serialize>(value: &T) -> Result<String, String> {
    let mut rendered = serde_json::to_string_pretty(value).map_err(|error| format!("serialize JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("serialize admission artifact: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROFILE: &str = "deterministic-core";
    const TEST_HASH: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // r[verify octet.effect_closure.dependency_summaries.target]
    #[test]
    fn metadata_feature_selection_is_narrow_and_deterministic() {
        let args = vec![
            "--lib".to_owned(),
            "--features".to_owned(),
            "serde,proof".to_owned(),
            "--all-targets".to_owned(),
            "--no-default-features".to_owned(),
        ];
        assert_eq!(metadata_cargo_args(&args).expect("metadata args"), vec![
            "--features",
            "serde,proof",
            "--no-default-features"
        ]);
        assert!(selected_target(&args).is_err());
        assert_eq!(
            selected_target(&["--target=x86_64-unknown-linux-gnu".to_owned()]).expect("explicit target"),
            "x86_64-unknown-linux-gnu"
        );
        assert!(selected_target(&["--target".to_owned(), "a".to_owned(), "--target=b".to_owned()]).is_err());
    }

    #[test]
    fn metadata_args_forward_manifest_config_and_short_features() {
        let args = [
            "--lib",
            "-F",
            "extra",
            "-Fother",
            "--config",
            "net.offline=true",
            "--config=build.rustflags=[]",
            "--manifest-path",
            "p/Cargo.toml",
            "--manifest-path=q/Cargo.toml",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--release",
            "--frozen",
        ]
        .map(str::to_owned);
        assert_eq!(metadata_cargo_args(&args).expect("metadata args"), vec![
            "-F",
            "extra",
            "-Fother",
            "--config",
            "net.offline=true",
            "--config=build.rustflags=[]",
            "--manifest-path",
            "p/Cargo.toml",
            "--manifest-path=q/Cargo.toml",
            "--frozen",
        ]);
        for flag in ["--manifest-path", "--config", "--features", "-F"] {
            assert_eq!(
                metadata_cargo_args(&["--lib".to_owned(), flag.to_owned()]),
                Err(format!("Cargo option `{flag}` is missing its value"))
            );
        }
    }

    #[test]
    fn workspace_path_validation_rejects_escape_and_accepts_relative() {
        let root = Path::new("/workspace");
        assert_eq!(
            checked_workspace_path(root, Path::new("policy.json"), "policy").expect("relative policy"),
            PathBuf::from("/workspace/policy.json")
        );
        assert!(checked_workspace_path(root, Path::new("../policy.json"), "policy").is_err());
        assert!(checked_workspace_path(root, Path::new("/outside/policy.json"), "policy").is_err());
    }

    const TEST_SOURCE_PATH: &str = "src/lib.rs";
    const TEST_TARGET: &str = "test-target";
    const TEST_TOOLCHAIN: &str = "test-toolchain";
    const TEST_RUSTC_VERSION: &str = "rustc-test";

    struct BundleFixture {
        command_bytes: Vec<u8>,
        check_status_bytes: Vec<u8>,
        function_address_bytes: Vec<u8>,
        policy_bytes: Vec<u8>,
        graph_bytes: Vec<u8>,
        input: ClosureInput,
    }

    fn test_function_address(fill: char) -> String {
        format!("b3:{}", fill.to_string().repeat(64))
    }

    fn bundle_fixture(
        source: &str,
        records: &[(&str, char)],
        call_summaries: Vec<effect_closure::CallSummary>,
        providers: Vec<ProviderIdentity>,
    ) -> BundleFixture {
        let command_bytes = normalized_text_bytes("cargo octet admit --profile deterministic-core --closure");
        let source_identity = crate::determinism::full_hash(source);
        let check_status_bytes = pretty_json_bytes(&serde_json::json!({
            "status": "clean",
            "exit_code": EXIT_ADMITTED,
            "metadata": {
                "rustc_version": TEST_RUSTC_VERSION,
                "toolchain": TEST_TOOLCHAIN,
                "profile_name": TEST_PROFILE,
                "profile_hash": TEST_HASH,
                "config_hash": TEST_HASH
            },
            "error_findings": 0
        }))
        .expect("check status JSON");
        let records = records
            .iter()
            .map(|(name, fill)| FunctionAddressRecord {
                path: TEST_SOURCE_PATH.to_owned(),
                name: (*name).to_owned(),
                function_address: test_function_address(*fill),
                source_identity: source_identity.clone(),
            })
            .collect::<Vec<_>>();
        let function_address_bytes = pretty_json_bytes(&serde_json::json!({
            "mode": FUNCTION_ADDRESS_MODE_DENY,
            "source_envelopes": [{
                "path": TEST_SOURCE_PATH,
                "source_identity": source_identity
            }],
            "records": records,
            "evidence_chain_handoff": {
                "canonical_roots": [{
                    "kind": FUNCTION_ADDRESS_ROOT_KIND,
                    "digest": TEST_HASH
                }]
            }
        }))
        .expect("function-address JSON");
        let policy = WorkspacePolicy {
            schema_version: effect_closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
            profile_id: TEST_PROFILE.to_owned(),
            function_address_baseline: "function-addresses.json".to_owned(),
            module_roles: vec![effect_closure::ModuleRoleAssignment {
                path: TEST_SOURCE_PATH.to_owned(),
                role: effect_closure::ModuleRole::Core,
            }],
            call_summaries,
            non_claims: vec!["bounded test policy".to_owned()],
        };
        let policy_bytes = pretty_json_bytes(&policy).expect("policy JSON");
        let graph = DependencyGraphArtifact {
            schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
            target: TEST_TARGET.to_owned(),
            toolchain: TEST_TOOLCHAIN.to_owned(),
            rustc_version: TEST_RUSTC_VERSION.to_owned(),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            providers: providers.clone(),
        };
        let graph_bytes = pretty_json_bytes(&graph).expect("graph JSON");
        let input = ClosureInput {
            profile_id: TEST_PROFILE.to_owned(),
            profile_hash: TEST_HASH.to_owned(),
            config_hash: TEST_HASH.to_owned(),
            command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
            check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
            function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
            function_address_record_root: TEST_HASH.to_owned(),
            workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&policy_bytes),
            cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&graph_bytes),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            target: TEST_TARGET.to_owned(),
            toolchain: TEST_TOOLCHAIN.to_owned(),
            sources: vec![SourceEnvelope {
                path: TEST_SOURCE_PATH.to_owned(),
                source: source.to_owned(),
                source_identity,
            }],
            function_addresses: records,
            resolved_providers: providers,
            catalog: effect_closure::embedded_catalog().expect("catalog"),
            policy,
        };
        BundleFixture {
            command_bytes,
            check_status_bytes,
            function_address_bytes,
            policy_bytes,
            graph_bytes,
            input,
        }
    }

    fn write_bundle(dir: &Path, fixture: &BundleFixture, receipt: &EffectClosureReceipt) {
        let mut artifacts = BTreeMap::from([
            (COMMAND_FILE.to_owned(), fixture.command_bytes.clone()),
            (STATUS_FILE.to_owned(), pretty_json_bytes(&status_from_receipt(receipt)).expect("status JSON")),
            (SUMMARY_FILE.to_owned(), render_summary(receipt).into_bytes()),
            (effect_closure::RECEIPT_FILE.to_owned(), pretty_json_bytes(receipt).expect("receipt JSON")),
            (WORKSPACE_POLICY_ARTIFACT.to_owned(), fixture.policy_bytes.clone()),
            (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), fixture.graph_bytes.clone()),
            (CHECK_STATUS_ARTIFACT.to_owned(), fixture.check_status_bytes.clone()),
            (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), fixture.function_address_bytes.clone()),
        ]);
        let provenance = render_provenance(&artifacts, TEST_PROFILE).expect("provenance");
        artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());
        for (locator, bytes) in &artifacts {
            let path = dir.join(locator);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create artifact parent");
            }
            fs::write(path, bytes).expect("write artifact");
        }
    }

    fn toolchain_summary(
        caller_fill: char,
        shape: &str,
        effects: Vec<effect_closure::Effect>,
    ) -> (effect_closure::CallSummary, ProviderIdentity) {
        let provider = ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: "alloc".to_owned(),
            version: TEST_RUSTC_VERSION.to_owned(),
            source: PROVIDER_SOURCE_TOOLCHAIN.to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        };
        // Fixture callers are file-root functions whose call path starts with an undeclared name.
        let leading = shape.trim_start_matches("call:").split("::").next().unwrap_or_default();
        let scope = format!("module=self; {leading}=prelude");
        let summary = effect_closure::CallSummary {
            call_site_id: effect_closure::call_site_identity(
                &test_function_address(caller_fill),
                TEST_SOURCE_PATH,
                &scope,
                shape,
                0,
            ),
            review_label: format!("reviewed `{shape}` toolchain call"),
            callee: shape.trim_start_matches("call:").to_owned(),
            provider: provider.clone(),
            effects,
            evidence: effect_closure::EvidenceReference {
                kind: "toolchain-api-review-v1".to_owned(),
                locator: "rustc-version#alloc".to_owned(),
                blake3: effect_closure::provider_identity_blake3(&provider),
            },
        };
        (summary, provider)
    }

    /// Rewrites the stored receipt the way the previous schema wrote it: schema label `v1` and
    /// call edges without the `scope` field that schema v2 requires.
    fn downgrade_receipt_to_v1(receipt: &EffectClosureReceipt) -> Vec<u8> {
        let mut value = serde_json::to_value(receipt).expect("receipt JSON");
        value["schema_version"] = serde_json::Value::from("octet-effect-closure-receipt/v1");
        for node in value["nodes"].as_array_mut().expect("receipt nodes") {
            for call in node["calls"].as_array_mut().expect("node calls") {
                call.as_object_mut().expect("call object").remove("scope");
            }
        }
        serde_json::to_vec_pretty(&value).expect("v1 receipt JSON")
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn previous_receipt_schema_is_reported_unsupported() {
        let temp = tempfile::tempdir().expect("tempdir");
        let fixture = bundle_fixture("fn root() { mystery(); }\n", &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert!(!receipt.nodes[0].calls.is_empty());
        write_bundle(temp.path(), &fixture, &receipt);
        fs::write(temp.path().join(effect_closure::RECEIPT_FILE), downgrade_receipt_to_v1(&receipt))
            .expect("write v1 receipt");

        let report = verify_bundle_inner(temp.path(), false).expect("verification report");

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported effect-closure receipt schema")
                    && diagnostic.contains("octet-effect-closure-receipt/v1")),
            "{:?}",
            report.diagnostics
        );
        assert!(
            !report.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed")),
            "{:?}",
            report.diagnostics
        );
    }

    // r[verify octet.effect_closure.evidence.visible]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn complete_admission_bundle_verifies_and_tampering_is_invalid() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(CHECK_DIR)).expect("create check dir");
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        write_bundle(temp.path(), &fixture, &receipt);

        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);
        let incomplete_parent = verify_bundle(temp.path()).expect("parent verification report");
        assert_eq!(incomplete_parent.status, STATUS_INVALID);
        assert!(
            incomplete_parent
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("strict-profile parent artifact verification failed"))
        );

        fs::write(temp.path().join(SUMMARY_FILE), b"tampered\n").expect("tamper summary");
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("summary.txt is stale")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("provenance.jsonl is stale")));
    }

    fn forge_admitted(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
        receipt.diagnostics.clear();
        receipt.status = AdmissionStatus::Admitted;
        effect_closure::finalize_receipt(receipt)
    }

    fn assert_forgery_rejected(fixture: &BundleFixture, forged: &EffectClosureReceipt, expected: &str) {
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), fixture, forged);
        let report = verify_bundle_inner(temp.path(), false).expect("verification report");
        assert_eq!(report.status, STATUS_INVALID, "forged admitted receipt verified");
        assert!(
            report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "expected `{expected}` in {:?}",
            report.diagnostics
        );
    }

    /// Rebinds the fixture to a workspace policy whose module roles differ from the default
    /// single `core` assignment, keeping every bound identity consistent with the new policy bytes.
    fn with_module_roles(
        mut fixture: BundleFixture,
        module_roles: Vec<effect_closure::ModuleRoleAssignment>,
    ) -> BundleFixture {
        fixture.input.policy.module_roles = module_roles;
        fixture.policy_bytes = pretty_json_bytes(&fixture.input.policy).expect("policy JSON");
        fixture.input.workspace_policy_blake3 = crate::evidence::artifact_blake3_hex(&fixture.policy_bytes);
        fixture
    }

    /// Rewrites the first call of a denied single-node receipt into the reviewed-summary edge the
    /// policy summary would produce, the way an attacker would hand-edit a denied bundle.
    fn forge_reviewed_edge(
        mut receipt: EffectClosureReceipt,
        summary: &effect_closure::CallSummary,
    ) -> EffectClosureReceipt {
        let node = &mut receipt.nodes[0];
        let call = &mut node.calls[0];
        assert_eq!(call.call_site_id, summary.call_site_id);
        call.resolution = effect_closure::CallResolution::ReviewedSummary;
        call.provider = Some(summary.provider.clone());
        call.effects.clone_from(&summary.effects);
        call.review_label = Some(summary.review_label.clone());
        call.evidence = Some(summary.evidence.clone());
        node.direct_effects.clone_from(&summary.effects);
        node.closure_effects.clone_from(&summary.effects);
        receipt.providers = vec![summary.provider.clone()];
        forge_admitted(receipt)
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_receipt_must_match_bound_function_addresses_and_policy_summaries() {
        let source = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";
        let (summary, provider) = toolchain_summary('b', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        let fixture = bundle_fixture(source, &[("root", 'b')], vec![summary], vec![provider]);
        let admitted = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &admitted);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        let (summary, provider) = toolchain_summary('b', "call:Vec::new", vec![effect_closure::Effect::Panic]);
        let fixture = bundle_fixture(source, &[("root", 'b')], vec![summary], vec![provider]);
        let mut narrowed = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(narrowed.status, AdmissionStatus::Denied);
        narrowed.nodes[0].calls[0].effects.clear();
        narrowed.nodes[0].direct_effects.clear();
        narrowed.nodes[0].closure_effects.clear();
        assert_forgery_rejected(&fixture, &forge_admitted(narrowed), "does not match its workspace-policy summary");

        let fixture = bundle_fixture("fn root() {}\n", &[("ghost", 'c'), ("root", 'd')], Vec::new(), Vec::new());
        let unmatched = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(unmatched.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_admitted(unmatched),
            "do not exactly match the bound function-address",
        );

        let (summary, provider) = toolchain_summary('e', "call:gone", Vec::new());
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'e')], vec![summary], vec![provider]);
        let stale = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale.status, AdmissionStatus::Denied);
        assert_forgery_rejected(&fixture, &forge_admitted(stale), "is not used by the admitted receipt");
    }

    // r[verify octet.effect_closure.roles.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_receipt_must_bind_exactly_one_core_role_per_source() {
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'f')], Vec::new(), Vec::new());
        let fixture = with_module_roles(fixture, vec![effect_closure::ModuleRoleAssignment {
            path: TEST_SOURCE_PATH.to_owned(),
            role: effect_closure::ModuleRole::Shell,
        }]);
        let shell = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(shell.status, AdmissionStatus::Denied);
        assert_forgery_rejected(&fixture, &forge_admitted(shell.clone()), "exactly one workspace-policy role `core`");
        let mut relabelled_core = shell;
        relabelled_core.nodes[0].role = effect_closure::ModuleRole::Core;
        assert_forgery_rejected(&fixture, &forge_admitted(relabelled_core), "exactly one workspace-policy role `core`");

        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'f')], Vec::new(), Vec::new());
        let fixture = with_module_roles(fixture, vec![effect_closure::ModuleRoleAssignment {
            path: "src/other.rs".to_owned(),
            role: effect_closure::ModuleRole::Core,
        }]);
        let unroled = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(unroled.status, AdmissionStatus::Denied);
        assert_eq!(unroled.nodes[0].role, effect_closure::ModuleRole::Core, "unroled paths default to core");
        assert_forgery_rejected(&fixture, &forge_admitted(unroled), "exactly one workspace-policy role `core`");
    }

    const ALLOCATING_SOURCE: &str = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_toolchain_summary_evidence_must_bind_its_provider_identity() {
        let source = ALLOCATING_SOURCE;
        let (mut summary, provider) = toolchain_summary('9', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        let fixture = bundle_fixture(source, &[("root", '9')], vec![summary.clone()], vec![provider.clone()]);
        let provider_bound = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(provider_bound.status, AdmissionStatus::Admitted, "{:?}", provider_bound.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &provider_bound);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        summary.evidence.blake3 = crate::determinism::full_hash("a different toolchain provider");
        let fixture = bundle_fixture(source, &[("root", '9')], vec![summary.clone()], vec![provider]);
        let stale_toolchain = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale_toolchain.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_reviewed_edge(stale_toolchain, &summary),
            "is not bound to the recorded Cargo.lock or provider identity",
        );
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_cargo_summary_evidence_must_bind_the_recorded_cargo_lock() {
        let source = ALLOCATING_SOURCE;
        let (mut summary, _) = toolchain_summary('8', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        summary.provider = ProviderIdentity {
            kind: ProviderKind::CargoPackage,
            name: "vecs".to_owned(),
            version: "1.0.0".to_owned(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        };
        summary.evidence = effect_closure::EvidenceReference {
            kind: "cargo-lock-review-v1".to_owned(),
            locator: "Cargo.lock#vecs".to_owned(),
            blake3: TEST_HASH.to_owned(),
        };
        let fixture = bundle_fixture(source, &[("root", '8')], vec![summary.clone()], vec![summary.provider.clone()]);
        let lock_bound = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(lock_bound.status, AdmissionStatus::Admitted, "{:?}", lock_bound.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &lock_bound);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        summary.evidence.blake3 = crate::determinism::full_hash("an older Cargo.lock");
        let fixture = bundle_fixture(source, &[("root", '8')], vec![summary.clone()], vec![summary.provider.clone()]);
        let stale_lock = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale_lock.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_reviewed_edge(stale_lock, &summary),
            "is not bound to the recorded Cargo.lock or provider identity",
        );
    }

    #[test]
    fn artifact_root_preparation_refuses_foreign_directories_without_deleting() {
        let temp = tempfile::tempdir().expect("tempdir");

        let fresh = temp.path().join("fresh");
        prepare_artifact_root(&fresh).expect("missing artifact directory is created");
        prepare_artifact_root(&fresh).expect("empty artifact directory is accepted");

        let foreign = temp.path().join("foreign");
        fs::create_dir_all(foreign.join(DEPENDENCY_GRAPH_ARTIFACT)).expect("create foreign directory");
        fs::write(foreign.join(DEPENDENCY_GRAPH_ARTIFACT).join("data"), "keep\n").expect("foreign nested file");
        fs::write(foreign.join(COMMAND_FILE), "keep\n").expect("foreign command file");
        fs::write(foreign.join(STATUS_FILE), "{}\n").expect("foreign status file");
        let error = prepare_artifact_root(&foreign).err().expect("foreign directory must be refused");
        assert!(error.contains("not an existing Octet admission bundle"), "{error}");
        assert!(foreign.join(DEPENDENCY_GRAPH_ARTIFACT).join("data").is_file());
        assert!(foreign.join(COMMAND_FILE).is_file());
        assert!(foreign.join(STATUS_FILE).is_file());

        let previous = temp.path().join("previous");
        let root = ArtifactRoot::open_or_create(&previous).expect("previous bundle root");
        write_blocked_status(&root, TEST_PROFILE, EXIT_DENIED, b"cargo octet admit\n").expect("previous bundle");
        fs::write(previous.join(effect_closure::RECEIPT_FILE), "{}\n").expect("stale receipt");
        prepare_artifact_root(&previous).expect("previous admission bundle is accepted");
        assert!(!previous.join(STATUS_FILE).exists());
        assert!(!previous.join(effect_closure::RECEIPT_FILE).exists());

        let interrupted = temp.path().join("interrupted");
        fs::create_dir_all(interrupted.join(CHECK_DIR)).expect("interrupted check dir");
        fs::write(interrupted.join(CHECK_STATUS_ARTIFACT), "{}\n").expect("check status marker");
        fs::write(interrupted.join(CHECK_PROVENANCE_ARTIFACT), "").expect("check provenance marker");
        fs::write(interrupted.join(COMMAND_FILE), "cargo octet admit\n").expect("interrupted command");
        prepare_artifact_root(&interrupted).expect("run stopped after its direct check is accepted");
        assert!(!interrupted.join(COMMAND_FILE).exists());
        assert!(interrupted.join(CHECK_STATUS_ARTIFACT).is_file());
    }

    #[test]
    fn verification_resolves_relative_artifact_dir_against_workspace_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path().canonicalize().expect("canonical workspace");
        fs::write(
            workspace.join("Cargo.toml"),
            "[workspace]\n\n[workspace.metadata.octet]\ndefault_scope = []\ncargo_check_args = []\n",
        )
        .expect("workspace manifest");
        let nested = workspace.join("crates/member");
        fs::create_dir_all(&nested).expect("nested directory");

        let relative = Path::new("target/octet-admission");
        assert_eq!(
            verification_artifact_dir(&nested, relative).expect("relative artifact dir"),
            workspace.join(relative)
        );
        let absolute = workspace.join("elsewhere");
        assert_eq!(verification_artifact_dir(&nested, &absolute).expect("absolute artifact dir"), absolute);
    }
}
