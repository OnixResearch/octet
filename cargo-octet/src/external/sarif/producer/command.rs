pub(crate) fn version_argv(profile: &super::Profile) -> Vec<String> {
    match profile.producer.as_str() {
        super::PRODUCER_CODEQL => string_argv(&["codeql", "version", "--format=json"]),
        _ => string_argv(&["semgrep", "--version"]),
    }
}

pub(super) fn semgrep_scan_argv(
    profile: &super::Profile,
    report_path: &std::path::Path,
    selected_paths: &[String],
) -> Vec<String> {
    let mut argv = string_argv(&[
        "semgrep",
        "scan",
        "--config",
        profile.rules.path.as_str(),
        "--sarif",
        "--output",
        report_path.to_string_lossy().as_ref(),
        "--metrics=off",
        "--disable-version-check",
    ]);
    argv.push(format!("--jobs={}", profile.limits.max_threads));
    argv.push(format!("--max-memory={}", profile.limits.max_memory_mb));
    argv.extend(selected_paths.iter().cloned());
    for glob in &profile.scope.exclude_globs {
        argv.push("--exclude".to_owned());
        argv.push(glob.clone());
    }
    argv
}

pub(super) fn string_argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

pub(super) fn executable_path_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

pub(super) fn stage_paths(stage_dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    (stage_dir.join(super::REPORT_FILE_NAME), stage_dir.join(super::DATABASE_DIR_NAME))
}

pub(super) fn probe_version(
    profile: &super::Profile,
    probe: &super::process::StepOutput,
) -> Result<String, (&'static str, String)> {
    if probe.exit_code != super::EXIT_SUCCESS {
        return Err((
            super::OUTCOME_TOOL_ERROR,
            format!("{} version probe exited {}", profile.driver_name, probe.exit_code),
        ));
    }
    let stdout = String::from_utf8_lossy(&probe.stdout);
    let observed = match profile.producer.as_str() {
        super::PRODUCER_CODEQL => serde_json::from_str::<serde_json::Value>(stdout.trim())
            .ok()
            .and_then(|value| value.get("version").and_then(serde_json::Value::as_str).map(ToOwned::to_owned))
            .unwrap_or_default(),
        _ => stdout.trim().to_owned(),
    };
    if observed != profile.executable.declared_version {
        return Err((
            super::OUTCOME_TOOL_ERROR,
            format!(
                "{} is pinned to {} but the resolved executable reports `{observed}`",
                profile.driver_name, profile.executable.declared_version
            ),
        ));
    }
    Ok(observed)
}

pub(super) fn codeql_create_argv(
    profile: &super::Profile,
    database_path: &std::path::Path,
    workspace_root: &std::path::Path,
) -> Vec<String> {
    let mut argv = string_argv(&["codeql", "database", "create"]);
    argv.push(executable_path_string(database_path));
    argv.push("--language=rust".to_owned());
    argv.push(format!("--build-mode={}", super::BUILD_MODE_NONE));
    argv.push(format!("--source-root={}", workspace_root.display()));
    argv.push(format!("--threads={}", profile.limits.max_threads));
    argv.push(format!("--ram={}", profile.limits.max_memory_mb));
    argv
}

pub(super) fn codeql_analyze_argv(
    profile: &super::Profile,
    database_path: &std::path::Path,
    report_path: &std::path::Path,
) -> Vec<String> {
    let mut argv = string_argv(&["codeql", "database", "analyze"]);
    argv.push(executable_path_string(database_path));
    argv.push("--format=sarif-latest".to_owned());
    argv.push(format!("--output={}", report_path.display()));
    argv.push("--sarif-group-rules-by-pack".to_owned());
    argv.push(format!("--threads={}", profile.limits.max_threads));
    argv.push(format!("--ram={}", profile.limits.max_memory_mb));
    for pack in &profile.query_packs {
        argv.push(format!("{}@{}", pack.name, pack.version));
    }
    argv
}
