const EXPECTED_COMMAND_PREFIX: [&str; 6] = ["cargo", "test", "--locked", "-p", "cargo-octet", "--lib"];
const EXPECTED_COMMAND_SUFFIX: [&str; 5] = ["--", "--exact", "--nocapture", "--test-threads", "1"];
const EXPECTED_COMMAND_LEN: usize = 12;
const EXPECTED_ADAPTER_PREFIX: &str = "hegel::tests::";
const EXPECTED_EVIDENCE_TIER: &str = "transcript_assertion";
const EXPECTED_VERIFICATION_ROLE: &str = "recorded_only";
const BUOY_DIMENSION_COUNT: usize = 16;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BuoyDimension {
    pub(crate) dimension_index: u64,
    pub(crate) dimension_id: String,
    pub(crate) buoy_reason: String,
    pub(crate) native_contract: String,
    pub(crate) production_validator: String,
    pub(crate) disposition: String,
    pub(crate) targeted_test_or_law: String,
    pub(crate) mutant_operator: String,
    pub(crate) expected_diagnostic_or_status: String,
    pub(crate) rationale: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BuoyMatrix {
    pub(crate) schema_version: String,
    pub(crate) source_repo: String,
    pub(crate) source_commit: String,
    pub(crate) source_symbol: String,
    pub(crate) source_span: String,
    pub(crate) dimensions: Vec<BuoyDimension>,
}

pub(crate) fn buoy_mutation_matrix() -> Result<BuoyMatrix, String> {
    let matrix: BuoyMatrix = serde_json::from_str(super::BUOY_MATRIX_JSON)
        .map_err(|error| format!("parse embedded Buoy mutation matrix: {error}"))?;
    if matrix.schema_version != super::BUOY_MATRIX_SCHEMA {
        return Err("embedded Buoy mutation matrix schema is unsupported".to_owned());
    }
    Ok(matrix)
}

pub(crate) fn validate_buoy_matrix() -> Vec<String> {
    let mut diagnostics = Vec::new();
    if super::prefixed_identity(super::BUOY_MATRIX_JSON.as_bytes()) != super::BUOY_MATRIX_IDENTITY {
        diagnostics.push("embedded Buoy mutation matrix identity mismatch".to_owned());
    }
    let matrix = match buoy_mutation_matrix() {
        Ok(value) => value,
        Err(error) => return vec![error],
    };
    let catalog_laws = super::law_catalog()
        .map(|catalog| catalog.laws.into_iter().map(|law| law.law_id).collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    if matrix.dimensions.len() != BUOY_DIMENSION_COUNT {
        diagnostics.push(format!(
            "Buoy mutation matrix expected {BUOY_DIMENSION_COUNT} dimensions, got {}",
            matrix.dimensions.len()
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let has_invalid_index_or_dup = matrix.dimensions.iter().enumerate().any(|(offset, row)| {
        let Ok(expected) = u64::try_from(offset.saturating_add(1)) else {
            return true;
        };
        row.dimension_index != expected || !seen.insert(row.dimension_id.as_str())
    });
    if has_invalid_index_or_dup {
        diagnostics.push("Buoy matrix has invalid dimension index or duplicate dimension_id".to_owned());
    }
    let has_incomplete_row = matrix.dimensions.iter().any(|row| {
        let is_valid_disposition = matches!(
            row.disposition.as_str(),
            "already_enforced_and_tested"
                | "enforced_missing_control"
                | "missing_in_selected_profile"
                | "not_applicable"
        );
        !is_valid_disposition
            || row.native_contract.trim().is_empty()
            || row.production_validator.trim().is_empty()
            || row.mutant_operator.trim().is_empty()
            || row.expected_diagnostic_or_status.trim().is_empty()
            || row.rationale.trim().is_empty()
    });
    if has_incomplete_row {
        diagnostics.push("Buoy matrix contains incomplete fields or unsupported disposition".to_owned());
    }
    let has_uncataloged_target = matrix.dimensions.iter().any(|row| {
        row.disposition != "not_applicable"
            && (row.targeted_test_or_law.trim().is_empty() || !catalog_laws.contains(&row.targeted_test_or_law))
    });
    if has_uncataloged_target {
        diagnostics.push("Buoy matrix targets an uncataloged law or empty test".to_owned());
    }
    diagnostics
}

pub(crate) fn is_valid_fixed_command(command_argv: &[String]) -> bool {
    if command_argv.len() != EXPECTED_COMMAND_LEN {
        return false;
    }
    let is_prefix_valid = command_argv[..6].iter().map(String::as_str).eq(EXPECTED_COMMAND_PREFIX);
    let is_adapter_valid = command_argv[6].starts_with(EXPECTED_ADAPTER_PREFIX);
    let is_suffix_valid = command_argv[7..].iter().map(String::as_str).eq(EXPECTED_COMMAND_SUFFIX);
    is_prefix_valid && is_adapter_valid && is_suffix_valid
}

pub(crate) fn validate(profile: &super::Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let registered = super::all_profiles()
        .ok()
        .and_then(|profiles| profiles.into_iter().find(|candidate| candidate.profile_id == profile.profile_id));
    let is_command_registered =
        registered.as_ref().is_some_and(|candidate| candidate.command_argv == profile.command_argv);
    if registered.is_none()
        || !REGISTERED_PROFILE_IDS.contains(&profile.profile_id.as_str())
        || !is_command_registered
        || !is_valid_fixed_command(&profile.command_argv)
        || profile.family.trim().is_empty()
        || profile.producer.package != "cargo-octet"
        || profile.producer.package_version != env!("CARGO_PKG_VERSION")
        || profile.producer.hegel_version != "0.14.27"
        || profile.producer.backend != "native"
        || profile.property_catalog != super::LAW_CATALOG_SCHEMA
    {
        diagnostics.push("Hegel producer profile identity or fixed command is unsupported".to_owned());
    }
    if profile.evidence_tier != EXPECTED_EVIDENCE_TIER
        || profile.verification_role != EXPECTED_VERIFICATION_ROLE
        || !profile.non_claims.iter().any(|claim| claim == super::REQUIRED_NON_CLAIM)
    {
        diagnostics.push("Hegel producer profile evidence boundary is unsupported".to_owned());
    }
    validate_paths_and_limits(profile, &mut diagnostics);
    diagnostics.extend(validate_catalog(profile));
    if profile.profile_id == "octet-freshness-mutation-laws-v1" {
        diagnostics.extend(validate_buoy_matrix());
    }
    diagnostics
}

fn validate_paths_and_limits(profile: &super::Profile, diagnostics: &mut Vec<String>) {
    let paths = profile.source_scope.iter().chain(profile.generator_inputs.iter()).chain([
        &profile.output_root,
        &profile.summary_path,
        &profile.counterexample_path,
    ]);
    for path in paths {
        if unsafe_relative_path(path) {
            diagnostics.push("Hegel producer profile contains an unsafe path".to_owned());
        }
    }
    if profile.source_scope.is_empty()
        || profile.source_scope.len() > profile.limits.max_source_files
        || profile.generator_inputs.is_empty()
        || profile.generator_inputs.len() > profile.limits.max_generator_files
        || profile.settings.case_budget == 0
        || profile.limits.max_summary_bytes == 0
        || profile.limits.max_counterexample_bytes == 0
        || profile.limits.max_log_bytes == 0
        || profile.limits.max_issue_codes == 0
        || profile.limits.timeout_ms == 0
        || profile.limits.poll_interval_ms == 0
        || profile.limits.teardown_timeout_ms == 0
    {
        diagnostics.push("Hegel producer profile contains an invalid limit".to_owned());
    }
}

pub(super) fn unsafe_relative_path(path: &str) -> bool {
    path.is_empty()
        || std::path::Path::new(path).is_absolute()
        || path.split('/').any(|component| component.is_empty() || component == "." || component == "..")
}

fn validate_catalog(profile: &super::Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let catalog = match super::law_catalog() {
        Ok(value) => value,
        Err(error) => return vec![error],
    };
    let registered_ids = super::all_profiles()
        .map(|profiles| profiles.into_iter().map(|item| item.profile_id).collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    if catalog.laws.is_empty() {
        diagnostics.push("Hegel law catalog is empty".to_owned());
    }
    if catalog
        .laws
        .iter()
        .any(|law| law.profile_id == profile.profile_id && law.case_budget != profile.settings.case_budget)
    {
        diagnostics.push("Hegel law catalog entry case_budget does not match profile".to_owned());
    }
    let mut ids = std::collections::BTreeSet::new();
    let has_bad_entry = catalog.laws.iter().any(|law| {
        law.law_id.trim().is_empty()
            || !registered_ids.contains(&law.profile_id)
            || law.family.trim().is_empty()
            || law.pure_core.trim().is_empty()
            || law.domain_boundary.trim().is_empty()
            || law.generator.trim().is_empty()
            || law.positive_property.trim().is_empty()
            || law.negative_property.trim().is_empty()
            || law.generated_input_benefit.trim().is_empty()
            || law.replay_method.trim().is_empty()
            || law.fixture_complement.trim().is_empty()
            || !law.effect_free
            || !ids.insert(law.law_id.as_str())
    });
    if has_bad_entry {
        diagnostics.push("Hegel law catalog entry is incomplete, duplicated, or effectful".to_owned());
    }
    if !catalog.laws.iter().any(|law| law.profile_id == profile.profile_id) {
        diagnostics.push("Hegel producer profile selects zero laws".to_owned());
    }
    diagnostics
}

pub(crate) const REGISTERED_PROFILE_IDS: [&str; 7] = [
    "octet-pure-core-laws-v1",
    "octet-evidence-identity-laws-v1",
    "octet-evidence-lineage-laws-v1",
    "octet-catalog-composition-laws-v1",
    "octet-reduction-acceptance-laws-v1",
    "octet-requirement-admission-laws-v1",
    "octet-freshness-mutation-laws-v1",
];
