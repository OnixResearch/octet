//! Wire-compatible strict profile and boolean API policy types.

#[derive(
    Clone,
    Copy,
    Debug,
    serde::Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize
)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionAddressModeFloor {
    Off,
    Observe,
    Warn,
    Deny,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionAddressScope {
    Workspace,
    Core,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct StrictProfile {
    pub profile_id: String,
    pub summary: String,
    pub required_lints: Vec<String>,
    pub effect_families: Vec<String>,
    pub requires_core_scopes: bool,
    pub minimum_function_address_mode: FunctionAddressModeFloor,
    pub function_address_scope: FunctionAddressScope,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct StrictProfileCatalog {
    pub schema_version: String,
    pub boolean_api_policy: BooleanApiPolicy,
    pub profiles: Vec<StrictProfile>,
}

/// Closed policy for the direct boolean-parameter lint, exported from Nickel.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct BooleanApiPolicy {
    pub public_scope: Vec<String>,
    pub private_min_direct_bool_parameters: usize,
    pub parameter_type: String,
    #[serde(flatten)]
    pub compatibility_exclusions: BooleanCompatibilityExclusions,
    pub exception_attribute: String,
    #[serde(flatten)]
    pub expectation_requirements: BooleanExpectationRequirements,
    pub broad_suppression: String,
    pub finding_baseline: String,
    pub warning_budget: String,
    pub unknown_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct BooleanCompatibilityExclusions {
    pub skip_expansions: bool,
    pub skip_external_trait_impls: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct BooleanExpectationRequirements {
    pub exception_reason_required: bool,
    pub exception_owner_required: bool,
}
