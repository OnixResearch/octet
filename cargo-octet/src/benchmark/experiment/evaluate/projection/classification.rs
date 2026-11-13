pub(super) fn classify_case(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    tool: &crate::benchmark::experiment::model::ToolPin,
    case: &crate::benchmark::experiment::model::CorpusCase,
    observation: Option<&crate::benchmark::experiment::model::ToolObservation>,
    metrics: &mut crate::benchmark::experiment::model::ToolTaskMetrics,
) {
    if classify_exceptional(case, observation, metrics) {
        return;
    }
    let Some(observation) = observation else {
        metrics.missing_execution_cases = metrics.missing_execution_cases.saturating_add(1);
        metrics.reasons.push(format!("case `{}` has no retained observation", case.case_id));
        return;
    };
    if is_comparable(profile, tool, case, observation, metrics) {
        classify_findings(case, observation, metrics);
    }
}

fn classify_exceptional(
    case: &crate::benchmark::experiment::model::CorpusCase,
    observation: Option<&crate::benchmark::experiment::model::ToolObservation>,
    metrics: &mut crate::benchmark::experiment::model::ToolTaskMetrics,
) -> bool {
    match case.expected {
        crate::benchmark::experiment::model::ExpectedOutcome::Disputed => {
            metrics.disputed_cases = metrics.disputed_cases.saturating_add(1);
            metrics.reasons.push(format!("case `{}` has a disputed expected outcome", case.case_id));
            return true;
        }
        crate::benchmark::experiment::model::ExpectedOutcome::Unsupported => {
            metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
            metrics.reasons.push(format!("case `{}` is declared unsupported", case.case_id));
            return true;
        }
        crate::benchmark::experiment::model::ExpectedOutcome::EmptyScope => {
            match observation {
                Some(observation)
                    if observation.status == crate::benchmark::experiment::model::ObservationStatus::Completed
                        && observation.scope.selected_sources.is_empty()
                        && observation.scope.analyzed_sources.is_empty() =>
                {
                    metrics.empty_scope_cases = metrics.empty_scope_cases.saturating_add(1);
                    metrics.reasons.push(format!(
                        "case `{}` selected an empty scope; zero findings are not clean evidence",
                        case.case_id
                    ));
                }
                Some(observation)
                    if matches!(
                        observation.status,
                        crate::benchmark::experiment::model::ObservationStatus::Unavailable
                            | crate::benchmark::experiment::model::ObservationStatus::Unsupported
                    ) =>
                {
                    metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
                }
                Some(observation) => {
                    metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
                    metrics.reasons.push(format!(
                        "case `{}` expected an empty scope but observation `{}` analyzed another scope",
                        case.case_id, observation.observation_id
                    ));
                }
                None => {
                    metrics.missing_execution_cases = metrics.missing_execution_cases.saturating_add(1);
                }
            }
            return true;
        }
        crate::benchmark::experiment::model::ExpectedOutcome::Finding
        | crate::benchmark::experiment::model::ExpectedOutcome::Clean => {}
    }
    false
}

fn is_comparable(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    tool: &crate::benchmark::experiment::model::ToolPin,
    case: &crate::benchmark::experiment::model::CorpusCase,
    observation: &crate::benchmark::experiment::model::ToolObservation,
    metrics: &mut crate::benchmark::experiment::model::ToolTaskMetrics,
) -> bool {
    match observation.status {
        crate::benchmark::experiment::model::ObservationStatus::Unavailable
        | crate::benchmark::experiment::model::ObservationStatus::Unsupported => {
            metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
            metrics.reasons.push(format!(
                "case `{}` is unsupported for `{}`; absence is not clean evidence",
                case.case_id, tool.tool_id
            ));
            return false;
        }
        crate::benchmark::experiment::model::ObservationStatus::TimedOut
        | crate::benchmark::experiment::model::ObservationStatus::Failed
        | crate::benchmark::experiment::model::ObservationStatus::NotRun
        | crate::benchmark::experiment::model::ObservationStatus::Unparsed => {
            metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
            metrics
                .reasons
                .push(format!("case `{}` observation `{}` did not complete", case.case_id, observation.observation_id));
            return false;
        }
        crate::benchmark::experiment::model::ObservationStatus::Completed => {}
    }
    if observation.compile_status != crate::benchmark::experiment::model::CompileStatus::Compiled {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` did not compile and cannot be compared",
            case.case_id, observation.observation_id
        ));
        return false;
    }
    if observation.environment.rust_toolchain != profile.rust_toolchain
        || observation.environment.target != profile.sample_plan.target
        || observation.environment.cache_state != profile.sample_plan.cache_state
        || observation.warm_state != profile.sample_plan.warm_state
        || observation.tool_version != tool.version_pin
    {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` has a differing toolchain, tool, or cache identity",
            case.case_id, observation.observation_id
        ));
        return false;
    }
    let has_matching_scope = observation.scope.selected_sources == case.scope.selected_sources
        && observation.scope.analyzed_sources.len() == case.scope.expected_analyzed_sources;
    if !has_matching_scope {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` analyzed a differing scope",
            case.case_id, observation.observation_id
        ));
        return false;
    }
    true
}

fn classify_findings(
    case: &crate::benchmark::experiment::model::CorpusCase,
    observation: &crate::benchmark::experiment::model::ToolObservation,
    metrics: &mut crate::benchmark::experiment::model::ToolTaskMetrics,
) {
    let is_detected = observation.findings.iter().any(|finding| !finding.suppressed);
    let is_suppressed =
        observation.findings.iter().any(|finding| finding.suppressed) || !observation.suppressions.is_empty();
    match case.expected {
        crate::benchmark::experiment::model::ExpectedOutcome::Finding => {
            if is_detected {
                metrics.true_positive = metrics.true_positive.saturating_add(1);
            } else if is_suppressed {
                metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
                metrics.suppressed_cases = metrics.suppressed_cases.saturating_add(1);
                metrics.reasons.push(format!("case `{}` finding was suppressed", case.case_id));
            } else {
                metrics.false_negative = metrics.false_negative.saturating_add(1);
            }
        }
        crate::benchmark::experiment::model::ExpectedOutcome::Clean => {
            if is_detected {
                metrics.false_positive = metrics.false_positive.saturating_add(1);
            } else if is_suppressed {
                metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
                metrics.suppressed_cases = metrics.suppressed_cases.saturating_add(1);
                metrics.reasons.push(format!("case `{}` has an unexpected suppression", case.case_id));
            } else {
                metrics.true_negative = metrics.true_negative.saturating_add(1);
            }
        }
        crate::benchmark::experiment::model::ExpectedOutcome::EmptyScope
        | crate::benchmark::experiment::model::ExpectedOutcome::Disputed
        | crate::benchmark::experiment::model::ExpectedOutcome::Unsupported => {}
    }
}
