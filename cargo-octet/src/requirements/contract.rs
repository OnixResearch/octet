use serde::Deserialize;
use serde::Serialize;

/// Requirement contract exported from Nickel.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Definition {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) reviewed_specification: ReviewedSpecification,
    pub(crate) source_root: Option<String>,
    pub(crate) clauses: Vec<RequirementClause>,
    #[serde(default = "Vec::new")]
    pub(crate) assumptions: Vec<AcceptedAssumption>,
    #[serde(default = "Vec::new")]
    pub(crate) exclusions: Vec<ExcludedBehavior>,
    #[serde(default = "Vec::new")]
    pub(crate) non_claims: Vec<String>,
}

/// Reviewed specification identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReviewedSpecification {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

/// One reviewed clause.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementClause {
    pub(crate) clause_id: String,
    pub(crate) statement: String,
    #[serde(default = "super::default_true")]
    pub(crate) required: bool,
    pub(crate) disposition: String,
    #[serde(default = "Vec::new")]
    pub(crate) checked_scope: Vec<String>,
    #[serde(default = "String::new")]
    pub(crate) expected_evidence_tier: String,
    #[serde(default = "Vec::new")]
    pub(crate) allowed_evidence_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) allowed_source_requirement_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) required_evidence_classes: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) required_negative_controls: Vec<String>,
    #[serde(default)]
    pub(crate) required_bindings: RequirementBindings,
    #[serde(default = "Vec::new")]
    pub(crate) source_bindings: Vec<SourceBinding>,
    pub(crate) assumption_id: Option<String>,
    pub(crate) exclusion_reason: Option<String>,
    pub(crate) required_provider_id: Option<String>,
}

/// Explicit bindings a freshness record must satisfy.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementBindings {
    #[serde(default = "Vec::new")]
    pub(crate) source_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) policy_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) tool_identity_hashes: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) artifact_hashes: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) checked_scope: Vec<String>,
    pub(crate) evidence_tier: Option<String>,
}

/// One expected source identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SourceBinding {
    pub(crate) locator: String,
    pub(crate) blake3: String,
}

/// One accepted assumption.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AcceptedAssumption {
    pub(crate) assumption_id: String,
    pub(crate) statement: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    #[serde(default = "Vec::new")]
    pub(crate) affected_clause_ids: Vec<String>,
    #[serde(default = "Vec::new")]
    pub(crate) required_empirical_evidence: Vec<String>,
}

/// One explicit exclusion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExcludedBehavior {
    pub(crate) exclusion_id: String,
    pub(crate) clause_id: String,
    pub(crate) reason: String,
    #[serde(default = "String::new")]
    pub(crate) reviewed_specification_blake3: String,
}

pub(super) mod validation;
