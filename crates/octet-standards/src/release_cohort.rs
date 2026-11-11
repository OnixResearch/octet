//! Release-cohort manifest validation for the public Octet publication channel.
//!
//! `standards/release/release-cohort.ncl` names one supported public channel and
//! the exact source, dependency, toolchain, package, notice, capability, and
//! acceptance-evidence identities that qualify it. The Nickel contracts in
//! `standards/release/contracts.ncl` reject malformed shapes. This module holds
//! the cross-field rules that a shape contract cannot express, and it re-checks
//! the exported JSON against the checked-in fixture bytes.
//!
//! The rules fail closed on the release mismatches the cohort must never hide:
//! an inventory bound to a different source revision, an omitted required
//! notice, imported content presented as cleared, a private input concealed by
//! a warm cache, a historical receipt presented as fresh, a receipt bound to
//! another source, changed evidence, or an unsupported profile presented as
//! verified.
//!
//! Validation is offline. Callers pass source-derived byte slices; this module
//! never reads the network, a Nix store, or a cache.

use serde_json::Value;

/// Schema version this module validates.
pub const SCHEMA_VERSION: &str = "octet-release-cohort/v1";

/// Notice files every qualified public export must carry.
///
/// Copying the root license alone does not satisfy these entries: they name the
/// attribution inventory and bundled third-party texts that a public tree needs
/// beside `LICENSE`.
pub const REQUIRED_NOTICE_PATHS: [&str; 3] = ["NOTICE", "THIRD_PARTY_NOTICES.md", "LICENSES/"];

/// Attribution rows the third-party notice inventory must keep.
pub const REQUIRED_ATTRIBUTION_IDS: [&str; 5] = [
    "dylint-support-sources",
    "clippy-match-def-path",
    "cairn-policy-provenance",
    "imported-standards-prose",
    "fetched-dependencies",
];

/// Package outputs a qualified cohort must publish.
pub const REQUIRED_PACKAGE_OUTPUTS: [&str; 9] = [
    "cargo-octet",
    "cargo-slotcar",
    "cargo-tigerstyle",
    "octet",
    "slotcar",
    "tigerstyle",
    "dylint-driver",
    "octet-standards",
    "octet-policy-artifacts",
];

/// Domain prefix of the acceptance-evidence identity string.
///
/// The input bytes are, in order: this prefix, a newline, the reviewed public
/// revision, a newline, the capability inventory bytes, a newline, the public
/// flake lock bytes, a newline, and the public `rust-toolchain.toml` bytes.
pub const EVIDENCE_DOMAIN: &str = "octet-release-evidence/v1";

/// One reason a release-cohort manifest is not release-ready.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    /// Stable rejection code.
    pub code: &'static str,
    /// Field-level detail naming the rejected value.
    pub detail: String,
}

impl Finding {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

/// Validate every cross-field rule of a release-cohort manifest.
///
/// Returns an empty vector when the manifest describes its source, inventory,
/// notices, dependencies, receipts, and profiles consistently. A returned
/// finding means the cohort must not be admitted for release, regardless of the
/// manifest's own `qualification.status`.
#[must_use]
pub fn validate(manifest: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_schema(manifest, &mut findings);
    check_source_binding(manifest, &mut findings);
    check_notices(manifest, &mut findings);
    check_dependencies(manifest, &mut findings);
    check_receipts(manifest, &mut findings);
    check_profiles(manifest, &mut findings);
    check_qualification(manifest, &mut findings);
    check_packages(manifest, &mut findings);
    check_inventory_shape(manifest, &mut findings);
    findings
}

/// Compare a cohort's exported inventory names with the inventory fixture.
///
/// The fixture is the public tree's own lint inventory file. Names are compared
/// in order after trimming blank lines, so an inventory bound to a newer or
/// older source revision cannot pass as the cohort's capability claim.
#[must_use]
pub fn validate_inventory(manifest: &Value, inventory: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let declared: Vec<&str> =
        rows(manifest, &["capability_inventory", "names"]).iter().filter_map(Value::as_str).collect();
    let observed: Vec<&str> = inventory.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    let declared_count = declared.len();
    let observed_count = observed.len();
    if declared != observed {
        findings.push(Finding::new(
            "inventory-fixture-mismatch",
            format!("cohort declares {declared_count} capability names but the inventory fixture has {observed_count}"),
        ));
    }
    findings
}

/// Recompute the acceptance-evidence identity for one cohort.
///
/// The digest binds the reviewed public revision to the inventory, flake lock,
/// and toolchain bytes that the cohort claims. Any changed input changes the
/// digest, which invalidates receipts carrying the previous value.
#[must_use]
pub fn evidence_digest(revision: &str, inventory: &[u8], flake_lock: &[u8], toolchain: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(EVIDENCE_DOMAIN.as_bytes());
    hasher.update(b"\n");
    hasher.update(revision.as_bytes());
    hasher.update(b"\n");
    hasher.update(inventory);
    hasher.update(b"\n");
    hasher.update(flake_lock);
    hasher.update(b"\n");
    hasher.update(toolchain);
    format!("b3:{}", hasher.finalize().to_hex())
}

/// Decide whether the cohort may be admitted as a release.
///
/// Release admission requires an empty finding set, a matching inventory and
/// evidence digest, and a manifest whose own qualification status is
/// `qualified`. A truthful `blocked` manifest validates without being admitted.
#[must_use]
pub fn release_ready(manifest: &Value, inventory: &str, flake_lock: &[u8], toolchain: &[u8]) -> Vec<Finding> {
    let mut findings = validate(manifest);
    findings.extend(validate_inventory(manifest, inventory));

    let revision = text(manifest, &["source", "public", "revision"]).unwrap_or_default();
    let observed = evidence_digest(revision, inventory.as_bytes(), flake_lock, toolchain);
    let declared = text(manifest, &["acceptance", "evidence_digest"]).unwrap_or_default();
    if observed != declared {
        findings.push(Finding::new(
            "evidence-digest-mismatch",
            format!("recomputed {observed} but acceptance declares {declared}"),
        ));
    }
    if text(manifest, &["qualification", "status"]) != Some("qualified") {
        findings.push(Finding::new("cohort-not-qualified", "manifest does not claim release qualification"));
    }
    findings
}

fn check_schema(manifest: &Value, findings: &mut Vec<Finding>) {
    if text(manifest, &["schema_version"]) != Some(SCHEMA_VERSION) {
        findings.push(Finding::new("schema-version", format!("expected schema_version {SCHEMA_VERSION}")));
    }
}

fn check_source_binding(manifest: &Value, findings: &mut Vec<Finding>) {
    let public = text(manifest, &["source", "public", "revision"]);
    let inventory = text(manifest, &["capability_inventory", "source_revision"]);
    if public.is_none() || inventory != public {
        findings.push(Finding::new(
            "inventory-source-mismatch",
            format!("inventory revision {inventory:?} does not name the public source revision {public:?}"),
        ));
    }

    let head = text(manifest, &["channel", "observed_branch_head"]);
    if flag(manifest, &["channel", "head_matches_reviewed_revision"]) && head != public {
        findings.push(Finding::new(
            "channel-head-mismatch",
            format!("channel head {head:?} does not name the reviewed revision {public:?}"),
        ));
    }

    let origin = text(manifest, &["source", "public", "derived_from_revision"]);
    let inventory_origin = text(manifest, &["capability_inventory", "origin_revision"]);
    if inventory_origin != origin {
        findings.push(Finding::new(
            "inventory-origin-mismatch",
            format!("inventory origin {inventory_origin:?} does not name the export origin {origin:?}"),
        ));
    }
}

fn check_notices(manifest: &Value, findings: &mut Vec<Finding>) {
    let required: Vec<&str> =
        rows(manifest, &["notices", "required"]).iter().filter_map(|row| text(row, &["path"])).collect();
    for path in REQUIRED_NOTICE_PATHS {
        if !required.contains(&path) {
            findings.push(Finding::new(
                "required-notice-omitted",
                format!("required public notice {path} is missing from notices.required"),
            ));
        }
    }

    let attributions: Vec<&str> =
        rows(manifest, &["notices", "attribution_ids"]).iter().filter_map(Value::as_str).collect();
    for id in REQUIRED_ATTRIBUTION_IDS {
        if !attributions.contains(&id) {
            findings.push(Finding::new(
                "required-notice-omitted",
                format!("attribution row {id} is missing from notices.attribution_ids"),
            ));
        }
    }

    let qualification = text(manifest, &["qualification", "status"]);
    for row in rows(manifest, &["notices", "imported_material"]) {
        let path = text(row, &["path"]).unwrap_or("<unnamed>");
        let resolution = text(row, &["resolution"]).unwrap_or_default();
        let included = flag(row, &["included_in_qualified_cohort"]);
        let evidence = text(row, &["permission_evidence"]).unwrap_or_default();

        if flag(row, &["root_license_relied_upon"]) {
            findings.push(Finding::new(
                "root-license-cannot-clear-imported-content",
                format!("{path} relies on the root license to clear imported material"),
            ));
        }
        if included && resolution != "permission_recorded" {
            findings.push(Finding::new(
                "imported-content-included-without-permission",
                format!("{path} is included in the qualified cohort with resolution {resolution}"),
            ));
        }
        if resolution == "permission_recorded" && evidence.trim().is_empty() {
            findings.push(Finding::new(
                "imported-content-included-without-permission",
                format!("{path} claims recorded permission without permission evidence"),
            ));
        }
        if resolution == "unresolved" && qualification == Some("qualified") {
            findings.push(Finding::new(
                "unresolved-imported-content-represented-as-cleared",
                format!("{path} is unresolved while the cohort claims qualification"),
            ));
        }
    }

    for row in rows(manifest, &["notices", "required"]) {
        let path = text(row, &["path"]).unwrap_or("<unnamed>");
        if !flag(row, &["present_in_public_tree"]) && qualification == Some("qualified") {
            findings.push(Finding::new(
                "notice-absent-with-qualified-claim",
                format!("{path} is absent from the public tree while the cohort claims qualification"),
            ));
        }
    }
}

fn check_dependencies(manifest: &Value, findings: &mut Vec<Finding>) {
    for row in rows(manifest, &["dependencies"]) {
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        if flag(row, &["warm_cache_dependency"]) {
            findings.push(Finding::new(
                "private-input-hidden-by-warm-cache",
                format!("{id} depends on a warm cache or pre-populated private input"),
            ));
        }
        if !flag(row, &["anonymous_at_review"]) {
            findings.push(Finding::new(
                "dependency-not-anonymous",
                format!("{id} was not retrievable anonymously at review time"),
            ));
        }
    }
}

fn check_receipts(manifest: &Value, findings: &mut Vec<Finding>) {
    let public = text(manifest, &["source", "public", "revision"]);
    let evidence = text(manifest, &["acceptance", "evidence_digest"]);

    for row in rows(manifest, &["acceptance", "receipts"]) {
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        if text(row, &["evidence_kind"]) != Some("fresh") {
            findings.push(Finding::new("receipt-not-fresh", format!("{id} is not a fresh acceptance receipt")));
        }
        if text(row, &["source_revision"]) != public {
            findings.push(Finding::new(
                "receipt-source-mismatch",
                format!("{id} binds a source other than the cohort's public revision"),
            ));
        }
        if text(row, &["evidence_digest"]) != evidence {
            findings.push(Finding::new(
                "evidence-digest-mismatch",
                format!("{id} carries an evidence digest other than the cohort's"),
            ));
        }
    }

    for row in rows(manifest, &["acceptance", "non_evidence"]) {
        let reference = text(row, &["reference"]).unwrap_or("<unnamed>");
        if text(row, &["evidence_role"]) != Some("non-evidence") {
            findings.push(Finding::new(
                "historical-receipt-labelled-evidence",
                format!("{reference} is historical and must be marked non-evidence"),
            ));
        }
    }
}

fn check_profiles(manifest: &Value, findings: &mut Vec<Finding>) {
    let fresh: Vec<&str> =
        rows(manifest, &["acceptance", "receipts"]).iter().filter_map(|row| text(row, &["id"])).collect();

    for row in rows(manifest, &["profiles"]) {
        if text(row, &["support_status"]) != Some("verified") {
            continue;
        }
        let id = text(row, &["id"]).unwrap_or("<unnamed>");
        let refs: Vec<&str> = rows(row, &["evidence_refs"]).iter().filter_map(Value::as_str).collect();
        let unsupported = refs.is_empty() || refs.iter().any(|reference| !fresh.contains(reference));
        if unsupported {
            findings.push(Finding::new(
                "unsupported-profile-claimed-verified",
                format!("{id} claims verified support without fresh cohort receipts"),
            ));
        }
    }
}

fn check_qualification(manifest: &Value, findings: &mut Vec<Finding>) {
    let status = text(manifest, &["qualification", "status"]);
    let reasons = rows(manifest, &["qualification", "blocking_reasons"]);
    if status == Some("qualified") && !reasons.is_empty() {
        findings.push(Finding::new(
            "qualified-with-blocking-reasons",
            "cohort claims qualification while blocking reasons remain",
        ));
    }
    if status == Some("blocked") && reasons.is_empty() {
        findings.push(Finding::new("blocked-without-reason", "cohort is blocked without a named blocking reason"));
    }
}

fn check_packages(manifest: &Value, findings: &mut Vec<Finding>) {
    let outputs: Vec<&str> = rows(manifest, &["packages"]).iter().filter_map(|row| text(row, &["output"])).collect();
    for output in REQUIRED_PACKAGE_OUTPUTS {
        if !outputs.contains(&output) {
            findings.push(Finding::new(
                "missing-package-output",
                format!("required package output {output} is not listed"),
            ));
        }
    }
}

fn check_inventory_shape(manifest: &Value, findings: &mut Vec<Finding>) {
    let declared = at(manifest, &["capability_inventory", "count"]).and_then(Value::as_u64);
    let names = rows(manifest, &["capability_inventory", "names"]).len();
    let excluded: Vec<&str> = rows(manifest, &["capability_inventory", "excluded_from_public"])
        .iter()
        .filter_map(|row| text(row, &["name"]))
        .collect();
    let development = at(manifest, &["capability_inventory", "development", "count"]).and_then(Value::as_u64);

    if declared != Some(u64_from_len(names)) {
        findings.push(Finding::new(
            "inventory-count-mismatch",
            format!("count {declared:?} does not equal {names} declared names"),
        ));
    }
    let expected_development = Some(u64_from_len(names) + u64_from_len(excluded.len()));
    if development != expected_development {
        findings.push(Finding::new(
            "inventory-count-mismatch",
            format!("development count {development:?} is not the public count plus reviewed exclusions"),
        ));
    }
    for name in &excluded {
        if rows(manifest, &["capability_inventory", "names"]).iter().any(|value| value.as_str() == Some(*name)) {
            findings.push(Finding::new(
                "inventory-excluded-name-present",
                format!("excluded capability {name} is also listed as public"),
            ));
        }
    }
}

fn u64_from_len(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = match key.parse::<usize>() {
            Ok(index) => current.get(index)?,
            Err(_) => current.get(*key)?,
        };
    }
    Some(current)
}

fn text<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    at(value, path)?.as_str()
}

fn flag(value: &Value, path: &[&str]) -> bool {
    at(value, path).and_then(Value::as_bool).unwrap_or(false)
}

fn rows<'a>(value: &'a Value, path: &[&str]) -> &'a [Value] {
    at(value, path).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use serde_json::Value;

    use super::evidence_digest;
    use super::release_ready;
    use super::validate;
    use super::validate_inventory;

    const COHORT_JSON: &str = "standards/release/generated/release-cohort.json";
    const INVENTORY_FIXTURE: &str = "tests/fixtures/release/public-lint-inventory-names.txt";
    const FLAKE_LOCK_FIXTURE: &str = "tests/fixtures/release/public-flake.lock";
    const TOOLCHAIN_FIXTURE: &str = "tests/fixtures/release/public-rust-toolchain.toml";
    const INVALID_FIXTURES: &str = "standards/release/fixtures/invalid";
    const RELEASE_CANDIDATE_DOC: &str = "docs/release-candidate.md";
    const DOWNSTREAM_ROLLOUT_DOC: &str = "docs/downstream-rollout.md";

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
    fn checked_in_cohort_validates_and_is_qualified() {
        let root = repo_root();
        let manifest = load_cohort(&root);
        let inventory = read(&root, INVENTORY_FIXTURE);
        let lock = read(&root, FLAKE_LOCK_FIXTURE);
        let toolchain = read(&root, TOOLCHAIN_FIXTURE);

        assert_eq!(validate(&manifest), Vec::new());
        assert_eq!(validate_inventory(&manifest, &inventory), Vec::new());

        let revision = manifest["source"]["public"]["revision"].as_str().expect("revision");
        let digest = evidence_digest(revision, inventory.as_bytes(), lock.as_bytes(), toolchain.as_bytes());
        assert_eq!(manifest["acceptance"]["evidence_digest"].as_str(), Some(digest.as_str()));
        assert_eq!(release_ready(&manifest, &inventory, lock.as_bytes(), toolchain.as_bytes()), Vec::new());

        let status = manifest["qualification"]["status"].as_str().expect("status");
        assert_eq!(status, "qualified");
        assert!(manifest["qualification"]["blocking_reasons"].as_array().expect("reasons").is_empty());
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

    #[test]
    fn documented_claims_match_the_public_inventory() {
        let root = repo_root();
        let manifest = load_cohort(&root);
        let candidate = read(&root, RELEASE_CANDIDATE_DOC);
        let rollout = read(&root, DOWNSTREAM_ROLLOUT_DOC);

        let cohort_id = manifest["cohort_id"].as_str().expect("cohort id");
        let flake_ref = manifest["channel"]["flake_ref"].as_str().expect("flake ref");
        let count = manifest["capability_inventory"]["count"].as_u64().expect("count").to_string();
        let claim = format!("{count} Dylint lints");

        for document in [&candidate, &rollout] {
            assert!(document.contains(cohort_id), "documentation misses {cohort_id}");
            assert!(document.contains(flake_ref), "documentation misses {flake_ref}");
        }
        assert!(candidate.contains(&claim), "release candidate misses {claim}");
        for excluded in manifest["capability_inventory"]["excluded_from_public"].as_array().expect("excluded") {
            let name = excluded["name"].as_str().expect("excluded name");
            assert!(candidate.contains(name), "release candidate misses exclusion {name}");
        }
    }
}
