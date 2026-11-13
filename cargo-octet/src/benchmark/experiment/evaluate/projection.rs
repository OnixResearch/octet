use std::collections::BTreeMap;
use std::collections::BTreeSet;

mod adoption;
mod classification;

const BASIS_POINTS_SCALE: u128 = 10_000;

pub(super) fn derive(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> Result<crate::benchmark::experiment::model::DerivedProjection, Vec<String>> {
    let availability = observations
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record.status.clone()))
        .collect::<BTreeMap<_, _>>();
    let observation_map: BTreeMap<(&str, &str, &str), &crate::benchmark::experiment::model::ToolObservation> =
        observations
            .observations
            .iter()
            .filter_map(|observation| {
                observation
                    .case_id
                    .as_deref()
                    .map(|case_id| ((observation.tool_id.as_str(), observation.task_id.as_str(), case_id), observation))
            })
            .collect();
    let mut architecture = Vec::with_capacity(profile.applicability.len());
    let mut assurance = Vec::with_capacity(profile.applicability.len());
    for entry in &profile.applicability {
        let tool_availability = availability
            .get(entry.tool_id.as_str())
            .cloned()
            .unwrap_or(crate::benchmark::experiment::model::Availability::Unavailable);
        let metrics = cell_metrics(profile, entry, tool_availability, &observation_map)?;
        push_metrics(&mut architecture, &mut assurance, metrics);
    }
    let adoption = adoption::derive(profile, observations)?;
    let provenance = adoption::provenance(observations);
    Ok(crate::benchmark::experiment::model::DerivedProjection {
        architecture,
        assurance,
        adoption,
        provenance,
    })
}

fn cell_metrics(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    entry: &crate::benchmark::experiment::model::Applicability,
    tool_availability: crate::benchmark::experiment::model::Availability,
    observation_map: &BTreeMap<(&str, &str, &str), &crate::benchmark::experiment::model::ToolObservation>,
) -> Result<crate::benchmark::experiment::model::ToolTaskMetrics, Vec<String>> {
    let Some(task) = profile.tasks.iter().find(|task| task.task_id == entry.task_id) else {
        return Err(vec![format!(
            "experiment applicability names unknown task `{}`",
            entry.task_id
        )]);
    };
    let Some(tool) = profile.tools.iter().find(|tool| tool.tool_id == entry.tool_id) else {
        return Err(vec![format!(
            "experiment applicability names unknown tool `{}`",
            entry.tool_id
        )]);
    };

    let mut metrics = empty_metrics(entry, task.category.clone(), tool_availability);
    if entry.status == crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported {
        metrics.unsupported_cases = count_u64(task_case_count(profile, &task.task_id))?;
        return Ok(metrics);
    }
    for case_id in &entry.case_ids {
        let Some(case) = profile.corpus.iter().find(|case| case.case_id == *case_id) else {
            return Err(vec![format!("experiment applicability names unknown case `{case_id}`")]);
        };
        let observation = observation_map.get(&(entry.tool_id.as_str(), entry.task_id.as_str(), case.case_id.as_str()));
        classification::classify_case(profile, tool, case, observation.copied(), &mut metrics);
    }
    let declared = entry.case_ids.iter().collect::<BTreeSet<_>>();
    let unclaimed = profile
        .corpus
        .iter()
        .filter(|case| case.task_id == entry.task_id && !declared.contains(&case.case_id))
        .count();
    metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(count_u64(unclaimed)?);
    metrics.comparable_cases = metrics
        .true_positive
        .saturating_add(metrics.false_positive)
        .saturating_add(metrics.false_negative)
        .saturating_add(metrics.true_negative);
    metrics.precision_basis_points =
        ratio_basis_points(metrics.true_positive, metrics.true_positive.saturating_add(metrics.false_positive));
    metrics.recall_basis_points =
        ratio_basis_points(metrics.true_positive, metrics.true_positive.saturating_add(metrics.false_negative));
    metrics.reasons.sort();
    metrics.reasons.dedup();
    Ok(metrics)
}

fn push_metrics(
    architecture: &mut Vec<crate::benchmark::experiment::model::ToolTaskMetrics>,
    assurance: &mut Vec<crate::benchmark::experiment::model::ToolTaskMetrics>,
    metrics: crate::benchmark::experiment::model::ToolTaskMetrics,
) {
    match metrics.category {
        crate::benchmark::experiment::model::Category::Architecture => architecture.push(metrics),
        crate::benchmark::experiment::model::Category::Assurance => assurance.push(metrics),
    }
}

fn empty_metrics(
    entry: &crate::benchmark::experiment::model::Applicability,
    category: crate::benchmark::experiment::model::Category,
    availability: crate::benchmark::experiment::model::Availability,
) -> crate::benchmark::experiment::model::ToolTaskMetrics {
    crate::benchmark::experiment::model::ToolTaskMetrics {
        category,
        task_id: entry.task_id.clone(),
        tool_id: entry.tool_id.clone(),
        applicability_status: entry.status.clone(),
        comparable_subset: entry.comparable_subset.clone(),
        availability,
        comparable_cases: 0,
        true_positive: 0,
        false_positive: 0,
        false_negative: 0,
        true_negative: 0,
        unsupported_cases: 0,
        inconclusive_cases: 0,
        incomparable_cases: 0,
        suppressed_cases: 0,
        empty_scope_cases: 0,
        disputed_cases: 0,
        missing_execution_cases: 0,
        precision_basis_points: None,
        recall_basis_points: None,
        reasons: Vec::with_capacity(entry.case_ids.len()),
    }
}

fn ratio_basis_points(numerator: u64, denominator: u64) -> Option<u64> {
    if denominator == 0 {
        return None;
    }
    u128::from(numerator)
        .checked_mul(BASIS_POINTS_SCALE)
        .and_then(|scaled| scaled.checked_div(u128::from(denominator)))
        .and_then(|value| u64::try_from(value).ok())
}

fn task_case_count(profile: &crate::benchmark::experiment::model::AnalysisProfile, task_id: &str) -> usize {
    profile.corpus.iter().filter(|case| case.task_id == task_id).count()
}

fn count_u64(count: usize) -> Result<u64, Vec<String>> {
    u64::try_from(count).map_err(|_| vec!["experiment count does not fit u64".to_owned()])
}
