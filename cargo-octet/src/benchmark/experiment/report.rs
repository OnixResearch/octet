use std::collections::BTreeMap;
use std::fmt::Write as _;

mod results;

pub fn render(artifact: &super::model::Artifact) -> Result<String, std::fmt::Error> {
    let mut output = String::new();
    let profile = &artifact.profile;
    writeln!(output, "# Cross-tool benchmark experiment report")?;
    writeln!(output)?;
    writeln!(output, "Source-linked report for profile `{}`.", profile.profile_id)?;
    writeln!(output)?;
    writeln!(output, "- Profile identity: `{}`", artifact.profile_identity)?;
    writeln!(output, "- Source revision: `{}`", profile.source_revision)?;
    writeln!(output, "- Rust toolchain: `{}`", profile.rust_toolchain)?;
    writeln!(output, "- Evidence tier: `{}`", profile.evidence_tier)?;
    writeln!(output, "- Machine artifact: `{}`", super::model::ARTIFACT_NAME)?;
    writeln!(output, "- Retained observations: `{}`", super::model::OBSERVATIONS_NAME)?;
    writeln!(output, "- Declared before measurement: `{}`", profile.declared_before_measurement)?;
    writeln!(output)?;

    write_provenance(&mut output, artifact)?;
    write_tool_pins(&mut output, artifact)?;
    write_applicability(&mut output, artifact)?;
    write_corpus(&mut output, artifact)?;
    results::write_metrics(
        &mut output,
        "Architecture detection results",
        &artifact.derived.architecture,
        &super::model::Category::Architecture,
    )?;
    results::write_metrics(
        &mut output,
        "Assurance results",
        &artifact.derived.assurance,
        &super::model::Category::Assurance,
    )?;
    write_retained_failures(&mut output, artifact)?;
    results::write_adoption(&mut output, artifact)?;
    write_controls(&mut output, artifact)?;
    write_boundary(&mut output, artifact)?;
    Ok(output)
}

fn write_provenance(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    let provenance = &artifact.derived.provenance;
    writeln!(output, "## Measurement provenance")?;
    writeln!(output)?;
    writeln!(output, "Executed locally:")?;
    writeln!(output)?;
    write_list(output, &provenance.executed_locally, "none")?;
    writeln!(output)?;
    writeln!(output, "Recorded fixtures:")?;
    writeln!(output)?;
    write_list(output, &provenance.recorded_fixtures, "none")?;
    writeln!(output)?;
    writeln!(output, "Unavailable executables:")?;
    writeln!(output)?;
    write_list(output, &provenance.unavailable, "none")?;
    writeln!(output)?;
    Ok(())
}

fn write_tool_pins(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    writeln!(output, "## Tool and configuration pins")?;
    writeln!(output)?;
    writeln!(output, "| Tool | Version | Source pin | Configuration | Availability |")?;
    writeln!(output, "| --- | --- | --- | --- | --- |")?;
    let availability = artifact
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    for tool in &artifact.profile.tools {
        let status = availability
            .get(tool.tool_id.as_str())
            .map_or("missing", |record| availability_label(&record.status));
        let config = if tool.config_path.is_empty() {
            format!("{:?}: {}", tool.configuration_status, tool.rules_pin)
        } else {
            format!("{} ({:?})", tool.config_path.join(", "), tool.configuration_status)
        };
        writeln!(
            output,
            "| `{}` | {} | {} | {} | {} |",
            tool.tool_id, tool.version_pin, tool.source_pin, config, status
        )?;
    }
    for tool in &artifact.profile.tools {
        let status = availability.get(tool.tool_id.as_str());
        if let Some(record) = status
            && let Some(reason) = record.reason.as_ref()
        {
            writeln!(output)?;
            writeln!(output, "- `{}` unavailable: {reason}", tool.tool_id)?;
        }
    }
    writeln!(output)?;
    Ok(())
}

fn write_applicability(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    writeln!(output, "## Applicability matrix")?;
    writeln!(output)?;
    let tools = &artifact.profile.tools;
    let mut header = String::from("| Task |");
    for tool in tools {
        write!(header, " {} |", tool.tool_id)?;
    }
    writeln!(output, "{header}")?;
    let mut separator = String::from("| --- |");
    for _ in tools {
        separator.push_str(" --- |");
    }
    writeln!(output, "{separator}")?;
    let matrix = artifact
        .profile
        .applicability
        .iter()
        .map(|entry| ((entry.task_id.as_str(), entry.tool_id.as_str()), &entry.status))
        .collect::<BTreeMap<_, _>>();
    for task in &artifact.profile.tasks {
        let mut row = format!("| `{}` |", task.task_id);
        for tool in tools {
            let status = matrix
                .get(&(task.task_id.as_str(), tool.tool_id.as_str()))
                .map_or("?", |status| applicability_mark(status));
            write!(row, " {status} |")?;
        }
        writeln!(output, "{row}")?;
    }
    writeln!(output)?;
    writeln!(output, "S = supported, P = partial, U = unsupported. Unsupported cells are not clean evidence.")?;
    writeln!(output)?;
    Ok(())
}

fn write_corpus(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    writeln!(output, "## Frozen corpus and adjudication")?;
    writeln!(output)?;
    writeln!(output, "| Case | Variant | Expected | Source link | Source identity | Rationale | Reviewer |")?;
    writeln!(output, "| --- | --- | --- | --- | --- | --- | --- |")?;
    for case in &artifact.profile.corpus {
        writeln!(
            output,
            "| `{}` | {:?} | {:?} | [{}]({}) | `{}` | {} | {} |",
            case.case_id,
            case.variant,
            case.expected,
            case.source_path,
            source_link(&case.source_path, &case.case_id),
            case.source_identity,
            case.rationale,
            case.reviewer_role
        )?;
    }
    writeln!(output)?;
    Ok(())
}

fn write_retained_failures(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    writeln!(output, "## Retained failures, timeouts, and unparsed runs")?;
    writeln!(output)?;
    let mut has_retained_failures = false;
    for observation in &artifact.observations {
        if observation.status == super::model::ObservationStatus::Completed {
            continue;
        }
        has_retained_failures = true;
        let case = observation.case_id.as_deref().unwrap_or("<tool-level>");
        let reason = observation
            .failure_reason
            .as_deref()
            .or(observation.unavailable_reason.as_deref())
            .unwrap_or("no reason recorded");
        writeln!(
            output,
            "- `{}` / `{}` / `{}`: {:?} ({reason})",
            observation.tool_id, observation.task_id, case, observation.status
        )?;
    }
    if !has_retained_failures {
        writeln!(output, "None.")?;
    }
    writeln!(output)?;
    Ok(())
}

fn write_controls(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    writeln!(output, "## Negative controls")?;
    writeln!(output)?;
    writeln!(output, "| Control | Outcome | Evidence |")?;
    writeln!(output, "| --- | --- | --- |")?;
    for control in &artifact.controls {
        writeln!(output, "| `{}` | {:?} | {} |", control.control_id, control.outcome, control.evidence)?;
    }
    writeln!(output)?;
    Ok(())
}

fn write_boundary(output: &mut String, artifact: &super::model::Artifact) -> std::fmt::Result {
    let boundary = &artifact.profile.coverage_boundary;
    writeln!(output, "## Coverage boundary and non-claims")?;
    writeln!(output)?;
    writeln!(output, "- Corpus: `{}`", boundary.corpus_id)?;
    writeln!(output, "- Bounded: `{}`", boundary.bounded)?;
    writeln!(output, "- Complete Rust coverage: `{}`", boundary.complete_rust_coverage)?;
    writeln!(output, "- {}", boundary.statement)?;
    writeln!(output)?;
    for claim in &artifact.profile.non_claims {
        writeln!(output, "- {claim}")?;
    }
    Ok(())
}

fn write_list(output: &mut String, values: &[String], empty: &str) -> std::fmt::Result {
    if values.is_empty() {
        writeln!(output, "- {empty}")?;
        return Ok(());
    }
    for value in values {
        writeln!(output, "- `{value}`")?;
    }
    Ok(())
}

fn availability_label(availability: &super::model::Availability) -> &'static str {
    match availability {
        super::model::Availability::Local => "local",
        super::model::Availability::Recorded => "recorded",
        super::model::Availability::Unavailable => "unavailable",
    }
}

fn applicability_mark(status: &super::model::ApplicabilityStatus) -> &'static str {
    match status {
        super::model::ApplicabilityStatus::Supported => "S",
        super::model::ApplicabilityStatus::Partial => "P",
        super::model::ApplicabilityStatus::Unsupported => "U",
    }
}

fn source_link(path: &str, case_id: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_owned();
    }
    format!("../../{path}#{case_id}")
}
