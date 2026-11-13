pub(super) fn verify_report_binding(
    report: &str,
    profile: &super::Profile,
    observed_version: &str,
) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(report).map_err(|error| format!("SARIF producer report is malformed: {error}"))?;
    let run = value
        .get("runs")
        .and_then(serde_json::Value::as_array)
        .and_then(|runs| runs.first())
        .ok_or_else(|| "SARIF producer report has no run to bind".to_owned())?;
    let driver = run
        .pointer("/tool/driver")
        .ok_or_else(|| "SARIF producer report has no tool driver to bind".to_owned())?;
    if !driver
        .get("name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|name| super::super::driver_name_matches(&profile.driver_name, name))
    {
        return Err("SARIF producer report driver does not match the selected profile".to_owned());
    }
    let driver_version = driver
        .get("semanticVersion")
        .or_else(|| driver.get("version"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "SARIF producer report does not declare its driver version".to_owned())?;
    if !driver_version.starts_with(observed_version) {
        return Err(format!(
            "SARIF producer report driver version `{driver_version}` does not match the executed `{observed_version}`"
        ));
    }
    if profile.producer != super::PRODUCER_CODEQL {
        return Ok(());
    }
    let extensions: &[serde_json::Value] =
        run.pointer("/tool/extensions").and_then(serde_json::Value::as_array).map_or(&[], Vec::as_slice);
    for pack in &profile.query_packs {
        let declared = extensions
            .iter()
            .find(|extension| extension.get("name").and_then(serde_json::Value::as_str) == Some(pack.name.as_str()));
        let Some(extension) = declared else {
            return Err(format!("SARIF producer report does not name the selected query pack `{}`", pack.name));
        };
        let version = extension.get("semanticVersion").and_then(serde_json::Value::as_str).unwrap_or_default();
        if !version.starts_with(&pack.version) {
            return Err(format!(
                "SARIF producer report names query pack `{}` at `{version}`, not the pinned `{}`",
                pack.name, pack.version
            ));
        }
    }
    Ok(())
}

pub(super) struct Paths<'a> {
    pub report: &'a std::path::Path,
    pub database: &'a std::path::Path,
}

pub(super) fn run(
    profile: &super::Profile,
    request: &super::process::ExecuteRequest<'_>,
    executor: &dyn super::Executor,
    paths: &Paths<'_>,
    execution: &mut super::Execution,
) -> Result<bool, String> {
    let argv = match profile.producer.as_str() {
        super::PRODUCER_CODEQL => super::command::codeql_analyze_argv(profile, paths.database, paths.report),
        _ => super::command::semgrep_scan_argv(profile, paths.report, &execution.scope.selected_paths),
    };
    super::selection::remove_if_present(paths.report)?;
    let scan = match executor.run(&argv, request.workspace_root, &profile.limits) {
        Ok(output) => output,
        Err(failure) => {
            super::recording::record_failure(execution, failure, "analyze", argv);
            return Ok(false);
        }
    };
    let step = super::StepReceipt {
        step: "analyze".to_owned(),
        argv,
        exit_code: Some(scan.exit_code),
        status: if scan.exit_code == super::EXIT_SUCCESS {
            super::STATUS_SUCCEEDED
        } else {
            super::STATUS_FAILED
        }
        .to_owned(),
    };
    if let Some(database) = execution.database.as_mut() {
        database.analysis = Some(step.clone());
    }
    execution.steps.push(step);
    if scan.exit_code != super::EXIT_SUCCESS {
        execution.outcome = super::OUTCOME_TOOL_ERROR.to_owned();
        execution.diagnostics = vec![format!(
            "{} exited {}; no SARIF report from a failed scan is evidence",
            profile.driver_name, scan.exit_code,
        )];
        return Ok(false);
    }
    Ok(true)
}

type IngestResult = Result<String, (&'static str, Vec<String>)>;

pub(super) fn ingest(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
    path: &std::path::Path,
    execution: &mut super::Execution,
) -> IngestResult {
    let report = read_report(profile, path)?;
    execution.report = Some(super::ReportReceipt {
        bytes: u64::try_from(report.len())
            .map_err(|error| (super::OUTCOME_FAILED, vec![format!("SARIF report length is out of range: {error}")]))?,
        identity: super::super::prefixed_identity(report.as_bytes()),
    });
    let observed_version = execution.executable.as_ref().map_or("", |executable| executable.observed_version.as_str());
    verify_report_binding(&report, profile, observed_version).map_err(|error| (super::OUTCOME_FAILED, vec![error]))?;
    validate_scope_binding(&report, profile, execution)?;
    let after = super::selection::snapshot_sources(profile, workspace_root)
        .map_err(|error| (super::OUTCOME_FAILED, vec![error]))?;
    if after.identity != execution.source_snapshot_identity {
        return Err((super::OUTCOME_FAILED, vec![
            "source changed while the scanner ran; the report is not bound to one snapshot".to_owned(),
        ]));
    }
    Ok(report)
}

fn read_report(profile: &super::Profile, path: &std::path::Path) -> IngestResult {
    let bytes = super::selection::read_stable_report(path, profile.limits.max_report_bytes)
        .map_err(|failure| (super::recording::outcome_for_failure(failure.kind), vec![failure.message]))?;
    if bytes.is_empty() {
        return Err((super::OUTCOME_INCOMPLETE, vec![
            "the scanner exited successfully but wrote an empty SARIF report".to_owned(),
        ]));
    }
    String::from_utf8(bytes)
        .map_err(|error| (super::OUTCOME_FAILED, vec![format!("SARIF producer report is not UTF-8: {error}")]))
}

fn validate_scope_binding(
    report: &str,
    profile: &super::Profile,
    execution: &super::Execution,
) -> Result<(), (&'static str, Vec<String>)> {
    let evaluation = super::super::evaluate(report, &profile.intake_profile_id);
    let Some(details) = evaluation.details.as_ref() else {
        return Err((super::OUTCOME_FAILED, evaluation.diagnostics));
    };
    let selected: std::collections::BTreeSet<&str> =
        execution.source_files.iter().map(|file| file.path.as_str()).collect();
    let outside = details
        .findings
        .iter()
        .filter(|finding| !selected.contains(finding.file.as_str()))
        .map(|finding| finding.file.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if !outside.is_empty() {
        return Err((super::OUTCOME_FAILED, vec![format!(
            "SARIF report references files outside the declared scope: {}",
            outside.into_iter().collect::<Vec<_>>().join(", "),
        )]));
    }
    Ok(())
}
