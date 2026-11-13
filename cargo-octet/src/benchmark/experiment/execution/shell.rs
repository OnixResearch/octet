use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

const PROBE_TIMEOUT_MS: u64 = 10_000;
pub(super) const ADOPTION_TIMEOUT_MS: u64 = 120_000;
const STREAM_LIMIT_BYTES: usize = 262_144;
const POLL_INTERVAL_MS: u64 = 25;
const TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const STDIN_LIMIT_BYTES: usize = 1;
static SCRATCH_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn probe_tool(
    tool: &crate::benchmark::experiment::model::ToolPin,
    runtime: &super::Runtime,
) -> crate::benchmark::experiment::model::ToolAvailability {
    let Some(program) = tool.probe_argv.first() else {
        return unavailable(tool, "probe argv is empty");
    };
    let Some(executable) = resolve_executable(program, runtime) else {
        return unavailable(tool, &format!("executable `{program}` is not available on PATH"));
    };
    let args = tool.probe_argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
    match execute(
        command(&executable, args, Path::new("/"), base_environment(runtime)),
        limits(PROBE_TIMEOUT_MS),
        &runtime.clock,
    ) {
        Ok(outcome) if outcome.succeeded => {
            let version_output = format!("{}{}", outcome.stdout, outcome.stderr);
            crate::benchmark::experiment::model::ToolAvailability {
                tool_id: tool.tool_id.clone(),
                status: crate::benchmark::experiment::model::Availability::Local,
                version_identity: crate::benchmark::prefixed_identity(version_output.as_bytes()),
                version_output,
                reason: None,
            }
        }
        Ok(outcome) => unavailable(tool, &format!("probe failed: {:?}", outcome.reason)),
        Err(error) => unavailable(tool, &format!("probe could not run: {error}")),
    }
}

#[cfg(test)]
pub fn run_command(root: &Path, argv: &[String], runtime: &super::Runtime) -> Result<super::CommandOutcome, String> {
    let Some(program) = argv.first() else {
        return Err("experiment command is empty".to_owned());
    };
    let executable = resolve_executable(program, runtime)
        .ok_or_else(|| format!("experiment executable `{program}` is not available on PATH"))?;
    let command_arguments = argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
    let root = absolute_root(root)?;
    let scratch = scratch_directory("command", runtime);
    let environment = super::scratch::prepare(&scratch, runtime)?;
    let outcome = execute(
        command(&executable, command_arguments, &root, environment),
        limits(ADOPTION_TIMEOUT_MS),
        &runtime.clock,
    );
    let cleanup = super::scratch::cleanup(&scratch);
    match (outcome, cleanup) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; {cleanup}")),
    }
}

pub(super) fn environment_observation(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    cache_state: crate::benchmark::experiment::model::CacheState,
) -> crate::benchmark::experiment::model::ObservationEnvironment {
    crate::benchmark::experiment::model::ObservationEnvironment {
        target: profile.sample_plan.target.clone(),
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        rust_toolchain: profile.rust_toolchain.clone(),
        cache_state,
    }
}

fn unavailable(
    tool: &crate::benchmark::experiment::model::ToolPin,
    reason: &str,
) -> crate::benchmark::experiment::model::ToolAvailability {
    crate::benchmark::experiment::model::ToolAvailability {
        tool_id: tool.tool_id.clone(),
        status: crate::benchmark::experiment::model::Availability::Unavailable,
        version_output: String::new(),
        version_identity: crate::benchmark::prefixed_identity(b""),
        reason: Some(reason.to_owned()),
    }
}

fn base_environment(runtime: &super::Runtime) -> Vec<(OsString, OsString)> {
    let mut environment = Vec::with_capacity(super::PRESERVED_ENVIRONMENT.len() + 3);
    for name in super::PRESERVED_ENVIRONMENT {
        if let Some(value) = runtime.environment.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone()) {
            environment.push((OsString::from(name), value));
        }
    }
    environment
}

pub(super) fn adoption_environment(
    home: &Path,
    temp: &Path,
    target: &Path,
    runtime: &super::Runtime,
) -> Vec<(OsString, OsString)> {
    let mut environment = base_environment(runtime);
    environment.push((OsString::from("HOME"), home.as_os_str().to_owned()));
    environment.push((OsString::from("TMPDIR"), temp.as_os_str().to_owned()));
    environment.push((OsString::from("CARGO_TARGET_DIR"), target.as_os_str().to_owned()));
    environment
}

pub(super) fn execute(
    command: bounded_exec::CommandSpec,
    limits: bounded_exec::ExecutionLimits,
    clock: &cap_std::time::MonotonicClock,
) -> Result<super::CommandOutcome, String> {
    let program = command.program.display().to_string();
    let started = clock.now();
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command,
        limits,
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![0], true, true)
            .map_err(|error| format!("construct experiment outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded experiment command: {error}"))?;
    let duration_ns = u64::try_from(clock.now().duration_since(started).as_nanos())
        .map_err(|_| "experiment duration does not fit u64 nanoseconds".to_owned())?;
    let stdout = String::from_utf8_lossy(&output.stdout.bytes).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr.bytes).into_owned();
    let is_succeeded = output.disposition == bounded_exec::Disposition::Succeeded;
    let is_timed_out = output.completion == bounded_exec::Completion::TimedOut;
    let is_truncated = output.stdout.truncated || output.stderr.truncated;
    let reason = if is_succeeded {
        None
    } else if is_timed_out {
        Some("experiment command timed out".to_owned())
    } else {
        Some(format!(
            "experiment command did not succeed: {:?} (exit {:?})",
            output.disposition, output.exit_code
        ))
    };
    Ok(super::CommandOutcome {
        program,
        stdout,
        stderr,
        exit_code: output.exit_code,
        succeeded: is_succeeded,
        timed_out: is_timed_out,
        duration_ns,
        truncated: is_truncated,
        reason,
    })
}

pub(super) fn resolve_executable(program: &str, runtime: &super::Runtime) -> Option<PathBuf> {
    if program.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(program);
        return is_executable(&path).then_some(path);
    }
    let path = runtime.environment.iter().find(|(key, _)| key == "PATH").map(|(_, value)| value)?;
    for directory in std::env::split_paths(path) {
        let candidate = directory.join(program);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

pub(super) fn absolute_root(root: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(root).map_err(|error| format!("canonicalize experiment root {}: {error}", root.display()))
}

pub(super) fn scratch_directory(tag: &str, runtime: &super::Runtime) -> PathBuf {
    let counter = SCRATCH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    runtime
        .scratch_root
        .join(format!("octet-benchmark-experiment-{}-{}-{counter}", std::process::id(), sanitize(tag)))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub(super) fn command(
    executable: &Path,
    args: Vec<OsString>,
    root: &Path,
    environment: Vec<(OsString, OsString)>,
) -> bounded_exec::CommandSpec {
    bounded_exec::CommandSpec {
        program: executable.to_path_buf(),
        args,
        current_dir: root.to_path_buf(),
        environment_mode: bounded_exec::EnvironmentMode::Clear,
        environment,
        input: bounded_exec::Input::Null,
    }
}

pub(super) fn limits(timeout_ms: u64) -> bounded_exec::ExecutionLimits {
    bounded_exec::ExecutionLimits {
        timeout_ms,
        stdin_max_bytes: STDIN_LIMIT_BYTES,
        stdout_max_bytes: STREAM_LIMIT_BYTES,
        stderr_max_bytes: STREAM_LIMIT_BYTES,
        poll_interval_ms: POLL_INTERVAL_MS,
        teardown_timeout_ms: TEARDOWN_TIMEOUT_MS,
    }
}
