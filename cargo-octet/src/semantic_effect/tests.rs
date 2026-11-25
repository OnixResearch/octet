use octet_architecture_ir::semantic::BodyObservation;
use octet_architecture_ir::semantic::CallResolutionKind;
use octet_architecture_ir::semantic::DropEdge;
use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::MemoryRegion;
use octet_architecture_ir::semantic::MemoryWrite;
use octet_architecture_ir::semantic::MirCallEdge;
use octet_architecture_ir::semantic::MirStage;
use octet_architecture_ir::semantic::UnitSummary;
use octet_architecture_ir::semantic::UnsupportedConstruct;
use octet_architecture_ir::semantic::UnsupportedKind;
use octet_architecture_ir::semantic::compute_unit_summary_blake3;

use super::evaluate::EvaluationInput;
use super::evaluate::evaluate;
use super::model::SemanticEffect;
use super::model::SemanticEffectProfile;
use super::model::TrustedContractSummary;
use super::model::UnwindMode;

fn base_profile(unwind_mode: UnwindMode) -> SemanticEffectProfile {
    let mut profile: SemanticEffectProfile =
        serde_json::from_str(include_str!("../../../standards/effect-closure/generated/semantic-effect-profile.json"))
            .expect("parse shipped semantic profile");
    profile.unwind_mode = unwind_mode;
    profile
}

fn body(name: &str) -> BodyObservation {
    BodyObservation {
        item_id: format!("item:{name}"),
        qualified_path: format!("core_fixture::{name}"),
        mir_stage: MirStage::OptimizedMir,
        is_unsafe: false,
        is_async: false,
        has_unwind_edges: false,
        has_implicit_panic_assert: false,
        basic_block_count: 2,
        step_count: 8,
        drops: Vec::new(),
        calls: Vec::new(),
        memory_writes: Vec::new(),
        unsupported: Vec::new(),
        partial_moves_observed: 0,
        ended_borrow_reassignments: 0,
        span_id: format!("span:{name}"),
    }
}

fn unit(bodies: Vec<BodyObservation>) -> UnitSummary {
    let mut summary = UnitSummary {
        schema_version: octet_architecture_ir::semantic::SEMANTIC_MIR_UNIT_SCHEMA_VERSION.to_owned(),
        unit_id: "u1".to_owned(),
        package_name: "core_fixture".to_owned(),
        target_name: "core_fixture".to_owned(),
        compiler_stage: super::model::COMPILER_STAGE.to_owned(),
        toolchain_blake3: "b3:toolchain".to_owned(),
        source_blake3: "b3:source".to_owned(),
        bodies,
        is_budget_exhausted: false,
        skipped_bodies: Vec::new(),
        summary_blake3: String::new(),
    };
    summary.summary_blake3 = compute_unit_summary_blake3(&summary).expect("hash unit summary");
    summary
}

fn addrs(names: &[&str]) -> Vec<crate::effect_closure::FunctionAddressRecord> {
    names
        .iter()
        .map(|name| crate::effect_closure::FunctionAddressRecord {
            path: "src/lib.rs".to_owned(),
            name: (*name).to_owned(),
            function_address: format!("addr:{name}"),
            source_identity: "b3:source".to_owned(),
        })
        .collect()
}

#[test]
fn pure_arithmetic_and_owned_local_mutation_are_admitted() {
    let mut pure_fn = body("pure_local");
    pure_fn.memory_writes.push(MemoryWrite {
        local_index: 1,
        projection: ".0".to_owned(),
        resolved_type: "u32".to_owned(),
        region: MemoryRegion::LocalField,
        is_observable_external: false,
        is_interior_mutability: false,
        span_id: None,
    });
    pure_fn.partial_moves_observed = 1;
    pure_fn.ended_borrow_reassignments = 1;

    let profile = base_profile(UnwindMode::ModeledCleanup);
    let summary = unit(vec![pure_fn]);
    let receipt = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["pure_local"]),
        trusted_summaries: &[],
        cargo_lock_blake3: "b3:lock",
    })
    .expect("evaluate");

    assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Admitted);
    assert!(receipt.observed_effects.is_empty());
    assert!(receipt.coverage.is_complete);
    assert!(super::verify_receipt(&receipt).is_empty());
}

#[test]
fn effectful_drop_and_parametric_trait_and_alias_mutation_are_denied() {
    let mut drop_impl = body("EffectfulDrop::drop");
    drop_impl.calls.push(MirCallEdge {
        callee_definition: Some("std::fs::remove_file".to_owned()),
        resolved_instance: Some("std::fs::remove_file".to_owned()),
        trait_definition: None,
        resolution: CallResolutionKind::ConcreteExternal,
        has_generic_params: false,
        span_id: None,
    });

    let mut caller = body("consume");
    caller.drops.push(DropEdge {
        dropped_type: "core_fixture::EffectfulDrop".to_owned(),
        destructor_definition: Some("core_fixture::EffectfulDrop::drop".to_owned()),
        path_kind: DropPathKind::NormalAndUnwind,
        is_resolved: true,
        has_effectful_candidate: true,
        span_id: None,
    });
    caller.calls.push(MirCallEdge {
        callee_definition: Some("core_fixture::EffectfulDrop::drop".to_owned()),
        resolved_instance: Some("core_fixture::EffectfulDrop::drop".to_owned()),
        trait_definition: Some("core::ops::drop::Drop".to_owned()),
        resolution: CallResolutionKind::ConcreteTraitImpl,
        has_generic_params: false,
        span_id: None,
    });

    let mut compare_fn = body("compare");
    compare_fn.calls.push(MirCallEdge {
        callee_definition: Some("core::cmp::PartialEq::eq".to_owned()),
        resolved_instance: None,
        trait_definition: Some("core::cmp::PartialEq".to_owned()),
        resolution: CallResolutionKind::ParametricTraitCall,
        has_generic_params: true,
        span_id: None,
    });

    let mut clear_fn = body("clear");
    clear_fn.memory_writes.push(MemoryWrite {
        local_index: 1,
        projection: "*".to_owned(),
        resolved_type: "u8".to_owned(),
        region: MemoryRegion::ParamMutRef,
        is_observable_external: true,
        is_interior_mutability: false,
        span_id: None,
    });

    let mut cell_fn = body("cell_write");
    cell_fn.memory_writes.push(MemoryWrite {
        local_index: 2,
        projection: String::new(),
        resolved_type: "core::cell::Cell<u8>".to_owned(),
        region: MemoryRegion::InteriorMutable,
        is_observable_external: true,
        is_interior_mutability: true,
        span_id: None,
    });

    let profile = base_profile(UnwindMode::ModeledCleanup);
    let summary = unit(vec![drop_impl, caller, compare_fn, clear_fn, cell_fn]);
    let receipt = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["EffectfulDrop::drop", "consume", "compare", "clear", "cell_write"]),
        trusted_summaries: &[],
        cargo_lock_blake3: "b3:lock",
    })
    .expect("evaluate");

    assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Denied);
    assert!(receipt.observed_effects.contains(&SemanticEffect::Fs));
    assert!(receipt.observed_effects.contains(&SemanticEffect::Unknown));
    assert!(receipt.observed_effects.contains(&SemanticEffect::ExternalMutation));
    assert!(receipt.observed_effects.contains(&SemanticEffect::InteriorMutation));
}

#[test]
fn unwind_rejection_and_unsupported_constructs_and_correspondence_failures_block() {
    let mut b = body("unwind_or_async");
    b.has_unwind_edges = true;
    b.unsupported.push(UnsupportedConstruct {
        kind: UnsupportedKind::AsyncCoroutine,
        detail: "async coroutine".to_owned(),
        span_id: None,
    });

    let profile = base_profile(UnwindMode::RejectUnwind);
    let mut summary = unit(vec![b]);
    let receipt = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["unwind_or_async", "missing_fn"]),
        trusted_summaries: &[],
        cargo_lock_blake3: "b3:lock",
    })
    .expect("evaluate");

    assert_eq!(receipt.status, crate::effect_closure::AdmissionStatus::Denied);
    assert!(receipt.observed_effects.contains(&SemanticEffect::UnwindDrop));
    assert!(receipt.observed_effects.contains(&SemanticEffect::Unknown));
    assert!(!receipt.coverage.is_complete);
    assert_eq!(receipt.coverage.missing_functions, vec!["missing_fn"]);

    summary.is_budget_exhausted = true;
    let budget_receipt = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["unwind_or_async"]),
        trusted_summaries: &[],
        cargo_lock_blake3: "b3:lock",
    })
    .expect("evaluate");
    assert_eq!(budget_receipt.status, crate::effect_closure::AdmissionStatus::Denied);
    assert!(!budget_receipt.coverage.is_complete);
}

#[test]
fn mutual_recursion_converges_and_stale_trusted_summary_is_rejected() {
    let mut f1 = body("ping");
    f1.calls.push(MirCallEdge {
        callee_definition: Some("core_fixture::pong".to_owned()),
        resolved_instance: Some("core_fixture::pong".to_owned()),
        trait_definition: None,
        resolution: CallResolutionKind::ConcreteLocal,
        has_generic_params: false,
        span_id: None,
    });
    let mut f2 = body("pong");
    f2.calls.push(MirCallEdge {
        callee_definition: Some("core_fixture::ping".to_owned()),
        resolved_instance: Some("core_fixture::ping".to_owned()),
        trait_definition: None,
        resolution: CallResolutionKind::ConcreteLocal,
        has_generic_params: false,
        span_id: None,
    });
    f2.calls.push(MirCallEdge {
        callee_definition: Some("ext::pure_helper".to_owned()),
        resolved_instance: Some("ext::pure_helper".to_owned()),
        trait_definition: None,
        resolution: CallResolutionKind::ConcreteExternal,
        has_generic_params: false,
        span_id: None,
    });

    let profile = base_profile(UnwindMode::ModeledCleanup);
    let summary = unit(vec![f1, f2]);
    let valid_trusted = vec![TrustedContractSummary {
        summary_id: "sum:1".to_owned(),
        target_definition: "ext::pure_helper".to_owned(),
        trait_definition: None,
        provider_name: "ext".to_owned(),
        provider_version: "0.1.0".to_owned(),
        cargo_lock_blake3: "b3:lock".to_owned(),
        implementation_blake3: "b3:impl".to_owned(),
        effects: vec![SemanticEffect::Alloc],
        review_label: "reviewed-alloc".to_owned(),
    }];
    let ok = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["ping", "pong"]),
        trusted_summaries: &valid_trusted,
        cargo_lock_blake3: "b3:lock",
    })
    .expect("evaluate");
    assert_eq!(ok.status, crate::effect_closure::AdmissionStatus::Admitted);
    assert_eq!(ok.observed_effects, vec![SemanticEffect::Alloc]);

    let stale_receipt = evaluate(&EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:prof",
        unit_summary: &summary,
        function_addresses: &addrs(&["ping", "pong"]),
        trusted_summaries: &valid_trusted,
        cargo_lock_blake3: "b3:wrong-lock",
    })
    .expect("evaluate");
    assert_eq!(stale_receipt.status, crate::effect_closure::AdmissionStatus::Denied);
}
