//! Generated policy artifacts consumed by standards tooling.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::model::EnforcementLevel;
use crate::model::EnforcementMode;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::SeedManifest;

/// Canonical registry artifact file name.
pub const POLICIES_JSON_FILE: &str = "tigerstyle-policies.json";
/// Checker configuration artifact file name.
pub const CHECK_CONFIG_TOML_FILE: &str = "tigerstyle-check-config.toml";
/// Seed manifest artifact file name.
pub const SEED_MANIFESTS_JSON_FILE: &str = "tigerstyle-seed-manifests.json";

const GENERATED_ARTIFACT_SCHEMA_VERSION: &str = "1";
const GENERATED_FROM: &str = "standards/policies/registry.ncl";

/// Deterministic seed-manifest export artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SeedManifestArtifact {
    /// Artifact schema version.
    pub schema_version: String,
    /// Source policy registry path.
    pub generated_from: String,
    /// Registry schema version this artifact was generated from.
    pub registry_schema_version: String,
    /// Seed manifests copied from the policy registry.
    pub seed_manifests: Vec<SeedManifest>,
}

/// Build the seed-manifest artifact from a registry.
#[must_use]
pub fn seed_manifest_artifact(registry: &PolicyRegistry) -> SeedManifestArtifact {
    SeedManifestArtifact {
        schema_version: GENERATED_ARTIFACT_SCHEMA_VERSION.to_owned(),
        generated_from: GENERATED_FROM.to_owned(),
        registry_schema_version: registry.schema_version.clone(),
        seed_manifests: registry.seed_manifests.clone(),
    }
}

/// Render deterministic checker configuration TOML from a registry.
#[must_use]
pub fn render_check_config_toml(registry: &PolicyRegistry) -> String {
    let hard_policy_ids = policy_ids_by_level(registry, EnforcementLevel::Hard);
    let structural_policy_ids = policy_ids_by_target_mode(registry, EnforcementMode::Structural);
    let verified_policy_ids = policy_ids_by_target_mode(registry, EnforcementMode::Verified);
    let families = policy_families(registry);
    let bounded_queue = bounded_queue_policy(registry);
    let bounded_queue_forbidden_apis =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.forbidden_apis);
    let bounded_queue_approved_apis =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.approved_bounded_apis);
    let bounded_queue_wrapper_scopes =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.approved_wrapper_scopes);
    let bounded_queue_capacity_witnesses =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.capacity_witnesses);
    let bounded_queue_backpressure_behaviors =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.backpressure_behaviors);
    let capability_policy = effect_capability_policy(registry);
    let capability_forbidden_ambient_apis =
        capability_mechanism_field(capability_policy, |mechanism| &mechanism.forbidden_ambient_apis);
    let capability_core_source_scopes =
        capability_mechanism_field(capability_policy, |mechanism| &mechanism.core_source_scopes);
    let capability_shell_constructor_scopes =
        capability_mechanism_field(capability_policy, |mechanism| &mechanism.shell_constructor_scopes);
    let capability_approved_names =
        capability_mechanism_field(capability_policy, |mechanism| &mechanism.approved_capability_names);
    let type_state_policy = nominal_type_boundary_policy(registry);
    let type_state_source_scopes =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.type_state_source_scopes);
    let type_state_raw_alias_patterns =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.raw_state_alias_patterns);
    let type_state_raw_transition_patterns =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.raw_transition_patterns);
    let type_state_required_state_markers =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.required_state_markers);
    let type_state_required_transition_methods =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.required_transition_methods);
    let proof_rail_backends = proof_rail_field(registry, |rail| &rail.backend);
    let proof_rail_commands = proof_rail_field(registry, |rail| &rail.command);
    let proof_rail_source_paths = proof_rail_fields(registry, |rail| &rail.source_paths);
    let proof_rail_proof_paths = proof_rail_fields(registry, |rail| &rail.proof_paths);
    let proof_rail_pass_signals = proof_rail_field(registry, |rail| &rail.pass_signal);
    let proof_rail_trust_boundaries = proof_rail_fields(registry, |rail| &rail.trust_boundaries);
    let proof_rail_receipt_bindings = proof_rail_field(registry, |rail| &rail.receipt_binding);
    let proof_rail_receipt_schemas = proof_rail_field(registry, |rail| &rail.receipt_provenance.schema_version);
    let proof_rail_receipt_runners = proof_rail_field(registry, |rail| &rail.receipt_provenance.runner);
    let proof_rail_receipt_environments = proof_rail_field(registry, |rail| &rail.receipt_provenance.environment);
    let proof_rail_receipt_requires = proof_rail_receipt_requirements(registry);
    let proof_rail_receipt_verifiers = proof_rail_field(registry, |rail| &rail.receipt_verifier.verifier_id);
    let proof_rail_receipt_verifier_digests = proof_rail_field(registry, |rail| &rail.receipt_verifier.verifier_digest);
    let proof_rail_receipt_toolchains = proof_rail_field(registry, |rail| &rail.receipt_verifier.toolchain_id);
    let proof_rail_receipt_toolchain_digests =
        proof_rail_field(registry, |rail| &rail.receipt_verifier.toolchain_digest);

    let mut toml = String::new();
    toml.push_str("schema_version = \"");
    toml.push_str(GENERATED_ARTIFACT_SCHEMA_VERSION);
    toml.push_str("\"\n");
    toml.push_str("generated_from = \"");
    toml.push_str(GENERATED_FROM);
    toml.push_str("\"\n");
    toml.push_str("registry_schema_version = \"");
    toml.push_str(&escape_toml_string(&registry.schema_version));
    toml.push_str("\"\n");
    append_toml_array(&mut toml, "checker_ids", &registry.checker_ids);
    append_toml_array(&mut toml, "families", &families);
    append_toml_array(&mut toml, "hard_policy_ids", &hard_policy_ids);
    append_toml_array(&mut toml, "structural_policy_ids", &structural_policy_ids);
    append_toml_array(&mut toml, "verified_policy_ids", &verified_policy_ids);
    append_toml_array(&mut toml, "bounded_queue_forbidden_apis", &bounded_queue_forbidden_apis);
    append_toml_array(&mut toml, "bounded_queue_approved_apis", &bounded_queue_approved_apis);
    append_toml_array(&mut toml, "bounded_queue_wrapper_scopes", &bounded_queue_wrapper_scopes);
    append_toml_array(&mut toml, "bounded_queue_capacity_witnesses", &bounded_queue_capacity_witnesses);
    append_toml_array(&mut toml, "bounded_queue_backpressure_behaviors", &bounded_queue_backpressure_behaviors);
    append_toml_array(&mut toml, "capability_forbidden_ambient_apis", &capability_forbidden_ambient_apis);
    append_toml_array(&mut toml, "capability_core_source_scopes", &capability_core_source_scopes);
    append_toml_array(&mut toml, "capability_shell_constructor_scopes", &capability_shell_constructor_scopes);
    append_toml_array(&mut toml, "capability_approved_names", &capability_approved_names);
    append_toml_array(&mut toml, "type_state_source_scopes", &type_state_source_scopes);
    append_toml_array(&mut toml, "type_state_raw_alias_patterns", &type_state_raw_alias_patterns);
    append_toml_array(&mut toml, "type_state_raw_transition_patterns", &type_state_raw_transition_patterns);
    append_toml_array(&mut toml, "type_state_required_state_markers", &type_state_required_state_markers);
    append_toml_array(&mut toml, "type_state_required_transition_methods", &type_state_required_transition_methods);
    append_toml_array(&mut toml, "proof_rail_backends", &proof_rail_backends);
    append_toml_array(&mut toml, "proof_rail_commands", &proof_rail_commands);
    append_toml_array(&mut toml, "proof_rail_source_paths", &proof_rail_source_paths);
    append_toml_array(&mut toml, "proof_rail_proof_paths", &proof_rail_proof_paths);
    append_toml_array(&mut toml, "proof_rail_pass_signals", &proof_rail_pass_signals);
    append_toml_array(&mut toml, "proof_rail_trust_boundaries", &proof_rail_trust_boundaries);
    append_toml_array(&mut toml, "proof_rail_receipt_bindings", &proof_rail_receipt_bindings);
    append_toml_array(&mut toml, "proof_rail_receipt_schemas", &proof_rail_receipt_schemas);
    append_toml_array(&mut toml, "proof_rail_receipt_runners", &proof_rail_receipt_runners);
    append_toml_array(&mut toml, "proof_rail_receipt_environments", &proof_rail_receipt_environments);
    append_toml_array(&mut toml, "proof_rail_receipt_requires", &proof_rail_receipt_requires);
    append_toml_array(&mut toml, "proof_rail_receipt_verifiers", &proof_rail_receipt_verifiers);
    append_toml_array(&mut toml, "proof_rail_receipt_verifier_digests", &proof_rail_receipt_verifier_digests);
    append_toml_array(&mut toml, "proof_rail_receipt_toolchains", &proof_rail_receipt_toolchains);
    append_toml_array(&mut toml, "proof_rail_receipt_toolchain_digests", &proof_rail_receipt_toolchain_digests);
    toml
}

fn proof_rail_field(registry: &PolicyRegistry, field: impl Fn(&crate::model::ProofRail) -> &String) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .map(|rail| field(rail).clone())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn proof_rail_receipt_requirements(registry: &PolicyRegistry) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .flat_map(|rail| {
            [
                rail.receipt_provenance.requires_backend.then_some("backend".to_owned()),
                rail.receipt_provenance.requires_trust_boundary.then_some("trust-boundary".to_owned()),
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn proof_rail_fields(
    registry: &PolicyRegistry,
    field: impl Fn(&crate::model::ProofRail) -> &Vec<String>,
) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .flat_map(|rail| field(rail).clone())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn bounded_queue_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.embedded.bounded-queues")
}

fn effect_capability_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.architecture.effect-capability-boundaries")
}

fn nominal_type_boundary_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.idioms.nominal-type-boundaries")
}

fn type_state_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == crate::model::MechanismKind::TypeState)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn capability_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy
                .mechanisms
                .iter()
                .find(|mechanism| mechanism.kind == crate::model::MechanismKind::CapabilityApi)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn bounded_queue_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy
                .mechanisms
                .iter()
                .find(|mechanism| mechanism.kind == crate::model::MechanismKind::BoundedPrimitive)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn policy_ids_by_level(registry: &PolicyRegistry, level: EnforcementLevel) -> Vec<String> {
    sorted_policy_ids(registry.policies.iter().filter(|policy| policy.enforcement_level == level))
}

fn policy_ids_by_target_mode(registry: &PolicyRegistry, mode: EnforcementMode) -> Vec<String> {
    sorted_policy_ids(registry.policies.iter().filter(|policy| policy.target_enforcement_mode == mode))
}

fn sorted_policy_ids<'a>(policies: impl Iterator<Item = &'a PolicyEntry>) -> Vec<String> {
    let mut ids = policies.map(|policy| policy.policy_id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids
}

fn policy_families(registry: &PolicyRegistry) -> Vec<String> {
    registry
        .policies
        .iter()
        .map(|policy| policy.pillar.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn append_toml_array(toml: &mut String, name: &str, values: &[String]) {
    toml.push_str(name);
    toml.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            toml.push_str(", ");
        }
        toml.push('"');
        toml.push_str(&escape_toml_string(value));
        toml.push('"');
    }
    toml.push_str("]\n");
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use crate::artifacts::render_check_config_toml;
    use crate::artifacts::seed_manifest_artifact;
    use crate::model::PolicyRegistry;

    #[test]
    fn seed_manifest_artifact_preserves_registry_schema() {
        let registry = PolicyRegistry {
            schema_version: "test-schema".to_owned(),
            checker_ids: Vec::new(),
            policies: Vec::new(),
            seed_manifests: Vec::new(),
        };

        let artifact = seed_manifest_artifact(&registry);

        assert_eq!(artifact.schema_version, "1");
        assert_eq!(artifact.registry_schema_version, "test-schema");
        assert!(artifact.seed_manifests.is_empty());
    }

    #[test]
    fn check_config_output_is_stable() {
        let registry = PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["checker-b".to_owned(), "checker-a".to_owned()],
            policies: Vec::new(),
            seed_manifests: Vec::new(),
        };

        let output = render_check_config_toml(&registry);

        assert!(output.starts_with("schema_version = \"1\"\n"));
        assert!(output.contains("checker_ids = [\"checker-b\", \"checker-a\"]\n"));
        assert!(output.contains("hard_policy_ids = []\n"));
    }
}
