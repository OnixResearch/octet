pub(super) mod admission;
pub(super) mod buoy_matrix;
pub(super) mod catalog;
pub(super) mod freshness;
pub(super) mod identity;
pub(super) mod lineage;
pub(super) mod reduction;

use super::ProfileLawSpec;

pub(super) fn specs_for_profile(profile_id: &str) -> Vec<ProfileLawSpec> {
    match profile_id {
        "octet-pure-core-laws-v1" => vec![(
            "octet.hegel.issue-code-normalization-idempotent",
            "pure_core",
            super::issue_code_normalization_is_idempotent,
        )],
        "octet-evidence-identity-laws-v1" => vec![
            (
                "octet.hegel.identity.canonical-hash-permutation-invariance",
                "identity",
                identity::canonical_hash_permutation_invariance,
            ),
            (
                "octet.hegel.identity.framed-provider-role-separation",
                "identity",
                identity::framed_provider_role_separation,
            ),
        ],
        "octet-evidence-lineage-laws-v1" => vec![
            (
                "octet.hegel.lineage.multihop-bfs-datalog-equivalence",
                "lineage",
                lineage::multihop_bfs_datalog_equivalence,
            ),
            (
                "octet.hegel.lineage.pointer-and-session-conservation",
                "lineage",
                lineage::pointer_and_session_conservation,
            ),
        ],
        "octet-catalog-composition-laws-v1" => vec![
            (
                "octet.hegel.catalog.effect-closure-and-profile-composition",
                "catalog_composition",
                catalog::effect_closure_and_profile_composition,
            ),
            (
                "octet.hegel.catalog.chaoscontrol-proof-style-boundary",
                "catalog_composition",
                catalog::chaoscontrol_proof_style_boundary,
            ),
        ],
        "octet-reduction-acceptance-laws-v1" => vec![
            (
                "octet.hegel.reduction.execution-capsule-and-cache-determinism",
                "reduction_acceptance",
                reduction::execution_capsule_and_cache_determinism,
            ),
            (
                "octet.hegel.reduction.negative-control-attribution-and-replay",
                "reduction_acceptance",
                reduction::negative_control_attribution_and_replay,
            ),
        ],
        "octet-requirement-admission-laws-v1" => vec![
            (
                "octet.hegel.admission.contract-evaluation-permutation-and-tier-separation",
                "requirement_admission",
                admission::contract_evaluation_permutation_and_tier_separation,
            ),
            (
                "octet.hegel.admission.effect-closure-summary-and-receipt-integrity",
                "requirement_admission",
                admission::effect_closure_summary_and_receipt_integrity,
            ),
        ],
        "octet-freshness-mutation-laws-v1" => vec![
            (
                "octet.hegel.freshness.buoy-single-role-mutation-and-role-confusion",
                "freshness",
                freshness::buoy_single_role_mutation_and_role_confusion,
            ),
            (
                "octet.hegel.freshness.envelope-relocation-and-transitive-drift",
                "freshness",
                freshness::envelope_relocation_and_transitive_drift,
            ),
        ],
        _ => Vec::new(),
    }
}
