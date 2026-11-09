//! Bounded Aeneas/Charon/Lean execution planning and shell orchestration.

use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use octet_standards::aeneas_observation::MeasuredLeanFile;
use octet_standards::aeneas_observation::observe_aeneas_trust_closure;
use octet_standards::aeneas_observation::observe_charon_extraction_policy;
use octet_standards::aeneas_observation::validate_observed_aeneas_translation;
use octet_standards::formal_proof::AeneasAssumptionPolicy;
use octet_standards::formal_proof::AeneasExecutionLimits;
use octet_standards::formal_proof::AeneasTheoremAdequacy;
use octet_standards::formal_proof::AeneasTheoremAdequacyDecision;
use octet_standards::formal_proof::CharonExtractionPolicy;
use octet_standards::formal_proof::assess_aeneas_theorem_source;
use octet_standards::formal_proof::validate_aeneas_theorem_statement_approval;
use octet_standards::formal_proof::validate_charon_extraction_contract;
use octet_standards::model::ProofRail;
use serde::Deserialize;

/// Backend name used by the opt-in Aeneas proof rail.
pub(crate) const AENEAS_LEAN_BACKEND: &str = "aeneas-lean";

const AENEAS_PROFILE_JSON: &str = include_str!("../../standards/proof-rails/aeneas-lean/generated/profile.json");
const CHARON_EXECUTABLE_ENV: &str = "OCTET_AENEAS_CHARON";
const AENEAS_EXECUTABLE_ENV: &str = "OCTET_AENEAS_AENEAS";
const LEAN_EXECUTABLE_ENV: &str = "OCTET_AENEAS_LEAN";
const AENEAS_LEAN_LIBRARY_ENV: &str = "OCTET_AENEAS_LEAN_LIBRARY";
const NETWORK_SANDBOX_ENV: &str = "OCTET_AENEAS_NETWORK_SANDBOX";
const CHARON_COMMAND_NAME: &str = "charon";
const AENEAS_COMMAND_NAME: &str = "aeneas";
const LEAN_COMMAND_NAME: &str = "lake";
const REQUIRED_CHARON_BASE_ARGUMENTS: &[&str] = &["cargo", "--preset", "aeneas", "--dest-file"];
const REQUIRED_CHARON_ARGUMENTS: &[&str] = &["cargo", "--preset", "aeneas", "--dest-file", "llbc.llbc"];
const REQUIRED_AENEAS_ARGUMENTS: &[&str] = &["-backend", "lean", "-emit-json"];
const REQUIRED_LEAN_SUBCOMMANDS: &[&str] = &["build"];
const AENEAS_LIBRARY_DIRECTORY: &str = "aeneas-lean-library";
const LAKEFILE_NAME: &str = "lakefile.lean";
const LAKE_MANIFEST_NAME: &str = "lake-manifest.json";
const LAKE_OVERRIDES_NAME: &str = "lake-overrides.json";
const LAKE_PACKAGE_OVERRIDES_NAME: &str = "package-overrides.json";
const LEAN_TOOLCHAIN_NAME: &str = "lean-toolchain";
const HOME_ENVIRONMENT_NAME: &str = "HOME";
const MATHLIB_NO_CACHE_ENVIRONMENT_NAME: &str = "MATHLIB_NO_CACHE_ON_UPDATE";
const MATHLIB_NO_CACHE_ENVIRONMENT_VALUE: &str = "1";
const PATH_ENVIRONMENT_NAME: &str = "PATH";
const SANDBOX_ARGUMENT_SEPARATOR: &str = "--";
const GIT_DIRECTORY_NAME: &str = ".git";
const TARGET_DIRECTORY_NAME: &str = "target";
const LAKE_DIRECTORY_NAME: &str = ".lake";
const NETWORK_DENIAL_FLAG: &str = "--unshare-net";
/// A fresh PID namespace makes the sandbox init process the namespace reaper, so killing the
/// sandbox tears down every grandchild (Cargo/rustc under Charon, Lean workers under Lake).
const PID_NAMESPACE_FLAG: &str = "--unshare-pid";
const DIE_WITH_PARENT_FLAG: &str = "--die-with-parent";
const NIX_STORE_DIRECTORY: &str = "/nix/store";
const PROC_MOUNT_PATH: &str = "/proc";
const DEV_MOUNT_PATH: &str = "/dev";
const PROCESS_POLL_INTERVAL_MS: u64 = 10;
const PROCESS_TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const PROCESS_STDIN_MAX_BYTES: usize = 1;
const ACCEPTED_EXIT_CODE: i32 = 0;
const ARGV_PAIR_WIDTH: usize = 2;
const STAGE_COUNT: usize = 3;
const SOURCE_PATH_INDEX: usize = 0;
const LLBC_PATH: &str = "llbc.llbc";
const THEOREM_PATH_INDEX: usize = 0;
const DEFAULT_SOURCE_PATH: &str = ".";
const DEFAULT_THEOREM_PATH: &str = "Theorem.lean";
/// Module file the generated Lake package builds as its `Theorem` root.
const LEAN_THEOREM_MODULE_FILE: &str = "Theorem.lean";
const AENEAS_PASS_SIGNAL: &str = "aeneas-proof-result: pass";
const TRANSLATION_MANIFEST_ARTIFACT: &str = "translation.json";
const GENERATED_LEAN_ARTIFACT: &str = "Llbc.lean";
const PACKAGE_LOCK_ARTIFACT: &str = "lake-manifest.json";
const LEAN_TOOLCHAIN_ARTIFACT: &str = "lean-toolchain";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasArtifact {
    pub(crate) locator: String,
    pub(crate) contents: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AeneasStageKind {
    /// Charon LLBC extraction.
    Charon,
    /// Aeneas Lean translation and JSON manifest emission.
    Aeneas,
    /// Lean theorem checking.
    Lean,
}

impl AeneasStageKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Charon => "charon",
            Self::Aeneas => "aeneas",
            Self::Lean => "lean",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasExecutionPlanInput {
    pub(crate) charon_argv: Vec<String>,
    pub(crate) aeneas_argv: Vec<String>,
    pub(crate) lean_argv: Vec<String>,
    pub(crate) charon_executable: PathBuf,
    pub(crate) aeneas_executable: PathBuf,
    pub(crate) lean_executable: PathBuf,
    pub(crate) aeneas_lean_library: PathBuf,
    pub(crate) network_sandbox: PathBuf,
    pub(crate) source_root: PathBuf,
    pub(crate) work_root: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) llbc_path: PathBuf,
    pub(crate) theorem_path: PathBuf,
    /// Exact preflighted theorem bytes that Lean checks as the `Theorem` root module.
    pub(crate) theorem_source: String,
    pub(crate) limits: AeneasExecutionLimits,
    pub(crate) environment_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReusedCharonStageInput {
    pub(crate) charon_argv: Vec<String>,
    pub(crate) charon_executable: PathBuf,
    pub(crate) network_sandbox: PathBuf,
    pub(crate) source_root: PathBuf,
    pub(crate) work_root: PathBuf,
    pub(crate) snapshot_path: PathBuf,
    pub(crate) limits: AeneasExecutionLimits,
    pub(crate) environment_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasStagePlan {
    pub(crate) kind: AeneasStageKind,
    pub(crate) argv: Vec<String>,
    pub(crate) working_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasExecutionPlan {
    pub(crate) stages: Vec<AeneasStagePlan>,
    pub(crate) source_root: PathBuf,
    pub(crate) work_root: PathBuf,
    pub(crate) limits: AeneasExecutionLimits,
    pub(crate) environment_allowlist: Vec<String>,
    pub(crate) aeneas_lean_library: PathBuf,
    pub(crate) theorem_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasStageObservation {
    pub(crate) kind: AeneasStageKind,
    pub(crate) argv: Vec<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) timed_out: bool,
    pub(crate) stdout: String,
    pub(crate) stdout_original_bytes: usize,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr: String,
    pub(crate) stderr_original_bytes: usize,
    pub(crate) stderr_truncated: bool,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasExecutionReport {
    pub(crate) observations: Vec<AeneasStageObservation>,
    pub(crate) diagnostics: Vec<String>,
}

// Each flag is an independent observed process fact copied into the proof receipt; they are not a
// state machine.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasCommandSummary {
    pub(crate) command_succeeded: bool,
    pub(crate) timed_out: bool,
    pub(crate) stdout_text: String,
    pub(crate) stdout_original_bytes: usize,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_text: String,
    pub(crate) stderr_original_bytes: usize,
    pub(crate) stderr_truncated: bool,
    pub(crate) command_blake3: String,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) stage_argvs: Vec<Vec<String>>,
    pub(crate) artifacts: Vec<AeneasArtifact>,
    pub(crate) provider_profile: Vec<u8>,
    /// Limits and allowlist the shell actually enforced; absent when no stage was planned.
    pub(crate) effective_policy: Option<AeneasEffectivePolicy>,
}

/// Execution policy the shell enforced for every stage of one rail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasEffectivePolicy {
    pub(crate) limits: AeneasExecutionLimits,
    pub(crate) environment_allowlist: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AeneasProfile {
    stage_commands: AeneasStageCommands,
    limits: AeneasExecutionLimits,
    environment_allowlist: Vec<String>,
    extraction_policy: CharonExtractionPolicy,
    assumption_policy: AeneasAssumptionPolicy,
}

// The `*_argv` names are the deserialized provider-profile JSON keys; renaming would break profile
// parsing.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
struct AeneasStageCommands {
    charon_argv: Vec<String>,
    aeneas_argv: Vec<String>,
    lean_argv: Vec<String>,
}

/// Build a deterministic, shell-free three-stage plan from typed inputs.
pub(crate) fn build_aeneas_execution_plan(input: AeneasExecutionPlanInput) -> Result<AeneasExecutionPlan, Vec<String>> {
    let mut diagnostics = Vec::new();
    validate_absolute_path(&mut diagnostics, "charon_executable", &input.charon_executable);
    validate_absolute_path(&mut diagnostics, "aeneas_executable", &input.aeneas_executable);
    validate_absolute_path(&mut diagnostics, "lean_executable", &input.lean_executable);
    validate_absolute_path(&mut diagnostics, "aeneas_lean_library", &input.aeneas_lean_library);
    validate_absolute_path(&mut diagnostics, "network_sandbox", &input.network_sandbox);
    validate_absolute_path(&mut diagnostics, "source_root", &input.source_root);
    validate_absolute_path(&mut diagnostics, "work_root", &input.work_root);
    if input.source_root == input.work_root {
        diagnostics.push("source_root and work_root must be distinct isolated roots".to_owned());
    }
    validate_relative_path(&mut diagnostics, "source_path", &input.source_path);
    validate_relative_path(&mut diagnostics, "llbc_path", &input.llbc_path);
    validate_relative_path(&mut diagnostics, "theorem_path", &input.theorem_path);
    validate_stage_argv(
        &mut diagnostics,
        AeneasStageKind::Charon,
        &input.charon_argv,
        CHARON_COMMAND_NAME,
        REQUIRED_CHARON_ARGUMENTS,
    );
    validate_stage_argv(
        &mut diagnostics,
        AeneasStageKind::Aeneas,
        &input.aeneas_argv,
        AENEAS_COMMAND_NAME,
        REQUIRED_AENEAS_ARGUMENTS,
    );
    validate_stage_argv(
        &mut diagnostics,
        AeneasStageKind::Lean,
        &input.lean_argv,
        LEAN_COMMAND_NAME,
        REQUIRED_LEAN_SUBCOMMANDS,
    );
    validate_environment_allowlist(&mut diagnostics, &input.environment_allowlist);
    if input.limits.timeout_ms == 0 {
        diagnostics.push("limits.timeout_ms must be positive".to_owned());
    }
    if input.limits.stdout_max_bytes == 0 {
        diagnostics.push("limits.stdout_max_bytes must be positive".to_owned());
    }
    if input.limits.stderr_max_bytes == 0 {
        diagnostics.push("limits.stderr_max_bytes must be positive".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let stages = vec![
        AeneasStagePlan {
            kind: AeneasStageKind::Charon,
            argv: sandboxed_argv(&input, &input.charon_executable, &input.charon_argv, None),
            working_dir: input.work_root.clone(),
        },
        AeneasStagePlan {
            kind: AeneasStageKind::Aeneas,
            argv: sandboxed_argv(&input, &input.aeneas_executable, &input.aeneas_argv, Some(&input.llbc_path)),
            working_dir: input.work_root.clone(),
        },
        AeneasStagePlan {
            kind: AeneasStageKind::Lean,
            argv: sandboxed_argv(&input, &input.lean_executable, &input.lean_argv, None),
            working_dir: input.work_root.clone(),
        },
    ];
    assert_eq!(stages.len(), STAGE_COUNT, "Aeneas plan must contain every provider stage");
    Ok(AeneasExecutionPlan {
        stages,
        source_root: input.source_root,
        work_root: input.work_root,
        limits: input.limits,
        environment_allowlist: input.environment_allowlist,
        aeneas_lean_library: input.aeneas_lean_library,
        theorem_source: input.theorem_source,
    })
}

fn validate_absolute_path(diagnostics: &mut Vec<String>, field: &str, path: &Path) {
    if !path.is_absolute() {
        diagnostics.push(format!("{field} must be an absolute pinned path"));
    }
}

fn validate_relative_path(diagnostics: &mut Vec<String>, field: &str, path: &Path) {
    if path.as_os_str().is_empty() || path.is_absolute() {
        diagnostics.push(format!("{field} must be a non-empty relative artifact path"));
        return;
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        diagnostics.push(format!("{field} must not escape the isolated work root"));
    }
}

fn validate_stage_argv(
    diagnostics: &mut Vec<String>,
    stage: AeneasStageKind,
    argv: &[String],
    expected_executable: &str,
    required_arguments: &[&str],
) {
    let field = format!("{}_argv", stage.as_str());
    if argv.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    if argv[0] != expected_executable {
        diagnostics.push(format!("{field}[0] must be `{expected_executable}`"));
    }
    for argument in argv {
        if argument.trim().is_empty() {
            diagnostics.push(format!("{field} contains an empty argv segment"));
        }
        if argument.chars().any(shell_metacharacter) {
            diagnostics.push(format!("{field} contains shell metacharacters; argv-only execution is required"));
        }
    }
    for required_argument in required_arguments {
        if !argv.iter().any(|argument| argument == required_argument) {
            diagnostics.push(format!("{field} is missing required argument `{required_argument}`"));
        }
    }
}

fn validate_environment_allowlist(diagnostics: &mut Vec<String>, names: &[String]) {
    if names.is_empty() {
        diagnostics.push("environment_allowlist must not be empty".to_owned());
    }
    for name in names {
        if name.trim().is_empty() {
            diagnostics.push("environment_allowlist contains an empty name".to_owned());
        } else if name.trim() != name || !is_environment_name(name) {
            diagnostics.push(format!("environment_allowlist entry `{name}` is not a canonical variable name"));
        }
    }
}

fn is_environment_name(name: &str) -> bool {
    name.chars().enumerate().all(|(index, character)| {
        if index == 0 {
            character.is_ascii_uppercase() || character == '_'
        } else {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        }
    })
}

fn shell_metacharacter(character: char) -> bool {
    matches!(character, '|' | '&' | ';' | '<' | '>' | '$' | '`' | '\n' | '\r')
}

fn sandboxed_argv(
    input: &AeneasExecutionPlanInput,
    executable: &Path,
    provider_argv: &[String],
    input_path: Option<&Path>,
) -> Vec<String> {
    sandboxed_provider_argv(
        &input.network_sandbox,
        &input.source_root,
        &input.work_root,
        executable,
        provider_argv,
        input_path,
    )
}

fn sandboxed_provider_argv(
    network_sandbox: &Path,
    source_root: &Path,
    work_root: &Path,
    executable: &Path,
    provider_argv: &[String],
    input_path: Option<&Path>,
) -> Vec<String> {
    let mut argv = vec![
        network_sandbox.display().to_string(),
        DIE_WITH_PARENT_FLAG.to_owned(),
        NETWORK_DENIAL_FLAG.to_owned(),
        PID_NAMESPACE_FLAG.to_owned(),
        "--dev".to_owned(),
        DEV_MOUNT_PATH.to_owned(),
        "--proc".to_owned(),
        PROC_MOUNT_PATH.to_owned(),
        "--ro-bind".to_owned(),
        NIX_STORE_DIRECTORY.to_owned(),
        NIX_STORE_DIRECTORY.to_owned(),
        "--ro-bind".to_owned(),
        source_root.display().to_string(),
        source_root.display().to_string(),
        "--bind".to_owned(),
        work_root.display().to_string(),
        work_root.display().to_string(),
        "--chdir".to_owned(),
        work_root.display().to_string(),
        "--".to_owned(),
        executable.display().to_string(),
    ];
    argv.extend(provider_argv.iter().skip(1).cloned());
    if let Some(input_path) = input_path {
        argv.push(input_path.display().to_string());
    }
    argv
}

pub(crate) fn execute_reused_charon_stage(input: ReusedCharonStageInput) -> AeneasStageObservation {
    let mut diagnostics = Vec::new();
    validate_absolute_path(&mut diagnostics, "charon_executable", &input.charon_executable);
    validate_absolute_path(&mut diagnostics, "network_sandbox", &input.network_sandbox);
    validate_absolute_path(&mut diagnostics, "source_root", &input.source_root);
    validate_absolute_path(&mut diagnostics, "work_root", &input.work_root);
    validate_relative_path(&mut diagnostics, "snapshot_path", &input.snapshot_path);
    validate_stage_argv(
        &mut diagnostics,
        AeneasStageKind::Charon,
        &input.charon_argv,
        CHARON_COMMAND_NAME,
        REQUIRED_CHARON_BASE_ARGUMENTS,
    );
    validate_environment_allowlist(&mut diagnostics, &input.environment_allowlist);
    if input.limits.timeout_ms == 0 || input.limits.stdout_max_bytes == 0 || input.limits.stderr_max_bytes == 0 {
        diagnostics.push("reused Charon limits must all be positive".to_owned());
    }
    let snapshot_argument = input.snapshot_path.display().to_string();
    let destination_matches = input
        .charon_argv
        .windows(ARGV_PAIR_WIDTH)
        .any(|window| window[0] == "--dest-file" && window[1] == snapshot_argument);
    if !destination_matches {
        diagnostics.push("reused Charon argv destination does not match snapshot_path".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return failed_reused_charon_observation(input.charon_argv, diagnostics);
    }
    if input.work_root.exists() {
        return failed_reused_charon_observation(input.charon_argv, vec![format!(
            "reused Charon work root `{}` already exists; refusing prior-run reuse",
            input.work_root.display()
        )]);
    }
    if let Err(error) = fs::create_dir_all(&input.work_root) {
        return failed_reused_charon_observation(input.charon_argv, vec![format!(
            "create reused Charon work root `{}`: {error}",
            input.work_root.display()
        )]);
    }
    if let Err(error) = copy_directory_contents(&input.source_root, &input.work_root, true) {
        return failed_reused_charon_observation(input.charon_argv, vec![error]);
    }
    let stage = AeneasStagePlan {
        kind: AeneasStageKind::Charon,
        argv: sandboxed_provider_argv(
            &input.network_sandbox,
            &input.source_root,
            &input.work_root,
            &input.charon_executable,
            &input.charon_argv,
            None,
        ),
        working_dir: input.work_root.clone(),
    };
    let context = AeneasExecutionPlan {
        stages: Vec::new(),
        source_root: input.source_root,
        work_root: input.work_root.clone(),
        limits: input.limits,
        environment_allowlist: input.environment_allowlist,
        aeneas_lean_library: PathBuf::new(),
        theorem_source: String::new(),
    };
    let mut observation = execute_stage(&stage, &context);
    if observation.exit_code == Some(0) && !input.work_root.join(input.snapshot_path).is_file() {
        observation
            .diagnostics
            .push("reused Charon stage succeeded without producing the declared snapshot".to_owned());
    }
    observation
}

fn failed_reused_charon_observation(argv: Vec<String>, diagnostics: Vec<String>) -> AeneasStageObservation {
    let stderr = diagnostics.join("; ");
    let stderr_original_bytes = stderr.len();
    AeneasStageObservation {
        kind: AeneasStageKind::Charon,
        argv,
        exit_code: None,
        timed_out: false,
        stdout: String::new(),
        stdout_original_bytes: 0,
        stdout_truncated: false,
        stderr,
        stderr_original_bytes,
        stderr_truncated: false,
        diagnostics,
    }
}

fn prepare_work_root(
    source_root: &Path,
    work_root: &Path,
    aeneas_lean_library: &Path,
    theorem_source: &str,
) -> Result<(), String> {
    if !source_root.is_dir() {
        return Err(format!("Aeneas source root `{}` is not a directory", source_root.display()));
    }
    if !aeneas_lean_library.is_dir() {
        return Err(format!("Aeneas Lean library `{}` is not a directory", aeneas_lean_library.display()));
    }
    copy_directory_contents(source_root, work_root, true)?;
    let library_destination = work_root.join(AENEAS_LIBRARY_DIRECTORY);
    copy_directory_contents(aeneas_lean_library, &library_destination, false)?;
    let toolchain_source = aeneas_lean_library.join(LEAN_TOOLCHAIN_NAME);
    let toolchain_destination = work_root.join(LEAN_TOOLCHAIN_NAME);
    fs::copy(&toolchain_source, &toolchain_destination).map_err(|error| {
        format!(
            "copy Aeneas Lean toolchain `{}` to `{}`: {error}",
            toolchain_source.display(),
            toolchain_destination.display()
        )
    })?;
    let provider_root = aeneas_lean_library
        .parent()
        .ok_or_else(|| format!("Aeneas Lean library `{}` has no provider root", aeneas_lean_library.display()))?;
    let overrides_source = provider_root.join(LAKE_OVERRIDES_NAME);
    let lake_directory = work_root.join(LAKE_DIRECTORY_NAME);
    fs::create_dir_all(&lake_directory)
        .map_err(|error| format!("create Aeneas Lake directory `{}`: {error}", lake_directory.display()))?;
    let overrides_destination = lake_directory.join(LAKE_PACKAGE_OVERRIDES_NAME);
    fs::copy(&overrides_source, &overrides_destination).map_err(|error| {
        format!(
            "copy Aeneas Lake overrides `{}` to `{}`: {error}",
            overrides_source.display(),
            overrides_destination.display()
        )
    })?;
    let manifest_source = provider_root.join(LAKE_MANIFEST_NAME);
    let manifest_destination = work_root.join(LAKE_MANIFEST_NAME);
    fs::copy(&manifest_source, &manifest_destination).map_err(|error| {
        format!(
            "copy Aeneas Lake manifest `{}` to `{}`: {error}",
            manifest_source.display(),
            manifest_destination.display()
        )
    })?;
    let lakefile = work_root.join(LAKEFILE_NAME);
    let lakefile_text = format!(
        r#"import Lake
open Lake DSL
package octet_aeneas_fixture
require aeneas from "./{AENEAS_LIBRARY_DIRECTORY}"
@[default_target]
lean_lib OctetAeneasFixture where
  roots := #[`Llbc, `Theorem]
"#
    );
    fs::write(&lakefile, lakefile_text)
        .map_err(|error| format!("write generated Aeneas Lake package `{}`: {error}", lakefile.display()))?;
    // Lean builds the fixed `Theorem` root, so it must hold exactly the preflighted, approved
    // theorem bytes whatever `proof_paths` names; a source-root `Theorem.lean` is never checked.
    let theorem_module = work_root.join(LEAN_THEOREM_MODULE_FILE);
    fs::write(&theorem_module, theorem_source)
        .map_err(|error| format!("write approved Aeneas theorem `{}`: {error}", theorem_module.display()))
}

fn copy_directory_contents(source: &Path, destination: &Path, skip_build_directories: bool) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("create copied Aeneas directory `{}`: {error}", destination.display()))?;
    let entries =
        fs::read_dir(source).map_err(|error| format!("read Aeneas directory `{}`: {error}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read Aeneas directory entry in `{}`: {error}", source.display()))?;
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("inspect Aeneas directory entry `{}`: {error}", entry.path().display()))?;
        if skip_build_directories && file_type.is_dir() && should_skip_source_directory(&name_text) {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(&name);
        if file_type.is_dir() {
            copy_directory_contents(&source_path, &destination_path, skip_build_directories)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!("copy Aeneas file `{}` to `{}`: {error}", source_path.display(), destination_path.display())
            })?;
        } else {
            return Err(format!("Aeneas source contains unsupported filesystem entry `{}`", source_path.display()));
        }
    }
    Ok(())
}

fn should_skip_source_directory(name: &str) -> bool {
    matches!(name, GIT_DIRECTORY_NAME | TARGET_DIRECTORY_NAME | LAKE_DIRECTORY_NAME)
}

/// Inspect one successful stage's outputs in the work root; returned diagnostics stop the run.
pub(crate) type AeneasStageGate<'a> = dyn FnMut(AeneasStageKind, &Path) -> Vec<String> + 'a;

/// Execute the validated plan in a fresh work root with a denied network namespace.
///
/// After each stage exits cleanly, `gate` inspects its outputs before the next stage runs. The
/// work root is removed afterwards on success and on failure so a rerun of the same rail starts
/// fresh; an already-existing root belongs to another run and is never reused or removed.
pub(crate) fn execute_aeneas_plan(plan: &AeneasExecutionPlan, gate: &mut AeneasStageGate<'_>) -> AeneasExecutionReport {
    let mut report = AeneasExecutionReport {
        observations: Vec::new(),
        diagnostics: Vec::new(),
    };
    if plan.work_root.exists() {
        report.diagnostics.push(format!(
            "Aeneas work root `{}` already exists; refusing to reuse a concurrent or interrupted run",
            plan.work_root.display()
        ));
        return report;
    }
    if let Err(error) = fs::create_dir_all(&plan.work_root) {
        report
            .diagnostics
            .push(format!("create isolated Aeneas work root `{}`: {error}", plan.work_root.display()));
        return report;
    }
    execute_stages_in_work_root(plan, gate, &mut report);
    if let Err(error) = fs::remove_dir_all(&plan.work_root) {
        report.diagnostics.push(format!("remove Aeneas work root `{}`: {error}", plan.work_root.display()));
    }
    report.diagnostics.extend(validate_aeneas_execution_report(&report));
    report.diagnostics.sort();
    report.diagnostics.dedup();
    report
}

fn execute_stages_in_work_root(
    plan: &AeneasExecutionPlan,
    gate: &mut AeneasStageGate<'_>,
    report: &mut AeneasExecutionReport,
) {
    if let Err(error) =
        prepare_work_root(&plan.source_root, &plan.work_root, &plan.aeneas_lean_library, &plan.theorem_source)
    {
        report.diagnostics.push(error);
        return;
    }
    for stage in &plan.stages {
        let mut observation = execute_stage(stage, plan);
        let exited_cleanly = !observation.timed_out
            && !observation.stdout_truncated
            && !observation.stderr_truncated
            && observation.exit_code == Some(0)
            && observation.diagnostics.is_empty();
        if exited_cleanly {
            observation.diagnostics.extend(gate(stage.kind, &plan.work_root));
        }
        let stop = !exited_cleanly || !observation.diagnostics.is_empty();
        report.observations.push(observation);
        if stop {
            break;
        }
    }
}

/// Validate captured stage observations without process, filesystem, or environment access.
pub(crate) fn validate_aeneas_execution_report(report: &AeneasExecutionReport) -> Vec<String> {
    let mut diagnostics = report.diagnostics.clone();
    if report.observations.len() != STAGE_COUNT {
        diagnostics.push(format!("Aeneas execution stopped before all {STAGE_COUNT} stages completed"));
    }
    for (index, observation) in report.observations.iter().enumerate() {
        if observation.kind as usize != index {
            diagnostics.push(format!(
                "Aeneas stage order mismatch at index {index}: observed `{}`",
                observation.kind.as_str()
            ));
        }
        if observation.exit_code != Some(0) {
            diagnostics.push(format!(
                "Aeneas `{}` stage exited unsuccessfully with {:?}",
                observation.kind.as_str(),
                observation.exit_code
            ));
        }
        if observation.timed_out {
            diagnostics.push(format!("Aeneas `{}` stage timed out", observation.kind.as_str()));
        }
        if observation.stdout_truncated || observation.stderr_truncated {
            diagnostics.push(format!("Aeneas `{}` stage exceeded a bounded log limit", observation.kind.as_str()));
        }
        diagnostics.extend(observation.diagnostics.iter().cloned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn execute_stage(stage: &AeneasStagePlan, plan: &AeneasExecutionPlan) -> AeneasStageObservation {
    let spawn_result = spawn_stage(stage, plan);
    match spawn_result {
        Ok(result) => result,
        Err(error) => AeneasStageObservation {
            kind: stage.kind,
            argv: stage.argv.clone(),
            exit_code: None,
            timed_out: false,
            stdout: String::new(),
            stdout_original_bytes: 0,
            stdout_truncated: false,
            stderr: error.clone(),
            stderr_original_bytes: error.len(),
            stderr_truncated: false,
            diagnostics: vec![format!("Aeneas `{}` stage could not run: {error}", stage.kind.as_str())],
        },
    }
}

fn provider_bin_directory(stage: &AeneasStagePlan) -> Option<PathBuf> {
    let separator_index = stage.argv.iter().position(|argument| argument == SANDBOX_ARGUMENT_SEPARATOR)?;
    let executable = stage.argv.get(separator_index + 1)?;
    Path::new(executable).parent().map(Path::to_path_buf)
}

fn prepend_provider_path(
    provider_bin_directory: Option<&Path>,
    inherited_path: Option<OsString>,
) -> Result<OsString, String> {
    let mut paths = Vec::new();
    if let Some(directory) = provider_bin_directory {
        paths.push(directory.to_path_buf());
    }
    if let Some(path) = inherited_path {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).map_err(|error| format!("construct provider PATH: {error}"))
}

/// Run one stage in its own process group under bounded capture.
///
/// The whole group is killed on timeout and after the direct child exits, and output-reader
/// teardown is bounded, so grandchildren that inherit the pipes cannot stall the shell.
fn spawn_stage(stage: &AeneasStagePlan, plan: &AeneasExecutionPlan) -> Result<AeneasStageObservation, String> {
    let executable =
        stage.argv.first().ok_or_else(|| format!("Aeneas `{}` stage has empty argv", stage.kind.as_str()))?;
    let provider_bin_directory = provider_bin_directory(stage);
    let mut environment = Vec::new();
    for name in &plan.environment_allowlist {
        let value = if name == HOME_ENVIRONMENT_NAME {
            Some(plan.work_root.clone().into_os_string())
        } else if name == MATHLIB_NO_CACHE_ENVIRONMENT_NAME {
            Some(OsString::from(MATHLIB_NO_CACHE_ENVIRONMENT_VALUE))
        } else if name == PATH_ENVIRONMENT_NAME {
            Some(prepend_provider_path(provider_bin_directory.as_deref(), env::var_os(name))?)
        } else {
            env::var_os(name)
        };
        if let Some(value) = value {
            environment.push((OsString::from(name), value));
        }
    }
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: PathBuf::from(executable),
            args: stage.argv.iter().skip(1).map(OsString::from).collect(),
            current_dir: stage.working_dir.clone(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment,
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: plan.limits.timeout_ms,
            stdin_max_bytes: PROCESS_STDIN_MAX_BYTES,
            stdout_max_bytes: plan.limits.stdout_max_bytes,
            stderr_max_bytes: plan.limits.stderr_max_bytes,
            poll_interval_ms: PROCESS_POLL_INTERVAL_MS.min(plan.limits.timeout_ms),
            teardown_timeout_ms: PROCESS_TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct Aeneas stage outcome policy: {error:?}"))?,
    })
    .map_err(|error| error.to_string())?;
    Ok(AeneasStageObservation {
        kind: stage.kind,
        argv: stage.argv.clone(),
        exit_code: output.exit_code,
        timed_out: output.completion == bounded_exec::Completion::TimedOut,
        stdout: String::from_utf8_lossy(&output.stdout.bytes).into_owned(),
        stdout_original_bytes: output.stdout.observed_bytes,
        stdout_truncated: output.stdout.truncated,
        stderr: String::from_utf8_lossy(&output.stderr.bytes).into_owned(),
        stderr_original_bytes: output.stderr.observed_bytes,
        stderr_truncated: output.stderr.truncated,
        diagnostics: Vec::new(),
    })
}

/// Explicit absolute provider paths supplied by the shell-owned configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AeneasProviderPaths {
    pub(crate) charon: PathBuf,
    pub(crate) aeneas: PathBuf,
    pub(crate) lean: PathBuf,
    pub(crate) aeneas_lean_library: PathBuf,
    pub(crate) network_sandbox: PathBuf,
}

/// Execute an Aeneas rail using explicit provider executable paths supplied by the shell.
pub(crate) fn execute_aeneas_from_environment(rail: &ProofRail, source_root: &Path) -> AeneasCommandSummary {
    let executable_paths = [
        (CHARON_EXECUTABLE_ENV, "Charon"),
        (AENEAS_EXECUTABLE_ENV, "Aeneas"),
        (LEAN_EXECUTABLE_ENV, "Lean"),
        (AENEAS_LEAN_LIBRARY_ENV, "Aeneas Lean library"),
        (NETWORK_SANDBOX_ENV, "network sandbox"),
    ];
    let mut configured = Vec::new();
    for (name, label) in executable_paths {
        match env::var_os(name) {
            Some(value) => configured.push(PathBuf::from(value)),
            None => return failed_summary(format!("missing pinned Aeneas {label} executable in {name}")),
        }
    }
    let [charon, aeneas, lean, aeneas_lean_library, network_sandbox] =
        <[PathBuf; 5]>::try_from(configured).expect("every provider path is configured");
    let provider = AeneasProviderPaths {
        charon,
        aeneas,
        lean,
        aeneas_lean_library,
        network_sandbox,
    };
    execute_aeneas_with_provider(rail, source_root, &provider)
}

/// Execute an Aeneas rail with already-resolved provider paths.
// The shell resolves inputs, runs the three stages, and folds their observations into one
// summary; each step feeds the next.
#[allow(clippy::too_many_lines)]
pub(crate) fn execute_aeneas_with_provider(
    rail: &ProofRail,
    source_root: &Path,
    provider: &AeneasProviderPaths,
) -> AeneasCommandSummary {
    let profile = match serde_json::from_str::<AeneasProfile>(AENEAS_PROFILE_JSON) {
        Ok(profile) => profile,
        Err(error) => return failed_summary(format!("embedded Aeneas profile is invalid: {error}")),
    };
    let source_root = match source_root.canonicalize() {
        Ok(path) => path,
        Err(error) => return failed_summary(format!("canonicalize Aeneas source root: {error}")),
    };
    let work_root = deterministic_work_root(&source_root, rail);
    let source_path = rail
        .source_paths
        .get(SOURCE_PATH_INDEX)
        .map_or_else(|| PathBuf::from(DEFAULT_SOURCE_PATH), PathBuf::from);
    let theorem_path = rail
        .proof_paths
        .get(THEOREM_PATH_INDEX)
        .map_or_else(|| PathBuf::from(DEFAULT_THEOREM_PATH), PathBuf::from);
    let mut planning_diagnostics = Vec::new();
    validate_relative_path(&mut planning_diagnostics, "theorem_path", &theorem_path);
    if !rail.environment_allowlist.is_empty()
        && sorted_names(&rail.environment_allowlist) != sorted_names(&profile.environment_allowlist)
    {
        planning_diagnostics.push(format!(
            "{AENEAS_LEAN_BACKEND} rail environment_allowlist must match the reviewed provider profile allowlist"
        ));
    }
    if !planning_diagnostics.is_empty() {
        return failed_summary(planning_diagnostics.join("; "));
    }
    let effective_policy = AeneasEffectivePolicy {
        limits: AeneasExecutionLimits {
            timeout_ms: configured_or_profile(rail.timeout_ms, profile.limits.timeout_ms),
            stdout_max_bytes: configured_or_profile(rail.stdout_max_bytes, profile.limits.stdout_max_bytes),
            stderr_max_bytes: configured_or_profile(rail.stderr_max_bytes, profile.limits.stderr_max_bytes),
        },
        environment_allowlist: profile.environment_allowlist.clone(),
    };
    let theorem_source = match fs::read_to_string(source_root.join(&theorem_path)) {
        Ok(source) => source,
        Err(error) => return failed_summary(format!("read Aeneas theorem source: {error}")),
    };
    if let Err(diagnostic) = validate_aeneas_theorem_preflight(rail, &theorem_source) {
        return failed_summary(diagnostic);
    }
    let stage_argvs = vec![
        profile.stage_commands.charon_argv.clone(),
        profile.stage_commands.aeneas_argv.clone(),
        profile.stage_commands.lean_argv.clone(),
    ];
    let input = AeneasExecutionPlanInput {
        charon_argv: profile.stage_commands.charon_argv,
        aeneas_argv: profile.stage_commands.aeneas_argv,
        lean_argv: profile.stage_commands.lean_argv,
        charon_executable: provider.charon.clone(),
        aeneas_executable: provider.aeneas.clone(),
        lean_executable: provider.lean.clone(),
        aeneas_lean_library: provider.aeneas_lean_library.clone(),
        network_sandbox: provider.network_sandbox.clone(),
        source_root,
        work_root,
        source_path,
        llbc_path: PathBuf::from(LLBC_PATH),
        theorem_path: theorem_path.clone(),
        theorem_source,
        limits: effective_policy.limits.clone(),
        environment_allowlist: effective_policy.environment_allowlist.clone(),
    };
    let plan = match build_aeneas_execution_plan(input) {
        Ok(plan) => plan,
        Err(diagnostics) => return failed_summary(diagnostics.join("; ")),
    };
    let mut gates = AeneasStageGates {
        reviewed_extraction_policy: &profile.extraction_policy,
        assumption_policy: &profile.assumption_policy,
        theorem_source: &plan.theorem_source,
        artifacts: Vec::new(),
    };
    let report = execute_aeneas_plan(&plan, &mut |kind, work_root| gates.inspect(kind, work_root));
    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut stdout_original_bytes = 0;
    let mut stderr_original_bytes = 0;
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;
    let mut timed_out = false;
    let mut command_argv = Vec::new();
    for observation in &report.observations {
        let _ = write!(stdout, "[{}]\n{}", observation.kind.as_str(), observation.stdout);
        let _ = write!(stderr, "[{}]\n{}", observation.kind.as_str(), observation.stderr);
        stdout_original_bytes += observation.stdout_original_bytes;
        stderr_original_bytes += observation.stderr_original_bytes;
        stdout_truncated |= observation.stdout_truncated;
        stderr_truncated |= observation.stderr_truncated;
        timed_out |= observation.timed_out;
        command_argv.extend(observation.argv.iter().cloned());
    }
    let diagnostics = validate_aeneas_execution_report(&report);
    let command_succeeded = diagnostics.is_empty();
    if command_succeeded {
        let _ = writeln!(stdout, "{AENEAS_PASS_SIGNAL} {}", theorem_path.display());
    }
    let artifacts = if command_succeeded { gates.artifacts } else { Vec::new() };
    AeneasCommandSummary {
        command_succeeded,
        timed_out,
        stdout_text: stdout,
        stdout_original_bytes,
        stdout_truncated,
        stderr_text: stderr,
        stderr_original_bytes,
        stderr_truncated,
        command_blake3: command_blake3(&command_argv),
        diagnostics,
        stage_argvs,
        artifacts,
        provider_profile: AENEAS_PROFILE_JSON.as_bytes().to_vec(),
        effective_policy: Some(effective_policy),
    }
}

fn sorted_names(names: &[String]) -> Vec<&str> {
    let mut sorted = names.iter().map(String::as_str).collect::<Vec<_>>();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

fn configured_or_profile<T: Copy + Default + PartialEq>(configured: T, profile: T) -> T {
    if configured == T::default() {
        profile
    } else {
        configured
    }
}

/// Per-stage output checks that run before the next provider stage may start.
struct AeneasStageGates<'a> {
    reviewed_extraction_policy: &'a CharonExtractionPolicy,
    assumption_policy: &'a AeneasAssumptionPolicy,
    theorem_source: &'a str,
    artifacts: Vec<AeneasArtifact>,
}

impl AeneasStageGates<'_> {
    fn inspect(&mut self, kind: AeneasStageKind, work_root: &Path) -> Vec<String> {
        let collected = match kind {
            AeneasStageKind::Charon => self.collect(work_root, &[LLBC_PATH]),
            AeneasStageKind::Aeneas => {
                self.collect(work_root, &[TRANSLATION_MANIFEST_ARTIFACT, GENERATED_LEAN_ARTIFACT])
            }
            AeneasStageKind::Lean => self.collect(work_root, &[PACKAGE_LOCK_ARTIFACT, LEAN_TOOLCHAIN_ARTIFACT]),
        };
        if let Err(diagnostic) = collected {
            return vec![diagnostic];
        }
        match kind {
            AeneasStageKind::Charon => self.check_extraction(),
            AeneasStageKind::Aeneas => self.check_translation_and_trust(),
            AeneasStageKind::Lean => Vec::new(),
        }
    }

    fn collect(&mut self, work_root: &Path, names: &[&str]) -> Result<(), String> {
        for name in names {
            let path = work_root.join(name);
            let contents =
                fs::read(&path).map_err(|error| format!("read Aeneas artifact `{}`: {error}", path.display()))?;
            self.artifacts.push(AeneasArtifact {
                locator: (*name).to_owned(),
                contents,
            });
        }
        Ok(())
    }

    fn text(&self, locator: &str) -> Result<&str, String> {
        let artifact = self
            .artifacts
            .iter()
            .find(|artifact| artifact.locator == locator)
            .ok_or_else(|| format!("Aeneas artifact `{locator}` was not collected"))?;
        std::str::from_utf8(&artifact.contents)
            .map_err(|error| format!("Aeneas artifact `{locator}` is not UTF-8: {error}"))
    }

    fn check_extraction(&self) -> Vec<String> {
        let observed =
            match self.text(LLBC_PATH).map_err(|error| vec![error]).and_then(observe_charon_extraction_policy) {
                Ok(observed) => observed,
                Err(diagnostics) => return diagnostics,
            };
        validate_charon_extraction_contract(&observed, self.reviewed_extraction_policy)
    }

    fn check_translation_and_trust(&self) -> Vec<String> {
        let texts =
            (self.text(LLBC_PATH), self.text(TRANSLATION_MANIFEST_ARTIFACT), self.text(GENERATED_LEAN_ARTIFACT));
        let (llbc, translation, generated) = match texts {
            (Ok(llbc), Ok(translation), Ok(generated)) => (llbc, translation, generated),
            (llbc, translation, generated) => {
                return [llbc.err(), translation.err(), generated.err()].into_iter().flatten().collect();
            }
        };
        let mut diagnostics = validate_observed_aeneas_translation(llbc, translation, &[MeasuredLeanFile {
            locator: GENERATED_LEAN_ARTIFACT,
            contents: generated.as_bytes(),
        }]);
        let closure = observe_aeneas_trust_closure(self.theorem_source, generated, self.assumption_policy);
        if !closure.undeclared_inputs.is_empty() {
            diagnostics
                .push(format!("Aeneas trust closure has undeclared inputs: {}", closure.undeclared_inputs.join(", ")));
        }
        if !closure.declared_admitted_declarations.is_empty() {
            diagnostics.push(format!(
                "Aeneas trust closure has admitted declarations: {}",
                closure.declared_admitted_declarations.join(", ")
            ));
        }
        diagnostics
    }
}

fn validate_aeneas_theorem_preflight(rail: &ProofRail, theorem_source: &str) -> Result<AeneasTheoremAdequacy, String> {
    let adequacy = assess_aeneas_theorem_source(theorem_source);
    if let Some(diagnostic) = theorem_adequacy_diagnostic(&adequacy) {
        return Err(diagnostic);
    }
    let observed_statement_blake3 = crate::evidence::artifact_blake3_hex(theorem_source.as_bytes());
    validate_aeneas_theorem_statement_approval(&rail.approved_theorem_statement_blake3, &observed_statement_blake3)?;
    Ok(adequacy)
}

fn theorem_adequacy_diagnostic(adequacy: &AeneasTheoremAdequacy) -> Option<String> {
    let decision_diagnostic = match adequacy.decision {
        AeneasTheoremAdequacyDecision::NonVacuous => None,
        AeneasTheoremAdequacyDecision::Vacuous => Some("Aeneas theorem adequacy rejected vacuous theorem".to_owned()),
        AeneasTheoremAdequacyDecision::Rejected => {
            Some("Aeneas theorem adequacy rejected malformed theorem".to_owned())
        }
    };
    if let Some(diagnostic) = decision_diagnostic {
        return Some(diagnostic);
    }
    if adequacy.forbidden_declarations.is_empty() {
        None
    } else {
        Some(format!(
            "Aeneas theorem adequacy rejected forbidden declarations: {}",
            adequacy.forbidden_declarations.join(", ")
        ))
    }
}

fn failed_summary(diagnostic: String) -> AeneasCommandSummary {
    AeneasCommandSummary {
        command_succeeded: false,
        timed_out: false,
        stdout_text: String::new(),
        stdout_original_bytes: 0,
        stdout_truncated: false,
        stderr_text: diagnostic.clone(),
        stderr_original_bytes: diagnostic.len(),
        stderr_truncated: false,
        command_blake3: command_blake3(&[]),
        diagnostics: vec![diagnostic],
        stage_argvs: Vec::new(),
        artifacts: Vec::new(),
        provider_profile: AENEAS_PROFILE_JSON.as_bytes().to_vec(),
        effective_policy: None,
    }
}

fn deterministic_work_root(source_root: &Path, rail: &ProofRail) -> PathBuf {
    let identity = format!("{}\n{}\n{}", source_root.display(), rail.backend, rail.command_argv.join("\u{1f}"));
    let digest = blake3::hash(identity.as_bytes()).to_hex().to_string();
    env::temp_dir().join("octet-aeneas").join(digest)
}

fn command_blake3(argv: &[String]) -> String {
    let encoded = serde_json::to_vec(argv).expect("Aeneas argv must serialize");
    blake3::hash(&encoded).to_hex().to_string()
}

/// Fake Aeneas provider whose stages replay the frozen real-fixture artifacts.
#[cfg(all(test, unix))]
pub(crate) mod fake_provider {
    use super::*;
    use crate::test_support::write_executable_script;

    pub(crate) const FROZEN_ARTIFACTS: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas"
    );
    pub(crate) const REAL_FIXTURE: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../standards/proof-rails/aeneas-lean/fixtures/valid/real_fixture");
    pub(crate) const THEOREM_ID: &str = "aeneas.fixture.safe_add";
    pub(crate) const TIMEOUT_MS: u64 = 30_000;
    pub(crate) const LOG_BYTES: usize = 65_536;

    /// Fake Charon body that replays the frozen real LLBC.
    pub(crate) fn replay_llbc() -> String {
        format!("cp '{FROZEN_ARTIFACTS}/llbc.llbc' llbc.llbc")
    }

    /// Fake Aeneas body that replays the frozen real translation manifest and generated Lean.
    pub(crate) fn replay_translation() -> String {
        format!(
            "cp '{FROZEN_ARTIFACTS}/translation.json' translation.json\ncp '{FROZEN_ARTIFACTS}/Llbc.lean' Llbc.lean"
        )
    }

    /// Create fake provider executables and the staged Lean library under `root`.
    pub(crate) fn provider(root: &Path, charon_body: &str, aeneas_body: &str, lean_body: &str) -> AeneasProviderPaths {
        let library_root = root.join(AENEAS_LIBRARY_DIRECTORY);
        fs::create_dir_all(&library_root).expect("Aeneas Lean library root");
        fs::write(library_root.join(LAKEFILE_NAME), "import Lake\n").expect("Aeneas library lakefile");
        fs::write(library_root.join(LEAN_TOOLCHAIN_NAME), "leanprover/lean4:v4.31.0\n")
            .expect("Aeneas library toolchain");
        fs::write(root.join(LAKE_OVERRIDES_NAME), "{\"schemaVersion\":\"1.2.0\",\"packages\":[]}")
            .expect("Lake override fixture");
        fs::write(
            root.join(LAKE_MANIFEST_NAME),
            "{\"version\":\"1.2.0\",\"fixedToolchain\":true,\"name\":\"fixture\",\"lakeDir\":\".lake\",\"packages\":[]}",
        )
        .expect("Lake manifest fixture");
        for (name, body) in [("charon", charon_body), ("aeneas", aeneas_body), ("lake", lean_body)] {
            write_executable_script(&root.join(name), &format!("#!/bin/sh\nset -e\n{body}\n"));
        }
        write_executable_script(
            &root.join("bwrap"),
            "#!/bin/sh\nwhile [ \"$1\" != \"--\" ]; do shift; done\nshift\nexec \"$@\"\n",
        );
        AeneasProviderPaths {
            charon: root.join("charon"),
            aeneas: root.join("aeneas"),
            lean: root.join("lake"),
            aeneas_lean_library: library_root,
            network_sandbox: root.join("bwrap"),
        }
    }

    /// Create a source root holding the real fixture crate and `theorem` at `theorem_path`.
    pub(crate) fn source_root(root: &Path, theorem_path: &str, theorem: &str) -> PathBuf {
        let source_root = root.join("source");
        fs::create_dir_all(source_root.join("src")).expect("fixture source directory");
        for name in ["Cargo.toml", "src/lib.rs"] {
            fs::copy(Path::new(REAL_FIXTURE).join(name), source_root.join(name)).expect("copy real fixture source");
        }
        let theorem_file = source_root.join(theorem_path);
        fs::create_dir_all(theorem_file.parent().expect("theorem parent")).expect("theorem directory");
        fs::write(theorem_file, theorem).expect("fixture theorem");
        source_root
    }

    /// The frozen real-fixture theorem source.
    pub(crate) fn real_theorem() -> String {
        fs::read_to_string(Path::new(REAL_FIXTURE).join(DEFAULT_THEOREM_PATH)).expect("real fixture theorem")
    }

    /// An `aeneas-lean` rail approving `theorem` at `theorem_path`.
    pub(crate) fn rail(theorem_path: &str, theorem: &str) -> ProofRail {
        ProofRail {
            backend: AENEAS_LEAN_BACKEND.to_owned(),
            source_paths: vec!["src/lib.rs".to_owned()],
            proof_paths: vec![theorem_path.to_owned()],
            theorem_id: THEOREM_ID.to_owned(),
            approved_theorem_statement_blake3: crate::evidence::artifact_blake3_hex(theorem.as_bytes()),
            trust_boundaries: vec!["fake Aeneas rail proves only its named theorem".to_owned()],
            timeout_ms: TIMEOUT_MS,
            stdout_max_bytes: LOG_BYTES,
            stderr_max_bytes: LOG_BYTES,
            environment_allowlist: [
                "PATH",
                "CHARON_TOOLCHAIN_IS_IN_PATH",
                "LD_LIBRARY_PATH",
                "HOME",
                "MATHLIB_NO_CACHE_ON_UPDATE",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            ..ProofRail::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::time::Instant;

    use super::*;

    const TEST_TIMEOUT_MS: u64 = 1_000;
    const TEST_SHORT_TIMEOUT_MS: u64 = 100;
    const TEST_LOG_BYTES: usize = 1_024;
    const TEST_OVERFLOW_LIMIT_BYTES: usize = 4;
    const TEST_TRANSLATION_FAILURE_EXIT_CODE: i32 = 7;
    const TEST_THEOREM_FAILURE_EXIT_CODE: i32 = 9;
    const TEST_SLEEP_COMMAND: &str = "sleep 1";
    const TEST_SOURCE_PATH: &str = "src/verified/addition.rs";
    const TEST_LLBC_PATH: &str = "llbc.json";
    const TEST_THEOREM_PATH: &str = "Theorem.lean";
    const TEST_ENVIRONMENT: &str = "PATH";
    const LIVE_PROVIDER_TIMEOUT_MS: u64 = 900_000;
    const LIVE_PROVIDER_LOG_BYTES: usize = 1_048_576;
    const LIVE_FIXTURE_SOURCE: &str = "src/lib.rs";
    const LIVE_FIXTURE_THEOREM: &str = "Theorem.lean";
    const TEST_TEARDOWN_DEADLINE_SECONDS: u64 = 10;
    const TEST_THEOREM_SOURCE: &str = "theorem approved : Nat = Nat := by rfl\n";

    fn no_stage_gates(_kind: AeneasStageKind, _work_root: &Path) -> Vec<String> {
        Vec::new()
    }

    fn valid_input(root: &Path) -> AeneasExecutionPlanInput {
        AeneasExecutionPlanInput {
            charon_argv: std::iter::once(CHARON_COMMAND_NAME.to_owned())
                .chain(REQUIRED_CHARON_ARGUMENTS.iter().map(|argument| (*argument).to_owned()))
                .collect(),
            aeneas_argv: std::iter::once(AENEAS_COMMAND_NAME.to_owned())
                .chain(REQUIRED_AENEAS_ARGUMENTS.iter().map(|argument| (*argument).to_owned()))
                .collect(),
            lean_argv: vec![LEAN_COMMAND_NAME.to_owned(), "build".to_owned()],
            charon_executable: root.join("charon"),
            aeneas_executable: root.join("aeneas"),
            lean_executable: root.join("lake"),
            aeneas_lean_library: root.join(AENEAS_LIBRARY_DIRECTORY),
            network_sandbox: root.join("bwrap"),
            source_root: root.join("source"),
            work_root: root.join("work"),
            source_path: PathBuf::from(TEST_SOURCE_PATH),
            llbc_path: PathBuf::from(TEST_LLBC_PATH),
            theorem_path: PathBuf::from(TEST_THEOREM_PATH),
            theorem_source: TEST_THEOREM_SOURCE.to_owned(),
            limits: AeneasExecutionLimits {
                timeout_ms: TEST_TIMEOUT_MS,
                stdout_max_bytes: TEST_LOG_BYTES,
                stderr_max_bytes: TEST_LOG_BYTES,
            },
            environment_allowlist: vec![
                TEST_ENVIRONMENT.to_owned(),
                HOME_ENVIRONMENT_NAME.to_owned(),
                MATHLIB_NO_CACHE_ENVIRONMENT_NAME.to_owned(),
            ],
        }
    }

    #[test]
    fn plan_is_argv_only_network_denied_and_stage_ordered() {
        let root = PathBuf::from("/tmp/octet-aeneas-plan");
        let plan = build_aeneas_execution_plan(valid_input(&root)).expect("valid plan");

        assert_eq!(plan.stages.len(), STAGE_COUNT);
        assert_eq!(plan.stages[0].kind, AeneasStageKind::Charon);
        assert_eq!(plan.stages[1].kind, AeneasStageKind::Aeneas);
        assert_eq!(plan.stages[2].kind, AeneasStageKind::Lean);
        for stage in &plan.stages {
            assert_eq!(stage.argv[1], DIE_WITH_PARENT_FLAG);
            assert_eq!(stage.argv[2], NETWORK_DENIAL_FLAG);
            assert_eq!(stage.argv[3], PID_NAMESPACE_FLAG);
            assert!(stage.argv.contains(&"--dev".to_owned()));
            assert!(stage.argv.contains(&DEV_MOUNT_PATH.to_owned()));
            assert!(stage.argv.contains(&SANDBOX_ARGUMENT_SEPARATOR.to_owned()));
        }
        assert_eq!(provider_bin_directory(&plan.stages[0]), Some(root.clone()));
        for argument in REQUIRED_AENEAS_ARGUMENTS {
            assert!(plan.stages[1].argv.contains(&(*argument).to_owned()));
        }
        for argument in REQUIRED_CHARON_ARGUMENTS {
            assert!(plan.stages[0].argv.contains(&(*argument).to_owned()));
        }
        assert!(plan.environment_allowlist.contains(&MATHLIB_NO_CACHE_ENVIRONMENT_NAME.to_owned()));
    }

    #[test]
    fn plan_rejects_unpinned_paths_shell_arguments_and_weak_limits() {
        let root = PathBuf::from("/tmp/octet-aeneas-plan");
        let mut input = valid_input(&root);
        input.aeneas_executable = PathBuf::from("aeneas");
        input.aeneas_argv.push("--bad;network".to_owned());
        input.lean_argv = vec![LEAN_COMMAND_NAME.to_owned()];
        input.environment_allowlist = vec!["PATH ".to_owned()];
        input.limits.timeout_ms = 0;

        let diagnostics = build_aeneas_execution_plan(input).expect_err("invalid plan must fail");

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("aeneas_executable")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("shell metacharacters")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("lean_argv")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("environment_allowlist")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("timeout_ms")));
    }

    #[cfg(unix)]
    #[test]
    fn shell_executes_all_stages_in_fresh_root_with_bounded_observations() {
        let temporary = tempfile::tempdir().expect("temporary test root");
        let plan = fixture_plan(
            temporary.path(),
            "printf '%s\\n' 'charon'; printf '%s\\n' \"$HOME\"",
            "printf '%s\\n' 'aeneas'",
            "printf 'lake:%s\\n' \"$MATHLIB_NO_CACHE_ON_UPDATE\"",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert_eq!(report.observations.len(), STAGE_COUNT);
        assert!(validate_aeneas_execution_report(&report).is_empty(), "report: {report:?}");
        assert!(report.observations[0].stdout.starts_with("charon\n"));
        assert!(report.observations[0].stdout.contains(&plan.work_root.display().to_string()));
        assert_eq!(report.observations[1].stdout.trim(), "aeneas");
        assert_eq!(report.observations[2].stdout.trim(), "lake:1");
        assert!(!plan.work_root.exists(), "work root survived a successful run");
    }

    #[cfg(unix)]
    #[test]
    fn shell_rejects_missing_aeneas_library_before_process_execution() {
        let temporary = tempfile::tempdir().expect("temporary test root");
        let plan = fixture_plan(
            temporary.path(),
            "printf '%s\\n' 'charon'",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        fs::remove_dir_all(&plan.aeneas_lean_library).expect("remove Aeneas library fixture");
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert!(report.observations.is_empty());
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("Aeneas Lean library")));
    }

    #[cfg(unix)]
    #[test]
    fn shell_rejects_missing_lake_overrides_before_process_execution() {
        let temporary = tempfile::tempdir().expect("temporary test root");
        let plan = fixture_plan(
            temporary.path(),
            "printf '%s\\n' 'charon'",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        fs::remove_file(temporary.path().join(LAKE_OVERRIDES_NAME)).expect("remove Lake overrides fixture");
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert!(report.observations.is_empty());
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("Lake overrides")));
    }

    #[cfg(unix)]
    #[test]
    fn shell_rejects_missing_lake_manifest_before_process_execution() {
        let temporary = tempfile::tempdir().expect("temporary test root");
        let plan = fixture_plan(
            temporary.path(),
            "printf '%s\\n' 'charon'",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        fs::remove_file(temporary.path().join(LAKE_MANIFEST_NAME)).expect("remove Lake manifest fixture");
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert!(report.observations.is_empty());
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("Lake manifest")));
    }

    #[cfg(unix)]
    #[test]
    fn shell_reports_spawn_failure_and_stops_before_later_stages() {
        let temporary = tempfile::tempdir().expect("temporary test root");
        let mut plan = fixture_plan(
            temporary.path(),
            "printf '%s\\n' 'charon'",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        plan.stages[0].argv[0] = temporary.path().join("missing-sandbox").display().to_string();
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert_eq!(report.observations.len(), 1);
        assert_eq!(report.observations[0].kind, AeneasStageKind::Charon);
        assert_eq!(report.observations[0].exit_code, None);
        assert!(!report.observations[0].diagnostics.is_empty(), "missing sandbox must be diagnosed: {report:?}");
    }

    #[cfg(unix)]
    #[test]
    fn shell_reports_translation_and_theorem_nonzero_results() {
        let translation_root = tempfile::tempdir().expect("translation test root");
        let translation_plan = fixture_plan(
            translation_root.path(),
            "printf '%s\\n' 'charon'",
            &format!("exit {TEST_TRANSLATION_FAILURE_EXIT_CODE}"),
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        let translation_report = execute_aeneas_plan(&translation_plan, &mut no_stage_gates);
        assert_eq!(translation_report.observations.len(), 2);
        assert!(translation_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("aeneas")));

        let theorem_root = tempfile::tempdir().expect("theorem test root");
        let theorem_plan = fixture_plan(
            theorem_root.path(),
            "printf '%s\\n' 'charon'",
            "printf '%s\\n' 'aeneas'",
            &format!("exit {TEST_THEOREM_FAILURE_EXIT_CODE}"),
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        let theorem_report = execute_aeneas_plan(&theorem_plan, &mut no_stage_gates);
        assert_eq!(theorem_report.observations.len(), STAGE_COUNT);
        assert!(theorem_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("lean")));
    }

    #[test]
    fn theorem_preflight_accepts_approved_source_and_rejects_mismatch_or_vacuity() {
        let theorem_source = "theorem approved : Nat = Nat := by rfl";
        let mut rail = ProofRail {
            approved_theorem_statement_blake3: crate::evidence::artifact_blake3_hex(theorem_source.as_bytes()),
            ..ProofRail::default()
        };
        let adequacy = validate_aeneas_theorem_preflight(&rail, theorem_source).expect("approved theorem preflight");
        assert_eq!(adequacy.decision, AeneasTheoremAdequacyDecision::NonVacuous);

        rail.approved_theorem_statement_blake3 = crate::evidence::artifact_blake3_hex(b"different theorem");
        let mismatch = validate_aeneas_theorem_preflight(&rail, theorem_source).expect_err("mismatched approval");
        assert!(mismatch.contains("mismatch"));

        let vacuous_source = "theorem vacuous : (True) := by trivial";
        rail.approved_theorem_statement_blake3 = crate::evidence::artifact_blake3_hex(vacuous_source.as_bytes());
        let vacuous = validate_aeneas_theorem_preflight(&rail, vacuous_source).expect_err("vacuous theorem");
        assert!(vacuous.contains("vacuous"));
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires the pinned Aeneas/Charon/Lean provider closure"]
    fn live_provider_executes_real_positive_fixture() {
        let source_root = live_fixture_root("valid/real_fixture");
        let rail = live_fixture_rail(LIVE_FIXTURE_THEOREM, &source_root);
        let summary = execute_aeneas_from_environment(&rail, &source_root);

        assert!(summary.command_succeeded, "real positive fixture failed: {summary:?}");
        assert!(summary.stdout_text.contains("[charon]"), "Charon stage was not observed: {summary:?}");
        assert!(summary.stdout_text.contains("[aeneas]"), "Aeneas stage was not observed: {summary:?}");
        assert!(summary.stdout_text.contains("[lean]"), "Lean stage was not observed: {summary:?}");
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires the pinned Aeneas/Charon/Lean provider closure"]
    fn live_provider_rejects_real_false_fixture() {
        let source_root = live_fixture_root("invalid/real_fixture_false");
        let rail = live_fixture_rail(LIVE_FIXTURE_THEOREM, &source_root);
        let summary = execute_aeneas_from_environment(&rail, &source_root);

        assert!(!summary.command_succeeded, "real false fixture unexpectedly passed: {summary:?}");
        assert!(!summary.timed_out, "real false fixture timed out instead of rejecting the theorem: {summary:?}");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("lean")));
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires the pinned Aeneas/Charon/Lean provider closure"]
    fn live_provider_rejects_real_vacuous_fixture() {
        let source_root = live_fixture_root("invalid/real_fixture_vacuous");
        let rail = live_fixture_rail(LIVE_FIXTURE_THEOREM, &source_root);
        let summary = execute_aeneas_from_environment(&rail, &source_root);

        assert!(!summary.command_succeeded, "real vacuous fixture unexpectedly passed: {summary:?}");
        assert!(!summary.timed_out, "real vacuous fixture timed out instead of rejecting: {summary:?}");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("vacuous")));
    }

    fn live_fixture_root(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../standards/proof-rails/aeneas-lean/fixtures")
            .join(relative)
            .canonicalize()
            .expect("live Aeneas fixture root")
    }

    fn live_fixture_rail(theorem_path: &str, source_root: &Path) -> ProofRail {
        let theorem_source = fs::read(source_root.join(theorem_path)).expect("live fixture theorem source");
        ProofRail {
            backend: AENEAS_LEAN_BACKEND.to_owned(),
            source_paths: vec![LIVE_FIXTURE_SOURCE.to_owned()],
            proof_paths: vec![theorem_path.to_owned()],
            approved_theorem_statement_blake3: crate::evidence::artifact_blake3_hex(&theorem_source),
            timeout_ms: LIVE_PROVIDER_TIMEOUT_MS,
            stdout_max_bytes: LIVE_PROVIDER_LOG_BYTES,
            stderr_max_bytes: LIVE_PROVIDER_LOG_BYTES,
            environment_allowlist: vec![
                TEST_ENVIRONMENT.to_owned(),
                "CHARON_TOOLCHAIN_IS_IN_PATH".to_owned(),
                "LD_LIBRARY_PATH".to_owned(),
                HOME_ENVIRONMENT_NAME.to_owned(),
                MATHLIB_NO_CACHE_ENVIRONMENT_NAME.to_owned(),
            ],
            ..ProofRail::default()
        }
    }

    #[cfg(unix)]
    #[test]
    fn shell_reports_timeout_and_bounded_log_overflow() {
        let timeout_root = tempfile::tempdir().expect("timeout test root");
        let timeout_plan = fixture_plan(
            timeout_root.path(),
            TEST_SLEEP_COMMAND,
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_SHORT_TIMEOUT_MS,
        );
        let timeout_report = execute_aeneas_plan(&timeout_plan, &mut no_stage_gates);
        assert_eq!(timeout_report.observations.len(), 1);
        assert!(timeout_report.observations[0].timed_out);
        assert!(timeout_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("timed out")));

        let overflow_root = tempfile::tempdir().expect("overflow test root");
        let overflow_plan = fixture_plan(
            overflow_root.path(),
            "printf '%s' '0123456789'",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_OVERFLOW_LIMIT_BYTES,
            TEST_LOG_BYTES,
            TEST_TIMEOUT_MS,
        );
        let overflow_report = execute_aeneas_plan(&overflow_plan, &mut no_stage_gates);
        assert_eq!(overflow_report.observations.len(), 1);
        assert!(overflow_report.observations[0].stdout_truncated);
        assert!(overflow_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("bounded log")));
    }

    #[cfg(unix)]
    fn fixture_plan(
        root: &Path,
        charon_body: &str,
        aeneas_body: &str,
        lean_body: &str,
        stdout_max_bytes: usize,
        stderr_max_bytes: usize,
        timeout_ms: u64,
    ) -> AeneasExecutionPlan {
        use crate::test_support::write_executable_script;

        let source_root = root.join("source");
        fs::create_dir(&source_root).expect("source root");
        let library_root = root.join(AENEAS_LIBRARY_DIRECTORY);
        fs::create_dir(&library_root).expect("Aeneas Lean library root");
        fs::write(library_root.join(LAKEFILE_NAME), "import Lake\n").expect("Aeneas library lakefile");
        fs::write(library_root.join(LEAN_TOOLCHAIN_NAME), "leanprover/lean4:v4.31.0\n")
            .expect("Aeneas library toolchain");
        fs::write(root.join(LAKE_OVERRIDES_NAME), "{\"schemaVersion\":\"1.2.0\",\"packages\":[]}")
            .expect("Lake override fixture");
        fs::write(
            root.join(LAKE_MANIFEST_NAME),
            "{\"version\":\"1.2.0\",\"fixedToolchain\":true,\"name\":\"fixture\",\"lakeDir\":\".lake\",\"packages\":[]}",
        )
        .expect("Lake manifest fixture");
        let scripts = [("charon", charon_body), ("aeneas", aeneas_body), ("lake", lean_body)];
        for (script_name, body) in scripts {
            write_executable_script(&root.join(script_name), &format!("#!/bin/sh\n{body}\n"));
        }
        write_executable_script(
            &root.join("bwrap"),
            "#!/bin/sh\nwhile [ \"$1\" != \"--\" ]; do shift; done\nshift\nexec \"$@\"\n",
        );
        let mut input = valid_input(root);
        input.limits = AeneasExecutionLimits {
            timeout_ms,
            stdout_max_bytes,
            stderr_max_bytes,
        };
        let plan = build_aeneas_execution_plan(input).expect("valid execution plan");
        assert!(plan.source_root.is_absolute());
        assert!(plan.work_root.is_absolute());
        plan
    }

    #[cfg(unix)]
    fn run_fake_rail(
        root: &Path,
        rail: &ProofRail,
        source_root: &Path,
        bodies: (&str, &str, &str),
    ) -> AeneasCommandSummary {
        let provider = fake_provider::provider(&root.join("provider"), bodies.0, bodies.1, bodies.2);
        execute_aeneas_with_provider(rail, source_root, &provider)
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[cfg(unix)]
    #[test]
    fn shell_rejects_undeclared_theorem_axiom_before_lean_checking() {
        let temporary = tempfile::tempdir().expect("axiom test root");
        let theorem = "import Llbc\n\naxiom cheat : real_fixture.safe_add 1#u32 2#u32 = .ok 4#u32\n\ntheorem cheat_used : real_fixture.safe_add 1#u32 2#u32 = .ok 4#u32 := cheat\n";
        let source_root = fake_provider::source_root(temporary.path(), DEFAULT_THEOREM_PATH, theorem);
        let rail = fake_provider::rail(DEFAULT_THEOREM_PATH, theorem);
        let summary = run_fake_rail(
            temporary.path(),
            &rail,
            &source_root,
            (&fake_provider::replay_llbc(), &fake_provider::replay_translation(), "exit 0"),
        );

        assert!(!summary.command_succeeded, "undeclared axiom reached formal proof: {summary:?}");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("axiom cheat")), "{summary:?}");
        assert!(!summary.stdout_text.contains("[lean]"), "Lean ran over an undeclared trust input: {summary:?}");
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[cfg(unix)]
    #[test]
    fn shell_rejects_weakened_charon_extraction_before_translation() {
        let temporary = tempfile::tempdir().expect("extraction test root");
        let theorem = fake_provider::real_theorem();
        let source_root = fake_provider::source_root(temporary.path(), DEFAULT_THEOREM_PATH, &theorem);
        let rail = fake_provider::rail(DEFAULT_THEOREM_PATH, &theorem);
        let charon = format!(
            "sed 's/\"skip_borrowck\":false/\"skip_borrowck\":true/' '{}/llbc.llbc' > llbc.llbc",
            fake_provider::FROZEN_ARTIFACTS
        );
        let summary = run_fake_rail(
            temporary.path(),
            &rail,
            &source_root,
            (&charon, &fake_provider::replay_translation(), "exit 0"),
        );

        assert!(!summary.command_succeeded, "borrow-check bypass reached formal proof: {summary:?}");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("skip_borrowck")), "{summary:?}");
        assert!(!summary.stdout_text.contains("[aeneas]"), "Aeneas ran over weakened LLBC: {summary:?}");
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[cfg(unix)]
    #[test]
    fn shell_rejects_stale_translation_manifest_before_lean_checking() {
        let temporary = tempfile::tempdir().expect("translation test root");
        let theorem = fake_provider::real_theorem();
        let source_root = fake_provider::source_root(temporary.path(), DEFAULT_THEOREM_PATH, &theorem);
        let rail = fake_provider::rail(DEFAULT_THEOREM_PATH, &theorem);
        let aeneas = format!(
            "sed 's/\"end_line\": 3/\"end_line\": 2/' '{root}/translation.json' > translation.json\ncp '{root}/Llbc.lean' Llbc.lean",
            root = fake_provider::FROZEN_ARTIFACTS
        );
        let summary =
            run_fake_rail(temporary.path(), &rail, &source_root, (&fake_provider::replay_llbc(), &aeneas, "exit 0"));

        assert!(!summary.command_succeeded, "stale translation manifest reached formal proof: {summary:?}");
        assert!(summary.diagnostics.iter().any(|diagnostic| diagnostic.contains("rust_span")), "{summary:?}");
        assert!(!summary.stdout_text.contains("[lean]"), "Lean ran over a stale manifest: {summary:?}");
    }

    // r[verify octet.aeneas_lean_proof_rail.safe_execution]
    #[cfg(unix)]
    #[test]
    fn shell_reruns_the_same_rail_and_removes_its_work_root() {
        let temporary = tempfile::tempdir().expect("rerun test root");
        let theorem = fake_provider::real_theorem();
        let source_root = fake_provider::source_root(temporary.path(), DEFAULT_THEOREM_PATH, &theorem);
        let rail = fake_provider::rail(DEFAULT_THEOREM_PATH, &theorem);
        let bodies = (fake_provider::replay_llbc(), fake_provider::replay_translation(), "exit 0".to_owned());
        let bodies = (bodies.0.as_str(), bodies.1.as_str(), bodies.2.as_str());
        let first = run_fake_rail(temporary.path(), &rail, &source_root, bodies);
        let second = run_fake_rail(&temporary.path().join("again"), &rail, &source_root, bodies);

        assert!(first.command_succeeded, "first run failed: {first:?}");
        assert!(second.command_succeeded, "rerun of the same rail failed: {second:?}");
        let work_root = deterministic_work_root(&source_root.canonicalize().expect("source root"), &rail);
        assert!(!work_root.exists(), "work root `{}` survived the run", work_root.display());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_gate]
    #[cfg(unix)]
    #[test]
    fn shell_lean_checks_the_preflighted_proof_path() {
        let temporary = tempfile::tempdir().expect("proof path test root");
        let theorem = fake_provider::real_theorem();
        let spec_path = "proofs/Spec.lean";
        let source_root = fake_provider::source_root(temporary.path(), spec_path, &theorem);
        fs::write(source_root.join(DEFAULT_THEOREM_PATH), "theorem unrelated : True := trivial\n")
            .expect("unrelated root theorem");
        let rail = fake_provider::rail(spec_path, &theorem);
        let summary = run_fake_rail(
            temporary.path(),
            &rail,
            &source_root,
            (
                &fake_provider::replay_llbc(),
                &fake_provider::replay_translation(),
                &format!("cmp -s {DEFAULT_THEOREM_PATH} {spec_path}"),
            ),
        );

        assert!(summary.command_succeeded, "Lean did not check the preflighted proof path: {summary:?}");
    }

    // r[verify octet.aeneas_lean_proof_rail.safe_execution]
    #[cfg(unix)]
    #[test]
    fn shell_timeout_tears_down_grandchildren_holding_output_pipes() {
        let temporary = tempfile::tempdir().expect("grandchild timeout root");
        let plan = fixture_plan(
            temporary.path(),
            "(sleep 30) &\nsleep 30",
            "printf '%s\\n' 'aeneas'",
            "printf '%s\\n' 'lake'",
            TEST_LOG_BYTES,
            TEST_LOG_BYTES,
            TEST_SHORT_TIMEOUT_MS,
        );
        let started = Instant::now();
        let report = execute_aeneas_plan(&plan, &mut no_stage_gates);

        assert!(
            started.elapsed() < Duration::from_secs(TEST_TEARDOWN_DEADLINE_SECONDS),
            "timed-out stage waited for a grandchild: {:?}",
            started.elapsed()
        );
        assert_eq!(report.observations.len(), 1);
        assert!(report.observations[0].timed_out);
    }

    // r[verify octet.aeneas_lean_proof_rail.safe_execution]
    #[cfg(unix)]
    #[test]
    fn shell_rejects_rail_environment_allowlist_that_differs_from_the_profile() {
        let temporary = tempfile::tempdir().expect("allowlist test root");
        let theorem = fake_provider::real_theorem();
        let source_root = fake_provider::source_root(temporary.path(), DEFAULT_THEOREM_PATH, &theorem);
        let mut rail = fake_provider::rail(DEFAULT_THEOREM_PATH, &theorem);
        rail.environment_allowlist.push("AWS_SECRET_ACCESS_KEY".to_owned());
        let summary = run_fake_rail(
            temporary.path(),
            &rail,
            &source_root,
            (&fake_provider::replay_llbc(), &fake_provider::replay_translation(), "exit 0"),
        );

        assert!(!summary.command_succeeded, "widened allowlist was accepted: {summary:?}");
        assert!(summary.stdout_text.is_empty(), "a stage ran with a widened allowlist: {summary:?}");
    }
}
