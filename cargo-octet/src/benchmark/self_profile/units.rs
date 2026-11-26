//! Per-unit `.mm_profdata` collection and session archival (#25).

pub(crate) struct UnitCollectRequest<'a> {
    pub(crate) prof_dir: Option<&'a std::path::Path>,
    pub(crate) archive_prof_dir: Option<&'a std::path::Path>,
    pub(crate) cache_state: &'a str,
    pub(crate) source_hash: &'a str,
    pub(crate) toolchain: &'a str,
    pub(crate) profile: &'a super::model::DiagnosticProfileConfig,
    pub(crate) driver_blake3: Option<&'a str>,
    pub(crate) lint_lib_blake3: Option<&'a str>,
}

pub(crate) fn collect_unit_rows(
    req: &UnitCollectRequest<'_>,
) -> Result<Vec<super::model::UnitProfileObservation>, String> {
    let Some(dir) = req.prof_dir else {
        return Ok(Vec::new());
    };
    let read_dir = if let Some(archive) = req.archive_prof_dir {
        if archive.exists() {
            std::fs::remove_dir_all(archive)
                .map_err(|error| format!("clear archive prof dir {}: {error}", archive.display()))?;
        }
        std::fs::create_dir_all(archive).map_err(|error| format!("create archive prof dir: {error}"))?;
        for entry in std::fs::read_dir(dir).map_err(|error| format!("read session prof dir: {error}"))? {
            let entry = entry.map_err(|error| format!("read session entry: {error}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("mm_profdata")
                && let Some(name) = path.file_name()
            {
                std::fs::rename(&path, archive.join(name))
                    .map_err(|error| format!("archive profdata {}: {error}", path.display()))?;
            }
        }
        archive
    } else {
        dir
    };
    let mut units = collect_profdata_units(read_dir, req.cache_state, req.source_hash, req.toolchain, req.profile)?;
    for unit in &mut units {
        unit.driver_blake3 = req.driver_blake3.map(str::to_owned);
        unit.lint_lib_blake3 = req.lint_lib_blake3.map(str::to_owned);
    }
    if units.is_empty() && req.cache_state == "cold" {
        return Err("cold instrumented compilation emitted zero .mm_profdata files".to_owned());
    }
    if units.is_empty() {
        units.push(super::model::UnitProfileObservation {
            unit_label: "workspace-unchanged".to_owned(),
            host_or_target_role: "target".to_owned(),
            cache_state: req.cache_state.to_owned(),
            build_status: super::model::UnitBuildStatus::NotCompiled,
            process_id: 0,
            command_line: String::new(),
            raw_profdata_path: String::new(),
            raw_profdata_blake3: String::new(),
            raw_bytes: 0,
            compiler_identity: req.toolchain.to_owned(),
            driver_blake3: req.driver_blake3.map(str::to_owned),
            lint_lib_blake3: req.lint_lib_blake3.map(str::to_owned),
            source_blake3: req.source_hash.to_owned(),
            total_events: 0,
            query_events: 0,
            activity_events: 0,
            cache_hit_events: 0,
            artifact_size_events: 0,
            distinct_threads: 0,
            summed_query_span_ns: 0,
            top_queries: Vec::new(),
        });
    }
    Ok(units)
}

pub(crate) fn collect_profdata_units(
    dir: &std::path::Path,
    cache_state: &str,
    source_hash: &str,
    toolchain: &str,
    profile: &super::model::DiagnosticProfileConfig,
) -> Result<Vec<super::model::UnitProfileObservation>, String> {
    let mut files = Vec::with_capacity(profile.budgets.max_units.min(64));
    for entry in std::fs::read_dir(dir).map_err(|error| format!("read profdata dir {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read profdata entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("mm_profdata") {
            if files.len() >= profile.budgets.max_units {
                return Err(format!("profdata unit count exceeds budget {}", profile.budgets.max_units));
            }
            files.push(path);
        }
    }
    files.sort();
    let mut observations = Vec::with_capacity(files.len());
    for path in files {
        let bytes = std::fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let raw_blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes));
        let parsed = super::measureme::parse_mm_profdata(
            &bytes,
            profile.budgets.max_mm_profdata_bytes,
            profile.budgets.max_events_per_unit,
        )?;
        // Cargo issues internal `rustc --crate-name ___ --print=...` probe processes even on
        // warm no-op invocations; skip those non-compilation probes when collecting unit profiles.
        if parsed.cmd.contains("--crate-name ___") || parsed.cmd.contains("--print=") {
            continue;
        }
        let unit_label = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("unknown").to_owned();
        observations.push(super::model::UnitProfileObservation {
            unit_label,
            host_or_target_role: "target".to_owned(),
            cache_state: cache_state.to_owned(),
            build_status: super::model::UnitBuildStatus::Compiled,
            process_id: parsed.process_id,
            command_line: parsed.cmd,
            raw_profdata_path: path.display().to_string(),
            raw_profdata_blake3: raw_blake3,
            raw_bytes: bytes.len(),
            compiler_identity: toolchain.to_owned(),
            driver_blake3: None,
            lint_lib_blake3: None,
            source_blake3: source_hash.to_owned(),
            total_events: parsed.total_events,
            query_events: parsed.query_events,
            activity_events: parsed.activity_events,
            cache_hit_events: parsed.cache_hit_events,
            artifact_size_events: parsed.artifact_size_events,
            distinct_threads: parsed.distinct_threads,
            summed_query_span_ns: parsed.summed_query_span_ns,
            top_queries: parsed.top_queries,
        });
    }
    Ok(observations)
}
