//! Embedded strict-profile catalog admission and diagnostic invariants.

pub(crate) const STRICT_PROFILE_CATALOG_JSON: &str =
    include_str!("../../../../standards/strict-profiles/generated/strict-rust-profiles.json");
const SUPPORTED_EFFECT_FAMILIES: &[&str] = &[
    crate::EFFECT_CONCURRENCY,
    crate::EFFECT_FAILURE,
    crate::EFFECT_RESOURCE,
    crate::EFFECT_TIME,
    crate::EFFECT_UNKNOWN,
];

#[derive(Default)]
struct ValidationDiagnostics {
    messages: Vec<String>,
}

impl ValidationDiagnostics {
    fn push(&mut self, message: String) {
        self.messages.push(message);
    }
}

fn validate_boolean_api_policy(
    policy: &crate::strict_profile::BooleanApiPolicy,
    diagnostics: &mut ValidationDiagnostics,
) {
    let required_scopes = ["free-function", "inherent-method", "crate-owned-trait"];
    if !policy.public_scope.iter().map(String::as_str).eq(required_scopes) {
        diagnostics.push("boolean API policy public_scope must name the closed callable set".to_owned());
    }
    if policy.private_min_direct_bool_parameters != 2 || policy.parameter_type != "direct-bool" {
        diagnostics.push("boolean API policy must cover direct bool and two private parameters".to_owned());
    }
    if !policy.compatibility_exclusions.skip_expansions || !policy.compatibility_exclusions.skip_external_trait_impls {
        diagnostics.push("boolean API policy must preserve generated and external trait signatures".to_owned());
    }
    if policy.exception_attribute != "local-expect"
        || !policy.expectation_requirements.exception_reason_required
        || !policy.expectation_requirements.exception_owner_required
        || policy.broad_suppression != "deny"
        || policy.finding_baseline != "deny"
        || policy.warning_budget != "deny"
        || policy.unknown_scope != "deny"
    {
        diagnostics.push(
            "boolean API policy must reject broad suppressions, baselines, budgets, and unknown scopes".to_owned(),
        );
    }
    if policy.non_claims.is_empty() || policy.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push("boolean API policy requires explicit non-claims".to_owned());
    }
}

/// Loads and validates the embedded strict-profile catalog.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the embedded JSON is malformed or violates the catalog
/// contract.
pub fn load_strict_profile_catalog() -> Result<crate::strict_profile::StrictProfileCatalog, String> {
    let catalog = serde_json::from_str::<crate::strict_profile::StrictProfileCatalog>(STRICT_PROFILE_CATALOG_JSON)
        .map_err(|error| format!("parse embedded strict-profile catalog: {error}"))?;
    let diagnostics = strict_profile_catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        Ok(catalog)
    } else {
        Err(format!("invalid embedded strict-profile catalog: {}", diagnostics.join("; ")))
    }
}

/// Returns one validated strict profile by identity.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the catalog is invalid or `profile_id` is unknown.
pub fn strict_profile(profile_id: &str) -> Result<crate::strict_profile::StrictProfile, String> {
    let catalog = load_strict_profile_catalog()?;
    if let Some(profile) = catalog.profiles.iter().find(|profile| profile.profile_id == profile_id) {
        return Ok(profile.clone());
    }

    let supported = sorted_ids(&catalog);
    Err(format!("unknown strict profile `{profile_id}`; supported profiles: {}", supported.join(", ")))
}

/// Returns the sorted identities from the validated strict-profile catalog.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the embedded catalog is malformed or invalid.
pub fn supported_profile_ids() -> Result<Vec<String>, String> {
    load_strict_profile_catalog().map(|catalog| sorted_ids(&catalog))
}

/// Reports deterministic contract violations for an in-memory strict-profile catalog.
#[must_use]
pub fn strict_profile_catalog_diagnostics(catalog: &crate::strict_profile::StrictProfileCatalog) -> Vec<String> {
    let mut diagnostics = ValidationDiagnostics::default();
    if catalog.schema_version != crate::strict_profile::STRICT_PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{}`, got `{}`",
            crate::strict_profile::STRICT_PROFILE_SCHEMA_VERSION,
            catalog.schema_version
        ));
    }
    if catalog.profiles.is_empty() {
        diagnostics.push("profiles must not be empty".to_owned());
    }
    validate_boolean_api_policy(&catalog.boolean_api_policy, &mut diagnostics);

    let mut profile_ids = std::collections::BTreeSet::new();
    for profile in &catalog.profiles {
        validate_profile(profile, &mut profile_ids, &mut diagnostics);
    }

    diagnostics.messages.sort();
    diagnostics.messages.dedup();
    diagnostics.messages
}

fn validate_profile(
    profile: &crate::strict_profile::StrictProfile,
    profile_ids: &mut std::collections::BTreeSet<String>,
    diagnostics: &mut ValidationDiagnostics,
) {
    let profile_id = profile.profile_id.as_str();
    if profile_id.trim().is_empty() {
        diagnostics.push("profile_id must not be empty".to_owned());
    } else if !profile_ids.insert(profile.profile_id.clone()) {
        diagnostics.push(format!("duplicate profile_id `{profile_id}`"));
    }
    if profile.summary.trim().is_empty() {
        diagnostics.push(format!("profile `{profile_id}` summary must not be empty"));
    }
    validate_required_lints(profile, diagnostics);
    validate_effect_families(profile, diagnostics);
    validate_scope_contract(profile, diagnostics);
    validate_non_claims(profile, diagnostics);
}

fn validate_required_lints(profile: &crate::strict_profile::StrictProfile, diagnostics: &mut ValidationDiagnostics) {
    let profile_id = profile.profile_id.as_str();
    if profile.required_lints.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` required_lints must not be empty"));
        return;
    }

    let mut lint_names = std::collections::BTreeSet::new();
    for lint_name in &profile.required_lints {
        if !lint_names.insert(lint_name.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` repeats required lint `{lint_name}`"));
        }
        if crate::metadata_for_lint(lint_name).is_none() {
            diagnostics.push(format!("profile `{profile_id}` names unknown required lint `{lint_name}`"));
        }
    }
}

fn validate_effect_families(profile: &crate::strict_profile::StrictProfile, diagnostics: &mut ValidationDiagnostics) {
    let profile_id = profile.profile_id.as_str();
    if profile.effect_families.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` effect_families must not be empty"));
        return;
    }

    let mut effect_families = std::collections::BTreeSet::new();
    for effect_family in &profile.effect_families {
        if !effect_families.insert(effect_family.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` repeats effect family `{effect_family}`"));
        }
        if !SUPPORTED_EFFECT_FAMILIES.contains(&effect_family.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` names unknown effect family `{effect_family}`"));
        }
    }
}

fn validate_scope_contract(profile: &crate::strict_profile::StrictProfile, diagnostics: &mut ValidationDiagnostics) {
    let profile_id = profile.profile_id.as_str();
    if profile.claim_scope != crate::strict_profile::STRICT_PROFILE_CLAIM_SCOPE {
        diagnostics.push(format!(
            "profile `{profile_id}` claim_scope must be `{}`",
            crate::strict_profile::STRICT_PROFILE_CLAIM_SCOPE
        ));
    }
    if profile.requires_core_scopes
        && profile.function_address_scope != crate::strict_profile::FunctionAddressScope::Core
    {
        diagnostics
            .push(format!("profile `{profile_id}` requires core scopes but function_address_scope is not `core`"));
    }
    if !profile.requires_core_scopes
        && profile.function_address_scope == crate::strict_profile::FunctionAddressScope::Core
    {
        diagnostics
            .push(format!("profile `{profile_id}` selects core function-address scope without requiring core scopes"));
    }
    if profile.minimum_function_address_mode == crate::strict_profile::FunctionAddressModeFloor::Off {
        diagnostics.push(format!("profile `{profile_id}` function-address floor must not be `off`"));
    }
}

fn validate_non_claims(profile: &crate::strict_profile::StrictProfile, diagnostics: &mut ValidationDiagnostics) {
    let profile_id = profile.profile_id.as_str();
    if profile.non_claims.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` non_claims must not be empty"));
        return;
    }
    if profile.non_claims.iter().any(|non_claim| non_claim.trim().is_empty()) {
        diagnostics.push(format!("profile `{profile_id}` non_claims must not contain empty text"));
    }
}

pub(crate) fn sorted_ids(catalog: &crate::strict_profile::StrictProfileCatalog) -> Vec<String> {
    let mut ids = catalog.profiles.iter().map(|profile| profile.profile_id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids
}
