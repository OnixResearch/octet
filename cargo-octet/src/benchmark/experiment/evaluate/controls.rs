mod unsupported;

pub(super) fn run(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> Vec<crate::benchmark::experiment::model::NegativeControlRecord> {
    vec![
        control_noncompiling(profile, artifact),
        control_stale_oracle(profile, artifact),
        control_differing_scope(profile, artifact),
        control_dropped_timeout(profile, artifact),
        control_stale_results(profile, artifact),
        control_hidden_environment(profile, artifact),
        unsupported::check(profile, observations),
        control_bounded_corpus(profile, artifact),
    ]
}

fn control_record(
    control_id: &str,
    diagnostics: &[String],
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let outcome = if diagnostics.is_empty() {
        crate::benchmark::experiment::model::ControlOutcome::Escaped
    } else {
        crate::benchmark::experiment::model::ControlOutcome::Contained
    };
    let preferred = [
        "noncompiling",
        "stale oracle",
        "scope",
        "missing",
        "toolchain",
        "complete Rust",
    ];
    let evidence = diagnostics
        .iter()
        .find(|diagnostic| preferred.iter().any(|token| diagnostic.contains(token)))
        .or_else(|| diagnostics.first())
        .cloned()
        .unwrap_or_default();
    crate::benchmark::experiment::model::NegativeControlRecord {
        control_id: control_id.to_owned(),
        outcome,
        evidence,
    }
}

fn control_noncompiling(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let target = mutated.observations.iter_mut().find(|observation| {
        observation
            .case_id
            .as_deref()
            .and_then(|case_id| profile.corpus.iter().find(|case| case.case_id == case_id))
            .is_some_and(|case| case.expected == crate::benchmark::experiment::model::ExpectedOutcome::Finding)
            && observation.findings.iter().any(|finding| !finding.suppressed)
    });
    if let Some(observation) = target {
        observation.compile_status = crate::benchmark::experiment::model::CompileStatus::NotCompiled;
    } else {
        let Some(observation) = mutated.observations.iter_mut().find(|observation| {
            observation.case_id.as_deref().is_some_and(|case_id| {
                profile.corpus.iter().any(|case| {
                    case.case_id == case_id
                        && case.expected == crate::benchmark::experiment::model::ExpectedOutcome::Finding
                })
            })
        }) else {
            return crate::benchmark::experiment::model::NegativeControlRecord {
                control_id: "noncompiling-violation-counted-as-detection".to_owned(),
                outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
                evidence: "no violating case observation".to_owned(),
            };
        };
        let Some(case) = observation
            .case_id
            .as_deref()
            .and_then(|case_id| profile.corpus.iter().find(|case| case.case_id == case_id))
        else {
            return crate::benchmark::experiment::model::NegativeControlRecord {
                control_id: "noncompiling-violation-counted-as-detection".to_owned(),
                outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
                evidence: "violating case is missing from the profile".to_owned(),
            };
        };
        observation.status = crate::benchmark::experiment::model::ObservationStatus::Completed;
        observation.compile_status = crate::benchmark::experiment::model::CompileStatus::NotCompiled;
        observation.exit_code = Some(0);
        observation.wall_duration_ns = Some(1);
        observation.findings.push(crate::benchmark::experiment::model::ObservedFinding {
            case_id: case.case_id.clone(),
            rule_id: "negative-control.noncompiling".to_owned(),
            path: case.source_path.clone(),
            line: 1,
            message: "negative-control finding attached to a noncompiling case".to_owned(),
            suppressed: false,
        });
    }
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("noncompiling-violation-counted-as-detection", &diagnostics)
}

fn control_stale_oracle(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.first_mut() else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "renamed-stale-oracle".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no retained observations".to_owned(),
        };
    };
    observation.source_identity = crate::benchmark::prefixed_identity(b"renamed-oracle");
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("renamed-stale-oracle", &diagnostics)
}

fn control_differing_scope(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.first_mut() else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "differing-scope-hidden".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no retained observations".to_owned(),
        };
    };
    observation.scope.selected_sources.push("hidden/other.rs".to_owned());
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("differing-scope-hidden", &diagnostics)
}

fn control_dropped_timeout(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(index) = mutated
        .observations
        .iter()
        .position(|observation| observation.status == crate::benchmark::experiment::model::ObservationStatus::TimedOut)
        .or_else(|| {
            mutated.observations.iter().position(|observation| {
                observation.case_id.is_some()
                    && !matches!(
                        observation.status,
                        crate::benchmark::experiment::model::ObservationStatus::Unavailable
                            | crate::benchmark::experiment::model::ObservationStatus::Unsupported
                    )
            })
        })
    else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "dropped-timeouts".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no comparable observation to drop".to_owned(),
        };
    };
    if let Some(observation) = mutated.observations.get_mut(index)
        && observation.status != crate::benchmark::experiment::model::ObservationStatus::TimedOut
    {
        observation.status = crate::benchmark::experiment::model::ObservationStatus::TimedOut;
        observation.failure_reason = Some("negative-control timeout".to_owned());
    }
    if index < mutated.observations.len() {
        mutated.observations.remove(index);
    }
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("dropped-timeouts", &diagnostics)
}

fn control_stale_results(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(metrics) = mutated.derived.architecture.first_mut() else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "stale-results".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no derived architecture metrics".to_owned(),
        };
    };
    metrics.true_positive = metrics.true_positive.saturating_add(1);
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("stale-results", &diagnostics)
}

fn control_hidden_environment(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.iter_mut().find(|observation| observation.case_id.is_some()) else {
        return crate::benchmark::experiment::model::NegativeControlRecord {
            control_id: "hidden-toolchain-cache-difference".to_owned(),
            outcome: crate::benchmark::experiment::model::ControlOutcome::NotRun,
            evidence: "no case observation".to_owned(),
        };
    };
    "nightly-unreviewed".clone_into(&mut observation.environment.rust_toolchain);
    observation.environment.cache_state = crate::benchmark::experiment::model::CacheState::Warm;
    observation.warm_state = crate::benchmark::experiment::model::WarmState::Warm;
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("hidden-toolchain-cache-difference", &diagnostics)
}

fn control_bounded_corpus(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> crate::benchmark::experiment::model::NegativeControlRecord {
    let mut mutated = artifact.clone();
    mutated.profile.coverage_boundary.complete_rust_coverage = true;
    let diagnostics = super::verify_artifact(profile, &mutated);
    control_record("bounded-corpus-presented-as-complete", &diagnostics)
}
