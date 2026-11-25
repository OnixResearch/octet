//! Diagnostic compiler self-profile and Measureme attribution (#25).

pub mod measureme;
pub mod model;
pub mod phases;
pub mod replay;
pub mod runner;
mod units;

#[cfg(test)]
mod tests;

pub(crate) const REPORT_FILE_NAME: &str = "self-profile-report.json";

/// CLI handler for `cargo octet benchmark self-profile`.
pub(crate) fn run_cli(args: &crate::cli::BenchmarkSelfProfileArgs) -> Result<i32, String> {
    let profile_bytes = std::fs::read(&args.profile_config)
        .map_err(|error| format!("read self-profile config {}: {error}", args.profile_config.display()))?;
    let profile_blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(&profile_bytes));
    let profile: model::DiagnosticProfileConfig = serde_json::from_slice(&profile_bytes)
        .map_err(|error| format!("parse self-profile config {}: {error}", args.profile_config.display()))?;

    if args.is_verify {
        let report_path = args.artifact.clone().unwrap_or_else(|| args.artifact_dir.join(REPORT_FILE_NAME));
        let report_bytes = std::fs::read(&report_path)
            .map_err(|error| format!("read self-profile report {}: {error}", report_path.display()))?;
        let report: model::SelfProfileReport = serde_json::from_slice(&report_bytes)
            .map_err(|error| format!("parse self-profile report {}: {error}", report_path.display()))?;
        let verification = replay::verify_report(&report, &profile, report_path.parent());
        let is_valid = verification.diagnostics.is_empty();
        if args.output_format == crate::cli::OutputFormat::Json {
            let text = serde_json::to_string_pretty(&verification)
                .map_err(|error| format!("serialize self-profile verification: {error}"))?;
            println!("{text}");
        } else {
            println!(
                "octet benchmark self-profile verify status={} verified_units={} diagnostics={}",
                verification.status,
                verification.verified_unit_count,
                verification.diagnostics.len()
            );
            for diag in &verification.diagnostics {
                println!("diagnostic: {diag}");
            }
        }
        return Ok(i32::from(!is_valid));
    }

    let report = runner::execute(&runner::RunOptions {
        profile: &profile,
        profile_config_blake3: &profile_blake3,
        workspace_root: &args.workspace_root,
        artifact_dir: &args.artifact_dir,
        is_compare_configurations: args.is_compare,
        controlled_change_file: args.controlled_change.as_deref(),
        driver_path: None,
        lint_library_path: None,
    })?;
    let bytes = crate::benchmark::pretty_bytes(&report, "self-profile report")?;
    std::fs::write(args.artifact_dir.join(REPORT_FILE_NAME), &bytes)
        .map_err(|error| format!("write self-profile report: {error}"))?;

    if args.output_format == crate::cli::OutputFormat::Json {
        let text = String::from_utf8(bytes).map_err(|error| format!("utf8 self-profile report: {error}"))?;
        print!("{text}");
    } else {
        println!(
            "octet benchmark self-profile status=recorded comparisons={} baseline_ns={} instrumented_cold_ns={} overhead_bps={}",
            report.comparisons.len(),
            report.baseline_uninstrumented_wall_ns,
            report.instrumented_cold_wall_ns,
            report.profiling_overhead_basis_points
        );
    }
    Ok(0)
}
