//! Full bundle replay controls, including resealed provenance and phase status.
use octet_architecture_ir as ir;
use octet_standards::project_architecture_gate as gate;
use octet_standards::project_architecture_policy as policy;

use super::*;

fn synthetic_policy() -> policy::ProjectArchitecturePolicy {
    // Reuse the reviewed Nickel-derived DTO for bounded collector defaults. This is synthetic input,
    // not evidence that Nickel or a compiler produced these observations.
    let mut policy: policy::ProjectArchitecturePolicy = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../standards/project-architecture/octet-self-architecture.json"
    )))
    .expect("policy fixture");
    policy.project_id = "ownership-replay-fixture".to_owned();
    policy.mode = policy::ArchitectureMode::Gate;
    policy.production.packages = vec!["fixture".to_owned()];
    policy.production.source_scopes = vec!["src".to_owned()];
    policy.production.targets = vec![policy::ArchitectureTarget {
        triple: "x86_64-unknown-linux-gnu".to_owned(),
        features: Vec::new(),
        default_features: true,
    }];
    policy.roles = [
        (policy::ArchitectureScopeKind::Package, "fixture", policy::ArchitectureRole::CompositionRoot),
        (policy::ArchitectureScopeKind::Source, "src", policy::ArchitectureRole::CompositionRoot),
        (policy::ArchitectureScopeKind::Source, "tests", policy::ArchitectureRole::Test),
    ]
    .into_iter()
    .map(|(scope_kind, scope, role)| policy::ArchitectureRoleAssignment {
        scope_kind,
        scope: scope.to_owned(),
        role,
    })
    .collect();
    policy.ports.clear();
    policy.dependency_rules.clear();
    policy.provider_classifications.clear();
    policy.capability_classifications.clear();
    policy.outbound_effects.clear();
    policy.core_no_std.clear();
    policy.waivers.clear();
    policy.admission = None;
    policy
}

fn synthetic_unit(policy_id: &str, test: bool) -> ir::CompilationUnit {
    let mut unit = ir::CompilationUnit {
        unit_id: String::new(),
        package_name: "fixture".to_owned(),
        package_version: "0.1.0".to_owned(),
        target_name: if test { "integration" } else { "fixture" }.to_owned(),
        target_kind: if test { "test+test" } else { "bin" }.to_owned(),
        source_path: if test { "tests/integration.rs" } else { "src/main.rs" }.to_owned(),
        crate_types: vec!["bin".to_owned()],
        target_triple: "x86_64-unknown-linux-gnu".to_owned(),
        features: Vec::new(),
        default_features: true,
        role: if test {
            ir::CompilerArchitectureRole::Test
        } else {
            ir::CompilerArchitectureRole::CompositionRoot
        },
        toolchain_blake3: "a".repeat(64),
        collector_blake3: "b".repeat(64),
        policy_blake3: policy_id.to_owned(),
        source_blake3: "c".repeat(64),
        invocation_blake3: "d".repeat(64),
    };
    unit.unit_id = ir::compilation_unit_identity(&unit);
    unit
}

fn insert_json(files: &mut BTreeMap<String, Vec<u8>>, name: &str, value: &impl serde::Serialize) {
    files.insert(name.to_owned(), serde_json::to_vec(value).expect("fixture JSON"));
}

fn store_receipt(files: &mut BTreeMap<String, Vec<u8>>, receipt: &gate::ProjectArchitectureReceipt) {
    insert_json(files, PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME, receipt);
    let mut status: serde_json::Value = serde_json::from_slice(&files[STATUS_ARTIFACT_NAME]).expect("status");
    status["phases"] = serde_json::json!({"lint": {"status": receipt.lint_phase_status}, "architecture": {"status": receipt.status, "receipt_id": receipt.receipt_id}});
    insert_json(files, STATUS_ARTIFACT_NAME, &status);
}

fn reseal_receipt(receipt: &mut gate::ProjectArchitectureReceipt) {
    receipt.receipt_id.clear();
    receipt.receipt_id =
        ir::canonical_identity("octet.project-architecture-gate.receipt/v2", receipt).expect("receipt identity");
}

fn synthetic_files() -> BTreeMap<String, Vec<u8>> {
    let mut files = valid_input(OutputFormat::Human)
        .files
        .into_iter()
        .map(|file| (file.name, file.contents))
        .collect::<BTreeMap<_, _>>();
    let policy = synthetic_policy();
    insert_json(&mut files, PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME, &policy);
    files.insert(
        PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME.to_owned(),
        b"# Synthetic replay input, not independently evaluated Nickel evidence.\n{}\n".to_vec(),
    );
    let policy_id = crate::evidence::artifact_blake3_hex(&files[PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME]);
    let production = synthetic_unit(&policy_id, false);
    let test = synthetic_unit(&policy_id, true);
    let mut configuration = ir::CompilerArchitectureFact::Configuration {
        fact_id: String::new(),
        predicate: "test".to_owned(),
        active: true,
        span_id: None,
    };
    let fact_id = ir::fact_identity(&configuration);
    if let ir::CompilerArchitectureFact::Configuration { fact_id: current, .. } = &mut configuration {
        *current = fact_id;
    }
    let mut unknown = ir::UnknownCompilerFact {
        unknown_id: String::new(),
        kind: ir::UnknownFactKind::DynamicDispatch,
        owner_item_id: None,
        target_identity: Some("test dispatch".to_owned()),
        span_id: None,
        detail: "synthetic unresolved test target".to_owned(),
    };
    unknown.unknown_id = ir::unknown_fact_identity(&unknown);
    let shards = [production, test]
        .into_iter()
        .map(|unit| {
            let is_test = unit.role == ir::CompilerArchitectureRole::Test;
            let mut shard = ir::CompilerArchitectureShard {
                schema_version: ir::COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION.to_owned(),
                shard_id: String::new(),
                unit,
                collection_status: ir::CollectionStatus::Complete,
                spans: Vec::new(),
                facts: if is_test {
                    vec![configuration.clone()]
                } else {
                    Vec::new()
                },
                unknowns: if is_test { vec![unknown.clone()] } else { Vec::new() },
                non_claims: vec!["signer-authenticity".to_owned()],
            };
            shard.shard_id = ir::shard_identity(&shard);
            shard
        })
        .collect::<Vec<_>>();
    let normalized = ir::normalize_architecture_shards(ir::NormalizationInput {
        policy_blake3: policy_id.clone(),
        limits: policy.collection_limits.clone(),
        expected_units: shards
            .iter()
            .map(|shard| ir::ExpectedCompilationUnit {
                unit: shard.unit.clone(),
            })
            .collect(),
        shards,
        invalid_shards: Vec::new(),
        non_claims: vec!["signer-authenticity".to_owned()],
    });
    let normalized = normalized.expect("valid normalization inputs");
    let ir = normalized.architecture_ir.expect("complete synthetic normalization");
    let mut graph = gate::ArchitectureCargoGraph {
        schema_version: gate::PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION.to_owned(),
        cargo_graph_id: String::new(),
        cargo_lock_blake3: "a".repeat(64),
        cargo_metadata_blake3: "b".repeat(64),
        selected_packages: vec!["fixture".to_owned()],
        target_triple: "x86_64-unknown-linux-gnu".to_owned(),
        features: Vec::new(),
        default_features: true,
        edges: Vec::new(),
    };
    graph.cargo_graph_id = gate::architecture_cargo_graph_identity(&graph);
    let receipt = gate::evaluate_project_architecture(&gate::ProjectArchitectureGateInput {
        policy: &policy,
        policy_source_blake3: &crate::evidence::artifact_blake3_hex(
            &files[PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME],
        ),
        policy_blake3: &policy_id,
        lint_phase_status: "clean",
        command_blake3: &crate::evidence::artifact_blake3_hex(&files[COMMAND_ARTIFACT_NAME]),
        cargo_graph: &graph,
        architecture_ir: &ir,
        coverage: &normalized.coverage,
        compiler_receipt: &normalized.receipt,
        effect_closure: None,
    });
    assert_eq!(receipt.status, gate::ArchitecturePhaseStatus::Clean, "{:?}", receipt.findings);
    insert_json(&mut files, PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME, &graph);
    insert_json(&mut files, ARCHITECTURE_IR_ARTIFACT_NAME, &ir);
    insert_json(&mut files, ARCHITECTURE_COVERAGE_ARTIFACT_NAME, &normalized.coverage);
    insert_json(&mut files, ARCHITECTURE_RECEIPT_ARTIFACT_NAME, &normalized.receipt);
    insert_json(&mut files, ARCHITECTURE_ISSUES_ARTIFACT_NAME, &normalized.issues);
    store_receipt(&mut files, &receipt);
    files
}

fn reseal_bundle(mut files: BTreeMap<String, Vec<u8>>) -> ArtifactBundleInput {
    let mut records = valid_records(OutputFormat::Human);
    for record in &mut records {
        record.blake3 = crate::evidence::artifact_blake3_hex(&files[&record.artifact_locator]);
    }
    for (names, source, tier) in [
        (ARCHITECTURE_ARTIFACT_NAMES, ARCHITECTURE_IR_SOURCE_ID, EvidenceTier::StaticLint),
        (PROJECT_ARCHITECTURE_ARTIFACT_NAMES, PROJECT_ARCHITECTURE_SOURCE_ID, EvidenceTier::PolicyCheck),
    ] {
        for name in names {
            let mut record = record(name, &files[*name], tier);
            record.source_id = source.to_owned();
            records.push(record);
        }
    }
    files.insert(PROVENANCE_ARTIFACT_NAME.to_owned(), render_records(&records).into_bytes());
    ArtifactBundleInput {
        artifact_dir: ARTIFACT_DIR.to_owned(),
        files: files.into_iter().map(|(name, contents)| BundleFile { name, contents }).collect(),
    }
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn full_bundle_replay_rejects_resealed_test_disposition_drift() {
    let mut files = synthetic_files();
    let valid = verify_bundle(reseal_bundle(files.clone()));
    assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);
    let mut receipt: gate::ProjectArchitectureReceipt =
        serde_json::from_slice(&files[PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME]).expect("receipt");
    let disposition = receipt.test_only.as_mut().expect("partition");
    assert!(!disposition.fact_ids.is_empty() && !disposition.unknown_ids.is_empty());
    disposition.fact_ids.clear();
    disposition.unknown_ids.clear();
    reseal_receipt(&mut receipt);
    assert!(gate::verify_project_architecture_receipt(&receipt).is_empty(), "self-consistency is not replay");
    store_receipt(&mut files, &receipt);
    let invalid = verify_bundle(reseal_bundle(files));
    assert_eq!(invalid.status, STATUS_INVALID);
    assert_eq!(invalid.diagnostics, [
        "project architecture receipt does not match pure reevaluation of stored artifacts".to_owned()
    ]);
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn full_bundle_replay_rejects_resealed_orphan_test_unknown() {
    let mut files = synthetic_files();
    let mut ir: ir::CompilerArchitectureIr = serde_json::from_slice(&files[ARCHITECTURE_IR_ARTIFACT_NAME]).expect("IR");
    for row in &mut ir.unit_memberships {
        row.unknown_ids.clear();
    }
    ir.architecture_ir_id = ir::ir_identity(&ir);
    insert_json(&mut files, ARCHITECTURE_IR_ARTIFACT_NAME, &ir);
    let mut compiler: ir::CompilerArchitectureReceipt =
        serde_json::from_slice(&files[ARCHITECTURE_RECEIPT_ARTIFACT_NAME]).expect("compiler receipt");
    compiler.architecture_ir_id = Some(ir.architecture_ir_id.clone());
    compiler.receipt_id = ir::receipt_identity(&compiler);
    insert_json(&mut files, ARCHITECTURE_RECEIPT_ARTIFACT_NAME, &compiler);
    let mut receipt: gate::ProjectArchitectureReceipt =
        serde_json::from_slice(&files[PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME]).expect("receipt");
    receipt.architecture_ir_id = ir.architecture_ir_id;
    receipt.compiler_receipt_id = compiler.receipt_id;
    reseal_receipt(&mut receipt);
    store_receipt(&mut files, &receipt);
    let invalid = verify_bundle(reseal_bundle(files));
    assert_eq!(invalid.status, STATUS_INVALID);
    assert!(
        invalid
            .diagnostics
            .iter()
            .any(|error| error.contains("unknown memberships contain orphan or unreferenced identities")),
        "{:?}",
        invalid.diagnostics
    );
    assert!(invalid.diagnostics.iter().any(|error| error.contains("pure reevaluation")));
}
