use std::io::Read as _;

const EXPORT_FILE_NAME: &str = "kani-export.json";
const OWNED_EXPORT_SUBDIR: &str = ".octet/kani-export";

pub fn run(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    assumption_review_path: Option<&std::path::Path>,
    export_dir_override: Option<&std::path::Path>,
    process_env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<(String, i32), String> {
    let runtime = super::probe::production_runtime(process_env)?;
    run_with(workspace_root, profile, &runtime, assumption_review_path, export_dir_override)
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    runtime: &super::probe::ToolRuntime,
    assumption_review_path: Option<&std::path::Path>,
    export_dir_override: Option<&std::path::Path>,
) -> Result<(String, i32), String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize Kani workspace {}: {error}", workspace_root.display()))?;
    let (source_manifest_identity, lockfile_identity) =
        super::probe::compute_source_manifest_identity(&workspace_root, &profile.limits)?;
    let assumption_review =
        read_assumption_review(&workspace_root, assumption_review_path, profile.limits.max_export_bytes)?;
    let toolchain = super::probe::inspect_toolchain(&workspace_root, profile, runtime)?;
    let (export_dir, owned_dir) = prepare_owned_export_dir(&workspace_root, export_dir_override)?;
    let export_path = export_dir.join(EXPORT_FILE_NAME);
    let argv = build_harness_argv(profile, &export_path)?;
    let result = execute_bounded(&workspace_root, profile, runtime, &argv, &export_path);
    if let Some(owned) = owned_dir
        && owned.exists()
    {
        std::fs::remove_dir_all(&owned).map_err(|error| format!("clean owned Kani export dir: {error}"))?;
    }
    let exec_result = result?;
    let observation = super::Observation {
        schema_version: super::OBSERVATION_SCHEMA.to_owned(),
        profile_id: profile.id.clone(),
        profile_identity: super::EXPECTED_PROFILE_IDENTITY.to_owned(),
        source_manifest_identity: source_manifest_identity.clone(),
        expected_source_manifest_identity: Some(source_manifest_identity),
        lockfile_identity: lockfile_identity.clone(),
        expected_lockfile_identity: lockfile_identity,
        command_argv: argv,
        toolchain,
        exit_code: exec_result.exit_code,
        timed_out: Some(exec_result.is_timed_out),
        truncated: Some(exec_result.is_truncated),
        bounded_log_identity: exec_result.bounded_log_identity,
        raw_export_json: exec_result.raw_export_json,
        assumption_review,
    };
    let rendered =
        serde_json::to_string(&observation).map_err(|error| format!("serialize Kani producer observation: {error}"))?;
    Ok((rendered, exec_result.exit_code))
}

struct BoundedOutcome {
    exit_code: i32,
    is_timed_out: bool,
    is_truncated: bool,
    bounded_log_identity: String,
    raw_export_json: String,
}

fn execute_bounded(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    runtime: &super::probe::ToolRuntime,
    argv: &[String],
    export_path: &std::path::Path,
) -> Result<BoundedOutcome, String> {
    let subcommand_args = argv.get(1..).ok_or_else(|| "Kani command_argv missing cargo subcommand".to_owned())?;
    let mut arg_list = Vec::with_capacity(subcommand_args.len());
    for item in subcommand_args {
        arg_list.push(std::ffi::OsString::from(item));
    }
    let exec = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: runtime.cargo.clone(),
            args: arg_list,
            current_dir: workspace_root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: super::probe::build_child_environment(runtime),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: profile.limits.timeout_seconds.saturating_mul(1_000),
            stdin_max_bytes: 1,
            stdout_max_bytes: profile.limits.max_output_bytes,
            stderr_max_bytes: profile.limits.max_output_bytes,
            poll_interval_ms: profile.limits.poll_interval_ms,
            teardown_timeout_ms: profile.limits.teardown_timeout_ms,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![0], true, true)
            .map_err(|error| format!("construct Kani outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("execute Kani producer: {error}"))?;
    let is_timed_out = exec.disposition == bounded_exec::Disposition::TimedOut;
    let is_truncated = matches!(exec.disposition, bounded_exec::Disposition::OutputLimitExceeded(_))
        || exec.stdout.truncated
        || exec.stderr.truncated;
    let exit_code = exec.exit_code.unwrap_or(1);
    let mut combined_log = String::from_utf8_lossy(&exec.stdout.bytes).into_owned();
    combined_log.push_str(&String::from_utf8_lossy(&exec.stderr.bytes));
    let bounded_log_identity = super::prefixed_identity(combined_log.as_bytes());
    let raw_export_json = if export_path.is_file() {
        read_bounded_export(export_path, profile.limits.max_export_bytes)?
    } else {
        String::new()
    };
    Ok(BoundedOutcome {
        exit_code,
        is_timed_out,
        is_truncated,
        bounded_log_identity,
        raw_export_json,
    })
}

fn prepare_owned_export_dir(
    workspace_root: &std::path::Path,
    override_dir: Option<&std::path::Path>,
) -> Result<(std::path::PathBuf, Option<std::path::PathBuf>), String> {
    if let Some(dir) = override_dir {
        if dir.exists() {
            let entries = std::fs::read_dir(dir)
                .map_err(|error| format!("inspect Kani export dir {}: {error}", dir.display()))?;
            for entry in entries {
                let entry = entry.map_err(|error| format!("read export dir entry: {error}"))?;
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("json") {
                    return Err(format!(
                        "Kani export directory {} contains stale JSON artifact {}; remove prior exports before running",
                        dir.display(),
                        entry.path().display()
                    ));
                }
            }
        } else {
            std::fs::create_dir_all(dir)
                .map_err(|error| format!("create Kani export dir {}: {error}", dir.display()))?;
        }
        return Ok((dir.to_path_buf(), None));
    }
    let dir = workspace_root.join(OWNED_EXPORT_SUBDIR);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|error| format!("clean Kani export dir {}: {error}", dir.display()))?;
    }
    std::fs::create_dir_all(&dir).map_err(|error| format!("create owned Kani export dir: {error}"))?;
    Ok((dir.clone(), Some(dir)))
}

fn build_harness_argv(profile: &super::Profile, export_path: &std::path::Path) -> Result<Vec<String>, String> {
    let harness = profile
        .harnesses
        .first()
        .ok_or_else(|| "Kani producer profile declares zero harnesses".to_owned())?;
    let mut argv = Vec::with_capacity(profile.command_prefix.len() + profile.kani_pin.export_flags.len() + 5);
    argv.extend(profile.command_prefix.iter().cloned());
    argv.push("--harness".to_owned());
    argv.push(harness.name.clone());
    argv.push("--default-unwind".to_owned());
    argv.push(harness.unwind.to_string());
    argv.extend(profile.kani_pin.export_flags.iter().cloned());
    argv.push(export_path.display().to_string());
    Ok(argv)
}

fn read_bounded_export(path: &std::path::Path, max_bytes: u64) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|error| format!("stat Kani export {}: {error}", path.display()))?;
    if meta.len() > max_bytes {
        return Err(format!("Kani export {} exceeds {} byte limit", path.display(), max_bytes));
    }
    let file = std::fs::File::open(path).map_err(|error| format!("open Kani export {}: {error}", path.display()))?;
    let cap = usize::try_from(meta.len()).map_err(|_| "Kani export length does not fit usize".to_owned())?;
    let mut bytes = Vec::with_capacity(cap);
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read Kani export {}: {error}", path.display()))?;
    let observed = u64::try_from(bytes.len()).map_err(|_| "Kani export byte count overflow".to_owned())?;
    if observed > max_bytes {
        return Err("Kani export grew beyond byte limit during read".to_owned());
    }
    String::from_utf8(bytes).map_err(|error| format!("Kani export {} is not UTF-8: {error}", path.display()))
}

fn read_assumption_review(
    workspace_root: &std::path::Path,
    review_path: Option<&std::path::Path>,
    max_bytes: u64,
) -> Result<Option<super::AssumptionReviewRecord>, String> {
    let Some(path) = review_path else {
        return Ok(None);
    };
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };
    let text = read_bounded_export(&resolved, max_bytes)?;
    let parsed: super::AssumptionReviewRecord = serde_json::from_str(&text)
        .map_err(|error| format!("parse Kani assumption review {}: {error}", resolved.display()))?;
    Ok(Some(parsed))
}
