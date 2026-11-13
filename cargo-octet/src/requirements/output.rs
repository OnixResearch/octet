pub(super) fn write_decision(
    decision: &super::evaluation::decision::Outcome,
    output: Option<&std::path::Path>,
    format: crate::cli::OutputFormat,
) -> Result<(), String> {
    let text =
        serde_json::to_string_pretty(decision).map_err(|error| format!("serialize requirement decision: {error}"))?;
    if let Some(output) = output {
        std::fs::write(output, format!("{text}\n"))
            .map_err(|error| format!("write requirement decision {}: {error}", output.display()))?;
    }
    match format {
        crate::cli::OutputFormat::Json => println!("{text}"),
        crate::cli::OutputFormat::Human | crate::cli::OutputFormat::Sarif => {
            println!(
                "octet requirements check contract={} status={} clauses={} assumptions={} exclusions={} diagnostics={}",
                decision.contract_id,
                decision.aggregate_result,
                decision.clauses.len(),
                decision.assumptions.len(),
                decision.exclusions.len(),
                decision.diagnostics.len()
            );
            for clause in &decision.clauses {
                println!(
                    "clause={} disposition={} required={} status={} evidence=[{}]",
                    clause.clause_id,
                    clause.disposition,
                    clause.required,
                    clause.status,
                    clause.evidence_ids.join(",")
                );
                for diagnostic in &clause.diagnostics {
                    println!("  clause-diagnostic: {diagnostic}");
                }
            }
            for assumption in &decision.assumptions {
                println!(
                    "assumption={} status={} identity={} supporting=[{}]",
                    assumption.assumption_id,
                    assumption.status,
                    assumption.identity,
                    assumption.supporting_evidence_ids.join(",")
                );
            }
            for exclusion in &decision.exclusions {
                println!(
                    "excluded={} clause={} reason={}",
                    exclusion.exclusion_id, exclusion.clause_id, exclusion.reason
                );
            }
            for limitation in &decision.limitations {
                println!("limitation: {limitation}");
            }
            for diagnostic in &decision.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &decision.non_claims {
                println!("non-claim: {non_claim}");
            }
        }
    }
    Ok(())
}

pub(super) fn print_freshness_human(report: &super::freshness::InventoryReport) {
    println!(
        "octet requirements freshness contract={} status={} clauses={} snapshot={} completeness={} diagnostics={}",
        report.contract_id,
        report.aggregate_result,
        report.clauses.len(),
        report.snapshot_id,
        report.completeness,
        report.diagnostics.len()
    );
    for clause in &report.clauses {
        println!(
            "clause={} required={} status={} matched=[{}] related={}",
            clause.clause_id,
            clause.required,
            clause.status,
            clause.matched_record_ids.join(","),
            clause.related_records.len()
        );
        for diagnostic in &clause.diagnostics {
            println!("  clause-diagnostic: {diagnostic}");
        }
        for related in &clause.related_records {
            println!(
                "  related={} stable_id={} depth={} relation={} semantics={}",
                related.record_id, related.stable_id, related.depth, related.relation, related.semantics
            );
        }
    }
    for diagnostic in &report.diagnostics {
        println!("diagnostic: {diagnostic}");
    }
    for non_claim in &report.non_claims {
        println!("non-claim: {non_claim}");
    }
}
