use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CoverageStatus;
use octet_architecture_ir::ItemKind;
use octet_standards::project_architecture_policy::parse_project_architecture_policy;

use super::SelectedProjectArchitecturePolicy;

const TOOLCHAIN: &str = "nightly-2026-03-21";
const FIXTURE_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "src/lib.rs",
    "src/main.rs",
    "tests/integration.rs",
    "examples/sample.rs",
    "benches/sample.rs",
    "macro/Cargo.toml",
    "macro/src/lib.rs",
];
const SOURCE_SCOPES: &[&str] = &["src", "tests", "examples", "benches", "build.rs", "macro/src"];
const PACKAGES: &[&str] = &["architecture-target-fixture", "architecture-target-macro"];

fn policy(root: &Path, host: &str) -> SelectedProjectArchitecturePolicy {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../../standards/project-architecture/octet-self-architecture.json"))
            .expect("template policy");
    value["project_id"] = "cargo-target-fixture".into();
    value["mode"] = "inventory".into();
    value["production"] = serde_json::json!({
        "packages": PACKAGES, "source_scopes": SOURCE_SCOPES,
        "targets": [{"triple": host, "features": ["extra"], "default_features": true}],
    });
    let mut roles = Vec::new();
    for package in PACKAGES {
        roles.push(serde_json::json!({"scope_kind":"package","scope":package,"role":"domain-core"}));
    }
    for scope in SOURCE_SCOPES {
        roles.push(serde_json::json!({"scope_kind":"source","scope":scope,"role":"domain-core"}));
    }
    value["roles"] = roles.into();
    value["dependency_rules"] = serde_json::json!([{
        "caller_role":"domain-core", "allowed_callee_roles":["domain-core"], "edge_kinds":["normal","build"],
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
    value["capability_classifications"] = serde_json::json!([{
        "family": "filesystem", "core_handling": "forbidden", "executor_role": "outbound-adapter",
        "rationale": "fixture filesystem effects must remain visible and forbidden in the core",
    }]);
    value["admission"] = serde_json::Value::Null;
    let bytes = serde_json::to_vec(&value).expect("serialize fixture policy");
    let policy = parse_project_architecture_policy(&bytes).expect("validate typed fixture policy");
    let identity = blake3::hash(&bytes).to_hex().to_string();
    SelectedProjectArchitecturePolicy {
        source_path: root.join("fixture-policy.ncl"),
        export_path: root.join("fixture-policy.json"),
        manifest_path: root.join("fixture-policy.manifest.json"),
        source_blake3: identity.clone(),
        policy_blake3: identity,
        policy,
    }
}

fn required_artifact(name: &str) -> PathBuf {
    let path = PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("set {name} for the compiler control")));
    assert!(path.is_absolute() && path.is_file(), "missing compiler control artifact");
    path
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_has_complete_compiler_coverage() {
    run_compiler_fixture(false);
}

// r[verify octet.architecture.features.equivalence]
// r[verify octet.architecture.features.rejection]
// r[verify octet.architecture.features.evidence]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_has_complete_compiler_coverage_with_all_features() {
    run_compiler_fixture(true);
}

fn run_compiler_fixture(all_features: bool) {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("owner root");
    let source = owner.join("fixtures/architecture-targets");
    let temporary = tempfile::TempDir::new().expect("fixture workspace");
    let root = temporary.path();
    for relative in FIXTURE_FILES {
        let target = root.join(relative);
        fs::create_dir_all(target.parent().expect("fixture parent")).expect("create fixture parent");
        fs::copy(source.join(relative), target).expect("copy controlled fixture source");
    }
    let (_, host) = super::toolchain_identity(TOOLCHAIN).expect("pinned host");
    let policy = policy(root, &host);
    let scope = vec!["--workspace".to_owned()];
    let args = if all_features {
        vec!["--all-targets".to_owned(), "--all-features".to_owned()]
    } else {
        vec!["--all-targets".to_owned(), "--features".to_owned(), "extra".to_owned()]
    };
    if all_features {
        let mut incomplete = self::policy(root, &host);
        incomplete.policy.production.targets[0].features.clear();
        let rejected = super::prepare_collector(root, &incomplete, TOOLCHAIN, &library, &scope, &args);
        assert!(rejected.err().expect("under-declared row must fail").contains("does not match"));
        assert!(!root.join(".octet").exists(), "mismatch must not publish a collector context");
    }
    let prepared = super::prepare_collector(root, &policy, TOOLCHAIN, &library, &scope, &args)
        .expect("plan actual Cargo compilation units");
    for kind in ["lib", "bin", "test", "example", "bench", "proc-macro", "custom-build"] {
        assert!(
            prepared
                .expected_units
                .iter()
                .any(|expected| expected.unit.target_kind.split('+').next() == Some(kind)),
            "missing {kind} unit"
        );
    }
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["check", "--locked", "--workspace"])
        .args(&args)
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("library selection"))
        .env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &prepared.context_path)
        .output()
        .expect("compile fixture through real collector");
    assert!(output.status.success(), "compiler failed: {}", String::from_utf8_lossy(&output.stderr));
    let artifacts = super::normalize_collected_shards(&prepared).expect("normalize actual shards");
    assert!(artifacts.incomplete_error.is_none(), "{:?}", artifacts.incomplete_error);
    let coverage: CompilerArchitectureCoverage = serde_json::from_str(&artifacts.coverage).expect("coverage");
    assert_eq!(coverage.status, CoverageStatus::Complete);
    assert_eq!(coverage.expected_unit_ids.len(), prepared.expected_units.len());
    let ir: CompilerArchitectureIr = serde_json::from_str(&artifacts.architecture_ir).expect("IR");
    assert!(
        ir.facts
            .iter()
            .any(|fact| matches!(fact, octet_architecture_ir::CompilerArchitectureFact::Effect { .. }))
    );
    assert_associated_item_classification(&ir);
    assert!(
        ir.unknowns
            .iter()
            .all(|unknown| unknown.kind != octet_architecture_ir::UnknownFactKind::UnsupportedExpansion),
        "fixture expansions must retain compiler-resolved provenance: {:?}",
        ir.unknowns
    );
    eprintln!(
        "actual Cargo target coverage: {} units, all_features={all_features}",
        coverage.observed_unit_ids.len()
    );
    let shard = fs::read_dir(&prepared.shard_directory)
        .expect("shards")
        .next()
        .expect("one shard")
        .expect("entry")
        .path();
    fs::remove_file(shard).expect("remove one controlled observation");
    let omitted = super::normalize_collected_shards(&prepared).expect("normalize omitted unit");
    assert!(omitted.incomplete_error.is_some(), "missing compiler unit must fail closed");
}

fn assert_associated_item_classification(ir: &CompilerArchitectureIr) {
    for (suffix, expected_kind) in [
        ("::Value", ItemKind::TypeAlias),
        ("::PRESENT", ItemKind::Constant),
        ("::Associated::identity", ItemKind::Function),
    ] {
        let mut observed = false;
        for fact in &ir.facts {
            let CompilerArchitectureFact::Item {
                qualified_path,
                item_kind,
                signature_id,
                ..
            } = fact
            else {
                continue;
            };
            if !qualified_path.ends_with(suffix) {
                continue;
            }
            observed = true;
            assert_eq!(*item_kind, expected_kind, "wrong classification for {qualified_path}");
            assert_eq!(signature_id.is_some(), expected_kind == ItemKind::Function);
        }
        assert!(observed, "missing associated item {suffix}");
    }
}
