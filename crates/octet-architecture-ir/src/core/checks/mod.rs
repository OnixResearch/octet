mod bounds;
mod relations;

pub(super) fn validate_shard(
    shard: &crate::model::CompilerArchitectureShard,
    policy_blake3: &str,
    limits: &crate::model::ArchitectureCollectionLimits,
    issues: &mut super::diagnostics::Issues,
) -> Result<(), String> {
    let unit_id = Some(shard.unit.unit_id.as_str());
    if shard.schema_version != crate::model::COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION {
        issues.push(super::digest::issue(
            "shard-schema",
            unit_id,
            None,
            "compiler architecture shard schema is not supported",
        ));
    }
    validate_unit(&shard.unit, policy_blake3, issues);
    if issues.stale_identity(&shard.shard_id, || super::digest::shard_identity(shard)) {
        issues.push(super::digest::issue(
            "shard-identity",
            unit_id,
            None,
            "compiler architecture shard identity is stale or malformed",
        ));
    }
    if shard.collection_status != crate::model::CollectionStatus::Complete {
        issues.push(super::digest::issue(
            "shard-incomplete",
            unit_id,
            None,
            "collector marked the compiler architecture shard incomplete",
        ));
    }
    if shard.non_claims.is_empty() || shard.non_claims.iter().any(|value| value.trim().is_empty()) {
        issues.push(super::digest::issue(
            "shard-non-claims",
            unit_id,
            None,
            "compiler architecture shard must contain non-empty non-claims",
        ));
    }

    let shard_bytes = crate::framing::canonical_json(shard)?.len();
    super::normalization::validate_count_limit("shard-bytes", shard_bytes, limits.max_shard_bytes, unit_id, issues);
    super::normalization::validate_count_limit("spans", shard.spans.len(), limits.max_spans, unit_id, issues);
    super::normalization::validate_count_limit("unknowns", shard.unknowns.len(), limits.max_unknowns, unit_id, issues);
    bounds::validate_fact_limits(&shard.facts, limits, unit_id, issues);
    validate_spans(&shard.spans, limits, unit_id, issues);
    validate_facts(&shard.facts, &shard.spans, unit_id, issues);
    relations::validate_unknowns(&shard.unknowns, &shard.spans, &shard.facts, unit_id, issues);
    Ok(())
}

/// Verify unit fields and inner identities without trusting a resealed outer IR digest.
#[must_use]
// r[impl octet.project_architecture_gate.test_ownership]
pub fn verify_ir_unit_bindings(ir: &crate::model::CompilerArchitectureIr) -> Vec<String> {
    let mut issues = super::diagnostics::Issues::default();
    for unit in &ir.units {
        validate_unit(unit, &ir.policy_blake3, &mut issues);
    }
    issues.into_messages(false)
}

pub(super) fn validate_unit(
    unit: &crate::model::CompilationUnit,
    policy_blake3: &str,
    issues: &mut super::diagnostics::Issues,
) {
    let unit_id = Some(unit.unit_id.as_str());
    for (field, value) in [
        ("package_name", unit.package_name.as_str()),
        ("package_version", unit.package_version.as_str()),
        ("target_name", unit.target_name.as_str()),
        ("target_kind", unit.target_kind.as_str()),
        ("target_triple", unit.target_triple.as_str()),
    ] {
        if value.trim().is_empty() {
            issues.push(super::digest::issue(
                "unit-field",
                unit_id,
                None,
                &format!("compilation unit field `{field}` is empty"),
            ));
        }
    }
    for (field, value) in [
        ("toolchain_blake3", unit.toolchain_blake3.as_str()),
        ("collector_blake3", unit.collector_blake3.as_str()),
        ("policy_blake3", unit.policy_blake3.as_str()),
        ("source_blake3", unit.source_blake3.as_str()),
        ("invocation_blake3", unit.invocation_blake3.as_str()),
    ] {
        if !super::valid_blake3(value) {
            issues.push(super::digest::issue(
                "unit-identity-field",
                unit_id,
                None,
                &format!("compilation unit field `{field}` is not BLAKE3 hex"),
            ));
        }
    }
    if unit.policy_blake3 != policy_blake3 {
        issues.push(super::digest::issue(
            "unit-policy",
            unit_id,
            None,
            "compilation unit policy identity does not match the active policy",
        ));
    }
    if issues.stale_identity(&unit.unit_id, || super::digest::compilation_unit_identity(unit)) {
        issues.push(super::digest::issue(
            "unit-identity",
            unit_id,
            None,
            "compilation unit identity is stale or malformed",
        ));
    }
    if !super::valid_workspace_path(&unit.source_path) {
        issues.push(super::digest::issue(
            "unit-source-root",
            unit_id,
            None,
            "compilation unit source root is not workspace-relative and canonical",
        ));
    }
    if unit.crate_types.is_empty() {
        issues.push(super::digest::issue("unit-crate-types", unit_id, None, "compilation unit has no crate type"));
    }
    if unit.features.windows(2).any(|pair| pair[0] >= pair[1]) {
        issues.push(super::digest::issue(
            "unit-features",
            unit_id,
            None,
            "compilation unit features are not sorted and unique",
        ));
    }
}

fn validate_spans(
    spans: &[crate::model::SourceSpan],
    limits: &crate::model::ArchitectureCollectionLimits,
    unit_id: Option<&str>,
    issues: &mut super::diagnostics::Issues,
) {
    let mut ids = std::collections::BTreeSet::new();
    for span in spans {
        if !ids.insert(span.span_id.as_str()) {
            issues.push(super::digest::issue(
                "duplicate-span",
                unit_id,
                None,
                "compiler architecture shard contains a duplicate span",
            ));
        }
        validate_span_fields(span, limits, unit_id, issues);
    }
}

fn validate_span_fields(
    span: &crate::model::SourceSpan,
    limits: &crate::model::ArchitectureCollectionLimits,
    unit_id: Option<&str>,
    issues: &mut super::diagnostics::Issues,
) {
    match &span.origin {
        crate::model::SourceOrigin::Workspace { source_path } if !super::valid_workspace_path(source_path) => {
            issues.push(super::digest::issue(
                "span-path",
                unit_id,
                None,
                "workspace source span path is not normalized and workspace-relative",
            ));
        }
        crate::model::SourceOrigin::ExternalGenerated { provenance_blake3 }
            if !super::valid_blake3(provenance_blake3) =>
        {
            issues.push(super::digest::issue(
                "span-external-provenance",
                unit_id,
                None,
                "external generated source span provenance is not BLAKE3 hex",
            ));
        }
        _ => {}
    }
    if span.byte_end < span.byte_start {
        issues.push(super::digest::issue("span-range", unit_id, None, "source span byte range is reversed"));
    }
    if span.line_start == 0 || span.column_start == 0 {
        issues.push(super::digest::issue(
            "span-coordinate",
            unit_id,
            None,
            "source span line and column must be one-based",
        ));
    }
    let byte_length = span.byte_end.saturating_sub(span.byte_start);
    match usize::try_from(byte_length) {
        Ok(byte_length) => {
            super::normalization::validate_count_limit(
                "span-bytes",
                byte_length,
                limits.max_span_bytes,
                unit_id,
                issues,
            );
        }
        Err(_) => issues.push(super::digest::issue(
            "span-byte-range",
            unit_id,
            None,
            "source span byte length is not addressable on this host",
        )),
    }
    if issues.stale_identity(&span.span_id, || super::digest::source_span_identity(span)) {
        issues.push(super::digest::issue("span-identity", unit_id, None, "source span identity is stale or malformed"));
    }
}

/// Check observation identities and reference closure without assigning ownership.
#[must_use]
pub fn verify_observation_set(
    facts: &[crate::model::CompilerArchitectureFact],
    unknowns: &[crate::model::UnknownCompilerFact],
    spans: &[crate::model::SourceSpan],
    limits: &crate::model::ArchitectureCollectionLimits,
) -> Vec<String> {
    let mut issues = super::diagnostics::Issues::default();
    validate_spans(spans, limits, None, &mut issues);
    validate_facts(facts, spans, None, &mut issues);
    relations::validate_unknowns(unknowns, spans, facts, None, &mut issues);
    issues.into_messages(true)
}

fn validate_facts(
    facts: &[crate::model::CompilerArchitectureFact],
    spans: &[crate::model::SourceSpan],
    unit_id: Option<&str>,
    issues: &mut super::diagnostics::Issues,
) {
    let span_ids = spans.iter().map(|span| span.span_id.as_str()).collect::<std::collections::BTreeSet<_>>();
    let mut fact_ids = std::collections::BTreeSet::new();
    let mut item_ids = std::collections::BTreeSet::new();
    let mut type_ids = std::collections::BTreeSet::new();
    let mut trait_ids = std::collections::BTreeSet::new();
    let mut signature_ids = std::collections::BTreeSet::new();

    for fact in facts {
        if !fact_ids.insert(fact.fact_id()) {
            issues.push(super::digest::issue(
                "duplicate-fact",
                unit_id,
                Some(fact.fact_id()),
                "compiler architecture shard contains a duplicate fact",
            ));
        }
        if issues.stale_identity(fact.fact_id(), || super::digest::fact_identity(fact)) {
            issues.push(super::digest::issue(
                "fact-identity",
                unit_id,
                Some(fact.fact_id()),
                "compiler architecture fact identity is stale or malformed",
            ));
        }
        match fact {
            crate::model::CompilerArchitectureFact::Item { item_id, .. } => {
                item_ids.insert(item_id.as_str());
            }
            crate::model::CompilerArchitectureFact::Type { type_id, .. } => {
                type_ids.insert(type_id.as_str());
            }
            crate::model::CompilerArchitectureFact::Trait { trait_id, .. } => {
                trait_ids.insert(trait_id.as_str());
            }
            crate::model::CompilerArchitectureFact::Signature { signature_id, .. } => {
                signature_ids.insert(signature_id.as_str());
            }
            _ => {}
        }
    }

    let index = relations::FactReferenceIndex {
        spans: span_ids,
        items: item_ids,
        types: type_ids,
        traits: trait_ids,
        signatures: signature_ids,
    };
    for fact in facts {
        relations::validate_fact_references(fact, &index, unit_id, issues);
    }
}
