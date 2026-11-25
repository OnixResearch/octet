//! Replay validation for retained `.mm_profdata` artifacts and self-profile summaries (#25).

use std::path::Path;

use super::model::DiagnosticProfileConfig;
use super::model::ReplayVerificationReport;
use super::model::SelfProfileReport;
use super::model::UnitBuildStatus;

pub(crate) fn verify_report(
    report: &SelfProfileReport,
    profile: &DiagnosticProfileConfig,
    artifact_root: Option<&Path>,
) -> ReplayVerificationReport {
    let mut diagnostics = Vec::new();
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
            if unit.build_status == UnitBuildStatus::NotCompiled {
                continue;
            }
            if unit.raw_profdata_path.trim().is_empty() {
                diagnostics.push(format!("compiled unit `{}` has no raw_profdata_path", unit.unit_label));
                continue;
            }
            let raw_path = resolve_profdata_path(&unit.raw_profdata_path, artifact_root);
            let bytes = match std::fs::read(&raw_path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    diagnostics.push(format!("missing raw profdata {}: {error}", raw_path.display()));
                    continue;
                }
            };
            let actual_blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(&bytes));
            if actual_blake3 != unit.raw_profdata_blake3 {
                diagnostics.push(format!(
                    "raw profdata digest mismatch for `{}`: expected {}, got {actual_blake3}",
                    unit.unit_label, unit.raw_profdata_blake3
                ));
                continue;
            }
            let parsed = match super::measureme::parse_mm_profdata(
                &bytes,
                profile.budgets.max_mm_profdata_bytes,
                profile.budgets.max_events_per_unit,
            ) {
                Ok(parsed) => parsed,
                Err(error) => {
                    diagnostics.push(format!("replay parse failed for `{}`: {error}", unit.unit_label));
                    continue;
                }
            };
            if parsed.process_id != unit.process_id || parsed.cmd != unit.command_line {
                diagnostics.push(format!("replay process/command mismatch for `{}`", unit.unit_label));
                continue;
            }
            if parsed.total_events != unit.total_events
                || parsed.query_events != unit.query_events
                || parsed.activity_events != unit.activity_events
                || parsed.cache_hit_events != unit.cache_hit_events
                || parsed.summed_query_span_ns != unit.summed_query_span_ns
            {
                diagnostics.push(format!("replay event summary mismatch for `{}`", unit.unit_label));
                continue;
            }
            if unit.compiler_identity != report.toolchain_identity {
                diagnostics.push(format!(
                    "unit `{}` compiler identity `{}` does not match report `{}`",
                    unit.unit_label, unit.compiler_identity, report.toolchain_identity
                ));
                continue;
            }
            verified_unit_count += 1;
        }
    }

    let status = if diagnostics.is_empty() {
        "verified".to_owned()
    } else {
        "invalid".to_owned()
    };
    ReplayVerificationReport {
        schema_version: super::model::REPORT_SCHEMA_VERSION.to_owned(),
        status,
        verified_unit_count,
        diagnostics,
        non_claims: vec![
            "Replay validates raw .mm_profdata identities and deterministically regenerates event summaries; wall-clock timing values across re-executions are not claimed deterministic.".to_owned(),
        ],
    }
}

fn resolve_profdata_path(raw: &str, artifact_root: Option<&Path>) -> std::path::PathBuf {
    let candidate = Path::new(raw);
    if candidate.is_absolute() || candidate.exists() {
        return candidate.to_path_buf();
    }
    if let Some(root) = artifact_root {
        let joined = root.join(raw);
        if joined.exists() {
            return joined;
        }
    }
    candidate.to_path_buf()
}
