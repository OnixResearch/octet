use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::MemoryRegion;
use octet_architecture_ir::semantic::UnitSummary;
use octet_architecture_ir::semantic::UnsupportedKind;
use octet_architecture_ir::semantic::compute_unit_summary_blake3;

const PROBE_CARGO_TOML: &str = r#"[package]
name = "semantic-probe"
version = "0.1.0"
edition = "2024"
publish = false
"#;

const PROBE_LIB_RS: &str = r#"pub struct EffectfulDrop {
    pub flag: u32,
}

impl Drop for EffectfulDrop {
    fn drop(&mut self) {
        self.flag = 0;
    }
}

pub fn pure_local(x: u32) -> u32 {
    let mut acc = x;
    acc = acc.wrapping_add(1);
    acc
}

pub fn consume<T>(value: T) {
    let _moved = value;
}

pub fn trigger_drop() {
    consume(EffectfulDrop { flag: 1 });
}

pub fn compare<T: PartialEq>(left: &T, right: &T) -> bool {
    *left == *right
}

pub trait Projection<'a> {
    type Output;
}

pub struct Slot;

impl<'a> Projection<'a> for Slot {
    type Output = &'a mut u8;
}

pub fn clear(out: <Slot as Projection<'_>>::Output) {
    *out = 0;
}

pub fn interior_mut(cell: &core::cell::Cell<u32>) {
    cell.set(7);
}

pub fn dyn_dispatch(callable: &dyn Fn() -> u32) -> u32 {
    callable()
}

pub fn fn_ptr(callable: fn() -> u32) -> u32 {
    callable()
}
"#;

const PURE_LIB_RS: &str = r#"pub struct Pair {
    pub left: u32,
    pub right: u32,
}

pub fn pure_local_and_field(x: u32) -> u32 {
    let mut pair = Pair { left: x, right: 2 };
    pair.left = pair.left.wrapping_add(pair.right);
    pair.left
}
"#;

// r[verify octet.semantic_effect.mir_collector_probes]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_rustc_mir_collects_implicit_drops_traits_projections_and_admits_pure_crate() {
    let library = super::required_artifact("OCTET_UI_LIBRARY");
    let driver = super::required_artifact("OCTET_UI_DRIVER");
    let (_, host) = crate::architecture_ir::toolchain_identity(super::TOOLCHAIN).expect("host triple");

    let mixed_summary = run_semantic_probe(&library, &driver, &host, PROBE_LIB_RS);
    assert_eq!(compute_unit_summary_blake3(&mixed_summary).expect("hash"), mixed_summary.summary_blake3);

    let find_body = |suffix: &str| {
        mixed_summary
            .bodies
            .iter()
            .find(|body| body.qualified_path.ends_with(suffix))
            .unwrap_or_else(|| panic!("missing probe body {suffix}"))
    };
    let pure_body = find_body("::pure_local");
    assert!(pure_body.drops.is_empty() && pure_body.unsupported.is_empty());

    let consume_body = find_body("::trigger_drop");
    assert!(consume_body.drops.iter().any(|edge| {
        matches!(edge.path_kind, DropPathKind::Normal | DropPathKind::UnwindCleanup | DropPathKind::NormalAndUnwind)
    }));

    let compare_body = find_body("::compare");
    assert!(compare_body.calls.iter().any(|edge| {
        edge.callee_definition.as_deref().is_some_and(|p| p.contains("PartialEq::eq"))
            || edge.trait_definition.as_deref().is_some_and(|p| p.contains("PartialEq"))
    }));

    let clear_body = find_body("::clear");
    assert!(clear_body.memory_writes.iter().any(|w| w.region == MemoryRegion::ParamMutRef));

    let interior_body = find_body("::interior_mut");
    assert!(
        !interior_body.calls.is_empty()
            || interior_body.memory_writes.iter().any(|w| w.region == MemoryRegion::InteriorMutable)
    );

    let dyn_body = find_body("::dyn_dispatch");
    assert!(
        dyn_body.unsupported.iter().any(|u| u.kind == UnsupportedKind::DynamicDispatch) || !dyn_body.calls.is_empty()
    );

    let fn_ptr_body = find_body("::fn_ptr");
    assert!(fn_ptr_body.unsupported.iter().any(|u| u.kind == UnsupportedKind::UnresolvedFnPtr));

    let pure_summary = run_semantic_probe(&library, &driver, &host, PURE_LIB_RS);
    let profile: crate::semantic_effect::model::SemanticEffectProfile = serde_json::from_str(include_str!(
        "../../../../standards/effect-closure/generated/semantic-effect-profile.json"
    ))
    .expect("shipped semantic profile");
    let fn_addrs = pure_summary
        .bodies
        .iter()
        .map(|body| crate::effect_closure::FunctionAddressRecord {
            path: "src/lib.rs".to_owned(),
            name: body.qualified_path.clone(),
            function_address: body.item_id.clone(),
            source_identity: pure_summary.source_blake3.clone(),
        })
        .collect::<Vec<_>>();
    let receipt = crate::semantic_effect::evaluate::evaluate(&crate::semantic_effect::evaluate::EvaluationInput {
        profile: &profile,
        profile_blake3: "b3:shipped-profile",
        unit_summary: &pure_summary,
        function_addresses: &fn_addrs,
        trusted_summaries: &[],
        cargo_lock_blake3: "b3:probe-lock",
    })
    .expect("evaluate pure semantic probe");
    assert_eq!(
        receipt.status,
        crate::effect_closure::AdmissionStatus::Admitted,
        "pure MIR probe must be admitted: {:?}",
        receipt.diagnostics
    );
    assert!(crate::semantic_effect::verify_receipt(&receipt).is_empty());
}

fn run_semantic_probe(library: &std::path::Path, driver: &std::path::Path, host: &str, lib_rs: &str) -> UnitSummary {
    let temp = tempfile::TempDir::new().expect("probe dir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("src")).expect("src dir");
    std::fs::write(root.join("Cargo.toml"), PROBE_CARGO_TOML).expect("Cargo.toml");
    std::fs::write(root.join("Cargo.lock"), "# probe lock\nversion = 4\n").expect("Cargo.lock");
    std::fs::write(root.join("src/lib.rs"), lib_rs).expect("src/lib.rs");

    let policy = probe_policy(root, host);

    let scope = vec!["--workspace".to_owned()];
    let args = vec!["--lib".to_owned()];
    let prepared =
        crate::architecture_ir::prepare_semantic_collector(root, &policy, super::TOOLCHAIN, library, &scope, &args)
            .expect("prepare semantic collector");

    let output = std::process::Command::new("cargo")
        .arg("check")
        .arg("--lib")
        .current_dir(root)
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("dylint libs"))
        .env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &prepared.context_path)
        .output()
        .expect("run cargo check with semantic collector");
    assert!(output.status.success(), "probe compile failed: {}", String::from_utf8_lossy(&output.stderr));

    let unit_id = &prepared.expected_units[0].unit.unit_id;
    let semantic_path = prepared.shard_directory.join(format!("{unit_id}.semantic.json"));
    let bytes = std::fs::read(&semantic_path).expect("read emitted semantic unit summary");
    serde_json::from_slice(&bytes).expect("parse emitted semantic unit summary")
}

fn probe_policy(root: &std::path::Path, host: &str) -> crate::architecture_ir::SelectedProjectArchitecturePolicy {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../../standards/project-architecture/octet-self-architecture.json"))
            .expect("template policy");
    value["project_id"] = "semantic-probe-fixture".into();
    value["mode"] =
        serde_json::to_value(octet_standards::project_architecture_policy::ArchitectureMode::Gate).expect("gate mode");
    value["production"] = serde_json::json!({
        "packages": ["semantic-probe"],
        "source_scopes": ["src"],
        "targets": [{"triple": host, "features": [], "default_features": true}],
    });
    value["roles"] = serde_json::json!([
        {"scope_kind": "package", "scope": "semantic-probe", "role": "domain-core"},
        {"scope_kind": "source", "scope": "src", "role": "domain-core"}
    ]);
    value["dependency_rules"] = serde_json::json!([{
        "caller_role": "domain-core",
        "allowed_callee_roles": ["domain-core"],
        "edge_kinds": ["normal", "build"]
    }]);
    for field in [
        "provider_classifications",
        "ports",
        "capability_classifications",
        "outbound_effects",
        "core_no_std",
        "waivers",
    ] {
        value[field] = serde_json::json!([]);
    }
    value["admission"] = serde_json::Value::Null;
    let bytes = serde_json::to_vec(&value).expect("serialize probe policy");
    let parsed = octet_standards::project_architecture_policy::parse_project_architecture_policy(&bytes)
        .expect("parse probe policy");
    let identity = blake3::hash(&bytes).to_hex().to_string();
    crate::architecture_ir::SelectedProjectArchitecturePolicy {
        source_path: root.join("fixture-policy.ncl"),
        export_path: root.join("fixture-policy.json"),
        manifest_path: root.join("fixture-policy.manifest.json"),
        source_blake3: identity.clone(),
        policy_blake3: identity,
        waiver_policy_blake3: octet_standards::project_architecture_policy::architecture_waiver_policy_blake3(&parsed),
        policy: parsed,
    }
}
