use std::path::Path;

pub(super) fn collect(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    corpus_root: &Path,
    runtime: &super::Runtime,
) -> Result<crate::benchmark::experiment::model::ObservationSet, Vec<String>> {
    let corpus_root = super::shell::absolute_root(corpus_root).map_err(|error| vec![error])?;
    let mut tool_availability = Vec::with_capacity(profile.tools.len());
    let case_count = profile.applicability.iter().map(|entry| entry.case_ids.len()).sum();
    let mut observations = Vec::with_capacity(case_count);
    let mut adoption = Vec::with_capacity(profile.adoption.tools.len() * 4);
    for tool in &profile.tools {
        let probe = super::shell::probe_tool(tool, runtime);
        let has_local_measurement = append_adoption(
            &super::adoption::Measurement {
                profile,
                tool,
                availability: &probe.status,
                runtime,
            },
            &corpus_root,
            &mut adoption,
        );
        let status = if probe.status == crate::benchmark::experiment::model::Availability::Unavailable {
            crate::benchmark::experiment::model::Availability::Unavailable
        } else if has_local_measurement {
            crate::benchmark::experiment::model::Availability::Local
        } else {
            crate::benchmark::experiment::model::Availability::Recorded
        };
        append_cases(profile, tool, &probe.status, &mut observations);
        tool_availability.push(crate::benchmark::experiment::model::ToolAvailability { status, ..probe });
    }
    Ok(crate::benchmark::experiment::model::ObservationSet {
        schema_version: crate::benchmark::experiment::model::OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        tool_availability,
        observations,
        adoption,
    })
}

fn append_adoption(
    measurement: &super::adoption::Measurement<'_>,
    corpus_root: &Path,
    adoption: &mut Vec<crate::benchmark::experiment::model::AdoptionObservation>,
) -> bool {
    let profile = measurement.profile;
    let tool = measurement.tool;
    let mut has_local_measurement = false;
    for plan in profile.adoption.tools.iter().filter(|plan| plan.tool_id == tool.tool_id) {
        let (initial, incremental) =
            super::adoption::measure_target(measurement, corpus_root, &profile.adoption.clean_consumer, plan);
        has_local_measurement |= initial.measured || incremental.measured;
        adoption.push(initial);
        adoption.push(incremental);
        let target = &profile.adoption.real_consumer;
        let real_root = Path::new(&target.workspace_path);
        let (initial, incremental) = if real_root.is_dir() {
            super::adoption::measure_target(measurement, real_root, target, plan)
        } else {
            super::adoption::unavailable_pair(
                measurement,
                target,
                "immutable real-consumer selection is not checked out",
            )
        };
        adoption.push(initial);
        adoption.push(incremental);
    }
    has_local_measurement
}

fn append_cases(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    tool: &crate::benchmark::experiment::model::ToolPin,
    availability: &crate::benchmark::experiment::model::Availability,
    observations: &mut Vec<crate::benchmark::experiment::model::ToolObservation>,
) {
    let is_executable_available = *availability != crate::benchmark::experiment::model::Availability::Unavailable;
    for entry in profile.applicability.iter().filter(|entry| entry.tool_id == tool.tool_id) {
        if entry.status == crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported {
            continue;
        }
        for case_id in &entry.case_ids {
            let Some(case) = profile.corpus.iter().find(|case| case.case_id == *case_id) else {
                continue;
            };
            let scope = crate::benchmark::experiment::model::ObservationScope {
                selected_sources: case.scope.selected_sources.clone(),
                analyzed_sources: Vec::new(),
                scope_identity: String::new(),
            };
            let scope = crate::benchmark::experiment::model::ObservationScope {
                scope_identity: super::super::evaluate::scope_identity(&scope),
                ..scope
            };
            let stdout = String::new();
            let stderr = String::new();
            let status = if is_executable_available {
                crate::benchmark::experiment::model::ObservationStatus::Unparsed
            } else {
                crate::benchmark::experiment::model::ObservationStatus::Unavailable
            };
            let reason = if is_executable_available {
                "execute mode probes availability and adoption only; finding extraction requires a parser and is not claimed"
            } else {
                "executable is unavailable; unsupported scope is not clean evidence"
            };
            observations.push(crate::benchmark::experiment::model::ToolObservation {
                observation_id: format!("{}:{}:{}", tool.tool_id, entry.task_id, case.case_id),
                tool_id: tool.tool_id.clone(),
                task_id: entry.task_id.clone(),
                case_id: Some(case.case_id.clone()),
                source_identity: case.source_identity.clone(),
                tool_version: tool.version_pin.clone(),
                origin: crate::benchmark::experiment::model::MeasurementOrigin::Local,
                environment: super::shell::environment_observation(
                    profile,
                    crate::benchmark::experiment::model::CacheState::Cold,
                ),
                scope,
                status,
                compile_status: crate::benchmark::experiment::model::CompileStatus::Unknown,
                findings: Vec::new(),
                suppressions: Vec::new(),
                stdout_identity: crate::benchmark::prefixed_identity(stdout.as_bytes()),
                stderr_identity: crate::benchmark::prefixed_identity(stderr.as_bytes()),
                stdout,
                stderr,
                exit_code: None,
                wall_duration_ns: None,
                warm_state: crate::benchmark::experiment::model::WarmState::Cold,
                sample_index: 0,
                failure_reason: None,
                unavailable_reason: Some(reason.to_owned()),
            });
        }
    }
}
