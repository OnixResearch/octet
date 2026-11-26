const AMBIENT_PASSTHROUGH: &[&str] = &["HOME", "PATH", "TMPDIR", "TEMP", "TMP", "USER", "LANG", "LC_ALL"];
const IGNORED_CARGO_ENV: &[&str] = &[
    "RUSTUP_TOOLCHAIN",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "MIRIFLAGS",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRuntime {
    pub cargo: std::path::PathBuf,
    pub rustc: std::path::PathBuf,
    pub xargo_rust_src: Option<std::path::PathBuf>,
    pub passthrough_env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    pub caller_overrides: Vec<String>,
}

pub struct ChildEnvironment {
    pub env: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    pub sysroot_identity: Option<String>,
}

pub fn production_runtime(vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Result<ToolRuntime, String> {
    runtime_from_pairs(vars)
}

pub fn runtime_from_pairs(vars: &[(std::ffi::OsString, std::ffi::OsString)]) -> Result<ToolRuntime, String> {
    let mut passthrough_env = Vec::with_capacity(AMBIENT_PASSTHROUGH.len());
    for key in AMBIENT_PASSTHROUGH {
        if let Some((_, value)) = vars.iter().find(|(k, _)| k.to_string_lossy() == *key) {
            passthrough_env.push((std::ffi::OsString::from(key), value.clone()));
        }
    }
    let caller_overrides: Vec<String> = IGNORED_CARGO_ENV
        .iter()
        .filter_map(|key| {
            vars.iter().find(|(k, v)| k.to_string_lossy() == *key && !v.is_empty()).map(|_| (*key).to_owned())
        })
        .collect();
    let mut xargo_rust_src = None;
    for key in ["XARGO_RUST_SRC", "MIRI_SYSROOT"] {
        if xargo_rust_src.is_none()
            && let Some((_, value)) = vars.iter().find(|(k, _)| k.to_string_lossy() == key)
        {
            xargo_rust_src = Some(std::path::PathBuf::from(value));
        }
    }
    let cargo = resolve_program(&passthrough_env, "cargo")?;
    let rustc = resolve_program(&passthrough_env, "rustc")?;
    Ok(ToolRuntime {
        cargo,
        rustc,
        xargo_rust_src,
        passthrough_env,
        caller_overrides,
    })
}

pub fn resolve_program(
    env: &[(std::ffi::OsString, std::ffi::OsString)],
    program: &str,
) -> Result<std::path::PathBuf, String> {
    let search_path = env
        .iter()
        .find(|(key, _)| key == "PATH")
        .map(|(_, value)| value)
        .ok_or_else(|| "dynamic qualification runtime requires PATH".to_owned())?;
    std::env::split_paths(search_path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("dynamic qualification runtime cannot locate `{program}` on PATH"))
}

pub fn build_child_environment(profile: &super::Profile, runtime: &ToolRuntime) -> Result<ChildEnvironment, String> {
    reject_conflicting_overrides(profile, &runtime.caller_overrides)?;
    let mut env = Vec::with_capacity(runtime.passthrough_env.len() + 8);
    env.extend(runtime.passthrough_env.iter().cloned());
    env.push((std::ffi::OsString::from("RUSTC"), runtime.rustc.clone().into_os_string()));
    env.push((std::ffi::OsString::from("RUSTC_WRAPPER"), std::ffi::OsString::new()));
    env.push((std::ffi::OsString::from("RUSTC_WORKSPACE_WRAPPER"), std::ffi::OsString::new()));
    let mut sysroot_identity = None;
    if let Some(miri) = profile.miri.as_ref() {
        env.push((std::ffi::OsString::from("MIRIFLAGS"), std::ffi::OsString::from(miri.miri_flags.join(" "))));
        let rust_src = runtime
            .xargo_rust_src
            .as_ref()
            .filter(|path| path.exists())
            .ok_or_else(|| "Miri qualification requires existing XARGO_RUST_SRC or MIRI_SYSROOT".to_owned())?;
        env.push((std::ffi::OsString::from("XARGO_RUST_SRC"), rust_src.clone().into_os_string()));
        sysroot_identity = Some(super::prefixed_identity(rust_src.to_string_lossy().as_bytes()));
    }
    if let Some(loom) = profile.loom.as_ref() {
        env.push((
            std::ffi::OsString::from("CARGO_ENCODED_RUSTFLAGS"),
            std::ffi::OsString::from(&loom.encoded_rustflags),
        ));
        env.push((std::ffi::OsString::from("RUSTFLAGS"), std::ffi::OsString::from(&loom.rustflags)));
        env.push((
            std::ffi::OsString::from("LOOM_MAX_PREEMPTIONS"),
            std::ffi::OsString::from(loom.preemption_bound.to_string()),
        ));
        env.push((
            std::ffi::OsString::from("LOOM_MAX_BRANCHES"),
            std::ffi::OsString::from(loom.max_branches.to_string()),
        ));
        env.push((
            std::ffi::OsString::from("LOOM_MAX_PERMUTATIONS"),
            std::ffi::OsString::from(loom.max_permutations.to_string()),
        ));
    }
    Ok(ChildEnvironment { env, sysroot_identity })
}

fn reject_conflicting_overrides(profile: &super::Profile, overrides: &[String]) -> Result<(), String> {
    if profile.tool == "loom" && overrides.iter().any(|name| name == "RUSTFLAGS" || name == "CARGO_ENCODED_RUSTFLAGS") {
        return Err("Loom qualification rejects ambient RUSTFLAGS or CARGO_ENCODED_RUSTFLAGS overrides".to_owned());
    }
    if profile.tool == "miri" && overrides.iter().any(|name| name == "MIRIFLAGS") {
        return Err("Miri qualification rejects ambient MIRIFLAGS overrides".to_owned());
    }
    Ok(())
}

pub fn inspect_toolchain(
    workspace_root: &std::path::Path,
    profile: &super::Profile,
    runtime: &ToolRuntime,
) -> Result<super::CompilerProbe, String> {
    let child = build_child_environment(profile, runtime)?;
    let cargo_stdout = exec_command(workspace_root, &runtime.cargo, &["--version"], &child.env, &profile.limits)?;
    let rustc_stdout = exec_command(workspace_root, &runtime.rustc, &["-vV"], &child.env, &profile.limits)?;
    let (rustc_release, commit_hash, host_triple) = parse_rustc_vv(&rustc_stdout)?;
    let tool_version = if profile.tool == "miri" {
        exec_command(workspace_root, &runtime.cargo, &["miri", "-V"], &child.env, &profile.limits)?
    } else {
        cargo_stdout.clone()
    };
    validate_probed_toolchain(profile, &rustc_release, &host_triple, &tool_version, child.sysroot_identity.as_deref())?;
    Ok(super::CompilerProbe {
        cargo_version: cargo_stdout.trim().to_owned(),
        rustc_release,
        commit_hash,
        host_triple,
        tool_kind: profile.tool.clone(),
        tool_version: tool_version.trim().to_owned(),
        sysroot_identity: child.sysroot_identity,
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
            .map_err(|error| format!("construct probe outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("execute probe `{}`: {error}", program.display()))?;
    if output.disposition != bounded_exec::Disposition::Succeeded || output.exit_code != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr.bytes);
        return Err(format!("probe `{}` failed ({:?}): {stderr}", program.display(), output.disposition));
    }
    String::from_utf8(output.stdout.bytes).map_err(|error| format!("probe stdout is not UTF-8: {error}"))
}

fn parse_rustc_vv(stdout: &str) -> Result<(String, String, String), String> {
    let mut release = None;
    let mut commit_hash = None;
    let mut host = None;
    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("release:") {
            release = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("commit-hash:") {
            commit_hash = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("host:") {
            host = Some(value.trim().to_owned());
        }
    }
    Ok((
        release.ok_or_else(|| "rustc -vV output missing `release:`".to_owned())?,
        commit_hash.ok_or_else(|| "rustc -vV output missing `commit-hash:`".to_owned())?,
        host.ok_or_else(|| "rustc -vV output missing `host:`".to_owned())?,
    ))
}

fn validate_probed_toolchain(
    profile: &super::Profile,
    rustc_release: &str,
    host_triple: &str,
    tool_version: &str,
    sysroot_identity: Option<&str>,
) -> Result<(), String> {
    let Some(miri) = profile.miri.as_ref() else {
        return Ok(());
    };
    if rustc_release != miri.toolchain_pin {
        return Err(format!("probed rustc `{rustc_release}` != pinned Miri `{}`", miri.toolchain_pin));
    }
    if host_triple != miri.target_triple {
        return Err(format!("probed host `{host_triple}` != pinned Miri target `{}`", miri.target_triple));
    }
    if !tool_version.contains("miri") {
        return Err(format!("cargo miri probe returned unexpected tool version `{tool_version}`"));
    }
    if sysroot_identity.is_none_or(str::is_empty) {
        return Err("Miri qualification requires an observed sysroot identity".to_owned());
    }
    Ok(())
}
