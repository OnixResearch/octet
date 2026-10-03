/// Normalize compiler shards into a coverage report, issues, receipt, and optional IR.
///
/// # Errors
///
/// Returns an error if invalid-shard capacity overflows `usize` or serializing a shard or the
/// assembled IR fails. Invalid source facts, policy bindings, and coverage instead become issues
/// in the returned outcome.
pub fn normalize_architecture_shards(
    input: crate::model::NormalizationInput,
) -> Result<crate::model::NormalizationOutcome, String> {
    let mut issues = validate_limit_shape(&input.limits);
    let expected_by_id = validate_expected_units(&input, &mut issues);
    let (valid_shards, invalid_shard_ids) = collect_shards(&input, &mut issues)?;

    validate_count_limit("units", expected_by_id.len(), input.limits.max_units, None, &mut issues);
    let observed_shard_count = input.shards.len().saturating_add(input.invalid_shards.len());
    validate_count_limit("shards", observed_shard_count, input.limits.max_shards, None, &mut issues);

    let coverage = summarize_coverage(&expected_by_id, &valid_shards, invalid_shard_ids, &mut issues)?;
    let architecture_ir = if coverage.status == crate::model::CoverageStatus::Complete {
        let candidate = build_ir(&input, &valid_shards, &mut issues)?;
        issues.is_empty().then_some(candidate)
    } else {
        None
    };
    issues.sort();
    issues.dedup();

    let status = if architecture_ir.is_some() && issues.is_empty() {
        crate::model::CollectionStatus::Complete
    } else {
        crate::model::CollectionStatus::Incomplete
    };
    let issues = issues.into_result()?;
    let mut receipt = crate::model::CompilerArchitectureReceipt {
        schema_version: crate::model::COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
        receipt_id: String::new(),
        status,
        policy_blake3: input.policy_blake3,
        coverage_id: coverage.coverage_id.clone(),
        architecture_ir_id: architecture_ir.as_ref().map(|ir| ir.architecture_ir_id.clone()),
        issue_ids: issues.iter().map(|current| current.issue_id.clone()).collect(),
        non_claims: super::diagnostics::sorted_unique(input.non_claims),
    };
    receipt.receipt_id = super::digest::receipt_identity(&receipt)?;

    Ok(crate::model::NormalizationOutcome {
        architecture_ir,
        coverage,
        issues,
        receipt,
    })
}

fn summarize_coverage(
    expected_by_id: &std::collections::BTreeMap<String, crate::model::ExpectedCompilationUnit>,
    valid_shards: &[crate::model::CompilerArchitectureShard],
    mut invalid_shard_ids: Vec<String>,
    issues: &mut super::diagnostics::Issues,
) -> Result<crate::model::CompilerArchitectureCoverage, String> {
    let observed_counts = unit_counts(valid_shards);
    let expected_ids = expected_by_id.keys().cloned().collect::<Vec<_>>();
    let observed_ids = observed_counts.keys().cloned().collect::<Vec<_>>();
    let missing_ids = difference(&expected_ids, &observed_ids);
    let unexpected_ids = difference(&observed_ids, &expected_ids);
    let duplicate_ids = observed_counts
        .iter()
        .filter_map(|(unit_id, count)| (*count > 1).then_some(unit_id.clone()))
        .collect::<Vec<_>>();

    record_coverage_gaps(&missing_ids, &unexpected_ids, &duplicate_ids, issues);

    invalid_shard_ids.sort();
    invalid_shard_ids.dedup();
    issues.sort();
    issues.dedup();

    let coverage_status = if missing_ids.is_empty()
        && unexpected_ids.is_empty()
        && duplicate_ids.is_empty()
        && invalid_shard_ids.is_empty()
        && issues.is_empty()
    {
        crate::model::CoverageStatus::Complete
    } else {
        crate::model::CoverageStatus::Incomplete
    };
    let mut coverage = crate::model::CompilerArchitectureCoverage {
        schema_version: crate::model::COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION.to_owned(),
        coverage_id: String::new(),
        status: coverage_status,
        expected_unit_ids: expected_ids,
        observed_unit_ids: observed_ids,
        missing_unit_ids: missing_ids,
        unexpected_unit_ids: unexpected_ids,
        duplicate_unit_ids: duplicate_ids,
        invalid_shard_ids,
    };
    coverage.coverage_id = super::digest::coverage_identity(&coverage)?;
    Ok(coverage)
}

fn record_coverage_gaps(
    missing: &[String],
    unexpected: &[String],
    duplicates: &[String],
    issues: &mut super::diagnostics::Issues,
) {
    for unit_id in missing {
        issues.push(super::digest::issue(
            "coverage-missing-unit",
            Some(unit_id),
            None,
            "expected compilation unit has no valid shard",
        ));
    }
    for unit_id in unexpected {
        issues.push(super::digest::issue(
            "coverage-unexpected-unit",
            Some(unit_id),
            None,
            "valid shard does not match an expected compilation unit",
        ));
    }
    for unit_id in duplicates {
        issues.push(super::digest::issue(
            "coverage-duplicate-unit",
            Some(unit_id),
            None,
            "compilation unit has more than one valid shard",
        ));
    }
}

fn collect_shards(
    input: &crate::model::NormalizationInput,
    issues: &mut super::diagnostics::Issues,
) -> Result<(Vec<crate::model::CompilerArchitectureShard>, Vec<String>), String> {
    let mut valid_shards = Vec::with_capacity(input.shards.len());
    let invalid_capacity = input
        .invalid_shards
        .len()
        .checked_add(input.shards.len())
        .ok_or_else(|| "compiler architecture invalid-shard capacity overflowed usize".to_owned())?;
    let mut invalid_shard_ids = Vec::with_capacity(invalid_capacity);

    for invalid in &input.invalid_shards {
        invalid_shard_ids.push(invalid.shard_id.clone());
        issues.push(super::digest::issue(&invalid.code, None, None, &invalid.message));
    }

    for shard in &input.shards {
        let before = issues.len();
        super::checks::validate_shard(shard, &input.policy_blake3, &input.limits, issues)?;
        if issues.len() == before {
            valid_shards.push(shard.clone());
        } else {
            invalid_shard_ids.push(shard.shard_id.clone());
        }
    }
    Ok((valid_shards, invalid_shard_ids))
}

fn validate_limit_shape(limits: &crate::model::ArchitectureCollectionLimits) -> super::diagnostics::Issues {
    let values = [
        ("max_units", limits.max_units),
        ("max_shards", limits.max_shards),
        ("max_modules", limits.max_modules),
        ("max_items", limits.max_items),
        ("max_types", limits.max_types),
        ("max_traits", limits.max_traits),
        ("max_implementations", limits.max_implementations),
        ("max_signatures", limits.max_signatures),
        ("max_calls", limits.max_calls),
        ("max_constructors", limits.max_constructors),
        ("max_effects", limits.max_effects),
        ("max_expansions", limits.max_expansions),
        ("max_configurations", limits.max_configurations),
        ("max_unknowns", limits.max_unknowns),
        ("max_spans", limits.max_spans),
        ("max_span_bytes", limits.max_span_bytes),
        ("max_shard_bytes", limits.max_shard_bytes),
        ("max_artifact_bytes", limits.max_artifact_bytes),
    ];
    values
        .into_iter()
        .filter(|(_, value)| *value == 0)
        .map(|(name, _)| {
            super::digest::issue("invalid-limit", None, None, &format!("collection limit `{name}` must be positive"))
        })
        .collect()
}

fn validate_expected_units(
    input: &crate::model::NormalizationInput,
    issues: &mut super::diagnostics::Issues,
) -> std::collections::BTreeMap<String, crate::model::ExpectedCompilationUnit> {
    let mut units = std::collections::BTreeMap::new();
    for expected in &input.expected_units {
        super::checks::validate_unit(&expected.unit, &input.policy_blake3, issues);
        if units.insert(expected.unit.unit_id.clone(), expected.clone()).is_some() {
            issues.push(super::digest::issue(
                "duplicate-expected-unit",
                Some(&expected.unit.unit_id),
                None,
                "expected compilation units contain a duplicate identity",
            ));
        }
    }
    units
}

fn build_ir(
    input: &crate::model::NormalizationInput,
    shards: &[crate::model::CompilerArchitectureShard],
    issues: &mut super::diagnostics::Issues,
) -> Result<crate::model::CompilerArchitectureIr, String> {
    let mut units = shards.iter().map(|shard| shard.unit.clone()).collect::<Vec<_>>();
    let mut spans = shards.iter().flat_map(|shard| shard.spans.clone()).collect::<Vec<_>>();
    let mut facts = shards.iter().flat_map(|shard| shard.facts.clone()).collect::<Vec<_>>();
    let mut unknowns = shards.iter().flat_map(|shard| shard.unknowns.clone()).collect::<Vec<_>>();
    let unit_memberships = crate::membership::memberships_from_shards(shards);
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    spans.sort();
    spans.dedup();
    facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    facts.dedup();
    unknowns.sort();
    unknowns.dedup();

    let mut ir = crate::model::CompilerArchitectureIr {
        schema_version: crate::model::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION.to_owned(),
        architecture_ir_id: String::new(),
        policy_blake3: input.policy_blake3.clone(),
        units,
        unit_memberships,
        spans,
        facts,
        unknowns,
        non_claims: super::diagnostics::sorted_unique(input.non_claims.clone()),
    };
    ir.architecture_ir_id = super::digest::ir_identity(&ir)?;
    for diagnostic in crate::verify_ir_unit_memberships(&ir) {
        issues.push(super::digest::issue("ir-unit-membership", None, None, &diagnostic));
    }
    let artifact_bytes = crate::framing::canonical_json(&ir)?.len();
    validate_count_limit("artifact-bytes", artifact_bytes, input.limits.max_artifact_bytes, None, issues);
    Ok(ir)
}

fn unit_counts(shards: &[crate::model::CompilerArchitectureShard]) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for shard in shards {
        *counts.entry(shard.unit.unit_id.clone()).or_insert(0) += 1;
    }
    counts
}

fn difference(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<std::collections::BTreeSet<_>>();
    left.iter().filter(|item| !right.contains(item)).cloned().collect()
}

pub(super) fn validate_count_limit(
    name: &str,
    observed: usize,
    limit: usize,
    unit_id: Option<&str>,
    issues: &mut super::diagnostics::Issues,
) {
    if observed > limit {
        issues.push(super::digest::issue(
            "limit-overflow",
            unit_id,
            None,
            &format!("collection `{name}` count {observed} exceeds limit {limit}"),
        ));
    }
}
