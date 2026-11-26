//! Deterministic projection construction, conversion-receipt hashing, and read-only verification.

/// Ingests one saved producer report cleanly without executing external commands, returning
/// distinct converted projection and conversion receipt DTOs.
pub(crate) fn ingest_report(
    profile: &super::IntakeProfile,
    report_bytes: &[u8],
    source_root: &std::path::Path,
) -> Result<(super::Projection, super::ConversionReceipt), String> {
    let (projection, receipt) = build_once(profile, report_bytes, source_root)?;
    let (replayed_projection, replayed_receipt) = build_once(profile, report_bytes, source_root)?;
    let is_replay_deterministic = projection.projection_blake3 == replayed_projection.projection_blake3
        && receipt.receipt_blake3 == replayed_receipt.receipt_blake3;
    if !is_replay_deterministic {
        return Err("structural report intake is non-deterministic across replay".to_owned());
    }
    Ok((projection, receipt))
}

/// Verifies saved report, projection, and conversion receipt bytes in read-only mode against
/// `source_root`.
pub(crate) fn verify_conversion(
    registry: &super::IntakeProfileRegistry,
    report_bytes: &[u8],
    projection_bytes: &[u8],
    receipt_bytes: &[u8],
    source_root: &std::path::Path,
) -> Result<super::ConversionReceipt, String> {
    let saved_projection: super::Projection = serde_json::from_slice(projection_bytes)
        .map_err(|error| format!("parse saved structural projection: {error}"))?;
    let saved_receipt: super::ConversionReceipt = serde_json::from_slice(receipt_bytes)
        .map_err(|error| format!("parse saved structural conversion receipt: {error}"))?;
    if saved_projection.schema_version != super::PROJECTION_SCHEMA_V1 {
        return Err(format!("unsupported projection schema `{}`", saved_projection.schema_version));
    }
    if saved_receipt.schema_version != super::RECEIPT_SCHEMA_V1 {
        return Err(format!("unsupported receipt schema `{}`", saved_receipt.schema_version));
    }
    let profile = super::select_profile(registry, &saved_receipt.profile_id)?;
    let (recomputed_projection, recomputed_receipt) = ingest_report(profile, report_bytes, source_root)?;
    let mut diagnostics = recomputed_receipt.diagnostics.clone();
    check_saved_identities(
        &saved_projection,
        &saved_receipt,
        &recomputed_projection,
        &recomputed_receipt,
        &mut diagnostics,
    );
    let status =
        resolve_status(&recomputed_projection.unsupported_reasons, &recomputed_receipt.source_checks, &diagnostics);
    let mut out = super::ConversionReceipt {
        status,
        diagnostics,
        receipt_blake3: String::new(),
        ..recomputed_receipt
    };
    out.receipt_blake3 = compute_conversion_hash(&out)?;
    Ok(out)
}

fn build_once(
    profile: &super::IntakeProfile,
    report_bytes: &[u8],
    source_root: &std::path::Path,
) -> Result<(super::Projection, super::ConversionReceipt), String> {
    let adapted = dispatch_adapter(profile, report_bytes)?;
    let report_blake3 = super::blake3_hex(report_bytes);
    let projection = finalize_projection(profile, &adapted, report_blake3.clone())?;
    let (source_checks, source_diagnostics) = super::source::verify_records(&projection.records, source_root);
    let mut diagnostics = adapted.diagnostics;
    diagnostics.extend(source_diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let status = resolve_status(&projection.unsupported_reasons, &source_checks, &diagnostics);
    let mut receipt = super::ConversionReceipt {
        schema_version: super::RECEIPT_SCHEMA_V1.to_owned(),
        profile_id: profile.profile_id.clone(),
        producer: profile.producer.clone(),
        original_report_schema: adapted.report_schema,
        original_report_blake3: report_blake3,
        projection_schema_version: super::PROJECTION_SCHEMA_V1.to_owned(),
        projection_blake3: projection.projection_blake3.clone(),
        mapping_id: profile.mapping_id.clone(),
        mapping_version: profile.mapping_version.clone(),
        namespace: profile.namespace_prefix.clone(),
        evidence_class: profile.evidence_class.clone(),
        evidence_tier: profile.evidence_tier.clone(),
        status,
        command_executed: false,
        replay_deterministic: true,
        active_record_count: projection.records.len(),
        historical_record_count: projection.historical_records.len(),
        source_checks,
        diagnostics,
        non_claims: projection.non_claims.clone(),
        receipt_blake3: String::new(),
    };
    receipt.receipt_blake3 = compute_conversion_hash(&receipt)?;
    Ok((projection, receipt))
}

fn dispatch_adapter(profile: &super::IntakeProfile, report_bytes: &[u8]) -> Result<super::AdapterOutput, String> {
    match profile.producer.as_str() {
        "cairn" => super::cairn::adapt_report(profile, report_bytes),
        "buoy-comparison-pilot" => super::buoy::adapt_report(profile, report_bytes),
        other => Ok(super::AdapterOutput {
            report_schema: String::new(),
            records: Vec::new(),
            historical_records: Vec::new(),
            unsupported_reasons: vec![format!("unsupported producer `{other}`")],
            diagnostics: Vec::new(),
        }),
    }
}

fn finalize_projection(
    profile: &super::IntakeProfile,
    adapted: &super::AdapterOutput,
    report_blake3: String,
) -> Result<super::Projection, String> {
    let mut projection = super::Projection {
        schema_version: super::PROJECTION_SCHEMA_V1.to_owned(),
        profile_id: profile.profile_id.clone(),
        producer: profile.producer.clone(),
        original_report_schema: adapted.report_schema.clone(),
        original_report_blake3: report_blake3,
        mapping_id: profile.mapping_id.clone(),
        mapping_version: profile.mapping_version.clone(),
        namespace: profile.namespace_prefix.clone(),
        evidence_class: profile.evidence_class.clone(),
        evidence_tier: profile.evidence_tier.clone(),
        records: adapted.records.clone(),
        historical_records: adapted.historical_records.clone(),
        unsupported_reasons: adapted.unsupported_reasons.clone(),
        non_claims: profile.non_claims.clone(),
        projection_blake3: String::new(),
    };
    let mut val =
        serde_json::to_value(&projection).map_err(|error| format!("serialize projection hash input: {error}"))?;
    if let Some(obj) = val.as_object_mut() {
        obj.remove("projection_blake3");
    }
    projection.projection_blake3 = super::domain_hash_json(super::PROJECTION_DOMAIN, &val)?;
    Ok(projection)
}

fn check_saved_identities(
    saved_projection: &super::Projection,
    saved_receipt: &super::ConversionReceipt,
    recomputed_projection: &super::Projection,
    recomputed_receipt: &super::ConversionReceipt,
    diagnostics: &mut Vec<String>,
) {
    if saved_projection.projection_blake3 != recomputed_projection.projection_blake3 {
        diagnostics.push(format!(
            "saved projection identity `{}` does not match recomputed `{}`",
            saved_projection.projection_blake3, recomputed_projection.projection_blake3
        ));
    }
    if saved_receipt.original_report_blake3 != recomputed_receipt.original_report_blake3 {
        diagnostics.push(format!(
            "saved original_report_blake3 `{}` does not match report `{}`",
            saved_receipt.original_report_blake3, recomputed_receipt.original_report_blake3
        ));
    }
    if saved_receipt.projection_blake3 != saved_projection.projection_blake3 {
        diagnostics.push("receipt `projection_blake3` disagrees with saved projection".to_owned());
    }
    if saved_receipt.receipt_blake3 != recomputed_receipt.receipt_blake3 {
        diagnostics.push(format!(
            "saved receipt_blake3 `{}` does not match recomputed `{}`",
            saved_receipt.receipt_blake3, recomputed_receipt.receipt_blake3
        ));
    }
    if saved_receipt.command_executed {
        diagnostics.push("saved conversion receipt unlawfully claims runtime command execution".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
}

fn resolve_status(
    unsupported_reasons: &[String],
    source_checks: &[super::SourceMatchOutcome],
    diagnostics: &[String],
) -> String {
    if !unsupported_reasons.is_empty() {
        return super::STATUS_UNSUPPORTED.to_owned();
    }
    let has_stale_source = source_checks.iter().any(|check| check.status != super::SOURCE_STATUS_CURRENT);
    if has_stale_source {
        return super::STATUS_STALE_BINDING.to_owned();
    }
    if !diagnostics.is_empty() {
        return super::STATUS_FAILED.to_owned();
    }
    super::STATUS_VALID.to_owned()
}

fn compute_conversion_hash(receipt: &super::ConversionReceipt) -> Result<String, String> {
    let mut val = serde_json::to_value(receipt).map_err(|error| format!("serialize receipt hash input: {error}"))?;
    if let Some(obj) = val.as_object_mut() {
        obj.remove("receipt_blake3");
    }
    super::domain_hash_json(super::RECEIPT_DOMAIN, &val)
}

/// Renders canonical deterministic JSON (sorted object keys, no whitespace) iteratively.
pub(crate) fn canonical_json(value: &serde_json::Value) -> String {
    enum Frame<'a> {
        Val(&'a serde_json::Value),
        Lit(&'static str),
        Key(&'a str),
    }
    let mut out = String::new();
    let mut stack = vec![Frame::Val(value)];
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Lit(lit) => out.push_str(lit),
            Frame::Key(key) => {
                out.push_str(&serde_json::to_string(key).unwrap_or_default());
                out.push(':');
            }
            Frame::Val(serde_json::Value::Null) => out.push_str("null"),
            Frame::Val(serde_json::Value::Bool(flag)) => out.push_str(if *flag { "true" } else { "false" }),
            Frame::Val(serde_json::Value::Number(num)) => out.push_str(&num.to_string()),
            Frame::Val(serde_json::Value::String(text)) => {
                out.push_str(&serde_json::to_string(text).unwrap_or_default());
            }
            Frame::Val(serde_json::Value::Array(items)) => {
                out.push('[');
                stack.push(Frame::Lit("]"));
                for idx in (0..items.len()).rev() {
                    if idx.saturating_add(1) < items.len() {
                        stack.push(Frame::Lit(","));
                    }
                    stack.push(Frame::Val(&items[idx]));
                }
            }
            Frame::Val(serde_json::Value::Object(map)) => {
                out.push('{');
                stack.push(Frame::Lit("}"));
                let mut entries = map.iter().collect::<Vec<_>>();
                entries.sort_by(|left, right| left.0.cmp(right.0));
                for idx in (0..entries.len()).rev() {
                    if idx.saturating_add(1) < entries.len() {
                        stack.push(Frame::Lit(","));
                    }
                    stack.push(Frame::Val(entries[idx].1));
                    stack.push(Frame::Key(entries[idx].0));
                }
            }
        }
    }
    out
}

/// Verifies a saved report/projection/receipt bundle and checks active non-historical clause match.
pub(crate) fn verify_clause_files(
    bundle_paths: [&[u8]; 4],
    expected_profile: &str,
    artifact_root: &std::path::Path,
    clause_ids: (&str, &[String]),
    checked_scope: &[String],
) -> Result<super::ConversionReceipt, (String, Vec<String>)> {
    let registry =
        super::parse_profile_registry(bundle_paths[0]).map_err(|err| (super::STATUS_FAILED.to_owned(), vec![err]))?;
    let verified = verify_conversion(&registry, bundle_paths[1], bundle_paths[2], bundle_paths[3], artifact_root)
        .map_err(|err| (super::STATUS_FAILED.to_owned(), vec![err]))?;
    if verified.profile_id != expected_profile {
        return Err((super::STATUS_FAILED.to_owned(), vec![format!(
            "expected profile `{expected_profile}`, got `{}`",
            verified.profile_id
        )]));
    }
    if verified.status != super::STATUS_VALID {
        return Err((verified.status.clone(), verified.diagnostics.clone()));
    }
    let projection: super::Projection = serde_json::from_slice(bundle_paths[2])
        .map_err(|err| (super::STATUS_FAILED.to_owned(), vec![format!("decode projection: {err}")]))?;
    let (clause_id, allowed_reqs) = clause_ids;
    let has_match = projection.records.iter().any(|record| {
        let is_req = record.requirement_id == clause_id || allowed_reqs.iter().any(|id| id == &record.requirement_id);
        let has_locator = record.anchor.as_ref().is_some_and(|a| checked_scope.iter().any(|s| s == &a.scoped_locator));
        let has_path = checked_scope.iter().any(|s| s == &record.source_path);
        is_req && (has_locator || has_path)
    });
    if !has_match {
        return Err((super::STATUS_FAILED.to_owned(), vec![format!(
            "carries no active non-historical anchor matching clause `{clause_id}` and checked_scope"
        )]));
    }
    Ok(verified)
}
