use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(super) fn validate(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(observations.adoption.len() * 3 + profile.adoption.tools.len() * 4);
    let plans = profile.adoption.tools.iter().map(|plan| (plan.tool_id.as_str(), plan)).collect::<BTreeMap<_, _>>();
    let clean = profile.adoption.clean_consumer.selection_id.as_str();
    let real = profile.adoption.real_consumer.selection_id.as_str();
    let mut keys = BTreeSet::new();
    for observation in &observations.adoption {
        if observation.observation_id.is_empty()
            || !plans.contains_key(observation.tool_id.as_str())
            || observation.target_selection.is_empty()
            || !keys.insert((
                observation.tool_id.as_str(),
                observation.target_selection.as_str(),
                phase_key(observation.phase),
            ))
        {
            diagnostics.push("experiment adoption observation is incomplete or duplicated".to_owned());
        }
        if observation.target_selection != clean && observation.target_selection != real {
            diagnostics.push(format!(
                "experiment adoption observation `{}` names an unknown target selection",
                observation.observation_id
            ));
        }
        validate_payload(observation, diagnostics);
        let availability = observations
            .tool_availability
            .iter()
            .find(|record| record.tool_id == observation.tool_id)
            .map(|record| record.status.clone());
        if availability == Some(crate::benchmark::experiment::model::Availability::Unavailable) && observation.measured
        {
            diagnostics.push(format!(
                "experiment adoption observation `{}` claims a measurement for an unavailable tool",
                observation.observation_id
            ));
        }
    }
    for plan in &profile.adoption.tools {
        for target in [clean, real] {
            for phase in [
                crate::benchmark::experiment::model::AdoptionPhase::Initial,
                crate::benchmark::experiment::model::AdoptionPhase::Incremental,
            ] {
                if !keys.contains(&(plan.tool_id.as_str(), target, phase_key(phase))) {
                    diagnostics.push(format!(
                        "experiment adoption observation for `{}`/{target}/{} is missing",
                        plan.tool_id,
                        phase_key(phase)
                    ));
                }
            }
        }
    }
}

fn phase_key(phase: crate::benchmark::experiment::model::AdoptionPhase) -> &'static str {
    match phase {
        crate::benchmark::experiment::model::AdoptionPhase::Initial => "initial",
        crate::benchmark::experiment::model::AdoptionPhase::Incremental => "incremental",
    }
}

fn validate_payload(
    observation: &crate::benchmark::experiment::model::AdoptionObservation,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(3);
    if observation.stdout_identity != crate::benchmark::prefixed_identity(observation.stdout.as_bytes())
        || observation.stderr_identity != crate::benchmark::prefixed_identity(observation.stderr.as_bytes())
    {
        diagnostics.push(format!(
            "experiment adoption observation `{}` raw output identity is stale",
            observation.observation_id
        ));
    }
    if observation.measured {
        if observation.exit_code != Some(0) || observation.wall_duration_ns.is_none_or(|value| value == 0) {
            diagnostics.push(format!(
                "experiment adoption observation `{}` claims a measured run without a successful timing",
                observation.observation_id
            ));
        }
    } else if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
        && observation.exit_code.is_none_or(|code| code == 0)
    {
        diagnostics.push(format!(
            "experiment adoption observation `{}` is unmeasured without a reason or failing exit",
            observation.observation_id
        ));
    }
    if observation.environment.target.is_empty()
        || observation.environment.rust_toolchain.is_empty()
        || observation.dependencies.is_empty()
    {
        diagnostics
            .push(format!("experiment adoption observation `{}` lacks environment facts", observation.observation_id));
    }
}
