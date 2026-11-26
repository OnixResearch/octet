pub(in crate::requirements) fn evaluate(
    evidence: &super::super::evidence::Bundle,
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    artifact_base: Option<&std::path::Path>,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    let mut evidence_ids = Vec::new();
    for deck in &evidence.proof_decks {
        if !clause.allowed_evidence_ids.contains(&deck.evidence_id) {
            continue;
        }
        let deck_path =
            match super::super::resolve_artifact_locator(source_root, artifact_base, &deck.deck, "proof deck") {
                Ok(path) => path,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };
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
            let locators = CardLocators {
                artifact_root: &deck.artifact_root,
                source_root,
                artifact_base,
            };
            validate_card(card, clause, locators, &mut card_diagnostics);
            if card_diagnostics.is_empty() {
                evidence_ids.push(card_id.to_owned());
            } else {
                diagnostics.extend(card_diagnostics);
            }
        }
    }
    if !clause.required_negative_controls.is_empty() && !evidence_ids.is_empty() {
        verify_required_negative_controls(
            evidence,
            clause,
            source_root,
            artifact_base,
            &evidence_ids,
            diagnostics,
        );
    }
    if evidence_ids.is_empty() && diagnostics.is_empty() {
        diagnostics.push(format!(
            "clause `{}` has no retained formal-proof card among allowed evidence ids",
            clause.clause_id
        ));
    }
    let status = if evidence_ids.is_empty() || !diagnostics.is_empty() {
        super::super::STATUS_MISSING
    } else {
        super::super::STATUS_PASSED
    };
    (status.to_owned(), evidence_ids)
}

/// The three path inputs every proof-card check resolves through.
#[derive(Clone, Copy)]
pub(in crate::requirements) struct CardLocators<'a> {
    pub(in crate::requirements) artifact_root: &'a str,
    pub(in crate::requirements) source_root: &'a std::path::Path,
    pub(in crate::requirements) artifact_base: Option<&'a std::path::Path>,
}

fn verify_required_negative_controls(
    evidence: &super::super::evidence::Bundle,
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    artifact_base: Option<&std::path::Path>,
    evidence_ids: &[String],
    diagnostics: &mut Vec<String>,
) {
    let mut satisfied = std::collections::BTreeSet::new();
    for deck in &evidence.proof_decks {
        if !clause.allowed_evidence_ids.contains(&deck.evidence_id) {
            continue;
        }
        let Ok(deck_path) =
            super::super::resolve_artifact_locator(source_root, artifact_base, &deck.deck, "proof deck")
        else {
            continue;
        };
        if let Ok(deck_json) = super::super::read_json_bounded::<serde_json::Value>(&deck_path)
            && let Some(cards) = deck_json.get("cards").and_then(serde_json::Value::as_array)
        {
            for card in cards {
                let Some(card_id) = card.get("proof_or_property_id").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                if evidence_ids.iter().any(|id| id == card_id)
                    && let Some(mutation_id) = super::negative::demonstrated_mutation_id(card)
                {
                    satisfied.insert(mutation_id.to_owned());
                }
            }
        }
    }
    for required_control in &clause.required_negative_controls {
        if !satisfied.contains(required_control.as_str()) {
            diagnostics.push(format!(
                "clause `{}` is missing required negative control `{required_control}`",
                clause.clause_id
            ));
        }
    }
}

pub(in crate::requirements) fn validate_card(
    card: &serde_json::Value,
    clause: &super::super::contract::RequirementClause,
    locators: CardLocators<'_>,
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
    if !clause.allowed_source_requirement_ids.is_empty() {
        match card.get("source_requirement_or_policy_id").and_then(serde_json::Value::as_str) {
            Some(source_requirement) if !source_requirement.trim().is_empty() => {
                if !clause.allowed_source_requirement_ids.iter().any(|allowed| allowed == source_requirement) {
                    diagnostics.push(format!(
                        "proof card source requirement `{source_requirement}` is not allowed for clause `{}`",
                        clause.clause_id
                    ));
                }
            }
            _ => diagnostics.push(format!(
                "proof card is missing `source_requirement_or_policy_id` for clause `{}`",
                clause.clause_id
            )),
        }
    }
    match card.get("checked_rust_source_scope").and_then(serde_json::Value::as_str) {
        Some(scope) if !scope.trim().is_empty() => {
            if !clause.checked_scope.iter().any(|allowed| allowed == scope) {
                diagnostics.push(format!(
                    "proof card checked scope `{scope}` is not allowed for clause `{}`",
                    clause.clause_id
                ));
            }
        }
        _ => diagnostics
            .push(format!("proof card is missing `checked_rust_source_scope` for clause `{}`", clause.clause_id)),
    }
    let card_diagnostics = card.get("diagnostics").and_then(serde_json::Value::as_array);
    if card_diagnostics.is_some_and(|items| !items.is_empty()) {
        diagnostics.push("proof card carries diagnostics".to_owned());
    }
    let non_claims = card.get("non_claims").and_then(serde_json::Value::as_array);
    if non_claims.is_none_or(std::vec::Vec::is_empty) {
        diagnostics.push("proof card must carry explicit non-claims".to_owned());
    }
    if !clause.required_negative_controls.is_empty() {
        super::negative::validate_control(card, clause, diagnostics);
    }
    validate_artifacts(card, locators.artifact_root, locators.source_root, locators.artifact_base, diagnostics);
    validate_envelope(card, clause, diagnostics);
}

pub(in crate::requirements) fn validate_artifacts(
    card: &serde_json::Value,
    artifact_root: &str,
    source_root: &std::path::Path,
    artifact_base: Option<&std::path::Path>,
    diagnostics: &mut Vec<String>,
) {
    let artifact_root =
        match super::super::resolve_artifact_locator(source_root, artifact_base, artifact_root, "artifact root") {
            Ok(path) => path,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                return;
            }
        };
    if let Some(identities) = card.get("artifact_blake3_identities").and_then(serde_json::Value::as_array) {
        if identities.is_empty() {
            diagnostics.push("proof card has empty artifact_blake3_identities".to_owned());
            return;
        }
        for identity in identities {
            let Some(locator) = identity
                .get("artifact")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
            else {
                diagnostics.push("proof artifact entry is missing `artifact` locator".to_owned());
                continue;
            };
            let Some(expected) =
                identity.get("blake3").and_then(serde_json::Value::as_str).filter(|value| !value.trim().is_empty())
            else {
                diagnostics.push(format!("proof artifact `{locator}` entry is missing `blake3` digest"));
                continue;
            };
            let path = super::super::resolve_path(&artifact_root, locator);
            match crate::envelope::read_regular_non_symlink_file(&path) {
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
