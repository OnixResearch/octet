//! Admission integration for the opt-in semantic MIR effect profile (#17).

pub(crate) const SEMANTIC_EFFECT_RECEIPT_ARTIFACT: &str = "semantic-effect-receipt.json";

/// Evaluates an opt-in `--semantic-effect-profile` during `cargo octet admit` when
/// compiler semantic MIR unit summaries are present under the workspace collector shard directory.
pub(crate) fn evaluate_admitted_semantic_profile(
    workspace_root: &std::path::Path,
    profile_path: &std::path::Path,
    artifact_dir: &std::path::Path,
    function_addresses: &[crate::effect_closure::FunctionAddressRecord],
    cargo_lock_blake3: &str,
) -> Result<super::model::SemanticEffectReceipt, String> {
    let resolved_profile = if profile_path.is_absolute() {
        profile_path.to_path_buf()
    } else {
        workspace_root.join(profile_path)
    };
    let profile_bytes = std::fs::read(&resolved_profile)
        .map_err(|error| format!("read semantic effect profile {}: {error}", resolved_profile.display()))?;
    let profile_blake3 = format!("b3:{}", crate::evidence::artifact_blake3_hex(&profile_bytes));
    let profile: super::model::SemanticEffectProfile = serde_json::from_slice(&profile_bytes)
        .map_err(|error| format!("parse semantic effect profile {}: {error}", resolved_profile.display()))?;

    let shard_dir = workspace_root.join(crate::architecture_ir::ARCHITECTURE_SHARD_DIRECTORY);
    let unit_summary = load_latest_semantic_unit_summary(&shard_dir)?;
    let receipt = super::evaluate::evaluate(&super::evaluate::EvaluationInput {
        profile: &profile,
        profile_blake3: &profile_blake3,
        unit_summary: &unit_summary,
        function_addresses,
        trusted_summaries: &[],
        cargo_lock_blake3,
    })?;

    let mut receipt_bytes =
        serde_json::to_vec_pretty(&receipt).map_err(|error| format!("serialize semantic effect receipt: {error}"))?;
    receipt_bytes.push(b'\n');
    std::fs::write(artifact_dir.join(SEMANTIC_EFFECT_RECEIPT_ARTIFACT), &receipt_bytes)
        .map_err(|error| format!("write semantic effect receipt: {error}"))?;
    Ok(receipt)
}

fn load_latest_semantic_unit_summary(
    shard_dir: &std::path::Path,
) -> Result<octet_architecture_ir::semantic::UnitSummary, String> {
    let entries = std::fs::read_dir(shard_dir).map_err(|error| {
        format!("semantic effect profile requires compiler MIR shard directory {}: {error}", shard_dir.display())
    })?;
    let mut paths = Vec::with_capacity(16);
    for entry in entries {
        let entry = entry.map_err(|error| format!("read shard entry: {error}"))?;
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with(".semantic.json"))
            && paths.len() < 4096
        {
            paths.push(path);
        }
    }
    paths.sort();
    let Some(first) = paths.first() else {
        return Err(format!(
            "no compiler semantic MIR summary (`*.semantic.json`) found in {}; ensure architecture collection ran with semantic effects enabled",
            shard_dir.display()
        ));
    };
    let bytes =
        std::fs::read(first).map_err(|error| format!("read semantic MIR summary {}: {error}", first.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse semantic MIR summary {}: {error}", first.display()))
}
