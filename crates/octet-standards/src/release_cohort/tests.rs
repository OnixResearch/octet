//! Public release admission and negative-control fixtures.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

use super::evidence_digest;
use super::ready;
use super::validate;
use super::validate_inventory;

const COHORT_JSON: &str = "standards/release/generated/release-cohort.json";
const INVENTORY_FIXTURE: &str = "tests/fixtures/release/public-lint-inventory-names.txt";
const FLAKE_LOCK_FIXTURE: &str = "tests/fixtures/release/public-flake.lock";
const TOOLCHAIN_FIXTURE: &str = "tests/fixtures/release/public-rust-toolchain.toml";
const INVALID_FIXTURES: &str = "standards/release/fixtures/invalid";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).expect(relative)
}

fn load_cohort(root: &Path) -> Value {
    serde_json::from_str(&read(root, COHORT_JSON)).expect("cohort json")
}

fn apply_overrides(manifest: &mut Value, overrides: &[Value]) {
    for entry in overrides {
        let path = entry["path"].as_str().expect("override path");
        let pointer = format!("/{}", path.replace('.', "/"));
        let value = entry["value"].clone();
        let slot = manifest.pointer_mut(&pointer).expect("override path resolves");
        *slot = value;
    }
}

#[test]
fn blocked_cohort_cannot_be_admitted_with_valid_receipts() {
    let root = repo_root();
    let mut manifest = load_cohort(&root);
    manifest["qualification"] = serde_json::json!({
        "status": "blocked",
        "blocking_reasons": [{
            "id": "pending-owner-review",
            "summary": "Owner review is pending.",
            "required_action": "Complete the source-bound review."
        }]
    });
    let inventory = read(&root, INVENTORY_FIXTURE);
    let lock = read(&root, FLAKE_LOCK_FIXTURE);
    let toolchain = read(&root, TOOLCHAIN_FIXTURE);

    assert_eq!(validate(&manifest), Vec::new());
    assert_eq!(validate_inventory(&manifest, &inventory), Vec::new());

    let revision = manifest["source"]["public"]["revision"].as_str().expect("revision");
    let digest = evidence_digest(revision, inventory.as_bytes(), lock.as_bytes(), toolchain.as_bytes());
    assert_eq!(manifest["acceptance"]["evidence_digest"].as_str(), Some(digest.as_str()));
    let findings = ready(&manifest, &inventory, lock.as_bytes(), toolchain.as_bytes());
    assert_eq!(findings.iter().map(|finding| finding.code).collect::<Vec<_>>(), vec!["cohort-not-qualified"]);
}

#[test]
fn negative_controls_are_rejected() {
    let root = repo_root();
    let base = load_cohort(&root);
    let mut seen = 0_usize;

    for entry in fs::read_dir(root.join(INVALID_FIXTURES)).expect("invalid fixtures") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("fixture source");
        let fixture: Value = serde_json::from_str(&source).expect("fixture json");
        let expected: Vec<&str> = fixture["expected_findings"]
            .as_array()
            .expect("expected_findings")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        let mut manifest = base.clone();
        apply_overrides(&mut manifest, fixture["overrides"].as_array().expect("overrides"));

        let inventory = match fixture["inventory"].as_str() {
            Some(relative) => read(&root, relative),
            None => read(&root, INVENTORY_FIXTURE),
        };
        let mut codes: Vec<&str> = validate(&manifest).iter().map(|finding| finding.code).collect();
        codes.extend(validate_inventory(&manifest, &inventory).iter().map(|finding| finding.code));

        for code in &expected {
            assert!(codes.contains(code), "{} did not report {code}; reported {codes:?}", path.display());
            seen += 1;
        }
    }
    assert!(seen >= 8, "expected at least eight negative controls, saw {seen}");
}
