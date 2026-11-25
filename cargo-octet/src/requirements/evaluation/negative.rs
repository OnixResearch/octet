pub(in crate::requirements) fn validate_control(
    card: &serde_json::Value,
    clause: &super::super::contract::RequirementClause,
    diagnostics: &mut Vec<String>,
) {
    let Some(negative) = card.get("negative_check") else {
        diagnostics.push(format!(
            "clause `{}` requires an attributable negative control but the card has none",
            clause.clause_id
        ));
        return;
    };
    if negative.get("schema_version").and_then(serde_json::Value::as_str)
        != Some(super::super::NEGATIVE_CHECK_SCHEMA_VERSION_V2)
    {
        diagnostics.push("negative control is not a v2 attributable record".to_owned());
        return;
    }
    if negative.get("status").and_then(serde_json::Value::as_str) != Some(super::super::NEGATIVE_STATUS_DEMONSTRATED) {
        diagnostics.push("negative control status is not property_demonstrated".to_owned());
    }
    if negative.get("mutant_card_status").and_then(serde_json::Value::as_str) != Some(super::super::STATUS_FAILED) {
        diagnostics.push("negative control mutant_card_status is not failed".to_owned());
    }
    if let Some(mutant_tier) = negative.get("mutant_card_evidence_tier").and_then(serde_json::Value::as_str)
        && !mutant_tier.is_empty()
        && mutant_tier != super::super::DISPOSITION_FORMAL_PROOF
    {
        diagnostics.push(format!("negative control mutant_card_evidence_tier `{mutant_tier}` is not formal_proof"));
    }
    let Some(evidence) = negative.get("evidence").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control has no retained verifier evidence".to_owned());
        return;
    };
    let string = |field: &str| evidence.get(field).and_then(serde_json::Value::as_str).unwrap_or_default();
    if string("verifier").trim().is_empty() {
        diagnostics.push("negative control evidence has no verifier identity".to_owned());
    }
    let card_id = card.get("proof_or_property_id").and_then(serde_json::Value::as_str).unwrap_or_default();
    if string("proof_or_property_id") != card_id {
        diagnostics.push("negative control evidence proof_or_property_id does not match the proof card".to_owned());
    }
    if string("approved_specification_id").trim().is_empty() {
        diagnostics.push("negative control evidence has no approved specification identity".to_owned());
    }
    if string("source_requirement_or_policy_id").trim().is_empty() {
        diagnostics.push("negative control evidence has no source requirement identity".to_owned());
    }
    let scope = string("checked_rust_source_scope");
    if scope.trim().is_empty() {
        diagnostics.push("negative control evidence has no checked_rust_source_scope".to_owned());
    } else if !clause.checked_scope.iter().any(|allowed| allowed == scope) {
        diagnostics.push(format!(
            "negative control evidence checked_rust_source_scope `{scope}` is not allowed for clause `{}`",
            clause.clause_id
        ));
    }
    let evidence_tier = string("evidence_tier");
    if evidence_tier != super::super::DISPOSITION_FORMAL_PROOF {
        diagnostics.push(format!("negative control evidence tier `{evidence_tier}` is not formal_proof"));
    }
    validate_control_runs(negative, evidence, card_id, diagnostics);
}

fn validate_control_runs(
    negative: &serde_json::Value,
    evidence: &serde_json::Map<String, serde_json::Value>,
    card_id: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(baseline) = evidence.get("baseline").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no baseline verifier run".to_owned());
        return;
    };
    let Some(mutant) = evidence.get("mutant").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no mutant verifier run".to_owned());
        return;
    };
    if baseline.get("outcome").and_then(serde_json::Value::as_str) != Some("accepted") {
        diagnostics.push("negative control baseline verifier run was not accepted".to_owned());
    }
    if baseline
        .get("rejection_attribution")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.is_empty())
    {
        diagnostics.push("negative control baseline run unexpectedly attributes a rejection".to_owned());
    }
    if mutant.get("outcome").and_then(serde_json::Value::as_str) != Some("rejected") {
        diagnostics.push("negative control mutant verifier run was not rejected".to_owned());
    }
    let attribution = mutant.get("rejection_attribution").and_then(serde_json::Value::as_str);
    if attribution != Some(card_id) {
        diagnostics.push(format!(
            "negative control rejection is attributed to `{}` instead of the expected obligation `{card_id}`",
            attribution.unwrap_or("missing")
        ));
    }
    validate_mutation(negative, evidence, diagnostics);
}

pub(in crate::requirements) fn validate_mutation(
    negative: &serde_json::Value,
    evidence: &serde_json::Map<String, serde_json::Value>,
    diagnostics: &mut Vec<String>,
) {
    let Some(mutation) = evidence.get("mutation").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no declared mutation identity".to_owned());
        return;
    };
    if mutation.get("patch_blake3").and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
        diagnostics.push("negative control mutation has no patch identity".to_owned());
    }
    let baseline_source =
        mutation.get("baseline_source_blake3").and_then(serde_json::Value::as_str).unwrap_or_default();
    let mutant_source = mutation.get("mutant_source_blake3").and_then(serde_json::Value::as_str).unwrap_or_default();
    if baseline_source.is_empty() || mutant_source.is_empty() {
        diagnostics.push("negative control mutation does not retain both source identities".to_owned());
    } else if baseline_source == mutant_source {
        diagnostics.push("negative control mutant source bytes are unchanged".to_owned());
    }
    let Some(measured) = negative.get("measured").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control record has no measured identities".to_owned());
        return;
    };
    for field in [
        "baseline_source_blake3",
        "mutant_source_blake3",
        "mutation_id",
        "patch_blake3",
    ] {
        if measured.get(field).and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
            diagnostics.push(format!("negative control measured identity `{field}` is missing"));
        }
    }
    if measured.get("baseline_source_blake3").and_then(serde_json::Value::as_str) != Some(baseline_source) {
        diagnostics.push("negative control measured baseline source does not match the declared mutation".to_owned());
    }
    if measured.get("mutant_source_blake3").and_then(serde_json::Value::as_str) != Some(mutant_source) {
        diagnostics.push("negative control measured mutant source does not match the declared mutation".to_owned());
    }
    let mutation_id = mutation.get("mutation_id").and_then(serde_json::Value::as_str).unwrap_or_default();
    if mutation_id.trim().is_empty()
        || measured.get("mutation_id").and_then(serde_json::Value::as_str) != Some(mutation_id)
    {
        diagnostics.push("negative control mutation_id is missing or inconsistent with measured identities".to_owned());
    }
}

pub(in crate::requirements) fn demonstrated_mutation_id(card: &serde_json::Value) -> Option<&str> {
    card.get("negative_check")
        .and_then(|negative| negative.get("measured"))
        .and_then(|measured| measured.get("mutation_id"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
}
