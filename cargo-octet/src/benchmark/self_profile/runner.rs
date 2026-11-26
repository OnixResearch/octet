//! Bounded execution runner for compiler `-Z self-profile` and configuration comparison (#25).

pub(crate) struct RunOptions<'a> {
    pub profile: &'a super::model::CaptureConfig,
    pub profile_config_blake3: &'a str,
    pub workspace_root: &'a std::path::Path,
    pub artifact_dir: &'a std::path::Path,
    pub is_compare_configurations: bool,
    pub controlled_change_file: Option<&'a std::path::Path>,
    pub driver_path: Option<&'a std::path::Path>,
    pub lint_library_path: Option<&'a std::path::Path>,
}

struct UnitRunRequest<'a> {
    workspace_root: &'a std::path::Path,
    target_dir: &'a std::path::Path,
    prof_dir: Option<&'a std::path::Path>,
    archive_prof_dir: Option<&'a std::path::Path>,
    phase_dir: Option<&'a std::path::Path>,
    config_id: super::model::ConfigMode,
    cache_state: &'a str,
    source_hash: &'a str,
    toolchain: &'a str,
    profile: &'a super::model::CaptureConfig,
    driver_path: Option<&'a std::path::Path>,
    lint_library_path: Option<&'a std::path::Path>,
}

struct RunEnv<'a> {
    target_base: &'a std::path::Path,
    raw_dir: &'a std::path::Path,
    source_hash: &'a str,
    toolchain: &'a str,
}

pub(crate) fn execute(options: &RunOptions<'_>) -> Result<super::model::CaptureReport, String> {
    super::phases::validate_config(options.profile)?;
    std::fs::create_dir_all(options.artifact_dir)
        .map_err(|error| format!("create self-profile artifact dir: {error}"))?;
    let raw_dir = options.artifact_dir.join("raw-profdata");
    std::fs::create_dir_all(&raw_dir).map_err(|error| format!("create raw-profdata dir: {error}"))?;
    let source_hash = super::phases::hash_workspace_sources(options.workspace_root)?;
    let toolchain = super::phases::rustc_version_line()?;
    let target_base = options.artifact_dir.join("target-isolated");
    if target_base.exists() {
        std::fs::remove_dir_all(&target_base)
            .map_err(|error| format!("clear isolated target dir {}: {error}", target_base.display()))?;
    }
    let env_ctx = RunEnv {
        target_base: &target_base,
        raw_dir: &raw_dir,
        source_hash: &source_hash,
        toolchain: &toolchain,
    };
    let (baseline_run, cold_run, warm_run) = run_baseline_and_cache_runs(options, &env_ctx)?;
    let mut comparisons = vec![baseline_run.clone(), cold_run.clone(), warm_run];
    if let Some(rel_file) = options.controlled_change_file {
        comparisons.push(run_controlled_change(options, &env_ctx, rel_file)?);
    }
    if options.is_compare_configurations {
        append_configuration_comparisons(options, &env_ctx, &mut comparisons)?;
    }
    let overhead_bps = if baseline_run.wall_elapsed_ns == 0 {
        0
    } else {
        cold_run
            .wall_elapsed_ns
            .saturating_mul(10_000)
            .checked_div(baseline_run.wall_elapsed_ns)
            .ok_or_else(|| "overhead computation divisor zero".to_owned())?
    };
    let mut report = super::model::CaptureReport {
        schema_version: super::model::REPORT_SCHEMA_VERSION.to_owned(),
        profile_id: options.profile.profile_id.clone(),
        workspace_source_blake3: source_hash,
        toolchain_identity: toolchain,
        profile_config_blake3: options.profile_config_blake3.to_owned(),
        baseline_uninstrumented_wall_ns: baseline_run.wall_elapsed_ns,
        instrumented_cold_wall_ns: cold_run.wall_elapsed_ns,
        profiling_overhead_basis_points: overhead_bps,
        comparisons,
        unattributed_boundaries: vec![
            "rustc early_lint_checks and late_lint_checks query totals include compiler AST/HIR traversal overhead and do not isolate custom Dylint lint cost without explicit Octet phase spans".to_owned(),
            "Summed query durations across nested or parallel threads are attribution counters, not wall elapsed time".to_owned(),
        ],
        non_claims: options.profile.non_claims.clone(),
        report_blake3: String::new(),
    };
    report.report_blake3 = super::model::compute_report_blake3(&report)?;
    Ok(report)
}

fn run_baseline_and_cache_runs(
    options: &RunOptions<'_>,
    env_ctx: &RunEnv<'_>,
) -> Result<
    (
        super::model::ConfigurationComparisonRow,
        super::model::ConfigurationComparisonRow,
        super::model::ConfigurationComparisonRow,
    ),
    String,
> {
    let baseline_target = env_ctx.target_base.join("baseline-uninstrumented");
    let prof_target = env_ctx.target_base.join("instrumented");
    let session_dir = env_ctx.raw_dir.join("session");
    let cold_prof_dir = env_ctx.raw_dir.join("cold");
    let warm_prof_dir = env_ctx.raw_dir.join("warm-noop");
    let specs = [
        (&baseline_target, None, None, "cold"),
        (&prof_target, Some(session_dir.as_path()), Some(cold_prof_dir.as_path()), "cold"),
        (&prof_target, Some(session_dir.as_path()), Some(warm_prof_dir.as_path()), "warm-noop"),
    ];
    let mut rows = Vec::with_capacity(3);
    for (target_dir, prof_dir, archive_prof_dir, cache_state) in specs {
        rows.push(execute_unit_run(&UnitRunRequest {
            workspace_root: options.workspace_root,
            target_dir,
            prof_dir,
            archive_prof_dir,
            phase_dir: None,
            config_id: super::model::ConfigMode::PlainCompiler,
            cache_state,
            source_hash: env_ctx.source_hash,
            toolchain: env_ctx.toolchain,
            profile: options.profile,
            driver_path: None,
            lint_library_path: None,
        })?);
    }
    let warm = rows.pop().ok_or_else(|| "missing warm run".to_owned())?;
    let cold = rows.pop().ok_or_else(|| "missing cold run".to_owned())?;
    let base = rows.pop().ok_or_else(|| "missing baseline run".to_owned())?;
    Ok((base, cold, warm))
}

fn append_configuration_comparisons(
    options: &RunOptions<'_>,
    env_ctx: &RunEnv<'_>,
    comparisons: &mut Vec<super::model::ConfigurationComparisonRow>,
) -> Result<(), String> {
    let phases_root = options.artifact_dir.join("phase-spans");
    for spec in &options.profile.configurations {
        if spec.config_id == super::model::ConfigMode::PlainCompiler || comparisons.len() >= 16 {
            continue;
        }
        let tag = format!("{:?}", spec.config_id).to_lowercase();
        let row = execute_unit_run(&UnitRunRequest {
            workspace_root: options.workspace_root,
            target_dir: &env_ctx.target_base.join(format!("cmp-{tag}")),
            prof_dir: Some(&env_ctx.raw_dir.join(&tag)),
            archive_prof_dir: None,
            phase_dir: Some(&phases_root.join(&tag)),
            config_id: spec.config_id,
            cache_state: "cold",
            source_hash: env_ctx.source_hash,
            toolchain: env_ctx.toolchain,
            profile: options.profile,
            driver_path: options.driver_path,
            lint_library_path: options.lint_library_path,
        })?;
        comparisons.push(row);
    }
    Ok(())
}

fn run_controlled_change(
    options: &RunOptions<'_>,
    env_ctx: &RunEnv<'_>,
    rel_file: &std::path::Path,
) -> Result<super::model::ConfigurationComparisonRow, String> {
    let prof_target = env_ctx.target_base.join("instrumented");
    let session_prof_dir = env_ctx.raw_dir.join("session");
    let target_file = options.workspace_root.join(rel_file);
    let original = std::fs::read(&target_file)
        .map_err(|error| format!("read controlled change file {}: {error}", target_file.display()))?;
    let mut modified = original.clone();
    modified.extend_from_slice(b"\n// octet-self-profile-controlled-rebuild\n");
    std::fs::write(&target_file, &modified)
        .map_err(|error| format!("write controlled change file {}: {error}", target_file.display()))?;
    let new_hash = super::phases::hash_workspace_sources(options.workspace_root)?;
    let change_dir = env_ctx.raw_dir.join("controlled-rebuild");
    let result = execute_unit_run(&UnitRunRequest {
        workspace_root: options.workspace_root,
        target_dir: &prof_target,
        prof_dir: Some(&session_prof_dir),
        archive_prof_dir: Some(&change_dir),
        phase_dir: None,
        config_id: super::model::ConfigMode::PlainCompiler,
        cache_state: "controlled-rebuild",
        source_hash: &new_hash,
        toolchain: env_ctx.toolchain,
        profile: options.profile,
        driver_path: None,
        lint_library_path: None,
    });
    if let Err(error) = std::fs::write(&target_file, &original) {
        return Err(format!("restore controlled change file {}: {error}", target_file.display()));
    }
    result
}

fn reset_clean_dir(dir: Option<&std::path::Path>, label: &str) -> Result<(), String> {
    let Some(dir) = dir else {
        return Ok(());
    };
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|error| format!("clear {label} dir {}: {error}", dir.display()))?;
    }
    std::fs::create_dir_all(dir).map_err(|error| format!("create {label} dir: {error}"))
}

fn execute_unit_run(req: &UnitRunRequest<'_>) -> Result<super::model::ConfigurationComparisonRow, String> {
    reset_clean_dir(req.prof_dir, "prof")?;
    reset_clean_dir(req.phase_dir, "phase")?;

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("check")
        .arg("--workspace")
        .arg("--lib")
        .current_dir(req.workspace_root)
        .env("CARGO_TARGET_DIR", req.target_dir);
    if let Some(dir) = req.prof_dir {
        let flags =
            format!("-Z self-profile={} -Z self-profile-events={}", dir.display(), req.profile.self_profile_events);
        cmd.env("RUSTFLAGS", flags);
    }
    if req.config_id != super::model::ConfigMode::PlainCompiler
        && let (Some(driver), Some(library)) = (req.driver_path, req.lint_library_path)
    {
        let libs_json = serde_json::to_string(&[library]).map_err(|e| format!("encode DYLINT_LIBS: {e}"))?;
        cmd.env("RUSTC_WORKSPACE_WRAPPER", driver).env("DYLINT_NO_DEPS", "1").env("DYLINT_LIBS", libs_json);
    }
    if let Some(dir) = req.phase_dir {
        cmd.env("OCTET_SELF_PROFILE_PHASE_DIR", dir);
    }

    let started = std::time::Instant::now();
    let output = cmd.output().map_err(|error| format!("run cargo check for self-profile: {error}"))?;
    let wall_elapsed_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| "self-profile wall elapsed time overflow".to_owned())?;
    let exit_code = output.status.code().unwrap_or(1);
    if !output.status.success() {
        return Err(format!(
            "cargo check failed under {:?}/{} (exit {exit_code}): {}",
            req.config_id,
            req.cache_state,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    collect_unit_comparison_row(req, wall_elapsed_ns, exit_code)
}

fn collect_unit_comparison_row(
    req: &UnitRunRequest<'_>,
    wall_elapsed_ns: u64,
    exit_code: i32,
) -> Result<super::model::ConfigurationComparisonRow, String> {
    let driver_hash = super::phases::file_blake3_tag(req.driver_path)?;
    let lib_hash = super::phases::file_blake3_tag(req.lint_library_path)?;
    let units = super::units::collect_unit_rows(&super::units::UnitCollectRequest {
        prof_dir: req.prof_dir,
        archive_prof_dir: req.archive_prof_dir,
        cache_state: req.cache_state,
        source_hash: req.source_hash,
        toolchain: req.toolchain,
        profile: req.profile,
        driver_blake3: driver_hash.as_deref(),
        lint_lib_blake3: lib_hash.as_deref(),
    })?;
    let spec_note = req
        .profile
        .configurations
        .iter()
        .find(|spec| spec.config_id == req.config_id)
        .map_or_else(String::new, |spec| spec.claim_coverage_note.clone());
    let octet_phases = super::phases::collect_phase_records(
        req.config_id,
        wall_elapsed_ns,
        req.phase_dir,
        req.profile.budgets.phase_record_budget,
        &units,
    )?;
    Ok(super::model::ConfigurationComparisonRow {
        config_id: req.config_id,
        cache_state: req.cache_state.to_owned(),
        is_instrumented: req.prof_dir.is_some(),
        wall_elapsed_ns,
        policy_exit_code: exit_code,
        policy_findings_count: 0,
        claim_coverage_note: spec_note,
        units,
        octet_phases,
    })
}
