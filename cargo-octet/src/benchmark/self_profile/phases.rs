//! Explicit Octet collector/engine phase span serialization and workspace hashing (#25).

pub(crate) fn write_engine_phase_record(
    phase_dir: &std::path::Path,
    shard_normalization_ns: u64,
    policy_evaluation_ns: u64,
) -> Result<(), String> {
    std::fs::create_dir_all(phase_dir)
        .map_err(|error| format!("create self-profile phase dir {}: {error}", phase_dir.display()))?;
    let record = super::model::OctetPhaseSpanRecord {
        schema_version: super::model::PHASE_SPAN_SCHEMA_VERSION.to_owned(),
        unit_label: "cargo-octet-engine".to_owned(),
        process_id: std::process::id(),
        selected_mir_analysis_ns: 0,
        value_flow_bodies_analyzed: 0,
        value_flow_steps: 0,
        value_flow_rounds: 0,
        semantic_bodies_analyzed: 0,
        shard_serialization_ns: 0,
        shard_bytes_written: 0,
        shard_normalization_ns,
        policy_evaluation_ns,
        unattributed_note: "host rustc lint pass traversal remains outside explicit Octet collector spans".to_owned(),
    };
    let bytes = crate::benchmark::pretty_bytes(&record, "engine phase span")?;
    let out = phase_dir.join(format!("engine.{}.phases.json", std::process::id()));
    std::fs::write(&out, bytes).map_err(|error| format!("write engine phase span {}: {error}", out.display()))
}

pub(crate) fn collect_phase_records(
    config_id: super::model::ConfigMode,
    wall_elapsed_ns: u64,
    phase_dir: Option<&std::path::Path>,
    max_records: usize,
    units: &[super::model::UnitProfileObservation],
) -> Result<Vec<super::model::OctetPhaseSpanRecord>, String> {
    if config_id == super::model::ConfigMode::PlainCompiler {
        return Ok(Vec::new());
    }
    if let Some(dir) = phase_dir
        && dir.is_dir()
    {
        let mut loaded = load_phase_files(dir, max_records)?;
        if !loaded.is_empty() {
            loaded.sort_by(|a, b| a.unit_label.cmp(&b.unit_label).then_with(|| a.process_id.cmp(&b.process_id)));
            return Ok(loaded);
        }
    }
    Ok(units
        .iter()
        .take(max_records)
        .map(|unit| super::model::OctetPhaseSpanRecord {
            schema_version: super::model::PHASE_SPAN_SCHEMA_VERSION.to_owned(),
            unit_label: unit.unit_label.clone(),
            process_id: unit.process_id,
            selected_mir_analysis_ns: 0,
            value_flow_bodies_analyzed: 0,
            value_flow_steps: 0,
            value_flow_rounds: 0,
            semantic_bodies_analyzed: 0,
            shard_serialization_ns: 0,
            shard_bytes_written: 0,
            shard_normalization_ns: 0,
            policy_evaluation_ns: wall_elapsed_ns,
            unattributed_note: "rustc lint pass totals outside explicit Octet spans remain host-compiler-attributed"
                .to_owned(),
        })
        .collect())
}

fn load_phase_files(
    dir: &std::path::Path,
    max_records: usize,
) -> Result<Vec<super::model::OctetPhaseSpanRecord>, String> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|error| format!("read phase dir {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read phase dir entry: {error}"))?;
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(".phases.json")) {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.len() > max_records {
        return Err(format!("phase record count {} exceeds budget {max_records}", paths.len()));
    }
    let mut records = Vec::with_capacity(paths.len());
    for path in paths {
        let bytes = std::fs::read(&path).map_err(|error| format!("read phase file {}: {error}", path.display()))?;
        let record: super::model::OctetPhaseSpanRecord =
            serde_json::from_slice(&bytes).map_err(|error| format!("parse phase file {}: {error}", path.display()))?;
        if record.schema_version != super::model::PHASE_SPAN_SCHEMA_VERSION {
            return Err(format!("unsupported phase span schema `{}` in {}", record.schema_version, path.display()));
        }
        records.push(record);
    }
    Ok(records)
}

pub(crate) fn validate_profile_config(profile: &super::model::DiagnosticProfileConfig) -> Result<(), String> {
    if profile.schema_version != super::model::CONFIG_SCHEMA_VERSION {
        return Err(format!("unsupported self-profile config schema `{}`", profile.schema_version));
    }
    if profile.rustc_format_version != super::model::EXPECTED_MEASUREME_FORMAT_VERSION {
        return Err(format!(
            "unsupported rustc_format_version {}; expected {}",
            profile.rustc_format_version,
            super::model::EXPECTED_MEASUREME_FORMAT_VERSION
        ));
    }
    if profile.non_claims.is_empty() {
        return Err("self-profile config requires non-empty non_claims".to_owned());
    }
    Ok(())
}

pub(crate) fn hash_workspace_sources(root: &std::path::Path) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    let mut files = Vec::new();
    collect_rs_files(root, root, &mut files)?;
    files.sort();
    for rel in files {
        hasher.update(rel.as_bytes());
        let bytes = std::fs::read(root.join(&rel)).map_err(|error| format!("hash source file {rel}: {error}"))?;
        hasher.update(&bytes);
    }
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

fn collect_rs_files(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|error| format!("read dir {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read dir entry: {error}"))?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name_str.as_ref(), "target" | ".git" | ".octet" | "artifacts") {
                collect_rs_files(base, &path, out)?;
            }
        } else if matches!(path.extension().and_then(|e| e.to_str()), Some("rs" | "toml"))
            && let Ok(rel) = path.strip_prefix(base)
        {
            out.push(rel.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

pub(crate) fn rustc_version_line() -> Result<String, String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|error| format!("query rustc --version: {error}"))?;
    if !output.status.success() {
        return Err("rustc --version failed".to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(crate) fn file_blake3_tag(path: Option<&std::path::Path>) -> Result<Option<String>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes = std::fs::read(path).map_err(|error| format!("read artifact {}: {error}", path.display()))?;
    Ok(Some(format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes))))
}
