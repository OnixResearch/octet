//! Opt-in cross-tool experiment surface.
//!
//! `benchmark experiment` is intentionally separate from the same-command
//! performance comparator. Recorded mode consumes a retained observation file;
//! execute mode performs bounded probes and adoption measurements. Neither mode
//! upgrades an unsupported tool or scope into clean evidence.

mod evaluate;
mod execution;
pub(crate) mod model;
mod profile;
mod report;

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

pub use execution::PRESERVED_ENVIRONMENT;
pub use execution::Runtime;

/// Runs the `benchmark experiment` CLI surface.
pub(crate) fn run_cli(args: &crate::cli::BenchmarkExperimentArgs, runtime: Option<&Runtime>) -> Result<i32, String> {
    use std::fs;
    use std::path::Path;

    if args.verify {
        return verify_cli(args);
    }

    let corpus_root: &Path = if args.execute {
        &args.workspace_root
    } else {
        Path::new(".")
    };
    let input_text = match &args.input {
        Some(path) => Some(
            fs::read_to_string(path).map_err(|error| format!("read experiment input {}: {error}", path.display()))?,
        ),
        None => None,
    };
    let mode = if args.execute {
        execution::Mode::Execute(
            runtime.ok_or_else(|| "experiment execute mode requires runtime capabilities".to_owned())?,
        )
    } else {
        execution::Mode::Recorded
    };
    let run = execution::run(mode, &args.profile, input_text.as_deref(), corpus_root)
        .map_err(|diagnostics| format!("benchmark experiment failed:\n- {}", diagnostics.join("\n- ")))?;
    fs::create_dir_all(&args.artifact_dir)
        .map_err(|error| format!("create experiment artifact directory {}: {error}", args.artifact_dir.display()))?;
    fs::write(args.artifact_dir.join(model::ARTIFACT_NAME), &run.artifact_bytes)
        .map_err(|error| format!("write experiment artifact: {error}"))?;
    fs::write(args.artifact_dir.join(model::REPORT_NAME), run.report.as_bytes())
        .map_err(|error| format!("write experiment report: {error}"))?;
    let observations_bytes = crate::benchmark::pretty_bytes(&run.observations, "experiment observations")
        .map_err(|error| format!("serialize experiment observations: {error}"))?;
    fs::write(args.artifact_dir.join(model::OBSERVATIONS_NAME), observations_bytes)
        .map_err(|error| format!("write experiment observations: {error}"))?;
    match args.output_format {
        crate::cli::OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&run.artifact)
                .map_err(|error| format!("serialize experiment artifact: {error}"))?
        ),
        crate::cli::OutputFormat::Human | crate::cli::OutputFormat::Sarif => {
            println!(
                "octet benchmark experiment profile={} status=verified artifact={} report={} observations={} tool_observations={} controls={}",
                run.artifact.profile.profile_id,
                args.artifact_dir.join(model::ARTIFACT_NAME).display(),
                args.artifact_dir.join(model::REPORT_NAME).display(),
                args.artifact_dir.join(model::OBSERVATIONS_NAME).display(),
                run.artifact.observations.len(),
                run.artifact.controls.len()
            );
            println!("non-claim: {}", model::REQUIRED_NON_CLAIM);
        }
    }
    Ok(0)
}

fn verify_cli(args: &crate::cli::BenchmarkExperimentArgs) -> Result<i32, String> {
    let artifact_path = args.artifact.clone().unwrap_or_else(|| args.artifact_dir.join(model::ARTIFACT_NAME));
    let bytes = std::fs::read(&artifact_path)
        .map_err(|error| format!("read experiment artifact {}: {error}", artifact_path.display()))?;
    let artifact: model::Artifact = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse experiment artifact {}: {error}", artifact_path.display()))?;
    let diagnostics = evaluate::verify(&args.profile, &artifact);
    let status = if diagnostics.is_empty() { "verified" } else { "invalid" };
    match args.output_format {
        crate::cli::OutputFormat::Json => {
            let report = serde_json::json!({
                "schema_version": model::ARTIFACT_SCHEMA,
                "status": status,
                "artifact": artifact_path.display().to_string(),
                "diagnostics": diagnostics,
                "non_claims": [model::REQUIRED_NON_CLAIM],
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("serialize experiment verification: {error}"))?
            );
        }
        crate::cli::OutputFormat::Human | crate::cli::OutputFormat::Sarif => {
            println!(
                "octet benchmark experiment verify artifact={} status={} diagnostics={}",
                artifact_path.display(),
                status,
                diagnostics.len()
            );
            for diagnostic in &diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            println!("non-claim: {}", model::REQUIRED_NON_CLAIM);
        }
    }
    Ok(i32::from(!diagnostics.is_empty()))
}
