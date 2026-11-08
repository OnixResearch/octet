use std::io::Read as _;

use super::*;

const CORPUS_DIRECTORY: &str = "fuzz/corpus/octet-profile-intake";
const ARTIFACT_DIRECTORY: &str = "fuzz/artifacts/octet-profile-intake";
const COMPILER_PROFILE: &str = "nix:fenix-nightly-2026-03-21-x86_64-unknown-linux-gnu";
const CRASH_PREFIX: &str = "crash-";
const MINIMIZED_PREFIX: &str = "minimized-";
const INTEGRATION_FAILURE_EXIT: i32 = 2;
const ACCEPTED_EXIT_CODE: i32 = 0;
const CARGO_PROGRAM: &str = "cargo";
const PATH_ENVIRONMENT: &str = "PATH";
const ENVIRONMENT_ALLOWLIST: [&str; 12] = [
    PATH_ENVIRONMENT,
    "HOME",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "RUSTUP_TOOLCHAIN",
    "RUSTC",
    "LD_LIBRARY_PATH",
    "TMPDIR",
    "CC",
    "CXX",
    "SSL_CERT_FILE",
    "NIX_SSL_CERT_FILE",
];
const STDIN_MAX_BYTES: usize = 1;
const LOG_STREAM_COUNT: usize = 2;
const BUILD_ALLOWANCE_SECONDS: u64 = 840;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const POLL_INTERVAL_MS: u64 = 25;
const TEARDOWN_TIMEOUT_MS: u64 = 5_000;

struct Snapshot {
    entries: Vec<ArtifactIdentity>,
    total_bytes: u64,
}

impl Snapshot {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_ARTIFACT_COUNT),
            total_bytes: 0,
        }
    }

    fn push(&mut self, entry: ArtifactIdentity) -> Result<(), String> {
        if self.entries.len() >= MAX_ARTIFACT_COUNT {
            return Err("cargo-fuzz snapshot exceeds artifact count limit".to_owned());
        }
        self.total_bytes = self
            .total_bytes
            .checked_add(entry.length)
            .ok_or_else(|| "cargo-fuzz snapshot byte count overflow".to_owned())?;
        if self.total_bytes > MAX_ARTIFACT_BYTES {
            return Err("cargo-fuzz snapshot exceeds artifact byte limit".to_owned());
        }
        self.entries.push(entry);
        Ok(())
    }

    fn finish(mut self) -> Vec<ArtifactIdentity> {
        self.entries.sort();
        self.entries
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTools {
    pub cargo: std::path::PathBuf,
    pub environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

pub fn run(workspace_root: &std::path::Path) -> Result<(String, i32), String> {
    run_with(workspace_root, production_tools()?, run_limits()?)
}

/// Resolves `cargo` from the ambient `PATH` and keeps only the allowlisted variables that cargo,
/// cargo-fuzz, rustup, and the libFuzzer C++ build use to locate the selected toolchain.
pub fn production_tools() -> Result<RuntimeTools, String> {
    let environment = ENVIRONMENT_ALLOWLIST
        .iter()
        .filter_map(|name| std::env::var_os(name).map(|value| (std::ffi::OsString::from(name), value)))
        .collect::<Vec<_>>();
    let search_path = environment
        .iter()
        .find(|(name, _)| name == PATH_ENVIRONMENT)
        .map(|(_, value)| value)
        .ok_or_else(|| "cargo-fuzz runtime is missing PATH".to_owned())?;
    let cargo = std::env::split_paths(search_path)
        .map(|directory| directory.join(CARGO_PROGRAM))
        .find(|candidate| candidate.is_absolute() && candidate.is_file())
        .ok_or_else(|| "cargo-fuzz runtime cannot find an absolute cargo on PATH".to_owned())?;
    Ok(RuntimeTools { cargo, environment })
}

/// Bounds the run by the receipt budget: the libFuzzer time budget plus a build allowance, and
/// the receipt log limit split between stdout and stderr.
pub fn run_limits() -> Result<bounded_exec::ExecutionLimits, String> {
    let stream_limit = usize::try_from(MAX_RECEIPT_BYTES)
        .ok()
        .and_then(|limit| limit.checked_div(LOG_STREAM_COUNT))
        .filter(|limit| *limit > 0)
        .ok_or_else(|| "cargo-fuzz log limit cannot be divided between output streams".to_owned())?;
    let timeout_ms = MAX_TOTAL_TIME_SECONDS
        .checked_add(BUILD_ALLOWANCE_SECONDS)
        .and_then(|seconds| seconds.checked_mul(MILLISECONDS_PER_SECOND))
        .ok_or_else(|| "cargo-fuzz timeout overflowed".to_owned())?;
    Ok(bounded_exec::ExecutionLimits {
        timeout_ms,
        stdin_max_bytes: STDIN_MAX_BYTES,
        stdout_max_bytes: stream_limit,
        stderr_max_bytes: stream_limit,
        poll_interval_ms: POLL_INTERVAL_MS,
        teardown_timeout_ms: TEARDOWN_TIMEOUT_MS,
    })
}

pub fn run_with(
    workspace_root: &std::path::Path,
    tools: RuntimeTools,
    limits: bounded_exec::ExecutionLimits,
) -> Result<(String, i32), String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize cargo-fuzz workspace {}: {error}", workspace_root.display()))?;
    let initial_corpus = snapshot(&workspace_root, CORPUS_DIRECTORY)?;
    let command_argv = command_argv();
    let args = command_argv
        .get(1..)
        .ok_or_else(|| "cargo-fuzz command has no subcommand arguments".to_owned())?
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: tools.cargo,
            args,
            current_dir: workspace_root.clone(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: tools.environment,
            input: bounded_exec::Input::Null,
        },
        limits,
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct cargo-fuzz outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded cargo-fuzz command: {error}"))?;
    if !matches!(output.disposition, bounded_exec::Disposition::Succeeded | bounded_exec::Disposition::ExitFailed) {
        return Err(format!("cargo-fuzz command did not complete: {:?}", output.disposition));
    }
    let log_identity = bounded_log_identity(&output.stdout.bytes, &output.stderr.bytes)?;
    let final_corpus = snapshot(&workspace_root, CORPUS_DIRECTORY)?;
    let artifacts = snapshot(&workspace_root, ARTIFACT_DIRECTORY)?;
    let exit_code = output.exit_code.unwrap_or(INTEGRATION_FAILURE_EXIT);
    let crash_input = find_artifact(&artifacts, CRASH_PREFIX);
    let minimized_reproducer = find_artifact(&artifacts, MINIMIZED_PREFIX);
    let outcome = classify_outcome(exit_code, crash_input.as_ref());
    let replay_argv = minimized_reproducer.as_ref().map_or_else(Vec::new, |artifact| {
        vec![
            "cargo".to_owned(),
            "fuzz".to_owned(),
            "run".to_owned(),
            TARGET.to_owned(),
            artifact.path.clone(),
        ]
    });
    let details = ReceiptDetails {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: EXPECTED_PROFILE_IDENTITY.to_owned(),
        target: TARGET.to_owned(),
        engine: ENGINE.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        compiler_identity: prefixed_identity(COMPILER_PROFILE.as_bytes()),
        sanitizer: SANITIZER.to_owned(),
        command_argv,
        budget: Budget {
            max_total_time_seconds: MAX_TOTAL_TIME_SECONDS,
            max_runs: MAX_RUNS,
            seed: FIXED_SEED,
        },
        initial_corpus_root: corpus_root(&initial_corpus),
        initial_corpus,
        final_corpus_root: corpus_root(&final_corpus),
        final_corpus,
        dictionary: None,
        outcome: outcome.to_owned(),
        crash_input,
        minimized_reproducer,
        replay_argv,
        bounded_log_identity: log_identity,
        evidence_tier: EVIDENCE_TIER.to_owned(),
        claimed_properties: Vec::new(),
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    };
    let rendered =
        serde_json::to_string(&details).map_err(|error| format!("serialize cargo-fuzz run receipt: {error}"))?;
    Ok((rendered, exit_code))
}

fn snapshot(workspace_root: &std::path::Path, relative: &str) -> Result<Vec<ArtifactIdentity>, String> {
    let root = workspace_root.join(relative);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut pending = std::collections::VecDeque::with_capacity(MAX_ARTIFACT_COUNT);
    pending.push_back((root, relative.to_owned()));
    let mut snapshot = Snapshot::new();
    for _ in 0..MAX_ARTIFACT_COUNT {
        let Some((directory, logical_directory)) = pending.pop_front() else {
            return Ok(snapshot.finish());
        };
        let mut entries = std::fs::read_dir(&directory)
            .map_err(|error| format!("read cargo-fuzz directory {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("read cargo-fuzz directory entry: {error}"))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            append_entry(entry, &logical_directory, &mut pending, &mut snapshot)?;
        }
    }
    Err("cargo-fuzz snapshot exceeds directory traversal limit".to_owned())
}

fn append_entry(
    entry: std::fs::DirEntry,
    logical_directory: &str,
    pending: &mut std::collections::VecDeque<(std::path::PathBuf, String)>,
    snapshot: &mut Snapshot,
) -> Result<(), String> {
    let file_type = entry.file_type().map_err(|error| format!("inspect cargo-fuzz entry: {error}"))?;
    if file_type.is_symlink() {
        return Err(format!("cargo-fuzz artifact {} is a symlink", entry.path().display()));
    }
    let name = entry.file_name().into_string().map_err(|_| "cargo-fuzz artifact name is not UTF-8".to_owned())?;
    let logical_path = format!("{logical_directory}/{name}");
    if file_type.is_dir() {
        if pending.len() >= MAX_ARTIFACT_COUNT {
            return Err("cargo-fuzz snapshot exceeds pending directory limit".to_owned());
        }
        pending.push_back((entry.path(), logical_path));
        return Ok(());
    }
    if !file_type.is_file() {
        return Err(format!("cargo-fuzz artifact {} is not a regular file", entry.path().display()));
    }
    snapshot.push(file_identity(&entry.path(), logical_path)?)
}

fn file_identity(path: &std::path::Path, logical_path: String) -> Result<ArtifactIdentity, String> {
    let length = std::fs::metadata(path)
        .map_err(|error| format!("inspect cargo-fuzz artifact {}: {error}", path.display()))?
        .len();
    if length > MAX_ARTIFACT_BYTES {
        return Err(format!("cargo-fuzz artifact {} exceeds byte limit", path.display()));
    }
    let capacity = usize::try_from(length).map_err(|_| "cargo-fuzz artifact length does not fit usize".to_owned())?;
    let mut bytes = Vec::with_capacity(capacity);
    std::fs::File::open(path)
        .map_err(|error| format!("open cargo-fuzz artifact {}: {error}", path.display()))?
        .take(MAX_ARTIFACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read cargo-fuzz artifact {}: {error}", path.display()))?;
    let observed =
        u64::try_from(bytes.len()).map_err(|_| "cargo-fuzz artifact byte count does not fit u64".to_owned())?;
    if observed != length {
        return Err(format!("cargo-fuzz artifact {} changed during bounded read", path.display()));
    }
    Ok(ArtifactIdentity {
        path: logical_path,
        length,
        blake3: prefixed_identity(&bytes),
    })
}

fn bounded_log_identity(stdout: &[u8], stderr: &[u8]) -> Result<String, String> {
    let length = stdout.len().checked_add(stderr.len()).ok_or_else(|| "cargo-fuzz log length overflow".to_owned())?;
    let observed = u64::try_from(length).map_err(|_| "cargo-fuzz log length does not fit u64".to_owned())?;
    if observed > MAX_RECEIPT_BYTES {
        return Err("cargo-fuzz log exceeds bounded transcript limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(stdout);
    bytes.extend_from_slice(stderr);
    Ok(prefixed_identity(&bytes))
}

fn find_artifact(artifacts: &[ArtifactIdentity], prefix: &str) -> Option<ArtifactIdentity> {
    artifacts
        .iter()
        .find(|artifact| artifact.path.rsplit('/').next().is_some_and(|name| name.starts_with(prefix)))
        .cloned()
}

fn classify_outcome(exit_code: i32, crash_input: Option<&ArtifactIdentity>) -> &'static str {
    if exit_code == 0 {
        "clean"
    } else if crash_input.is_some() {
        "crash"
    } else {
        "tool_failure"
    }
}

fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED_BYTES: &[u8] = b"seed";
    const SHELL_MANAGED_VARIABLES: [&str; 4] = ["PWD", "OLDPWD", "SHLVL", "_"];
    const SELECTED_VARIABLE: &str = "OCTET_FUZZ_FIXTURE";
    const OWNER_ONLY_PERMISSIONS: u32 = 0o700;
    const FIXTURE_TIMEOUT_MS: u64 = 200;
    const FIXTURE_POLL_MS: u64 = 10;
    const FIXTURE_TEARDOWN_MS: u64 = 1_000;
    const FIXTURE_DEADLINE_SECONDS: u64 = 10;
    const TEXT_FILE_BUSY: &str = "Text file busy";
    const SPAWN_RETRIES: usize = 20;
    const SPAWN_RETRY_DELAY_MS: u64 = 10;

    #[test]
    fn bounded_snapshot_hashes_files_and_rejects_symlinks() {
        let directory = tempfile::tempdir().expect("create cargo-fuzz shell fixture");
        let corpus = directory.path().join(CORPUS_DIRECTORY);
        std::fs::create_dir_all(&corpus).expect("create corpus");
        std::fs::write(corpus.join("seed"), SEED_BYTES).expect("write seed");

        let observed = snapshot(directory.path(), CORPUS_DIRECTORY).expect("snapshot corpus");
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].path, format!("{CORPUS_DIRECTORY}/seed"));
        assert_eq!(observed[0].length, u64::try_from(SEED_BYTES.len()).expect("seed length fits u64"));
        assert_eq!(observed[0].blake3, prefixed_identity(SEED_BYTES));

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(corpus.join("seed"), corpus.join("link")).expect("create symlink");
            assert!(snapshot(directory.path(), CORPUS_DIRECTORY).expect_err("symlink rejected").contains("symlink"));
        }
    }

    #[test]
    fn outcome_and_log_classification_have_positive_and_negative_paths() {
        let crash = ArtifactIdentity {
            path: format!("{ARTIFACT_DIRECTORY}/{CRASH_PREFIX}fixture"),
            length: u64::try_from(SEED_BYTES.len()).expect("seed length fits u64"),
            blake3: prefixed_identity(SEED_BYTES),
        };
        assert_eq!(classify_outcome(0, None), "clean");
        assert_eq!(classify_outcome(INTEGRATION_FAILURE_EXIT, Some(&crash)), "crash");
        assert_eq!(classify_outcome(INTEGRATION_FAILURE_EXIT, None), "tool_failure");
        assert!(bounded_log_identity(b"out", b"err").is_ok());
        let oversized = vec![b'x'; usize::try_from(MAX_RECEIPT_BYTES + 1).expect("log limit fits usize")];
        assert!(bounded_log_identity(&oversized, &[]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_run_clears_ambient_environment_and_passes_fixed_argv() {
        assert!(std::env::vars_os().next().is_some(), "test process has an ambient environment");
        let directory = tempfile::tempdir().expect("create cargo-fuzz run fixture");
        let cargo = write_fixture_cargo(
            directory.path(),
            "export -p > child-env.txt\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\"; done > child-args.txt\n",
        );
        let tools = RuntimeTools {
            cargo,
            environment: vec![(SELECTED_VARIABLE.into(), "selected".into())],
        };
        let (rendered, exit_code) = run_fixture(directory.path(), &tools, run_limits().expect("limits")).expect("run");
        assert_eq!(exit_code, ACCEPTED_EXIT_CODE);
        let details: ReceiptDetails = serde_json::from_str(&rendered).expect("parse cargo-fuzz receipt");
        assert_eq!(details.outcome, "clean");
        let environment = std::fs::read_to_string(directory.path().join("child-env.txt")).expect("read child env");
        let mut names = environment
            .lines()
            .filter_map(|line| line.strip_prefix("export ").or_else(|| line.strip_prefix("declare -x ")))
            .filter_map(|line| line.split('=').next())
            .filter(|name| !SHELL_MANAGED_VARIABLES.contains(name))
            .collect::<Vec<_>>();
        names.sort_unstable();
        assert_eq!(names, vec![SELECTED_VARIABLE]);
        let args = std::fs::read_to_string(directory.path().join("child-args.txt")).expect("read child args");
        assert_eq!(
            args.lines().collect::<Vec<_>>(),
            command_argv()[1..].iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    #[cfg(unix)]
    #[test]
    fn bounded_run_times_out_and_kills_the_process_group() {
        let directory = tempfile::tempdir().expect("create cargo-fuzz timeout fixture");
        let cargo = write_fixture_cargo(directory.path(), "(while :; do :; done) &\nwhile :; do :; done\n");
        let limits = bounded_exec::ExecutionLimits {
            timeout_ms: FIXTURE_TIMEOUT_MS,
            poll_interval_ms: FIXTURE_POLL_MS,
            teardown_timeout_ms: FIXTURE_TEARDOWN_MS,
            ..run_limits().expect("limits")
        };
        let started = std::time::Instant::now();
        let tools = RuntimeTools {
            cargo,
            environment: Vec::new(),
        };
        let error = run_fixture(directory.path(), &tools, limits).expect_err("timed-out cargo-fuzz run must fail");
        assert!(error.contains("TimedOut"), "{error}");
        assert!(started.elapsed() < std::time::Duration::from_secs(FIXTURE_DEADLINE_SECONDS));
    }

    #[test]
    fn production_limits_derive_from_receipt_bounds() {
        let limits = run_limits().expect("limits");
        assert!(limits.timeout_ms > MAX_TOTAL_TIME_SECONDS * MILLISECONDS_PER_SECOND);
        let log_limit = usize::try_from(MAX_RECEIPT_BYTES).expect("log limit fits usize");
        assert!(limits.stdout_max_bytes + limits.stderr_max_bytes <= log_limit);
    }

    #[cfg(unix)]
    fn write_fixture_cargo(directory: &std::path::Path, body: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt as _;

        let path = directory.join("cargo-fixture");
        std::fs::write(&path, format!("#!/bin/sh\n{body}")).expect("write cargo fixture");
        let mut permissions = std::fs::metadata(&path).expect("inspect cargo fixture").permissions();
        permissions.set_mode(OWNER_ONLY_PERMISSIONS);
        std::fs::set_permissions(&path, permissions).expect("make cargo fixture executable");
        path
    }

    /// Retries a spawn that raced a sibling test's fork while the fixture script was still open
    /// for writing (`ETXTBSY`); no fixture process ran in that case.
    fn run_fixture(
        directory: &std::path::Path,
        tools: &RuntimeTools,
        limits: bounded_exec::ExecutionLimits,
    ) -> Result<(String, i32), String> {
        let mut outcome = run_with(directory, tools.clone(), limits);
        for _ in 0..SPAWN_RETRIES {
            if !outcome.as_ref().is_err_and(|error| error.contains(TEXT_FILE_BUSY)) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(SPAWN_RETRY_DELAY_MS));
            outcome = run_with(directory, tools.clone(), limits);
        }
        outcome
    }
}
