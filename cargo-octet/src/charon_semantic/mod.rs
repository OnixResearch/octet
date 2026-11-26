mod comparison;
mod consumer;
mod core;
mod model;
mod shell;

use std::collections::BTreeMap;

pub(crate) use model::ANALYSIS_ARTIFACT;
use model::ArtifactBinding;
pub(crate) use model::COMPARISON_ARTIFACT;
use model::CharonSemanticEnvelope;
use model::CharonSemanticEvaluation;
pub(crate) use model::CharonSemanticReceiptDetails;
use model::ComparisonParent;
use model::DETAILS_SCHEMA;
use model::ENVELOPE_SCHEMA;
use model::ENVELOPE_SCHEMA_V2;
use model::EXPECTED_COMPATIBILITY_IDENTITY;
pub(crate) use model::EXPECTED_PROFILE_IDENTITY;
use model::EXPECTED_PROVIDER_PROFILE_IDENTITY;
pub(crate) use model::PROFILE_ARTIFACT;
use model::PROFILE_ID;
pub(crate) use model::PROJECTION_ARTIFACT;
pub(crate) use model::RAW_SNAPSHOT_ARTIFACT;
use model::RECORDED_ONLY_ROLE;
use model::SemanticComparison;
use model::SemanticFinding;
pub(crate) use shell::execute_from_profile;

const PROFILE_JSON: &str = include_str!("../../../standards/charon-semantic/generated/profile.json");
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;

pub(crate) fn validate_profile_text(
    selected_profile: &str,
    profile_text: &str,
) -> Result<(String, Vec<String>), String> {
    if consumer::is_workspace_profile_json(profile_text) {
        let validated = consumer::parse_and_validate(profile_text)
            .map_err(|diagnostics| format!("invalid Charon workspace profile:\n- {}", diagnostics.join("\n- ")))?;
        if validated.profile.profile_id != selected_profile {
            return Err(format!(
                "Charon semantic `--profile {selected_profile}` does not match config profile `{}`",
                validated.profile.profile_id
            ));
        }
        Ok((validated.profile_identity, validated.profile.extraction.command_argv))
    } else {
        if selected_profile != PROFILE_ID {
            return Err(format!("unsupported Charon semantic profile `{selected_profile}`"));
        }
        if prefixed_identity(profile_text.as_bytes()) != EXPECTED_PROFILE_IDENTITY {
            return Err("Charon semantic profile bytes do not match the shipped profile identity".to_owned());
        }
        Ok((EXPECTED_PROFILE_IDENTITY.to_owned(), profile_command_argv()?))
    }
}

#[cfg(test)]
pub(crate) fn build_envelope(snapshot_text: String, comparison_parent_texts: Vec<String>) -> Result<String, String> {
    build_envelope_for_profile(snapshot_text, comparison_parent_texts, None)
}

pub(crate) fn build_envelope_for_profile(
    snapshot_text: String,
    comparison_parent_texts: Vec<String>,
    profile_text: Option<String>,
) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(&snapshot_text)
        .map_err(|error| format!("parse Charon snapshot before envelope construction: {error}"))?;
    for parent in &comparison_parent_texts {
        serde_json::from_str::<ComparisonParent>(parent)
            .map_err(|error| format!("parse Charon comparison parent before envelope construction: {error}"))?;
    }
    let schema_version = if profile_text.is_some() {
        ENVELOPE_SCHEMA_V2
    } else {
        ENVELOPE_SCHEMA
    };
    render_json(&CharonSemanticEnvelope {
        schema_version: schema_version.to_owned(),
        snapshot_text,
        comparison_parent_texts,
        profile_text,
    })
}

pub(crate) fn envelope_profile_identity(envelope_text: &str) -> Option<String> {
    let envelope = serde_json::from_str::<CharonSemanticEnvelope>(envelope_text).ok()?;
    match envelope.profile_text {
        Some(text) => Some(prefixed_identity(text.as_bytes())),
        None => Some(EXPECTED_PROFILE_IDENTITY.to_owned()),
    }
}

type ResolvedEnvelopeProfile = (String, Option<consumer::ValidatedWorkspace>);

fn resolve_envelope_profile(envelope: &CharonSemanticEnvelope) -> Result<ResolvedEnvelopeProfile, Vec<String>> {
    let is_v2 = envelope.schema_version == ENVELOPE_SCHEMA_V2;
    match (is_v2, envelope.profile_text.as_deref()) {
        (true, Some(profile_text)) if consumer::is_workspace_profile_json(profile_text) => {
            consumer::parse_and_validate(profile_text)
                .map(|validated| (validated.profile_identity.clone(), Some(validated)))
        }
        (true, Some(profile_text)) => {
            if prefixed_identity(profile_text.as_bytes()) == EXPECTED_PROFILE_IDENTITY {
                Ok((EXPECTED_PROFILE_IDENTITY.to_owned(), None))
            } else {
                Err(vec!["Charon semantic profile bytes do not match the shipped profile identity".to_owned()])
            }
        }
        (true, None) => Err(vec!["Charon semantic v2 envelope is missing profile_text".to_owned()]),
        (false, _) => Ok((EXPECTED_PROFILE_IDENTITY.to_owned(), None)),
    }
}

pub(crate) fn evaluate_envelope(text: &str) -> CharonSemanticEvaluation {
    let envelope = match serde_json::from_str::<CharonSemanticEnvelope>(text) {
        Ok(envelope) => envelope,
        Err(error) => return failed_evaluation(format!("parse Charon semantic input envelope: {error}")),
    };
    if envelope.schema_version != ENVELOPE_SCHEMA && envelope.schema_version != ENVELOPE_SCHEMA_V2 {
        return failed_evaluation(format!("unsupported Charon semantic envelope schema `{}`", envelope.schema_version));
    }
    let (profile_identity, workspace) = match resolve_envelope_profile(&envelope) {
        Ok(resolved) => resolved,
        Err(diagnostics) => {
            return CharonSemanticEvaluation {
                profile_id: None,
                details: None,
                diagnostics,
                findings: Vec::new(),
            };
        }
    };
    let (profile, core_result) = if let Some(validated) = workspace.as_ref() {
        (validated.profile.clone(), core::analyze_workspace_snapshot(validated, &envelope.snapshot_text))
    } else {
        let profile = match core::parse_profile(PROFILE_JSON) {
            Ok(profile) => profile,
            Err(error) => return failed_evaluation(error),
        };
        let result = core::analyze_snapshot(&profile, &envelope.snapshot_text);
        (profile, result)
    };
    let core_output = match core_result {
        Ok(output) => output,
        Err(diagnostics) => {
            return CharonSemanticEvaluation {
                profile_id: Some(profile.profile_id),
                details: None,
                diagnostics,
                findings: Vec::new(),
            };
        }
    };
    let parents = match parse_comparison_parents(&envelope.comparison_parent_texts) {
        Ok(parents) => parents,
        Err(diagnostics) => {
            return CharonSemanticEvaluation {
                profile_id: Some(profile.profile_id),
                details: None,
                diagnostics,
                findings: core_output.findings,
            };
        }
    };
    let comparison = match comparison::compare(
        &core_output.projection,
        &core_output.analysis.effect_candidates,
        &core_output.source_identity,
        &parents,
    ) {
        Ok(comparison) => comparison,
        Err(diagnostics) => {
            return CharonSemanticEvaluation {
                profile_id: Some(profile.profile_id),
                details: None,
                diagnostics,
                findings: core_output.findings,
            };
        }
    };
    let evaluation_ctx = EvaluationArtifactsContext {
        envelope: &envelope,
        profile,
        profile_identity,
        is_consumer_workspace: workspace.is_some(),
        core_output,
        comparison,
    };
    finalize_evaluation(evaluation_ctx)
}

struct EvaluationArtifactsContext<'a> {
    envelope: &'a CharonSemanticEnvelope,
    profile: model::CharonSemanticProfile,
    profile_identity: String,
    is_consumer_workspace: bool,
    core_output: core::CoreOutput,
    comparison: Option<SemanticComparison>,
}

fn finalize_evaluation(ctx: EvaluationArtifactsContext<'_>) -> CharonSemanticEvaluation {
    let mut findings = ctx.core_output.findings;
    if let Some(comparison) = &ctx.comparison {
        findings.extend(comparison_findings(comparison));
    }
    findings.sort_by(|left, right| {
        (&left.family, &left.message, &left.file, left.line).cmp(&(
            &right.family,
            &right.message,
            &right.file,
            right.line,
        ))
    });
    let snapshot_identity = prefixed_identity(ctx.envelope.snapshot_text.as_bytes());
    let projection_identity = identity_for_json(&ctx.core_output.projection);
    let analysis_identity = identity_for_json(&ctx.core_output.analysis);
    let comparison_identity = ctx.comparison.as_ref().map(identity_for_json);
    let bound_profile_identity = ctx.is_consumer_workspace.then_some(ctx.profile_identity.as_str());
    let artifacts = artifact_bindings(
        &snapshot_identity,
        &projection_identity,
        &analysis_identity,
        comparison_identity.as_deref(),
        bound_profile_identity,
    );
    let command_argv = ctx.is_consumer_workspace.then(|| ctx.profile.extraction.command_argv.clone());
    CharonSemanticEvaluation {
        profile_id: Some(ctx.profile.profile_id.clone()),
        details: Some(CharonSemanticReceiptDetails {
            schema_version: DETAILS_SCHEMA.to_owned(),
            profile_id: ctx.profile.profile_id,
            profile_identity: ctx.profile_identity,
            provider_profile_identity: EXPECTED_PROVIDER_PROFILE_IDENTITY.to_owned(),
            compatibility_matrix_identity: EXPECTED_COMPATIBILITY_IDENTITY.to_owned(),
            charon_revision: ctx.profile.provider.charon_revision,
            artifacts,
            snapshot_identity,
            projection_identity,
            analysis_identity,
            comparison_identity,
            verification_role: RECORDED_ONLY_ROLE.to_owned(),
            artifact_roles: ctx.profile.artifact_roles,
            limits: ctx.profile.limits,
            references: ctx.profile.references,
            projection: ctx.core_output.projection,
            analysis: ctx.core_output.analysis,
            comparison: ctx.comparison,
            command_argv,
        }),
        diagnostics: Vec::new(),
        findings,
    }
}

fn comparison_findings(comparison: &SemanticComparison) -> impl Iterator<Item = SemanticFinding> + '_ {
    comparison.rows.iter().filter(|row| row.status != "agree").map(|row| SemanticFinding {
        family: "semantic-comparison".to_owned(),
        severity: "manual_review".to_owned(),
        message: format!(
            "{} {} {}: Charon={:?}, parent={:?}",
            row.parent_authority, row.status, row.fact, row.charon_value, row.parent_value
        ),
        file: None,
        line: None,
    })
}

pub(crate) fn validate_details(details: &CharonSemanticReceiptDetails, non_claims: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if details.schema_version != DETAILS_SCHEMA {
        diagnostics.push(format!("unsupported Charon semantic receipt details schema `{}`", details.schema_version));
    }
    if details.profile_id == PROFILE_ID {
        if details.profile_identity != EXPECTED_PROFILE_IDENTITY {
            diagnostics.push("Charon semantic receipt profile identity drifted".to_owned());
        }
    } else if details.profile_id.trim().is_empty()
        || !is_b3_identity(&details.profile_identity)
        || details.command_argv.as_ref().is_none_or(Vec::is_empty)
    {
        diagnostics.push("Charon semantic receipt profile identity drifted".to_owned());
    }
    if details.provider_profile_identity != EXPECTED_PROVIDER_PROFILE_IDENTITY
        || details.compatibility_matrix_identity != EXPECTED_COMPATIBILITY_IDENTITY
    {
        diagnostics.push("Charon semantic receipt provider identity drifted".to_owned());
    }
    if details.verification_role != RECORDED_ONLY_ROLE {
        diagnostics.push("Charon semantic receipt must remain recorded_only".to_owned());
    }
    if details.projection_identity != identity_for_json(&details.projection) {
        diagnostics.push("Charon semantic projection identity does not match receipt details".to_owned());
    }
    if details.analysis_identity != identity_for_json(&details.analysis) {
        diagnostics.push("Charon semantic analysis identity does not match receipt details".to_owned());
    }
    match (&details.comparison, &details.comparison_identity) {
        (Some(comparison), Some(identity)) if *identity == identity_for_json(comparison) => {}
        (None, None) => {}
        _ => diagnostics.push("Charon semantic comparison identity does not match receipt details".to_owned()),
    }
    if details.comparison.as_ref().is_some_and(|comparison| comparison.authority_merged) {
        diagnostics.push("Charon semantic comparison must not merge parent authority".to_owned());
    }
    let expected_bindings = artifact_identity_bindings(details);
    let observed_bindings = details
        .artifacts
        .iter()
        .map(|artifact| (artifact.name.clone(), artifact.identity.clone()))
        .collect::<BTreeMap<_, _>>();
    if observed_bindings != expected_bindings || details.artifacts.len() != expected_bindings.len() {
        diagnostics.push("Charon semantic artifact role bindings do not match receipt identities".to_owned());
    }
    if details.artifacts.iter().any(|artifact| artifact.authority.contains("proof")) {
        diagnostics.push("Charon semantic artifact role attempted proof-authority promotion".to_owned());
    }
    let required_non_claims = [
        "does not prove Charon or rustc correctness",
        "does not prove effect absence",
        "does not produce formal-proof",
    ];
    for required in required_non_claims {
        if !non_claims.iter().any(|non_claim| non_claim.contains(required)) {
            diagnostics.push(format!("Charon semantic receipt is missing non-claim `{required}`"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn artifact_payloads(
    details: &CharonSemanticReceiptDetails,
    envelope_text: &str,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let envelope = serde_json::from_str::<CharonSemanticEnvelope>(envelope_text)
        .map_err(|error| format!("parse Charon semantic envelope for artifacts: {error}"))?;
    if prefixed_identity(envelope.snapshot_text.as_bytes()) != details.snapshot_identity {
        return Err("Charon raw snapshot identity changed before artifact write".to_owned());
    }
    let mut artifacts = BTreeMap::from([
        (RAW_SNAPSHOT_ARTIFACT.to_owned(), envelope.snapshot_text.into_bytes()),
        (PROJECTION_ARTIFACT.to_owned(), render_json(&details.projection)?.into_bytes()),
        (ANALYSIS_ARTIFACT.to_owned(), render_json(&details.analysis)?.into_bytes()),
    ]);
    if details.command_argv.is_some() {
        let Some(profile_text) = envelope.profile_text else {
            return Err("Charon workspace envelope missing profile_text before artifact write".to_owned());
        };
        if prefixed_identity(profile_text.as_bytes()) != details.profile_identity {
            return Err("Charon profile identity changed before artifact write".to_owned());
        }
        artifacts.insert(PROFILE_ARTIFACT.to_owned(), profile_text.into_bytes());
    }
    if let Some(comparison) = &details.comparison {
        artifacts.insert(COMPARISON_ARTIFACT.to_owned(), render_json(comparison)?.into_bytes());
    }
    Ok(artifacts)
}

pub(crate) fn expected_artifact_names(details: Option<&CharonSemanticReceiptDetails>) -> Vec<String> {
    let Some(details) = details else {
        return Vec::new();
    };
    let mut names = vec![
        RAW_SNAPSHOT_ARTIFACT.to_owned(),
        PROJECTION_ARTIFACT.to_owned(),
        ANALYSIS_ARTIFACT.to_owned(),
    ];
    if details.command_argv.is_some() {
        names.push(PROFILE_ARTIFACT.to_owned());
    }
    if details.comparison.is_some() {
        names.push(COMPARISON_ARTIFACT.to_owned());
    }
    names
}

pub(crate) fn artifact_identity_bindings(details: &CharonSemanticReceiptDetails) -> BTreeMap<String, String> {
    let mut bindings = BTreeMap::from([
        (RAW_SNAPSHOT_ARTIFACT.to_owned(), details.snapshot_identity.clone()),
        (PROJECTION_ARTIFACT.to_owned(), details.projection_identity.clone()),
        (ANALYSIS_ARTIFACT.to_owned(), details.analysis_identity.clone()),
    ]);
    if details.command_argv.is_some() {
        bindings.insert(PROFILE_ARTIFACT.to_owned(), details.profile_identity.clone());
    }
    if let Some(identity) = &details.comparison_identity {
        bindings.insert(COMPARISON_ARTIFACT.to_owned(), identity.clone());
    }
    bindings
}

pub(crate) fn profile_command_argv() -> Result<Vec<String>, String> {
    let profile = core::parse_profile(PROFILE_JSON)?;
    let diagnostics = core::validate_profile(&profile);
    if diagnostics.is_empty() {
        Ok(profile.extraction.command_argv)
    } else {
        Err(format!("invalid embedded Charon semantic profile:\n- {}", diagnostics.join("\n- ")))
    }
}

pub(crate) fn shipped_profile_id() -> &'static str {
    PROFILE_ID
}

pub(crate) fn is_b3_identity(value: &str) -> bool {
    value
        .strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|hex| hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

pub(crate) fn prefixed_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn artifact_bindings(
    snapshot_identity: &str,
    projection_identity: &str,
    analysis_identity: &str,
    comparison_identity: Option<&str>,
    profile_identity: Option<&str>,
) -> Vec<ArtifactBinding> {
    let mut artifacts = vec![
        ArtifactBinding {
            role: "snapshot".to_owned(),
            name: RAW_SNAPSHOT_ARTIFACT.to_owned(),
            identity: snapshot_identity.to_owned(),
            authority: "recorded-pinned-provider-output".to_owned(),
        },
        ArtifactBinding {
            role: "projection".to_owned(),
            name: PROJECTION_ARTIFACT.to_owned(),
            identity: projection_identity.to_owned(),
            authority: "recorded-static-semantic-projection".to_owned(),
        },
        ArtifactBinding {
            role: "analysis".to_owned(),
            name: ANALYSIS_ARTIFACT.to_owned(),
            identity: analysis_identity.to_owned(),
            authority: "recorded-conservative-effects-and-unknowns".to_owned(),
        },
    ];
    if let Some(identity) = profile_identity {
        artifacts.push(ArtifactBinding {
            role: "profile".to_owned(),
            name: PROFILE_ARTIFACT.to_owned(),
            identity: identity.to_owned(),
            authority: "recorded-consumer-workspace-profile".to_owned(),
        });
    }
    if let Some(identity) = comparison_identity {
        artifacts.push(ArtifactBinding {
            role: "comparison".to_owned(),
            name: COMPARISON_ARTIFACT.to_owned(),
            identity: identity.to_owned(),
            authority: "recorded-independent-source-token-comparison".to_owned(),
        });
    }
    artifacts.sort_by(|left, right| left.role.cmp(&right.role));
    artifacts
}

fn identity_for_json(value: &impl serde::Serialize) -> String {
    let bytes = render_json(value).expect("serializing typed Charon semantic evidence must succeed");
    prefixed_identity(bytes.as_bytes())
}

fn render_json(value: &impl serde::Serialize) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(value).map_err(|error| format!("serialize Charon semantic artifact: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn parse_comparison_parents(texts: &[String]) -> Result<Vec<ComparisonParent>, Vec<String>> {
    let mut parents = Vec::new();
    let mut diagnostics = Vec::new();
    for (index, text) in texts.iter().enumerate() {
        match serde_json::from_str::<ComparisonParent>(text) {
            Ok(parent) => parents.push(parent),
            Err(error) => diagnostics.push(format!("parse Charon comparison parent {index}: {error}")),
        }
    }
    if diagnostics.is_empty() {
        Ok(parents)
    } else {
        Err(diagnostics)
    }
}

fn failed_evaluation(diagnostic: String) -> CharonSemanticEvaluation {
    CharonSemanticEvaluation {
        profile_id: None,
        details: None,
        diagnostics: vec![diagnostic],
        findings: Vec::new(),
    }
}

#[cfg(test)]
mod consumer_tests;

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT_JSON: &str =
        include_str!("../../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/llbc.llbc");
    const EXPECTED_BASE_ARTIFACT_COUNT: usize = 3;

    #[test]
    fn envelope_evaluation_binds_role_separated_artifacts() {
        let envelope = build_envelope(SNAPSHOT_JSON.to_owned(), Vec::new()).expect("envelope must build");
        let evaluation = evaluate_envelope(&envelope);
        assert!(evaluation.diagnostics.is_empty());
        let details = evaluation.details.expect("details must exist");
        let artifacts = artifact_payloads(&details, &envelope).expect("artifacts must render");
        assert_eq!(artifacts.len(), EXPECTED_BASE_ARTIFACT_COUNT);
        let bindings = artifact_identity_bindings(&details);
        assert_eq!(bindings.len(), artifacts.len());
        for (name, bytes) in artifacts {
            assert_eq!(bindings.get(&name), Some(&prefixed_identity(&bytes)));
        }
        assert_eq!(details.verification_role, RECORDED_ONLY_ROLE);
    }

    #[test]
    fn detail_validation_rejects_property_promotion_and_tampering() {
        let envelope = build_envelope(SNAPSHOT_JSON.to_owned(), Vec::new()).expect("envelope must build");
        let mut details = evaluate_envelope(&envelope).details.expect("details must exist");
        details.verification_role = "formal_proof".to_owned();
        details.projection.items[0].qualified_name = "tampered".to_owned();
        details.artifacts[0].authority = "formal-proof".to_owned();
        let diagnostics = validate_details(&details, &[]);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("projection identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("proof-authority")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
    }

    #[test]
    fn malformed_envelope_fails_without_details() {
        let evaluation = evaluate_envelope("{}");
        assert!(evaluation.details.is_none());
        assert!(!evaluation.diagnostics.is_empty());
    }
}
