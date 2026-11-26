//! Replay validation for retained `.mm_profdata` artifacts and self-profile summaries (#25).

pub(crate) fn verify_report(
    report: &super::model::CaptureReport,
    profile: &super::model::CaptureConfig,
    artifact_root: Option<&std::path::Path>,
) -> super::model::ReplayVerificationReport {
    let mut diagnostics = Vec::with_capacity(16);
    let mut verified_unit_count = 0usize;

    if report.schema_version != super::model::REPORT_SCHEMA_VERSION {
        diagnostics.push(format!("unsupported report schema `{}`", report.schema_version));
    }
    match super::model::compute_report_blake3(report) {
        Ok(expected) if expected == report.report_blake3 => {}
        Ok(_) => diagnostics.push("self-profile report BLAKE3 digest mismatch".to_owned()),
        Err(error) => diagnostics.push(error),
    }

    for row in &report.comparisons {
        for unit in &row.units {
            if unit.build_status == super::model::UnitBuildStatus::NotCompiled {
                continue;
            }
            if diagnostics.len() >= 128 {
                break;
            }
            if verify_unit_observation(unit, report, profile, artifact_root, &mut diagnostics) {
                verified_unit_count += 1;
            }
        }
    }

    let status = if diagnostics.is_empty() {
        "verified".to_owned()
    } else {
        "invalid".to_owned()
    };
    super::model::ReplayVerificationReport {
        schema_version: super::model::REPORT_SCHEMA_VERSION.to_owned(),
        status,
        verified_unit_count,
        diagnostics,
        non_claims: profile.non_claims.clone(),
    }
}

fn verify_unit_observation(
    unit: &super::model::UnitObservation,
    report: &super::model::CaptureReport,
    profile: &super::model::CaptureConfig,
    artifact_root: Option<&std::path::Path>,
    diagnostics: &mut Vec<String>,
) -> bool {
    if unit.raw_profdata_path.trim().is_empty() {
        diagnostics.push(format!("compiled unit `{}` has no raw_profdata_path", unit.unit_label));
        return false;
    }
    let raw_path = resolve_profdata_path(&unit.raw_profdata_path, artifact_root);
    let bytes = match std::fs::read(&raw_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostics.push(format!("missing raw profdata {}: {error}", raw_path.display()));
            return false;
        }
    };
    let actual_blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes));
    if actual_blake3 != unit.raw_profdata_blake3 {
        diagnostics.push(format!(
            "raw profdata digest mismatch for `{}`: expected {}, got {actual_blake3}",
            unit.unit_label, unit.raw_profdata_blake3
        ));
        return false;
    }
    let parsed = match super::measureme::parse_mm_profdata(
        &bytes,
        profile.budgets.max_mm_profdata_bytes,
        profile.budgets.max_events_per_unit,
    ) {
        Ok(parsed) => parsed,
        Err(error) => {
            diagnostics.push(format!("replay parse failed for `{}`: {error}", unit.unit_label));
            return false;
        }
    };
    if parsed.process_id != unit.process_id || parsed.cmd != unit.command_line {
        diagnostics.push(format!("replay process/command mismatch for `{}`", unit.unit_label));
        return false;
    }
    if parsed.total_events != unit.total_events
        || parsed.query_events != unit.query_events
        || parsed.activity_events != unit.activity_events
        || parsed.cache_hit_events != unit.cache_hit_events
        || parsed.summed_query_span_ns != unit.summed_query_span_ns
    {
        diagnostics.push(format!("replay event summary mismatch for `{}`", unit.unit_label));
        return false;
    }
    if unit.compiler_identity != report.toolchain_identity {
        diagnostics.push(format!(
            "unit `{}` compiler identity `{}` does not match report `{}`",
            unit.unit_label, unit.compiler_identity, report.toolchain_identity
        ));
        return false;
    }
    true
}

fn resolve_profdata_path(raw: &str, artifact_root: Option<&std::path::Path>) -> std::path::PathBuf {
    let path = std::path::Path::new(raw);
    if path.is_absolute() || path.exists() {
        return path.to_path_buf();
    }
    if let Some(root) = artifact_root {
        let joined = root.join(path);
        if joined.exists() {
            return joined;
        }
    }
    path.to_path_buf()
}
