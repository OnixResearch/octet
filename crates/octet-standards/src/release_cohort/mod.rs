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

mod validation;

#[cfg(test)]
mod tests;

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
    validation::check_schema(manifest, &mut findings);
    validation::check_source_binding(manifest, &mut findings);
    validation::check_notices(manifest, &mut findings);
    validation::check_dependencies(manifest, &mut findings);
    validation::check_receipts(manifest, &mut findings);
    validation::check_profiles(manifest, &mut findings);
    validation::check_qualification(manifest, &mut findings);
    validation::check_packages(manifest, &mut findings);
    validation::check_inventory_shape(manifest, &mut findings);
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
    let declared: Vec<&str> = validation::rows(manifest, &["capability_inventory", "names"])
        .iter()
        .filter_map(Value::as_str)
        .collect();
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
pub fn ready(manifest: &Value, inventory: &str, flake_lock: &[u8], toolchain: &[u8]) -> Vec<Finding> {
    let mut findings = validate(manifest);
    findings.extend(validate_inventory(manifest, inventory));

    let revision = validation::text(manifest, &["source", "public", "revision"]).unwrap_or_default();
    let observed = evidence_digest(revision, inventory.as_bytes(), flake_lock, toolchain);
    let declared = validation::text(manifest, &["acceptance", "evidence_digest"]).unwrap_or_default();
    if observed != declared {
        findings.push(Finding::new(
            "evidence-digest-mismatch",
            format!("recomputed {observed} but acceptance declares {declared}"),
        ));
    }
    if validation::text(manifest, &["qualification", "status"]) != Some("qualified") {
        findings.push(Finding::new("cohort-not-qualified", "manifest does not claim release qualification"));
    }
    findings
}
