struct RawExecOutcome<'a> {
    probe: &'a super::CompilerProbe,
    combined_log: &'a str,
    observed_exit: Option<i32>,
    is_timed_out: bool,
    is_truncated: bool,
}

pub fn run(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    process_env: &[(std::ffi::OsString, std::ffi::OsString)],
) -> Result<(String, i32), String> {
    let runtime = super::probe::production_runtime(process_env)?;
    run_with(workspace_root, profile, &runtime)
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    runtime: &super::probe::ToolRuntime,
) -> Result<(String, i32), String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize dynamic workspace {}: {error}", workspace_root.display()))?;
    let (source_manifest_identity, lockfile_identity) =
        super::manifest::collect_source_digest(&workspace_root, &profile.limits)?;
    let compiler_probe = super::probe::inspect_toolchain(&workspace_root, profile, runtime)?;
    let child = super::probe::build_child_environment(profile, runtime)?;
    let subcommand_args = profile
        .command_argv
        .get(1..)
        .ok_or_else(|| "dynamic profile command_argv missing cargo subcommand".to_owned())?;
    let mut arg_list = Vec::with_capacity(subcommand_args.len());
    for item in subcommand_args {
        arg_list.push(std::ffi::OsString::from(item));
    }
    let exec_output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: runtime.cargo.clone(),
            args: arg_list,
            current_dir: workspace_root,
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: child.env,
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
            .map_err(|error| format!("construct dynamic run outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("execute dynamic profile `{}`: {error}", profile.id))?;

    let is_timed_out = exec_output.disposition == bounded_exec::Disposition::TimedOut;
    let is_truncated = matches!(exec_output.disposition, bounded_exec::Disposition::OutputLimitExceeded(_))
        || exec_output.stdout.truncated
        || exec_output.stderr.truncated;
    let mut combined = String::from_utf8_lossy(&exec_output.stdout.bytes).into_owned();
    combined.push_str(&String::from_utf8_lossy(&exec_output.stderr.bytes));
    let (observation, exit_code) = parse_execution_output(profile, &RawExecOutcome {
        probe: &compiler_probe,
        combined_log: &combined,
        observed_exit: exec_output.exit_code,
        is_timed_out,
        is_truncated,
    });
    let envelope = super::Input {
        schema_version: super::QUALIFICATION_INPUT_SCHEMA.to_owned(),
        profile_id: profile.id.clone(),
        profile_identity: super::EXPECTED_PROFILE_IDENTITY.to_owned(),
        source_manifest_identity: source_manifest_identity.clone(),
        expected_source_manifest_identity: Some(source_manifest_identity),
        lockfile_identity: lockfile_identity.clone(),
        expected_lockfile_identity: lockfile_identity,
        command_argv: profile.command_argv.clone(),
        compiler_probe,
        observed_runs: vec![observation],
    };
    let rendered = serde_json::to_string(&envelope)
        .map_err(|error| format!("serialize dynamic qualification envelope: {error}"))?;
    Ok((rendered, exit_code))
}

fn parse_execution_output(profile: &super::Profile, raw: &RawExecOutcome<'_>) -> (super::CellObservation, i32) {
    let bounded_log_identity = super::prefixed_identity(raw.combined_log.as_bytes());
    let exit_code = raw.observed_exit.unwrap_or(1);
    let parsed = parse_test_transcript(raw.combined_log, profile);
    let disposition = if raw.is_timed_out {
        "timeout"
    } else if raw.is_truncated {
        "incomplete"
    } else if exit_code == 0 && parsed.tests_failed == 0 && parsed.failure_messages.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let cell_id = profile.matrix_cells.first().map_or_else(|| "default-cell".to_owned(), |cell| cell.cell_id.clone());
    (
        super::CellObservation {
            cell_id,
            disposition: disposition.to_owned(),
            target_triple: raw.probe.host_triple.clone(),
            tool_version: raw.probe.tool_version.clone(),
            rustc_release: raw.probe.rustc_release.clone(),
            miri_flags: profile.miri.as_ref().map(|miri| miri.miri_flags.clone()),
            rustflags: profile.loom.as_ref().map(|loom| loom.rustflags.clone()),
            encoded_rustflags: profile.loom.as_ref().map(|loom| loom.encoded_rustflags.clone()),
            tests_started: parsed.tests_started,
            tests_passed: parsed.tests_passed,
            tests_failed: parsed.tests_failed,
            tests_ignored: parsed.tests_ignored,
            tests_filtered_out: parsed.tests_filtered_out,
            executed_tests: parsed.executed_tests,
            filters: Some(profile_filters(profile)),
            model_entered: Some(parsed.is_model_entered),
            model_completed: Some(parsed.is_model_completed),
            permutations_explored: Some(parsed.permutations_explored),
            exit_code,
            timed_out: Some(raw.is_timed_out),
            truncated: Some(raw.is_truncated),
            bounded_log_identity,
            failure_messages: Some(parsed.failure_messages),
        },
        exit_code,
    )
}

fn profile_filters(profile: &super::Profile) -> Vec<String> {
    profile
        .command_argv
        .iter()
        .position(|arg| arg == "--lib")
        .and_then(|idx| profile.command_argv.get(idx + 1))
        .filter(|arg| *arg != "--")
        .map_or_else(|| Vec::with_capacity(0), |arg| vec![(*arg).clone()])
}

struct TranscriptCounts {
    tests_started: u64,
    tests_passed: u64,
    tests_failed: u64,
    tests_ignored: u64,
    tests_filtered_out: u64,
    executed_tests: Vec<String>,
    is_model_entered: bool,
    is_model_completed: bool,
    permutations_explored: u64,
    failure_messages: Vec<String>,
}

fn parse_test_transcript(transcript: &str, profile: &super::Profile) -> TranscriptCounts {
    let mut counts = TranscriptCounts {
        tests_started: 0,
        tests_passed: 0,
        tests_failed: 0,
        tests_ignored: 0,
        tests_filtered_out: 0,
        executed_tests: Vec::with_capacity(profile.test_inventory.len()),
        is_model_entered: false,
        is_model_completed: false,
        permutations_explored: 0,
        failure_messages: Vec::with_capacity(4),
    };
    let entry_marker = profile.loom.as_ref().map(|loom| loom.expected_model_entry_marker.as_str());
    let done_marker = profile.loom.as_ref().map(|loom| loom.expected_model_completion_marker.as_str());
    for line in transcript.lines() {
        parse_transcript_line(line, entry_marker, done_marker, &mut counts);
    }
    counts
}

fn parse_transcript_line(
    line: &str,
    entry_marker: Option<&str>,
    done_marker: Option<&str>,
    counts: &mut TranscriptCounts,
) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("test ") {
        parse_test_line(rest, counts);
    }
    if trimmed.starts_with("test result:") {
        parse_summary_line(trimmed, counts);
    }
    if let Some(marker) = entry_marker
        && trimmed.contains(marker)
    {
        counts.is_model_entered = true;
    }
    if let Some(marker) = done_marker
        && trimmed.contains(marker)
    {
        counts.is_model_completed = true;
        counts.permutations_explored = parse_permutations(trimmed).unwrap_or(1).max(counts.permutations_explored);
    }
    if trimmed.contains("error: Undefined Behavior") || trimmed.contains("error[") || trimmed.contains("panicked at") {
        counts.failure_messages.push(trimmed.to_owned());
    }
}

fn parse_test_line(rest: &str, counts: &mut TranscriptCounts) {
    if let Some((name, outcome)) = rest.rsplit_once(" ... ") {
        let name = name.trim();
        if !name.is_empty() {
            counts.executed_tests.push(name.to_owned());
            counts.tests_started = counts.tests_started.saturating_add(1);
            match outcome.trim() {
                "ok" => counts.tests_passed = counts.tests_passed.saturating_add(1),
                "FAILED" => {
                    counts.tests_failed = counts.tests_failed.saturating_add(1);
                    counts.failure_messages.push(format!("test `{name}` FAILED"));
                }
                "ignored" => counts.tests_ignored = counts.tests_ignored.saturating_add(1),
                _ => {}
            }
        }
    }
}

fn parse_summary_line(line: &str, counts: &mut TranscriptCounts) {
    for part in line.split(';') {
        let words: Vec<&str> = part.split_whitespace().collect();
        for pair in words.windows(2) {
            if pair[1] == "filtered"
                && let Ok(number) = pair[0].parse::<u64>()
            {
                counts.tests_filtered_out = number;
            }
        }
    }
}

fn parse_permutations(line: &str) -> Option<u64> {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix("permutations="))
        .and_then(|value| value.parse::<u64>().ok())
}
