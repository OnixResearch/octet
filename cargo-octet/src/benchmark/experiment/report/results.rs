use std::collections::BTreeMap;
use std::fmt::Write as _;

pub(super) fn write_metrics(
    output: &mut String,
    title: &str,
    rows: &[crate::benchmark::experiment::model::ToolTaskMetrics],
    category: &crate::benchmark::experiment::model::Category,
) -> std::fmt::Result {
    writeln!(output, "## {title}")?;
    writeln!(output)?;
    writeln!(
        output,
        "| Tool | Task | Applicability | Availability | Comparable | TP | FP | FN | TN | Unsupported | Inconclusive | Incomparable | Precision bp | Recall bp |"
    )?;
    writeln!(output, "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")?;
    for metrics in rows.iter().filter(|metrics| &metrics.category == category) {
        writeln!(
            output,
            "| `{}` | `{}` | {:?} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            metrics.tool_id,
            metrics.task_id,
            metrics.applicability_status,
            super::availability_label(&metrics.availability),
            metrics.comparable_cases,
            metrics.true_positive,
            metrics.false_positive,
            metrics.false_negative,
            metrics.true_negative,
            metrics.unsupported_cases,
            metrics.inconclusive_cases,
            metrics.incomparable_cases,
            option_number(metrics.precision_basis_points),
            option_number(metrics.recall_basis_points)
        )?;
    }
    writeln!(output)?;
    writeln!(output, "Precision and recall use only the declared comparable subset.")?;
    writeln!(output)?;
    writeln!(output, "### Unsupported applicability cells")?;
    writeln!(output)?;
    let mut unsupported_by_tool = BTreeMap::new();
    for metrics in rows.iter().filter(|metrics| &metrics.category == category) {
        if metrics.applicability_status == crate::benchmark::experiment::model::ApplicabilityStatus::Unsupported {
            *unsupported_by_tool.entry(metrics.tool_id.as_str()).or_insert(0_u64) += 1;
        }
    }
    for (tool, count) in unsupported_by_tool {
        writeln!(output, "- `{tool}`: {count} unsupported task cells")?;
    }
    writeln!(output)?;
    Ok(())
}

pub(super) fn write_adoption(
    output: &mut String,
    artifact: &crate::benchmark::experiment::model::Artifact,
) -> std::fmt::Result {
    writeln!(output, "## Adoption cost")?;
    writeln!(output)?;
    writeln!(
        output,
        "Clean consumer: `{}` ({}@{}).",
        artifact.profile.adoption.clean_consumer.selection_id,
        artifact.profile.adoption.clean_consumer.source_repository,
        artifact.profile.adoption.clean_consumer.source_revision
    )?;
    writeln!(
        output,
        "Real consumer: `{}` ({}@{}).",
        artifact.profile.adoption.real_consumer.selection_id,
        artifact.profile.adoption.real_consumer.source_repository,
        artifact.profile.adoption.real_consumer.source_revision
    )?;
    writeln!(output)?;
    writeln!(
        output,
        "| Tool | Availability | Setup steps | Config files | Clean initial | Clean incremental | Real initial | Real incremental | Incremental/initial |"
    )?;
    writeln!(output, "| --- | --- | --- | --- | --- | --- | --- | --- | --- |")?;
    for summary in &artifact.derived.adoption {
        writeln!(
            output,
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            summary.tool_id,
            super::availability_label(&summary.availability),
            summary.setup_steps,
            summary.config_files,
            option_duration(summary.clean_initial_ns),
            option_duration(summary.clean_incremental_ns),
            option_duration(summary.real_initial_ns),
            option_duration(summary.real_incremental_ns),
            option_ratio(summary.incremental_over_initial_basis_points)
        )?;
    }
    writeln!(output)?;
    for summary in &artifact.derived.adoption {
        if summary.unavailable_reasons.is_empty() {
            continue;
        }
        writeln!(output, "- `{}` unavailable rows:", summary.tool_id)?;
        for reason in &summary.unavailable_reasons {
            writeln!(output, "  - {reason}")?;
        }
        if !summary.unavailable_metrics.is_empty() {
            writeln!(output, "  - unavailable metrics: {}", summary.unavailable_metrics.join(", "))?;
        }
    }
    writeln!(output)?;
    writeln!(output, "### Documented setup steps")?;
    writeln!(output)?;
    for tool in &artifact.profile.tools {
        writeln!(output, "- `{}` (source: {}):", tool.tool_id, tool.setup_source)?;
        for step in &tool.setup_steps {
            writeln!(output, "  - {step}")?;
        }
        if !tool.required_config_files.is_empty() {
            writeln!(output, "  - required config: {}", tool.required_config_files.join(", "))?;
        }
    }
    writeln!(output)?;
    Ok(())
}

fn option_number(value: Option<u64>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |value| value.to_string())
}

fn option_duration(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{value} ns"))
}

fn option_ratio(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{value} bp"))
}
