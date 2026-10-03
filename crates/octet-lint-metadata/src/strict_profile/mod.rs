pub const STRICT_PROFILE_ENV: &str = "OCTET_STRICT_PROFILE";
pub const STRICT_PROFILE_SCHEMA_VERSION: &str = "octet-strict-rust-profiles/v1";
pub const STRICT_PROFILE_CLAIM_SCOPE: &str = "direct-source-shape-policy";

pub use crate::rules::load_strict_profile_catalog;
pub use crate::rules::strict_profile;
pub use crate::rules::strict_profile_catalog_diagnostics;
pub use crate::rules::supported_profile_ids;
pub use crate::schema::BooleanApiPolicy;
pub use crate::schema::BooleanCompatibilityExclusions;
pub use crate::schema::BooleanExpectationRequirements;
pub use crate::schema::FunctionAddressModeFloor;
pub use crate::schema::FunctionAddressScope;
pub use crate::schema::StrictProfile;
pub use crate::schema::StrictProfileCatalog;

/// Returns the checked raw-boolean policy and a BLAKE3 identity of its canonical JSON.
///
/// # Errors
///
/// Returns a deterministic diagnostic if the embedded policy is invalid or its identity cannot be
/// serialized.
pub fn boolean_api_policy() -> Result<(BooleanApiPolicy, String), String> {
    let policy = load_strict_profile_catalog()?.boolean_api_policy;
    let identity = boolean_api_policy_identity(&policy)?;
    Ok((policy, identity))
}

fn boolean_api_policy_identity(policy: &BooleanApiPolicy) -> Result<String, String> {
    let canonical =
        serde_json::to_vec(&policy).map_err(|error| format!("serialize boolean API policy identity: {error}"))?;
    Ok(format!("b3:{}", blake3::hash(&canonical).to_hex()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PROFILE_IDS: &[&str] = &["bounded-systems", "deterministic-core", "robust"];

    #[test]
    fn embedded_catalog_accepts_every_shipped_profile() {
        let catalog = load_strict_profile_catalog().expect("embedded strict-profile catalog should validate");
        let ids = crate::rules::sorted_ids(&catalog);

        assert_eq!(ids, EXPECTED_PROFILE_IDS);
        for profile_id in EXPECTED_PROFILE_IDS {
            let profile = strict_profile(profile_id).expect("shipped profile should resolve");
            assert_eq!(profile.profile_id, *profile_id);
            assert!(!profile.required_lints.is_empty());
            assert!(!profile.non_claims.is_empty());
        }
    }

    #[test]
    fn unknown_profile_lists_supported_ids_in_deterministic_order() {
        let error = strict_profile("not-a-profile").expect_err("unknown profile should fail");

        assert!(error.contains("unknown strict profile `not-a-profile`"));
        assert!(error.contains("bounded-systems, deterministic-core, robust"));
    }

    #[test]
    fn malformed_catalog_reports_duplicate_unknown_and_scope_errors() {
        let mut catalog = load_strict_profile_catalog().expect("fixture catalog");
        let duplicate_id = catalog.profiles[0].profile_id.clone();
        catalog.profiles[1].profile_id = duplicate_id;
        catalog.profiles[1].required_lints.push("not_a_lint".to_owned());
        catalog.profiles[1].requires_core_scopes = true;
        catalog.profiles[1].function_address_scope = FunctionAddressScope::Workspace;
        catalog.profiles[1].non_claims.clear();

        let diagnostics = strict_profile_catalog_diagnostics(&catalog);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate profile_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown required lint `not_a_lint`")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("requires core scopes")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non_claims must not be empty")));
    }

    #[test]
    fn boolean_api_policy_rejects_suppression_and_rekeys_on_change() {
        let mut catalog = load_strict_profile_catalog().expect("checked policy");
        let initial = boolean_api_policy_identity(&catalog.boolean_api_policy).expect("policy hash");
        let (_, repeated) = boolean_api_policy().expect("repeat policy hash");
        assert_eq!(initial, repeated);
        catalog.boolean_api_policy.warning_budget = "allow".to_owned();
        catalog.boolean_api_policy.broad_suppression = "allow".to_owned();
        catalog.boolean_api_policy.public_scope.push("unknown-scope".to_owned());

        let diagnostics = strict_profile_catalog_diagnostics(&catalog);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("broad suppressions, baselines, budgets")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("closed callable set")));
        assert_ne!(initial, boolean_api_policy_identity(&catalog.boolean_api_policy).expect("changed policy hash"));
    }

    #[test]
    fn boolean_api_policy_retains_flat_generated_fields() {
        let source: serde_json::Value =
            serde_json::from_str(crate::rules::STRICT_PROFILE_CATALOG_JSON).expect("generated strict-profile catalog");
        let policy = load_strict_profile_catalog().expect("checked policy").boolean_api_policy;
        let serialized = serde_json::to_value(policy).expect("serialize checked policy");
        assert_eq!(serialized, source["boolean_api_policy"]);
    }
}
