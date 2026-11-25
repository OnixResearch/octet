use std::fs;
use std::path::Path;

use super::ClosureMetadataInput;
use super::CompletenessStatus;
use super::ResolvedDepEdge;
use super::ResolvedPackageInput;
use super::closure::compute_provider_envelopes;
use super::contract::EFFECT_CLOSURE_RECEIPT_CONTRACT_PATH;
use super::contract::render_receipt_contract;
use super::policy::BuildContextInput;
use super::policy::compute_build_context;
use super::policy::embedded_settings as embedded_policy;
use super::scan::PackageTargetMeta;
use super::scan::inspect_local_package as scan_local_package;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write fixture file");
}

#[test]
fn effect_closure_receipt_contract_is_fresh() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("repo root");
    let path = root.join(EFFECT_CLOSURE_RECEIPT_CONTRACT_PATH);
    let expected = fs::read_to_string(&path).expect("read generated receipt contract");
    assert_eq!(expected, render_receipt_contract(), "effect-closure receipt contract is stale");
}

#[test]
fn package_envelope_is_relocation_invariant_and_detects_body_and_include_edits() {
    let policy = embedded_policy().expect("embedded policy");
    let dir_a = tempfile::tempdir().expect("dir_a");
    let dir_b = tempfile::tempdir().expect("dir_b");
    for root in [dir_a.path(), dir_b.path()] {
        write(&root.join("Cargo.toml"), "[package]\nname = \"dep\"\nversion = \"0.1.0\"\n");
        write(&root.join("src/data.txt"), "hello-v1\n");
        write(
            &root.join("src/lib.rs"),
            "pub const MSG: &str = include_str!(\"data.txt\"); pub fn f() -> u32 { 1 }\n",
        );
    }
    let scan_a = scan_local_package(&dir_a.path().join("Cargo.toml"), &[], &policy);
    let scan_b = scan_local_package(&dir_b.path().join("Cargo.toml"), &[], &policy);
    assert_eq!(scan_a.completeness, CompletenessStatus::Complete);
    assert_eq!(scan_a.package_envelope_blake3, scan_b.package_envelope_blake3);
    assert_eq!(scan_a.generated_inputs.len(), 1);
    assert_eq!(scan_a.generated_inputs[0].locator, "src/data.txt");

    write(&dir_b.path().join("src/data.txt"), "hello-v2\n");
    let scan_inc = scan_local_package(&dir_b.path().join("Cargo.toml"), &[], &policy);
    assert_ne!(scan_a.package_envelope_blake3, scan_inc.package_envelope_blake3);

    write(&dir_b.path().join("src/data.txt"), "hello-v1\n");
    write(
        &dir_b.path().join("src/lib.rs"),
        "pub const MSG: &str = include_str!(\"data.txt\"); pub fn f() -> u32 { let _ = std::fs::read(\"x\"); 1 }\n",
    );
    let scan_body = scan_local_package(&dir_b.path().join("Cargo.toml"), &[], &policy);
    assert_ne!(scan_a.package_envelope_blake3, scan_body.package_envelope_blake3);
}

#[test]
fn package_envelope_rejects_symlinks_missing_files_and_unadmitted_build_inputs() {
    let policy = embedded_policy().expect("embedded policy");
    let missing = scan_local_package(Path::new("/nonexistent/dep/Cargo.toml"), &[], &policy);
    assert_eq!(missing.completeness, CompletenessStatus::Incomplete);

    let symlink_dir = tempfile::tempdir().expect("symlink_dir");
    write(&symlink_dir.path().join("Cargo.toml"), "[package]\nname = \"sym\"\nversion = \"0.1.0\"\n");
    write(&symlink_dir.path().join("real.rs"), "pub fn x() {}\n");
    std::os::unix::fs::symlink(symlink_dir.path().join("real.rs"), symlink_dir.path().join("src_link.rs"))
        .expect("symlink rs");
    let sym_scan = scan_local_package(&symlink_dir.path().join("Cargo.toml"), &[], &policy);
    assert_eq!(sym_scan.completeness, CompletenessStatus::Incomplete);

    let build_dir = tempfile::tempdir().expect("build_dir");
    write(&build_dir.path().join("Cargo.toml"), "[package]\nname = \"bld\"\nversion = \"0.1.0\"\n");
    write(&build_dir.path().join("build.rs"), "fn main() {}\n");
    write(&build_dir.path().join("src/lib.rs"), "pub fn x() {}\n");
    let targets = vec![PackageTargetMeta {
        kind: vec!["custom-build".to_owned()],
        src_path: build_dir.path().join("build.rs").display().to_string(),
    }];
    let build_scan = scan_local_package(&build_dir.path().join("Cargo.toml"), &targets, &policy);
    assert_eq!(build_scan.completeness, CompletenessStatus::UnsupportedGeneratedInput);

    let env_dir = tempfile::tempdir().expect("env_dir");
    write(&env_dir.path().join("Cargo.toml"), "[package]\nname = \"envd\"\nversion = \"0.1.0\"\n");
    write(&env_dir.path().join("src/lib.rs"), "pub fn x() -> &'static str { env!(\"FOO\") }\n");
    let env_scan = scan_local_package(&env_dir.path().join("Cargo.toml"), &[], &policy);
    assert_eq!(env_scan.completeness, CompletenessStatus::UnsupportedGeneratedInput);
}

#[test]
fn closure_propagates_transitive_edits_and_fails_on_resolve_cycle() {
    let policy = embedded_policy().expect("embedded policy");
    let root = tempfile::tempdir().expect("ws");
    write(&root.path().join("Cargo.lock"), "# lock\n");
    write(&root.path().join("ws_dep/Cargo.toml"), "[package]\nname = \"ws_dep\"\nversion = \"0.1.0\"\n");
    write(&root.path().join("ws_dep/src/lib.rs"), "pub fn f() -> u32 { trans_dep::g() }\n");
    write(&root.path().join("trans_dep/Cargo.toml"), "[package]\nname = \"trans_dep\"\nversion = \"0.1.0\"\n");
    write(&root.path().join("trans_dep/src/lib.rs"), "pub fn g() -> u32 { 1 }\n");

    let build_ctx = compute_build_context(
        &BuildContextInput {
            workspace_root: root.path(),
            target: "x86_64-unknown-linux-gnu",
            toolchain: "nightly-2026-03-21",
            rustc_version: "rustc 1.90.0-nightly",
            cargo_lock_blake3: "b3:1111111111111111111111111111111111111111111111111111111111111111",
            metadata_cargo_args: &[],
            rustflags: "",
        },
        &policy,
    );
    let make_input = |cyclic: bool| ClosureMetadataInput {
        target: "x86_64-unknown-linux-gnu".to_owned(),
        toolchain: "nightly-2026-03-21".to_owned(),
        cargo_lock_blake3: "b3:1111111111111111111111111111111111111111111111111111111111111111".to_owned(),
        build_context: build_ctx.clone(),
        has_resolve_graph: true,
        packages: vec![
            ResolvedPackageInput {
                id: "ws_dep".to_owned(),
                name: "ws_dep".to_owned(),
                version: "0.1.0".to_owned(),
                source: None,
                manifest_path: root.path().join("ws_dep/Cargo.toml"),
                is_workspace_member: true,
                features: Vec::new(),
                targets: Vec::new(),
                deps: vec![ResolvedDepEdge {
                    pkg_id: "trans_dep".to_owned(),
                    dep_kinds: vec!["normal".to_owned()],
                }],
            },
            ResolvedPackageInput {
                id: "trans_dep".to_owned(),
                name: "trans_dep".to_owned(),
                version: "0.1.0".to_owned(),
                source: None,
                manifest_path: root.path().join("trans_dep/Cargo.toml"),
                is_workspace_member: false,
                features: Vec::new(),
                targets: Vec::new(),
                deps: if cyclic {
                    vec![ResolvedDepEdge {
                        pkg_id: "ws_dep".to_owned(),
                        dep_kinds: vec!["normal".to_owned()],
                    }]
                } else {
                    Vec::new()
                },
            },
        ],
        toolchain_provider_names: vec!["core".to_owned()],
    };
    let before = compute_provider_envelopes(&make_input(false), &policy);
    let ws_before = before.iter().find(|r| r.provider.name == "ws_dep").expect("ws_dep");
    assert_eq!(ws_before.completeness, CompletenessStatus::Complete);

    write(&root.path().join("trans_dep/src/lib.rs"), "pub fn g() -> u32 { 2 }\n");
    let after = compute_provider_envelopes(&make_input(false), &policy);
    let ws_after = after.iter().find(|r| r.provider.name == "ws_dep").expect("ws_dep");
    assert_eq!(ws_before.package_envelope_blake3, ws_after.package_envelope_blake3);
    assert_ne!(ws_before.implementation_closure_blake3, ws_after.implementation_closure_blake3);

    let cyclic = compute_provider_envelopes(&make_input(true), &policy);
    let ws_cyclic = cyclic.iter().find(|r| r.provider.name == "ws_dep").expect("ws_dep");
    assert_eq!(ws_cyclic.completeness, CompletenessStatus::Incomplete);
}

mod containment;
