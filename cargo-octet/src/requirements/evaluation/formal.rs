pub(in crate::requirements) fn evaluate(
    evidence: &super::super::evidence::Bundle,
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    let mut evidence_ids = Vec::new();
    for deck in &evidence.proof_decks {
        if !clause.allowed_evidence_ids.contains(&deck.evidence_id) {
            continue;
        }
        let deck_path = super::super::resolve_path(source_root, &deck.deck);
        let deck_json = match super::super::read_json_bounded::<serde_json::Value>(&deck_path) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        if deck_json.get("schema_version").and_then(serde_json::Value::as_str)
            != Some(super::super::PROOF_CARDS_SCHEMA_VERSION)
        {
            diagnostics.push(format!("proof deck `{}` has an unsupported schema", deck_path.display()));
            continue;
        }
        let Some(cards) = deck_json.get("cards").and_then(serde_json::Value::as_array) else {
            diagnostics.push(format!("proof deck `{}` has no cards array", deck_path.display()));
            continue;
        };
        evidence_ids.reserve(cards.len());
        for card in cards {
            let Some(card_id) = card.get("proof_or_property_id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if !clause.allowed_evidence_ids.iter().any(|allowed| allowed == card_id) {
                continue;
            }
            let mut card_diagnostics = Vec::new();
            validate_card(card, clause, &deck.artifact_root, source_root, &mut card_diagnostics);
            if card_diagnostics.is_empty() {
                evidence_ids.push(card_id.to_owned());
            } else {
                diagnostics.extend(card_diagnostics);
            }
        }
    }
    if evidence_ids.is_empty() && diagnostics.is_empty() {
        diagnostics.push(format!(
            "clause `{}` has no retained formal-proof card among allowed evidence ids",
            clause.clause_id
        ));
    }
    let status = if evidence_ids.is_empty() {
        super::super::STATUS_MISSING
    } else {
        super::super::STATUS_PASSED
    };
    (status.to_owned(), evidence_ids)
}

pub(in crate::requirements) fn validate_card(
    card: &serde_json::Value,
    clause: &super::super::contract::RequirementClause,
    artifact_root: &str,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let status = card.get("status").and_then(serde_json::Value::as_str).unwrap_or_default();
    let tier = card.get("evidence_tier").and_then(serde_json::Value::as_str).unwrap_or_default();
    if status != super::super::CHECK_STATUS_PASSED {
        diagnostics.push(format!(
            "proof card status `{status}` is not `{CHECK_STATUS_PASSED}`",
            CHECK_STATUS_PASSED = super::super::CHECK_STATUS_PASSED
        ));
    }
    if tier != super::super::DISPOSITION_FORMAL_PROOF {
        diagnostics.push(format!(
            "proof card evidence tier `{tier}` is not `{DISPOSITION_FORMAL_PROOF}`",
            DISPOSITION_FORMAL_PROOF = super::super::DISPOSITION_FORMAL_PROOF
        ));
    }
    if let Some(source_requirement) = card.get("source_requirement_or_policy_id").and_then(serde_json::Value::as_str)
        && !clause.allowed_source_requirement_ids.is_empty()
        && !clause.allowed_source_requirement_ids.iter().any(|allowed| allowed == source_requirement)
    {
        diagnostics.push(format!(
            "proof card source requirement `{source_requirement}` is not allowed for clause `{}`",
            clause.clause_id
        ));
    }
    if let Some(scope) = card.get("checked_rust_source_scope").and_then(serde_json::Value::as_str)
        && !clause.checked_scope.iter().any(|allowed| allowed == scope)
    {
        diagnostics
            .push(format!("proof card checked scope `{scope}` is not allowed for clause `{}`", clause.clause_id));
    }
    let card_diagnostics = card.get("diagnostics").and_then(serde_json::Value::as_array);
    if card_diagnostics.is_some_and(|items| !items.is_empty()) {
        diagnostics.push("proof card carries diagnostics".to_owned());
    }
    let non_claims = card.get("non_claims").and_then(serde_json::Value::as_array);
    if non_claims.is_none_or(std::vec::Vec::is_empty) {
        diagnostics.push("proof card must carry explicit non-claims".to_owned());
    }
    if clause.required_negative_controls.is_empty() {
        // no negative control required
    } else {
        super::negative::validate_control(card, clause, diagnostics);
    }
    validate_artifacts(card, artifact_root, source_root, diagnostics);
    validate_envelope(card, clause, diagnostics);
}

pub(in crate::requirements) fn validate_artifacts(
    card: &serde_json::Value,
    artifact_root: &str,
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) {
    let artifact_root = super::super::resolve_path(source_root, artifact_root);
    if let Some(identities) = card.get("artifact_blake3_identities").and_then(serde_json::Value::as_array) {
        for identity in identities {
            let Some(locator) = identity.get("artifact").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(expected) = identity.get("blake3").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let path = super::super::resolve_path(&artifact_root, locator);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let actual = super::super::hash_bytes(&bytes);
                    if actual != expected {
                        diagnostics
                            .push(format!("proof artifact `{locator}` changed: expected {expected}, actual {actual}"));
                    }
                }
                Err(error) => diagnostics.push(format!("proof artifact `{locator}` is missing: {error}")),
            }
        }
    } else {
        diagnostics.push("proof card has no retained artifact identities".to_owned());
    }
}

pub(in crate::requirements) fn validate_envelope(
    card: &serde_json::Value,
    clause: &super::super::contract::RequirementClause,
    diagnostics: &mut Vec<String>,
) {
    if let Some(envelope) = card.get("formal_proof_envelope") {
        let envelope_schema = envelope.get("schema_version").and_then(serde_json::Value::as_str);
        if envelope_schema != Some(super::super::FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION) {
            diagnostics.push("formal proof envelope has an unsupported schema".to_owned());
        }
        if envelope.get("status").and_then(serde_json::Value::as_str) != Some(super::super::CHECK_STATUS_PASSED) {
            diagnostics.push("formal proof envelope status is not passed".to_owned());
        }
        if let Some(provider) = &clause.required_provider_id {
            let actual = envelope.get("provider_id").and_then(serde_json::Value::as_str);
            if actual != Some(provider.as_str()) {
                diagnostics.push(format!(
                    "formal proof provider `{}` does not match required provider `{provider}`",
                    actual.unwrap_or("missing")
                ));
            }
        }
    } else if clause.required_provider_id.is_some() {
        diagnostics.push("required provider evidence is missing a formal proof envelope".to_owned());
    }
}
