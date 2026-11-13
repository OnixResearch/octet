use std::collections::BTreeMap;
use std::collections::BTreeSet;

mod applicability;

const REQUIRED_CONTROL_IDS: [&str; 8] = [
    "noncompiling-violation-counted-as-detection",
    "renamed-stale-oracle",
    "differing-scope-hidden",
    "dropped-timeouts",
    "stale-results",
    "hidden-toolchain-cache-difference",
    "unsupported-counted-clean",
    "bounded-corpus-presented-as-complete",
];

pub(crate) fn validate(profile: &crate::benchmark::experiment::model::AnalysisProfile) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(3);
    if profile.profile_id != crate::benchmark::experiment::model::PROFILE_ID
        || profile.source_revision.is_empty()
        || profile.rust_toolchain.is_empty()
        || profile.evidence_tier != "transcript_assertion"
        || !profile.declared_before_measurement
    {
        diagnostics.push(
            "experiment profile identity, revision, toolchain, or declaration boundary is unsupported".to_owned(),
        );
    }
    if !profile
        .non_claims
        .iter()
        .any(|claim| claim == crate::benchmark::experiment::model::REQUIRED_NON_CLAIM)
    {
        diagnostics.push("experiment profile is missing the required non-claim".to_owned());
    }
    if !profile.coverage_boundary.bounded
        || profile.coverage_boundary.complete_rust_coverage
        || profile.coverage_boundary.statement.is_empty()
    {
        diagnostics.push("experiment coverage boundary must stay bounded and incomplete".to_owned());
    }
    validate_cases(profile, &mut diagnostics);
    validate_tasks(profile, &mut diagnostics);
    validate_tools(profile, &mut diagnostics);
    applicability::validate(profile, &mut diagnostics);
    validate_adoption_plan(profile, &mut diagnostics);
    validate_sample_plan(profile, &mut diagnostics);
    validate_controls(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_cases(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(profile.corpus.len() * 3 + profile.tasks.len());
    let mut case_ids = BTreeSet::new();
    let tasks = profile.tasks.iter().map(|task| (task.task_id.as_str(), task)).collect::<BTreeMap<_, _>>();
    for case in &profile.corpus {
        if case.case_id.is_empty()
            || !case_ids.insert(case.case_id.as_str())
            || case.source_path.is_empty()
            || case.rationale.is_empty()
            || case.reviewer_role.is_empty()
            || !super::super::is_b3(&case.source_identity)
        {
            diagnostics.push("experiment corpus contains an incomplete or duplicate case".to_owned());
        }
        let Some(task) = tasks.get(case.task_id.as_str()) else {
            diagnostics.push(format!("experiment case `{}` names an unknown task", case.case_id));
            continue;
        };
        if task.category != case.category {
            diagnostics.push(format!("experiment case `{}` category differs from its task", case.case_id));
        }
        if case.scope.empty != (case.scope.selected_sources.is_empty() && case.scope.expected_analyzed_sources == 0) {
            diagnostics.push(format!("experiment case `{}` has an inconsistent empty-scope declaration", case.case_id));
        }
    }
    for task in &profile.tasks {
        if !profile.corpus.iter().any(|case| case.task_id == task.task_id) {
            diagnostics.push(format!("experiment task `{}` has no corpus cases", task.task_id));
        }
    }
}

fn validate_tasks(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(profile.tasks.len() + 2);
    let mut task_ids = BTreeSet::new();
    for task in &profile.tasks {
        if task.task_id.is_empty()
            || task.description.is_empty()
            || task.comparable_subset.is_empty()
            || task.required_scope.is_empty()
            || !task_ids.insert(task.task_id.as_str())
        {
            diagnostics.push("experiment task is incomplete or duplicated".to_owned());
        }
    }
    for category in [
        crate::benchmark::experiment::model::Category::Architecture,
        crate::benchmark::experiment::model::Category::Assurance,
    ] {
        if !profile.tasks.iter().any(|task| task.category == category) {
            diagnostics.push("experiment profile is missing a required task category".to_owned());
        }
    }
}

fn validate_tools(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(profile.tools.len() * 3);
    let mut tool_ids = BTreeSet::new();
    for tool in &profile.tools {
        if tool.tool_id.is_empty()
            || tool.executable.is_empty()
            || tool.version_pin.is_empty()
            || tool.source_pin.is_empty()
            || tool.rules_pin.is_empty()
            || tool.setup_source.is_empty()
            || tool.setup_steps.is_empty()
            || tool.probe_argv.is_empty()
            || tool.claims.is_empty()
            || !tool_ids.insert(tool.tool_id.as_str())
        {
            diagnostics.push("experiment tool pin is incomplete or duplicated".to_owned());
        }
        if tool.config_path.len() != tool.config_identity.len()
            || tool.config_identity.iter().any(|identity| !super::super::is_b3(identity))
        {
            diagnostics.push(format!("experiment tool `{}` config pin is inconsistent", tool.tool_id));
        }
        if tool.configuration_status == crate::benchmark::experiment::model::ConfigurationStatus::CheckedIn
            && (tool.config_path.is_empty() || tool.config_identity.is_empty())
        {
            diagnostics.push(format!("experiment tool `{}` is missing a checked-in config pin", tool.tool_id));
        }
    }
}

fn validate_adoption_plan(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(2 + profile.adoption.tools.len() + profile.tools.len());
    let clean = &profile.adoption.clean_consumer;
    let real = &profile.adoption.real_consumer;
    if clean.selection_id.is_empty()
        || clean.source_repository.is_empty()
        || clean.source_revision.is_empty()
        || clean.workspace_path.is_empty()
        || !clean.immutable
    {
        diagnostics.push("experiment clean-consumer adoption target is incomplete".to_owned());
    }
    if real.selection_id.is_empty()
        || real.selection_id == clean.selection_id
        || real.source_repository.is_empty()
        || real.source_revision.is_empty()
        || !real.immutable
    {
        diagnostics.push("experiment real-consumer adoption target must be a distinct immutable selection".to_owned());
    }
    let mut planned = BTreeSet::new();
    for plan in &profile.adoption.tools {
        if plan.tool_id.is_empty()
            || plan.setup_steps.is_empty()
            || plan.initial_command.is_empty()
            || plan.incremental_command.is_empty()
            || !planned.insert(plan.tool_id.as_str())
        {
            diagnostics.push("experiment adoption plan is incomplete or duplicated".to_owned());
        }
    }
    for tool in &profile.tools {
        if !planned.contains(tool.tool_id.as_str()) {
            diagnostics.push(format!("experiment adoption plan is missing tool `{}`", tool.tool_id));
        }
    }
}

fn validate_sample_plan(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    if profile.sample_plan.sample_count == 0
        || profile.sample_plan.timeout_ms == 0
        || profile.sample_plan.target.is_empty()
        || profile.sample_plan.warm_state != crate::benchmark::experiment::model::WarmState::Cold
        || profile.sample_plan.cache_state != crate::benchmark::experiment::model::CacheState::Cold
    {
        diagnostics.push("experiment sample plan is incomplete or does not declare a cold run".to_owned());
    }
}

fn validate_controls(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(profile.negative_controls.len() + REQUIRED_CONTROL_IDS.len());
    let mut ids = BTreeSet::new();
    for control in &profile.negative_controls {
        if control.control_id.is_empty() || control.description.is_empty() || !ids.insert(control.control_id.as_str()) {
            diagnostics.push("experiment negative-control declaration is incomplete or duplicated".to_owned());
        }
    }
    for required in REQUIRED_CONTROL_IDS {
        if !ids.contains(required) {
            diagnostics.push(format!("experiment profile is missing negative control `{required}`"));
        }
    }
}
