//! Bounded execution runner for compiler `-Z self-profile` and configuration comparison (#25).

pub(crate) struct RunOptions<'a> {
    pub profile: &'a super::model::DiagnosticProfileConfig,
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
    profile: &'a super::model::DiagnosticProfileConfig,
    driver_path: Option<&'a std::path::Path>,
    lint_library_path: Option<&'a std::path::Path>,
}

pub(crate) fn execute(options: &RunOptions<'_>) -> Result<super::model::SelfProfileReport, String> {
    super::phases::validate_profile_config(options.profile)?;
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

    let (baseline_run, cold_run, warm_run) =
        run_baseline_and_cache_runs(options, &target_base, &raw_dir, &source_hash, &toolchain)?;
    let mut comparisons = vec![baseline_run.clone(), cold_run.clone(), warm_run];
    if let Some(rel_file) = options.controlled_change_file {
        let session_dir = raw_dir.join("session");
        let prof_target = target_base.join("instrumented");
        comparisons.push(run_controlled_change(options, &prof_target, &session_dir, &raw_dir, rel_file, &toolchain)?);
    }
    if options.is_compare_configurations {
        append_configuration_comparisons(options, &target_base, &raw_dir, &source_hash, &toolchain, &mut comparisons)?;
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

    let mut report = super::model::SelfProfileReport {
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

fn plain_run(
    options: &RunOptions<'_>,
    target_dir: &std::path::Path,
    prof_dir: Option<&std::path::Path>,
    archive_prof_dir: Option<&std::path::Path>,
    cache_state: &str,
    source_hash: &str,
    toolchain: &str,
) -> Result<super::model::ConfigurationComparisonRow, String> {
    run_cargo_unit(&UnitRunRequest {
        workspace_root: options.workspace_root,
        target_dir,
        prof_dir,
        archive_prof_dir,
        phase_dir: None,
        config_id: super::model::ConfigMode::PlainCompiler,
        cache_state,
        source_hash,
        toolchain,
        profile: options.profile,
        driver_path: None,
        lint_library_path: None,
    })
}

fn run_baseline_and_cache_runs(
    options: &RunOptions<'_>,
    target_base: &std::path::Path,
    raw_dir: &std::path::Path,
    source_hash: &str,
    toolchain: &str,
) -> Result<
    (
        super::model::ConfigurationComparisonRow,
        super::model::ConfigurationComparisonRow,
        super::model::ConfigurationComparisonRow,
    ),
    String,
> {
    let baseline_target = target_base.join("baseline-uninstrumented");
    let prof_target = target_base.join("instrumented");
    let session_dir = raw_dir.join("session");
    let baseline_run = plain_run(options, &baseline_target, None, None, "cold", source_hash, toolchain)?;
    let cold_prof_dir = raw_dir.join("cold");
    let cold_run =
        plain_run(options, &prof_target, Some(&session_dir), Some(&cold_prof_dir), "cold", source_hash, toolchain)?;
    let warm_prof_dir = raw_dir.join("warm-noop");
    let warm_run = plain_run(
        options,
        &prof_target,
        Some(&session_dir),
        Some(&warm_prof_dir),
        "warm-noop",
        source_hash,
        toolchain,
    )?;
    Ok((baseline_run, cold_run, warm_run))
}

fn append_configuration_comparisons(
    options: &RunOptions<'_>,
    target_base: &std::path::Path,
    raw_dir: &std::path::Path,
    source_hash: &str,
    toolchain: &str,
    comparisons: &mut Vec<super::model::ConfigurationComparisonRow>,
) -> Result<(), String> {
    let phases_root = options.artifact_dir.join("phase-spans");
    let driver_env = std::env::var_os("OCTET_UI_DRIVER").map(std::path::PathBuf::from);
    let lib_env = std::env::var_os("OCTET_UI_LIBRARY").map(std::path::PathBuf::from);
    let driver = options.driver_path.or(driver_env.as_deref());
    let library = options.lint_library_path.or(lib_env.as_deref());
    for spec in &options.profile.configurations {
        if spec.config_id == super::model::ConfigMode::PlainCompiler || comparisons.len() >= 16 {
            continue;
        }
        let tag = format!("{:?}", spec.config_id).to_lowercase();
        let mode_dir = raw_dir.join(&tag);
        let mode_phases = phases_root.join(&tag);
        let mode_target = target_base.join(format!("cmp-{tag}"));
        let row = run_cargo_unit(&UnitRunRequest {
            workspace_root: options.workspace_root,
            target_dir: &mode_target,
            prof_dir: Some(&mode_dir),
            archive_prof_dir: None,
            phase_dir: Some(&mode_phases),
            config_id: spec.config_id,
            cache_state: "cold",
            source_hash,
            toolchain,
            profile: options.profile,
            driver_path: driver,
            lint_library_path: library,
        })?;
        comparisons.push(row);
    }
    Ok(())
}

fn run_controlled_change(
    options: &RunOptions<'_>,
    prof_target: &std::path::Path,
    session_prof_dir: &std::path::Path,
    raw_dir: &std::path::Path,
    rel_file: &std::path::Path,
    toolchain: &str,
) -> Result<super::model::ConfigurationComparisonRow, String> {
    let target_file = options.workspace_root.join(rel_file);
    let original = std::fs::read(&target_file)
        .map_err(|error| format!("read controlled change file {}: {error}", target_file.display()))?;
    let mut modified = original.clone();
    modified.extend_from_slice(b"\n// octet-self-profile-controlled-rebuild\n");
    std::fs::write(&target_file, &modified)
        .map_err(|error| format!("write controlled change file {}: {error}", target_file.display()))?;

    let new_hash = super::phases::hash_workspace_sources(options.workspace_root)?;
    let change_dir = raw_dir.join("controlled-rebuild");
    let result = run_cargo_unit(&UnitRunRequest {
        workspace_root: options.workspace_root,
        target_dir: prof_target,
        prof_dir: Some(session_prof_dir),
        archive_prof_dir: Some(&change_dir),
        phase_dir: None,
        config_id: super::model::ConfigMode::PlainCompiler,
        cache_state: "controlled-rebuild",
        source_hash: &new_hash,
        toolchain,
        profile: options.profile,
        driver_path: None,
        lint_library_path: None,
    });
    if let Err(error) = std::fs::write(&target_file, &original) {
        return Err(format!("restore controlled change file {}: {error}", target_file.display()));
    }
    result
}

fn run_cargo_unit(req: &UnitRunRequest<'_>) -> Result<super::model::ConfigurationComparisonRow, String> {
    if let Some(dir) = req.prof_dir {
        if dir.exists() {
            std::fs::remove_dir_all(dir).map_err(|error| format!("clear prof dir {}: {error}", dir.display()))?;
        }
        std::fs::create_dir_all(dir).map_err(|error| format!("create prof dir: {error}"))?;
    }
    if let Some(dir) = req.phase_dir {
        if dir.exists() {
            std::fs::remove_dir_all(dir).map_err(|error| format!("clear phase dir {}: {error}", dir.display()))?;
        }
        std::fs::create_dir_all(dir).map_err(|error| format!("create phase dir: {error}"))?;
    }

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
