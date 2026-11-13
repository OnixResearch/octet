use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(crate) fn validate(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(1 + observations.observations.len() * 7);
    if observations.schema_version != crate::benchmark::experiment::model::OBSERVATIONS_SCHEMA
        || observations.profile_id != profile.profile_id
    {
        diagnostics.push("experiment observations schema or profile is unsupported".to_owned());
    }
    let tool_map = crate::benchmark::experiment::model::tool_map(profile);
    let task_map = crate::benchmark::experiment::model::task_map(profile);
    let case_map = crate::benchmark::experiment::model::case_map(profile);
    let applicability = crate::benchmark::experiment::model::applicability_map(profile);
    super::availability::validate(profile, observations, &mut diagnostics);
    let mut observation_ids = BTreeSet::new();
    let mut observation_keys = BTreeSet::new();
    for observation in &observations.observations {
        if observation.observation_id.is_empty()
            || !observation_ids.insert(observation.observation_id.as_str())
            || !observation_keys.insert((
                observation.tool_id.as_str(),
                observation.task_id.as_str(),
                observation.case_id.as_deref(),
            ))
        {
            diagnostics.push("experiment observation is incomplete or duplicated".to_owned());
        }
        if !tool_map.contains_key(observation.tool_id.as_str())
            || !task_map.contains_key(observation.task_id.as_str())
            || !super::super::is_b3(&observation.source_identity)
            || observation.tool_version.is_empty()
            || observation.environment.target.is_empty()
            || observation.environment.host_os.is_empty()
            || observation.environment.host_arch.is_empty()
            || observation.environment.rust_toolchain.is_empty()
        {
            diagnostics.push(format!("experiment observation `{}` is incomplete", observation.observation_id));
        }
        super::payload::validate(observation, &mut diagnostics);
        validate_binding(&case_map, &applicability, observation, &mut diagnostics);
    }

    validate_retained_cases(profile, observations, &mut diagnostics);
    super::adoption::validate(profile, observations, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_binding(
    case_map: &BTreeMap<&str, &crate::benchmark::experiment::model::CorpusCase>,
    applicability: &BTreeMap<(&str, &str), &crate::benchmark::experiment::model::Applicability>,
    observation: &crate::benchmark::experiment::model::ToolObservation,
    diagnostics: &mut Vec<String>,
) {
    match observation.case_id.as_deref() {
        Some(case_id) => match case_map.get(case_id) {
            Some(case) => {
                if case.task_id != observation.task_id || observation.source_identity != case.source_identity {
                    diagnostics.push(format!(
                        "experiment observation `{}` has a stale oracle source or task identity",
                        observation.observation_id
                    ));
                }
                if observation.scope.selected_sources != case.scope.selected_sources {
                    diagnostics.push(format!(
                        "experiment observation `{}` analyzed a different scope than the frozen case",
                        observation.observation_id
                    ));
                }
                validate_claim(applicability, observation, case_id, diagnostics);
                if case.expected == crate::benchmark::experiment::model::ExpectedOutcome::Finding
                    && observation.status == crate::benchmark::experiment::model::ObservationStatus::Completed
                    && observation.compile_status == crate::benchmark::experiment::model::CompileStatus::NotCompiled
                    && observation.findings.iter().any(|finding| !finding.suppressed)
                {
                    diagnostics.push(format!(
                        "experiment observation `{}` counts a noncompiling violation as detection",
                        observation.observation_id
                    ));
                }
            }
            None => diagnostics
                .push(format!("experiment observation `{}` names an unknown case", observation.observation_id)),
        },
        None => {
            if !matches!(
                observation.status,
                crate::benchmark::experiment::model::ObservationStatus::Unsupported
                    | crate::benchmark::experiment::model::ObservationStatus::Unavailable
            ) {
                diagnostics.push(format!(
                    "experiment tool-level observation `{}` is not unsupported",
                    observation.observation_id
                ));
            }
        }
    }
}

fn validate_claim(
    applicability: &BTreeMap<(&str, &str), &crate::benchmark::experiment::model::Applicability>,
    observation: &crate::benchmark::experiment::model::ToolObservation,
    case_id: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(entry) = applicability.get(&(observation.tool_id.as_str(), observation.task_id.as_str())) else {
        return;
    };
    if entry.status == crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported {
        diagnostics.push(format!(
            "experiment observation `{}` targets an unsupported applicability cell",
            observation.observation_id
        ));
    } else if !entry.case_ids.iter().any(|declared| declared == case_id) {
        diagnostics.push(format!(
            "experiment observation `{}` targets a case the tool does not claim",
            observation.observation_id
        ));
    }
}

fn validate_retained_cases(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(profile.applicability.iter().map(|entry| entry.case_ids.len()).sum());
    for entry in &profile.applicability {
        if entry.status == crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported {
            continue;
        }
        for case_id in &entry.case_ids {
            let is_present = observations.observations.iter().any(|observation| {
                observation.tool_id == entry.tool_id
                    && observation.task_id == entry.task_id
                    && observation.case_id.as_deref() == Some(case_id.as_str())
            });
            if !is_present {
                diagnostics.push(format!(
                    "experiment observation for `{}`/`{case_id}` is missing (dropped execution or timeout)",
                    entry.tool_id
                ));
            }
        }
    }
}
