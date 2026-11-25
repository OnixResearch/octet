use hegel::generators as gs;

use super::super::CaseTally;

const MAX_CODE_LEN: usize = 20;
const MAX_ITEMS: usize = 8;

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn effect_closure_and_profile_composition(tc: hegel::TestCase, db_path: Option<String>) {
    let raw_codes = tc.draw(gs::vecs(gs::text().max_size(MAX_CODE_LEN)).max_size(MAX_ITEMS));
    let duplicate_role_edge = tc.draw(gs::booleans());
    let zero_budget = tc.draw(gs::booleans());
    let _case = CaseTally;

    let norm_a = crate::hegel::normalize_issue_codes(&raw_codes);
    let mut reversed = raw_codes;
    reversed.reverse();
    let norm_b = crate::hegel::normalize_issue_codes(&reversed);
    assert_eq!(norm_a, norm_b);

    let mut profiles = crate::hegel::all_profiles().expect("profiles");
    for profile in &profiles {
        assert!(crate::hegel::profile_validation::validate(profile).is_empty(), "{profile:?}");
    }
    assert!(crate::hegel::profile_validation::validate_buoy_matrix().is_empty());

    if zero_budget {
        profiles[0].settings.case_budget = 0;
        assert!(!crate::hegel::profile_validation::validate(&profiles[0]).is_empty());
    }

    let mut catalog = crate::effect_closure::embedded_catalog().expect("embedded catalog");
    if duplicate_role_edge {
        let first_edge = catalog.role_edges[0].clone();
        catalog.role_edges.push(first_edge);
    }
    let catalog_diags = crate::effect_closure::validate_catalog(&catalog);
    if duplicate_role_edge {
        assert!(!catalog_diags.is_empty());
    } else {
        assert!(catalog_diags.is_empty(), "{catalog_diags:?}");
    }
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, database = db_path, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
pub(super) fn chaoscontrol_proof_style_boundary(tc: hegel::TestCase, db_path: Option<String>) {
    let extra_crate = tc.draw(gs::text().max_size(MAX_CODE_LEN));
    let drop_vmm = tc.draw(gs::booleans());
    let stale_registry = tc.draw(gs::booleans());
    let stale_trellis = tc.draw(gs::booleans());
    let overbroad = tc.draw(gs::booleans());
    let _case = CaseTally;

    let mut focused_crates = vec!["chaoscontrol-core".to_owned(), "chaoscontrol-vmm".to_owned()];
    if !extra_crate.trim().is_empty() && !focused_crates.contains(&extra_crate) {
        focused_crates.push(extra_crate);
    }
    if drop_vmm {
        focused_crates.retain(|c| c != "chaoscontrol-vmm");
    }
    let mut non_claims = vec![
        "Octet validates profile and input identity only; it does not prove ChaosControl behavioral correctness, runtime determinism, or VMM correctness".to_owned(),
    ];
    if overbroad {
        non_claims.push("proves runtime determinism".to_owned());
    }
    let contract = crate::evidence_contracts::ChaosControlProofStyleInputContract {
        schema_version: "octet-chaoscontrol-proof-style-input-contract/v1".to_owned(),
        consumer_repo: "github:OnixResearch/chaoscontrol".to_owned(),
        selected_profile: "proof-style-focused".to_owned(),
        focused_crates,
        policy_registry_digest: if stale_registry {
            "b3:0000000000000000000000000000000000000000000000000000000000000000".to_owned()
        } else {
            "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned()
        },
        trellis_primitive_manifest_digest: if stale_trellis {
            None
        } else {
            Some("b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned())
        },
        input_revision: "chaoscontrol-proof-style-fixture-revision".to_owned(),
        expected_command: vec![
            "cargo".to_owned(),
            "octet".to_owned(),
            "check".to_owned(),
            "--profile".to_owned(),
            "proof-style-focused".to_owned(),
        ],
        non_claims,
    };
    let receipt = crate::evidence_contracts::validate_chaoscontrol_proof_style_input(&contract);
    let is_clean = !drop_vmm && !stale_registry && !stale_trellis && !overbroad;
    assert_eq!(receipt.valid, is_clean, "{receipt:?}");
    if drop_vmm {
        assert!(receipt.issues.contains(&"focused_crate_missing"));
    }
    if stale_registry {
        assert!(receipt.issues.contains(&"policy_registry_digest_stale"));
    }
    if stale_trellis {
        assert!(receipt.issues.contains(&"trellis_manifest_digest_stale"));
    }
    if overbroad {
        assert!(receipt.issues.contains(&"overbroad_claim"));
    }
}
