pub(super) fn check(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let Some(case) = profile.corpus.iter().find(|case| {
        case.expected == crate::benchmark::experiment::model::ExpectedOutcome::Clean
            && case.category == crate::benchmark::experiment::model::Category::Architecture
    }) else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no clean architecture case".to_owned(),
        };
    };
    let Some(entry) = profile
        .applicability
        .iter()
        .find(|entry| entry.task_id == case.task_id && entry.case_ids.iter().any(|id| id == &case.case_id))
    else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no claimed clean case".to_owned(),
        };
    };
    let Ok(before) = super::super::projection::derive(profile, observations) else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::Escaped,
            evidence: "baseline projection failed".to_owned(),
        };
    };
    let before_metrics = find_metrics(&before, &entry.tool_id, &entry.task_id);
    let mut mutated = observations.clone();
    let Some(observation) = mutated.observations.iter_mut().find(|observation| {
        observation.tool_id == entry.tool_id
            && observation.task_id == entry.task_id
            && observation.case_id.as_deref() == Some(case.case_id.as_str())
    }) else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no observation for the selected clean case".to_owned(),
        };
    };
    observation.status = crate::benchmark::experiment::model::ObservationStatus::Unsupported;
    observation.unavailable_reason = Some("negative-control unsupported scope".to_owned());
    let Ok(after) = super::super::projection::derive(profile, &mutated) else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::Contained,
            evidence: "projection rejected the unsupported observation".to_owned(),
        };
    };
    let after_metrics = find_metrics(&after, &entry.tool_id, &entry.task_id);
    let has_increased_clean_evidence = after_metrics.is_some_and(|after_metrics| {
        before_metrics.is_some_and(|before_metrics| after_metrics.true_negative > before_metrics.true_negative)
    });
    if has_increased_clean_evidence {
        crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::Escaped,
            evidence: "unsupported scope increased true negatives".to_owned(),
        }
    } else {
        crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::Contained,
            evidence: "unsupported scope did not increase clean evidence".to_owned(),
        }
    }
}

fn find_metrics<'a>(
    projection: &'a crate::benchmark::experiment::model::DerivedProjection,
    tool_id: &str,
    task_id: &str,
) -> Option<&'a crate::benchmark::experiment::model::ToolTaskMetrics> {
    projection
        .architecture
        .iter()
        .chain(projection.assurance.iter())
        .find(|metrics| metrics.tool_id == tool_id && metrics.task_id == task_id)
}
