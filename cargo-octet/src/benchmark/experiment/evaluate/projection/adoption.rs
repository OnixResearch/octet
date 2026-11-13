use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub(super) fn derive(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> Result<Vec<crate::benchmark::experiment::model::AdoptionSummary>, Vec<String>> {
    let availability = observations
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record.status.clone()))
        .collect::<BTreeMap<_, _>>();
    let clean = profile.adoption.clean_consumer.selection_id.as_str();
    let real = profile.adoption.real_consumer.selection_id.as_str();
    let mut summaries = Vec::with_capacity(profile.adoption.tools.len());
    for plan in &profile.adoption.tools {
        let mut clean_initial = None;
        let mut clean_incremental = None;
        let mut real_initial = None;
        let mut real_incremental = None;
        let mut measured_targets = BTreeSet::new();
        let mut unavailable_reasons = BTreeSet::new();
        for observation in observations.adoption.iter().filter(|observation| observation.tool_id == plan.tool_id) {
            let duration = if observation.measured
                && observation.exit_code == Some(0)
                && observation.wall_duration_ns.is_some_and(|value| value > 0)
            {
                observation.wall_duration_ns
            } else {
                None
            };
            if duration.is_some() {
                measured_targets.insert(observation.target_selection.clone());
            } else if let Some(reason) = observation.unavailable_reason.as_ref() {
                unavailable_reasons.insert(reason.clone());
            }
            match (observation.target_selection.as_str(), &observation.phase) {
                (target, crate::benchmark::experiment::model::AdoptionPhase::Initial) if target == clean => {
                    clean_initial = duration;
                }
                (target, crate::benchmark::experiment::model::AdoptionPhase::Incremental) if target == clean => {
                    clean_incremental = duration;
                }
                (target, crate::benchmark::experiment::model::AdoptionPhase::Initial) if target == real => {
                    real_initial = duration;
                }
                (target, crate::benchmark::experiment::model::AdoptionPhase::Incremental) if target == real => {
                    real_incremental = duration;
                }
                _ => {}
            }
        }
        let incremental_over_initial_basis_points = match (clean_initial, clean_incremental) {
            (Some(initial), Some(incremental)) if initial > 0 => super::ratio_basis_points(incremental, initial),
            _ => None,
        };
        summaries.push(crate::benchmark::experiment::model::AdoptionSummary {
            tool_id: plan.tool_id.clone(),
            availability: availability
                .get(plan.tool_id.as_str())
                .cloned()
                .unwrap_or(crate::benchmark::experiment::model::Availability::Unavailable),
            setup_steps: super::count_u64(plan.setup_steps.len())?,
            config_files: super::count_u64(plan.required_config_files.len())?,
            clean_initial_ns: clean_initial,
            clean_incremental_ns: clean_incremental,
            real_initial_ns: real_initial,
            real_incremental_ns: real_incremental,
            incremental_over_initial_basis_points,
            measured_targets: measured_targets.into_iter().collect(),
            unavailable_metrics: plan.unavailable_metrics.clone(),
            unavailable_reasons: unavailable_reasons.into_iter().collect(),
        });
    }
    Ok(summaries)
}

pub(super) fn provenance(
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> crate::benchmark::experiment::model::ProvenanceSummary {
    let mut executed_locally = BTreeSet::new();
    let mut recorded_fixtures = BTreeSet::new();
    let mut unavailable = BTreeSet::new();
    for record in &observations.tool_availability {
        match record.status {
            crate::benchmark::experiment::model::Availability::Local => {
                executed_locally.insert(record.tool_id.clone());
            }
            crate::benchmark::experiment::model::Availability::Recorded => {
                recorded_fixtures.insert(record.tool_id.clone());
            }
            crate::benchmark::experiment::model::Availability::Unavailable => {
                unavailable.insert(record.tool_id.clone());
            }
        }
    }
    crate::benchmark::experiment::model::ProvenanceSummary {
        executed_locally: executed_locally.into_iter().collect(),
        recorded_fixtures: recorded_fixtures.into_iter().collect(),
        unavailable: unavailable.into_iter().collect(),
    }
}
