pub(super) fn record_failure(
    execution: &mut super::Execution,
    failure: super::process::StepFailure,
    step: &str,
    argv: Vec<String>,
) {
    execution.steps.push(super::StepReceipt {
        step: step.to_owned(),
        argv,
        exit_code: None,
        status: super::STATUS_FAILED.to_owned(),
    });
    execution.outcome = outcome_for_failure(failure.kind).to_owned();
    execution.diagnostics = vec![failure.message];
}

pub(super) fn outcome_for_failure(kind: super::process::FailureKind) -> &'static str {
    match kind {
        super::process::FailureKind::NotFound => super::OUTCOME_UNSUPPORTED,
        super::process::FailureKind::TimedOut => super::OUTCOME_TIMEOUT,
        super::process::FailureKind::OutputLimitExceeded => super::OUTCOME_INCOMPLETE,
        super::process::FailureKind::Cancelled
        | super::process::FailureKind::Terminated
        | super::process::FailureKind::Spawn => super::OUTCOME_TOOL_ERROR,
    }
}

pub(super) fn finish_diagnostics(mut execution: super::Execution) -> super::Execution {
    execution.diagnostics.sort();
    execution.diagnostics.dedup();
    execution
}

pub(super) fn limit_receipt(limits: &super::Limits) -> super::LimitReceipt {
    super::LimitReceipt {
        wall_clock_ms: limits.wall_clock_ms,
        stdout_max_bytes: limits.stdout_max_bytes,
        stderr_max_bytes: limits.stderr_max_bytes,
        max_report_bytes: limits.max_report_bytes,
        max_memory_mb: limits.max_memory_mb,
        max_threads: limits.max_threads,
    }
}

pub(super) fn rule_receipt(rules: &super::RuleSource) -> super::RuleReceipt {
    super::RuleReceipt {
        kind: rules.kind.clone(),
        path: rules.path.clone(),
        identity: rules.identity.clone(),
        identity_source: rules.identity_source.clone(),
    }
}

pub(super) fn query_pack_receipt(pack: &super::QueryPack) -> super::QueryPackReceipt {
    super::QueryPackReceipt {
        name: pack.name.clone(),
        version: pack.version.clone(),
        identity: pack.identity.clone(),
        identity_source: pack.identity_source.clone(),
    }
}

/// Validate one execution receipt's internal binding and outcome consistency.
pub(crate) fn validate_execution(execution: &super::Execution) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if execution.schema_version != super::EXECUTION_SCHEMA {
        diagnostics.push("SARIF producer execution schema is unsupported".to_owned());
    }
    if execution.evidence_origin != super::EVIDENCE_ORIGIN_EXECUTED {
        diagnostics.push("SARIF producer execution must be labeled octet_executed".to_owned());
    }
    if execution.evidence_tier != "static_lint" {
        diagnostics.push("SARIF producer execution must retain the static_lint evidence tier".to_owned());
    }
    if execution.profile_identity != super::PROFILE_IDENTITY_VALUE {
        diagnostics.push("SARIF producer profile identity is stale".to_owned());
    }
    let profile = match super::admission::selected_profile(&execution.profile_id) {
        Ok(profile) => Some(profile),
        Err(error) => {
            diagnostics.push(error);
            None
        }
    };
    if let Some(profile) = profile.as_ref()
        && (execution.producer != profile.producer
            || execution.driver_name != profile.driver_name
            || execution.intake_profile_id != profile.intake_profile_id
            || execution.intake_profile_identity != super::super::PROFILE_IDENTITY)
    {
        diagnostics.push("SARIF producer execution disagrees with its profile".to_owned());
    }
    if !matches!(
        execution.outcome.as_str(),
        super::OUTCOME_COMPLETED
            | super::OUTCOME_FAILED
            | super::OUTCOME_UNSUPPORTED
            | super::OUTCOME_TOOL_ERROR
            | super::OUTCOME_TIMEOUT
            | super::OUTCOME_INCOMPLETE
    ) {
        diagnostics.push("SARIF producer execution outcome is unsupported".to_owned());
    }
    match source_snapshot_identity(&execution.source_files) {
        Ok(observed) if observed != execution.source_snapshot_identity => {
            diagnostics.push("SARIF producer source snapshot identity is stale".to_owned());
        }
        Err(error) => diagnostics.push(error),
        Ok(_) => {}
    }
    validate_completion(execution, &mut diagnostics);
    validate_database(execution, profile.as_ref(), &mut diagnostics);
    if let Some(report) = execution.report.as_ref() {
        if report.bytes == 0 {
            diagnostics.push("SARIF producer report must not be empty".to_owned());
        }
        if report.bytes > execution.limits.max_report_bytes {
            diagnostics.push("SARIF producer report exceeds the declared limit".to_owned());
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_completion(execution: &super::Execution, diagnostics: &mut Vec<String>) {
    if execution.outcome == super::OUTCOME_COMPLETED {
        if execution.report.is_none() {
            diagnostics.push("a completed SARIF producer execution must retain its report".to_owned());
        }
        if execution.scope.selected_paths.is_empty() || execution.source_files.is_empty() {
            diagnostics.push("a completed SARIF producer execution must establish its source scope".to_owned());
        }
        if execution.steps.last().is_none_or(|step| step.status != super::STATUS_SUCCEEDED) {
            diagnostics.push("a completed SARIF producer execution must end in a successful step".to_owned());
        }
        if execution.producer == super::PRODUCER_CODEQL
            && execution.database.as_ref().is_none_or(|database| database.analysis.is_none())
        {
            diagnostics.push("a completed CodeQL execution must record database analysis".to_owned());
        }
    }
}

fn validate_database(execution: &super::Execution, profile: Option<&super::Profile>, diagnostics: &mut Vec<String>) {
    if let Some(database) = execution.database.as_ref() {
        if database.binding.source_snapshot_identity != execution.source_snapshot_identity {
            diagnostics.push("CodeQL database binding does not match the source snapshot".to_owned());
        }
        if let Some(profile) = profile {
            let expected = profile.query_packs.iter().map(query_pack_receipt).collect::<Vec<_>>();
            if database.binding.query_packs != expected {
                diagnostics.push("CodeQL database binding does not match the selected query packs".to_owned());
            }
        }
        if database.created
            && database.creation.as_ref().is_none_or(|creation| creation.status != super::STATUS_SUCCEEDED)
        {
            diagnostics.push("CodeQL database creation was not recorded as successful".to_owned());
        }
    }
}
/// Re-validate an executed report against the existing intake profile and bind it to the execution
/// receipt through the raw report identity.
#[cfg(test)]
pub(crate) fn evaluate_executed(report: &str, execution: &super::Execution) -> super::super::Evaluation {
    let mut diagnostics = validate_execution(execution);
    if !diagnostics.is_empty() {
        return super::super::Evaluation {
            details: None,
            diagnostics,
        };
    }
    let Some(receipt) = execution.report.as_ref() else {
        diagnostics.push("SARIF producer execution did not retain a report".to_owned());
        return super::super::Evaluation {
            details: None,
            diagnostics,
        };
    };
    if receipt.identity != super::super::prefixed_identity(report.as_bytes()) {
        diagnostics.push("SARIF producer report bytes do not match the execution receipt".to_owned());
        return super::super::Evaluation {
            details: None,
            diagnostics,
        };
    }
    if execution.outcome != super::OUTCOME_COMPLETED {
        diagnostics.push(format!("SARIF producer execution outcome `{}` is not completed evidence", execution.outcome));
        return super::super::Evaluation {
            details: None,
            diagnostics,
        };
    }
    super::super::evaluate(report, &execution.intake_profile_id)
}

/// The surrogate identity of one `CodeQL` query pack pin: `name` and declared `version` only. It is
/// not byte evidence for the locally resolved pack; the analysis report binding is.
pub(crate) fn query_pack_identity(name: &str, version: &str) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::QUERY_PACK_IDENTITY_DOMAIN);
    hasher.update(&length_frame(name.len())?);
    hasher.update(name.as_bytes());
    hasher.update(&length_frame(version.len())?);
    hasher.update(version.as_bytes());
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

pub(crate) fn query_pack_set_identity(packs: &[super::QueryPack]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::QUERY_PACK_SET_IDENTITY_DOMAIN);
    for pack in packs {
        hasher.update(&length_frame(pack.name.len())?);
        hasher.update(pack.name.as_bytes());
        hasher.update(&length_frame(pack.version.len())?);
        hasher.update(pack.version.as_bytes());
    }
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

pub(super) fn length_frame(length: usize) -> Result<[u8; 8], String> {
    u64::try_from(length)
        .map(u64::to_le_bytes)
        .map_err(|error| format!("SARIF identity length is out of range: {error}"))
}

pub(crate) fn execution_bytes(execution: &super::Execution) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(execution)
        .map_err(|error| format!("serialize SARIF producer execution receipt: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn execution_identity(execution: &super::Execution) -> Result<String, String> {
    Ok(super::super::prefixed_identity(&execution_bytes(execution)?))
}

pub(crate) fn source_snapshot_identity(files: &[super::SourceFile]) -> Result<String, String> {
    let mut sorted = files.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.path.cmp(&right.path));
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::SNAPSHOT_IDENTITY_DOMAIN);
    for file in &sorted {
        hasher.update(&length_frame(file.path.len())?);
        hasher.update(file.path.as_bytes());
        hasher.update(&length_frame(file.identity.len())?);
        hasher.update(file.identity.as_bytes());
    }
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}
