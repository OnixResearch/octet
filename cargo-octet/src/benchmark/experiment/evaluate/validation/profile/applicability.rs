use std::collections::BTreeSet;

pub(super) fn validate(profile: &crate::benchmark::experiment::model::AnalysisProfile, diagnostics: &mut Vec<String>) {
    diagnostics.reserve(
        profile.applicability.len() * 4
            + profile.applicability.iter().map(|entry| entry.case_ids.len()).sum::<usize>()
            + profile.tools.len() * profile.tasks.len(),
    );
    let case_map = crate::benchmark::experiment::model::case_map(profile);
    let task_map = crate::benchmark::experiment::model::task_map(profile);
    let tool_map = crate::benchmark::experiment::model::tool_map(profile);
    let mut seen = BTreeSet::new();
    for entry in &profile.applicability {
        if !seen.insert((entry.tool_id.as_str(), entry.task_id.as_str())) {
            diagnostics.push("experiment applicability matrix contains a duplicate cell".to_owned());
        }
        let Some(task) = task_map.get(entry.task_id.as_str()) else {
            diagnostics.push(format!("experiment applicability names unknown task `{}`", entry.task_id));
            continue;
        };
        if !tool_map.contains_key(entry.tool_id.as_str()) {
            diagnostics.push(format!("experiment applicability names unknown tool `{}`", entry.tool_id));
        }
        if entry.comparable_subset != task.comparable_subset || entry.scope != task.required_scope {
            diagnostics.push(format!(
                "experiment applicability for `{}`/`{}` does not bind the task subset or scope",
                entry.tool_id, entry.task_id
            ));
        }
        match entry.status {
            crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported => {
                if !entry.case_ids.is_empty() {
                    diagnostics.push(format!(
                        "experiment applicability for unsupported `{}`/`{}` still claims cases",
                        entry.tool_id, entry.task_id
                    ));
                }
            }
            crate::benchmark::experiment::model::ApplicabilityStatus::Supported
            | crate::benchmark::experiment::model::ApplicabilityStatus::Partial => {
                if entry.case_ids.is_empty() {
                    diagnostics.push(format!(
                        "experiment applicability for `{}`/`{}` claims no cases",
                        entry.tool_id, entry.task_id
                    ));
                }
                for case_id in &entry.case_ids {
                    match case_map.get(case_id.as_str()) {
                        Some(case) if case.task_id == entry.task_id => {}
                        Some(_) => diagnostics.push(format!(
                            "experiment applicability for `{}`/`{}` claims case `{case_id}` from another task",
                            entry.tool_id, entry.task_id
                        )),
                        None => diagnostics.push(format!(
                            "experiment applicability for `{}`/`{}` claims unknown case `{case_id}`",
                            entry.tool_id, entry.task_id
                        )),
                    }
                }
            }
        }
    }
    for tool in &profile.tools {
        for task in &profile.tasks {
            if !seen.contains(&(tool.tool_id.as_str(), task.task_id.as_str())) {
                diagnostics
                    .push(format!("experiment applicability matrix is missing `{}`/`{}`", tool.tool_id, task.task_id));
            }
        }
    }
}
