use std::collections::BTreeMap;

pub(super) fn validate(
    profile: &crate::benchmark::experiment::model::AnalysisProfile,
    observations: &crate::benchmark::experiment::model::ObservationSet,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.reserve(observations.tool_availability.len() * 3 + profile.tools.len());
    let tool_map = crate::benchmark::experiment::model::tool_map(profile);
    let mut availability = BTreeMap::new();
    for record in &observations.tool_availability {
        if record.tool_id.is_empty()
            || !tool_map.contains_key(record.tool_id.as_str())
            || availability.insert(record.tool_id.as_str(), record).is_some()
        {
            diagnostics.push("experiment tool availability is incomplete or duplicated".to_owned());
            continue;
        }
        if !super::super::is_b3(&record.version_identity) {
            diagnostics.push(format!("experiment tool `{}` version identity is malformed", record.tool_id));
        }
        if record.status == crate::benchmark::experiment::model::Availability::Unavailable {
            if record.reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics.push(format!("experiment unavailable tool `{}` has no reason", record.tool_id));
            }
        } else if record.version_output.is_empty() {
            diagnostics.push(format!("experiment available tool `{}` has no version output", record.tool_id));
        }
    }
    for tool in &profile.tools {
        if !availability.contains_key(tool.tool_id.as_str()) {
            diagnostics.push(format!("experiment tool availability is missing `{}`", tool.tool_id));
        }
    }
}
