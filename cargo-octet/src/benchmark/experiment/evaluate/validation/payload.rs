pub(super) fn validate(
    observation: &crate::benchmark::experiment::model::ToolObservation,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(2 + observation.findings.len() + observation.suppressions.len());
    if observation.scope.scope_identity != super::super::scope_identity(&observation.scope) {
        diagnostics.push(format!("experiment observation `{}` scope identity is stale", observation.observation_id));
    }
    if observation.stdout_identity != crate::benchmark::prefixed_identity(observation.stdout.as_bytes())
        || observation.stderr_identity != crate::benchmark::prefixed_identity(observation.stderr.as_bytes())
    {
        diagnostics
            .push(format!("experiment observation `{}` raw output identity is stale", observation.observation_id));
    }
    validate_status(observation, diagnostics);
    for finding in &observation.findings {
        if finding.case_id.is_empty()
            || finding.rule_id.is_empty()
            || finding.path.is_empty()
            || finding.message.is_empty()
            || observation.case_id.as_deref() != Some(finding.case_id.as_str())
        {
            diagnostics.push(format!("experiment observation `{}` has an invalid finding", observation.observation_id));
        }
    }
    for suppression in &observation.suppressions {
        if suppression.case_id.is_empty()
            || suppression.rule_id.is_empty()
            || suppression.reason.is_empty()
            || observation.case_id.as_deref() != Some(suppression.case_id.as_str())
        {
            diagnostics
                .push(format!("experiment observation `{}` has an invalid suppression", observation.observation_id));
        }
    }
}

fn validate_status(observation: &crate::benchmark::experiment::model::ToolObservation, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(2);
    match observation.status {
        crate::benchmark::experiment::model::ObservationStatus::Completed => {
            if observation.exit_code.is_none() || observation.wall_duration_ns.is_none() {
                diagnostics.push(format!(
                    "experiment completed observation `{}` lacks exit or duration",
                    observation.observation_id
                ));
            }
            if observation.compile_status == crate::benchmark::experiment::model::CompileStatus::Unknown {
                diagnostics.push(format!(
                    "experiment completed observation `{}` has unknown compile status",
                    observation.observation_id
                ));
            }
        }
        crate::benchmark::experiment::model::ObservationStatus::TimedOut
        | crate::benchmark::experiment::model::ObservationStatus::Failed => {
            if observation.failure_reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics.push(format!(
                    "experiment failed observation `{}` has no failure reason",
                    observation.observation_id
                ));
            }
        }
        crate::benchmark::experiment::model::ObservationStatus::Unavailable
        | crate::benchmark::experiment::model::ObservationStatus::Unsupported => {
            if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics
                    .push(format!("experiment unsupported observation `{}` has no reason", observation.observation_id));
            }
        }
        crate::benchmark::experiment::model::ObservationStatus::NotRun => {
            if observation.failure_reason.as_deref().unwrap_or_default().is_empty()
                && observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
            {
                diagnostics
                    .push(format!("experiment not-run observation `{}` has no reason", observation.observation_id));
            }
        }
        crate::benchmark::experiment::model::ObservationStatus::Unparsed => {
            if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
                && observation.failure_reason.as_deref().unwrap_or_default().is_empty()
            {
                diagnostics
                    .push(format!("experiment unparsed observation `{}` has no reason", observation.observation_id));
            }
        }
    }
}
