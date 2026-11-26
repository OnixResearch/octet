const AMBIENT_PASSTHROUGH: &[&str] = &["HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LANG", "LC_ALL"];
const MAX_SOURCE_FILE_BYTES: u64 = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRuntime {
    pub cargo: std::path::PathBuf,
    pub kani_rustc: std::path::PathBuf,
    pub cbmc: std::path::PathBuf,
    pub passthrough_env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

pub fn production_runtime(vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Result<ToolRuntime, String> {
    let mut passthrough_env = Vec::with_capacity(AMBIENT_PASSTHROUGH.len());
    for (key, value) in vars {
        let key_str = key.to_string_lossy();
        if AMBIENT_PASSTHROUGH.contains(&key_str.as_ref()) {
            passthrough_env.push((key.clone(), value.clone()));
        }
    }
    let cargo = resolve_program(&passthrough_env, "cargo")?;
    let kani_rustc = resolve_program(&passthrough_env, "rustc")?;
    let cbmc = resolve_program(&passthrough_env, "cbmc")?;
    Ok(ToolRuntime {
        cargo,
        kani_rustc,
        cbmc,
        passthrough_env,
    })
}

fn resolve_program(
    env: &[(std::ffi::OsString, std::ffi::OsString)],
    program: &str,
) -> Result<std::path::PathBuf, String> {
    let search_path = env
        .iter()
        .find(|(key, _)| key == "PATH")
        .map(|(_, value)| value)
        .ok_or_else(|| "Kani producer runtime requires PATH".to_owned())?;
    std::env::split_paths(search_path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("Kani producer runtime cannot locate `{program}` on PATH"))
}

pub fn build_child_environment(runtime: &ToolRuntime) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut env = Vec::with_capacity(runtime.passthrough_env.len() + 3);
    env.extend(runtime.passthrough_env.iter().cloned());
    env.push((std::ffi::OsString::from("RUSTC"), runtime.kani_rustc.clone().into_os_string()));
    env.push((std::ffi::OsString::from("RUSTC_WRAPPER"), std::ffi::OsString::new()));
    env.push((std::ffi::OsString::from("RUSTC_WORKSPACE_WRAPPER"), std::ffi::OsString::new()));
    env
}

pub fn inspect_toolchain(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    runtime: &ToolRuntime,
) -> Result<super::ToolchainObservation, String> {
    let env = build_child_environment(runtime);
    let kani_out = exec_command(workspace_root, &runtime.cargo, &["kani", "--version"], &env, &profile.limits)?;
    let rustc_out = exec_command(workspace_root, &runtime.kani_rustc, &["-vV"], &env, &profile.limits)?;
    let cbmc_out = exec_command(workspace_root, &runtime.cbmc, &["--version"], &env, &profile.limits)?;
    let (rustc_release, rustc_commit_hash) = parse_rustc_vv(&rustc_out)?;
    let kani_version = kani_out.trim().to_owned();
    let cbmc_version = cbmc_out.trim().to_owned();
    if !kani_version.contains(&profile.kani_pin.kani_version) {
        return Err(format!("probed Kani `{kani_version}` != pinned `{}`", profile.kani_pin.kani_version));
    }
    if rustc_release != profile.kani_pin.rustc_release {
        return Err(format!("probed Kani rustc `{rustc_release}` != pinned `{}`", profile.kani_pin.rustc_release));
    }
    if !cbmc_version.contains(&profile.kani_pin.cbmc_version) {
        return Err(format!("probed CBMC `{cbmc_version}` != pinned `{}`", profile.kani_pin.cbmc_version));
    }
    Ok(super::ToolchainObservation {
        kani_version,
        provider_revision: profile.kani_pin.provider_revision.clone(),
        closure_identity: profile.kani_pin.closure_identity.clone(),
        rustc_release,
        rustc_commit_hash,
        cbmc_version,
        solver_mode: profile.kani_pin.solver_mode.clone(),
        solver_identity: profile.kani_pin.solver_identity.clone(),
    })
}

fn exec_command(
    workspace_root: &std::path::Path,
    program: &std::path::Path,
    args: &[&str],
    environment: &[(std::ffi::OsString, std::ffi::OsString)],
    limits: &super::Limits,
) -> Result<String, String> {
    let mut arg_list = Vec::with_capacity(args.len());
    for item in args {
        arg_list.push(std::ffi::OsString::from(item));
    }
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: program.to_path_buf(),
            args: arg_list,
            current_dir: workspace_root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: environment.to_vec(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: limits.timeout_seconds.saturating_mul(1_000),
            stdin_max_bytes: 1,
            stdout_max_bytes: 32_768,
            stderr_max_bytes: 32_768,
            poll_interval_ms: limits.poll_interval_ms,
            teardown_timeout_ms: limits.teardown_timeout_ms,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![0], false, false)
            .map_err(|error| format!("construct Kani probe outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("execute Kani probe `{}`: {error}", program.display()))?;
    if output.disposition != bounded_exec::Disposition::Succeeded || output.exit_code != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr.bytes);
        return Err(format!("Kani probe `{}` failed ({:?}): {stderr}", program.display(), output.disposition));
    }
    String::from_utf8(output.stdout.bytes).map_err(|error| format!("Kani probe stdout not UTF-8: {error}"))
}

fn parse_rustc_vv(stdout: &str) -> Result<(String, String), String> {
    let mut release = None;
    let mut commit = None;
    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("release:") {
            release = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("commit-hash:") {
            commit = Some(value.trim().to_owned());
        }
    }
    Ok((
        release.ok_or_else(|| "Kani rustc -vV missing `release:`".to_owned())?,
        commit.ok_or_else(|| "Kani rustc -vV missing `commit-hash:`".to_owned())?,
    ))
}

pub fn compute_source_manifest_identity(
    workspace_root: &std::path::Path,
    limits: &super::Limits,
) -> Result<(String, Option<String>), String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let files = collect_rust_files_iterative(workspace_root, limits.max_source_files)?;
    let mut manifest = String::with_capacity(files.len() * 80);
    for rel in &files {
        let bytes = root.read_bounded(&crate::cap_fs::RootRelativePath::new(rel)?, MAX_SOURCE_FILE_BYTES)?;
        manifest.push_str(rel);
        manifest.push('\t');
        manifest.push_str(&super::prefixed_identity(&bytes));
        manifest.push('\n');
    }
    let lock_path = workspace_root.join("Cargo.lock");
    let lockfile = if lock_path.is_file() {
        let lock_bytes =
            root.read_bounded(&crate::cap_fs::RootRelativePath::new("Cargo.lock")?, MAX_SOURCE_FILE_BYTES)?;
        Some(super::prefixed_identity(&lock_bytes))
    } else {
        None
    };
    Ok((super::prefixed_identity(manifest.as_bytes()), lockfile))
}

fn collect_rust_files_iterative(base: &std::path::Path, max_files: usize) -> Result<Vec<String>, String> {
    let mut files = Vec::with_capacity(max_files.min(64));
    let mut pending = Vec::with_capacity(16);
    pending.push(base.to_path_buf());
    while let Some(current) = pending.pop() {
        let entries =
            std::fs::read_dir(&current).map_err(|error| format!("read dir {}: {error}", current.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("read dir entry: {error}"))?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else if matches!(name_str.as_ref(), "Cargo.toml" | "Cargo.lock") || name_str.ends_with(".rs") {
                if files.len() >= max_files {
                    return Err(format!("Kani source file count exceeds limit {max_files}"));
                }
                let rel = path.strip_prefix(base).map_err(|error| format!("strip workspace prefix: {error}"))?;
                files.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    Ok(files)
}
