use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub const IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION: &str = "octet-imported-proof-bundles/v1";
pub const HASH_ALGORITHM_BLAKE3: &str = "blake3";
pub const EVIDENCE_ORIGIN_EXTERNAL_TRELLIS: &str = "external_trellis_import";
pub const EVIDENCE_TIER_MANUAL_REVIEW: &str = "manual_review_required";
pub const EVIDENCE_TIER_FORMAL_PROOF: &str = "formal_proof";
const BLAKE3_HEX_LEN: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedProofBundleManifest {
    pub bundles: Vec<ImportedProofBundle>,
    pub generated_from: String,
    pub schema_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedProofBundle {
    pub bundle_id: String,
    pub evidence_origin: String,
    pub hash_algorithm: String,
    pub intended_evidence_tier: String,
    pub promotion_command_argv: Vec<String>,
    pub proof_artifact: ImportedArtifactIdentity,
    pub proof_path: String,
    pub receipt_artifacts: Vec<ImportedArtifactIdentity>,
    pub repository_label: String,
    pub repository_revision: String,
    pub source_artifact: ImportedArtifactIdentity,
    pub source_path: String,
    pub stderr_artifacts: Vec<ImportedArtifactIdentity>,
    pub stdout_artifacts: Vec<ImportedArtifactIdentity>,
    pub trust_boundary_notes: Vec<String>,
    pub verifier_command_argv: Vec<String>,
    pub verifier_identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedArtifactIdentity {
    pub blake3: String,
    pub bytes: u64,
    pub path: String,
}

pub struct ImportedProofArtifactBytes<'a> {
    pub source: &'a [u8],
    pub proof: &'a [u8],
    pub stdout: Vec<&'a [u8]>,
    pub stderr: Vec<&'a [u8]>,
    pub receipts: Vec<&'a [u8]>,
}

pub struct OwnedImportedProofArtifactBytes {
    pub source: Vec<u8>,
    pub proof: Vec<u8>,
    pub stdout: Vec<Vec<u8>>,
    pub stderr: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
}

impl OwnedImportedProofArtifactBytes {
    #[must_use]
    pub fn as_borrowed(&self) -> ImportedProofArtifactBytes<'_> {
        ImportedProofArtifactBytes {
            source: &self.source,
            proof: &self.proof,
            stdout: self.stdout.iter().map(Vec::as_slice).collect(),
            stderr: self.stderr.iter().map(Vec::as_slice).collect(),
            receipts: self.receipts.iter().map(Vec::as_slice).collect(),
        }
    }
}

#[must_use]
pub fn validate_imported_proof_manifest_shape(manifest: &ImportedProofBundleManifest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported imported proof bundle schema_version `{}`; expected `{IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION}`",
            manifest.schema_version
        ));
    }
    if manifest.generated_from.trim().is_empty() {
        diagnostics.push("imported proof bundle manifest missing generated_from".to_owned());
    }
    diagnostics.extend(duplicate_bundle_diagnostics(&manifest.bundles));
    for bundle in &manifest.bundles {
        diagnostics.extend(validate_imported_proof_bundle_shape(bundle));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[must_use]
pub fn validate_imported_proof_bundle_shape(bundle: &ImportedProofBundle) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required(&mut diagnostics, "bundle_id", &bundle.bundle_id);
    push_required(&mut diagnostics, "repository_label", &bundle.repository_label);
    push_required(&mut diagnostics, "repository_revision", &bundle.repository_revision);
    push_required(&mut diagnostics, "source_path", &bundle.source_path);
    push_required(&mut diagnostics, "proof_path", &bundle.proof_path);
    push_required(&mut diagnostics, "verifier_identity", &bundle.verifier_identity);
    push_required_collection(&mut diagnostics, "verifier_command_argv", &bundle.verifier_command_argv);
    push_required_collection(&mut diagnostics, "promotion_command_argv", &bundle.promotion_command_argv);
    push_required_collection(&mut diagnostics, "trust_boundary_notes", &bundle.trust_boundary_notes);

    if bundle.hash_algorithm != HASH_ALGORITHM_BLAKE3 {
        diagnostics.push(format!(
            "imported proof bundle `{}` uses unsupported hash_algorithm `{}`",
            bundle.bundle_id, bundle.hash_algorithm
        ));
    }
    if bundle.evidence_origin != EVIDENCE_ORIGIN_EXTERNAL_TRELLIS {
        diagnostics.push(format!(
            "imported proof bundle `{}` uses unsupported evidence_origin `{}`",
            bundle.bundle_id, bundle.evidence_origin
        ));
    }
    if bundle.intended_evidence_tier != EVIDENCE_TIER_MANUAL_REVIEW {
        diagnostics.push(format!(
            "imported proof bundle `{}` must stay `{EVIDENCE_TIER_MANUAL_REVIEW}`, got `{}`",
            bundle.bundle_id, bundle.intended_evidence_tier
        ));
    }
    if bundle.intended_evidence_tier == EVIDENCE_TIER_FORMAL_PROOF {
        diagnostics.push(format!(
            "imported proof bundle `{}` cannot claim formal_proof without Octet rerun receipt",
            bundle.bundle_id
        ));
    }
    diagnostics.extend(validate_imported_artifact_identity("source_artifact", &bundle.source_artifact));
    diagnostics.extend(validate_imported_artifact_identity("proof_artifact", &bundle.proof_artifact));
    for artifact in &bundle.stdout_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("stdout_artifacts", artifact));
    }
    for artifact in &bundle.stderr_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("stderr_artifacts", artifact));
    }
    for artifact in &bundle.receipt_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("receipt_artifacts", artifact));
    }
    diagnostics.extend(validate_imported_non_claims(bundle));
    diagnostics.extend(validate_imported_promotion_command(bundle));
    diagnostics
}

#[must_use]
pub fn validate_imported_proof_bundle_artifacts(
    bundle: &ImportedProofBundle,
    artifacts: &ImportedProofArtifactBytes<'_>,
) -> Vec<String> {
    let mut diagnostics = validate_imported_proof_bundle_shape(bundle);
    diagnostics.extend(validate_artifact_bytes("source_artifact", &bundle.source_artifact, artifacts.source));
    diagnostics.extend(validate_artifact_bytes("proof_artifact", &bundle.proof_artifact, artifacts.proof));
    diagnostics.extend(validate_artifact_byte_list("stdout_artifacts", &bundle.stdout_artifacts, &artifacts.stdout));
    diagnostics.extend(validate_artifact_byte_list("stderr_artifacts", &bundle.stderr_artifacts, &artifacts.stderr));
    diagnostics.extend(validate_artifact_byte_list(
        "receipt_artifacts",
        &bundle.receipt_artifacts,
        &artifacts.receipts,
    ));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub fn validate_imported_proof_manifest_files(
    root: &Path,
    manifest: &ImportedProofBundleManifest,
) -> Result<Vec<String>, String> {
    let shape_diagnostics = validate_imported_proof_manifest_shape(manifest);
    if !shape_diagnostics.is_empty() {
        return Ok(shape_diagnostics);
    }
    let owned_artifacts = manifest
        .bundles
        .iter()
        .map(|bundle| read_imported_proof_bundle_artifacts(root, bundle))
        .collect::<Result<Vec<_>, _>>()?;
    let borrowed_artifacts =
        owned_artifacts.iter().map(OwnedImportedProofArtifactBytes::as_borrowed).collect::<Vec<_>>();
    Ok(validate_imported_proof_manifest_artifacts(manifest, &borrowed_artifacts))
}

#[must_use]
pub fn validate_imported_proof_manifest_artifacts(
    manifest: &ImportedProofBundleManifest,
    artifacts: &[ImportedProofArtifactBytes<'_>],
) -> Vec<String> {
    let mut diagnostics = validate_imported_proof_manifest_shape(manifest);
    if manifest.bundles.len() != artifacts.len() {
        diagnostics.push(format!(
            "imported proof artifact byte set count {} does not match bundle count {}",
            artifacts.len(),
            manifest.bundles.len()
        ));
    }
    for (bundle, bytes) in manifest.bundles.iter().zip(artifacts.iter()) {
        diagnostics.extend(validate_imported_proof_bundle_artifacts(bundle, bytes));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn read_imported_proof_bundle_artifacts(
    root: &Path,
    bundle: &ImportedProofBundle,
) -> Result<OwnedImportedProofArtifactBytes, String> {
    Ok(OwnedImportedProofArtifactBytes {
        source: read_artifact_bytes(root, &bundle.source_artifact)?,
        proof: read_artifact_bytes(root, &bundle.proof_artifact)?,
        stdout: read_artifact_bytes_list(root, &bundle.stdout_artifacts)?,
        stderr: read_artifact_bytes_list(root, &bundle.stderr_artifacts)?,
        receipts: read_artifact_bytes_list(root, &bundle.receipt_artifacts)?,
    })
}

fn read_artifact_bytes_list(root: &Path, artifacts: &[ImportedArtifactIdentity]) -> Result<Vec<Vec<u8>>, String> {
    artifacts.iter().map(|artifact| read_artifact_bytes(root, artifact)).collect::<Result<Vec<_>, _>>()
}

fn read_artifact_bytes(root: &Path, artifact: &ImportedArtifactIdentity) -> Result<Vec<u8>, String> {
    let path = root.join(&artifact.path);
    fs::read(&path).map_err(|error| format!("read imported proof artifact `{}`: {error}", artifact.path))
}

#[must_use]
pub fn bundle_artifacts(bundle: &ImportedProofBundle) -> Vec<ImportedArtifactIdentity> {
    let mut artifacts = vec![bundle.source_artifact.clone(), bundle.proof_artifact.clone()];
    artifacts.extend(bundle.stdout_artifacts.clone());
    artifacts.extend(bundle.stderr_artifacts.clone());
    artifacts.extend(bundle.receipt_artifacts.clone());
    artifacts
}

fn duplicate_bundle_diagnostics(bundles: &[ImportedProofBundle]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut bundle_ids = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for bundle in bundles {
        if !bundle_ids.insert(bundle.bundle_id.clone()) {
            diagnostics.push(format!("duplicate imported proof bundle_id `{}`", bundle.bundle_id));
        }
        let local_identities = bundle_artifacts(bundle)
            .into_iter()
            .map(|artifact| format!("{}:{}", artifact.path, artifact.blake3))
            .collect::<BTreeSet<_>>();
        for identity in local_identities {
            if !identities.insert(identity.clone()) {
                diagnostics.push(format!("duplicate imported proof artifact identity `{identity}`"));
            }
        }
    }
    diagnostics
}

fn validate_artifact_byte_list(
    label: &str,
    identities: &[ImportedArtifactIdentity],
    byte_sets: &[&[u8]],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if identities.len() != byte_sets.len() {
        diagnostics.push(format!(
            "imported proof `{label}` byte set count {} does not match artifact count {}",
            byte_sets.len(),
            identities.len()
        ));
    }
    for (identity, bytes) in identities.iter().zip(byte_sets.iter()) {
        diagnostics.extend(validate_artifact_bytes(label, identity, bytes));
    }
    diagnostics
}

fn validate_artifact_bytes(label: &str, identity: &ImportedArtifactIdentity, bytes: &[u8]) -> Vec<String> {
    let mut diagnostics = validate_imported_artifact_identity(label, identity);
    let actual_bytes = u64::try_from(bytes.len()).expect("artifact bytes length should fit u64");
    if identity.bytes != actual_bytes {
        diagnostics.push(format!(
            "imported proof artifact `{label}` byte length mismatch for `{}`; expected {}, got {actual_bytes}",
            identity.path, identity.bytes
        ));
    }
    let actual_blake3 = blake3::hash(bytes).to_hex().to_string();
    if is_blake3_hex(&identity.blake3) && identity.blake3 != actual_blake3 {
        diagnostics.push(format!(
            "imported proof artifact `{label}` hash mismatch for `{}`; expected `{}`, got `{actual_blake3}`",
            identity.path, identity.blake3
        ));
    }
    diagnostics
}

fn validate_imported_artifact_identity(label: &str, identity: &ImportedArtifactIdentity) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required(&mut diagnostics, &format!("{label}.path"), &identity.path);
    push_required(&mut diagnostics, &format!("{label}.blake3"), &identity.blake3);
    if !is_normalized_relative_path(&identity.path) {
        diagnostics
            .push(format!("imported proof artifact `{label}` path `{}` is not normalized relative", identity.path));
    }
    if !is_blake3_hex(&identity.blake3) {
        diagnostics
            .push(format!("imported proof artifact `{label}` has malformed BLAKE3 digest `{}`", identity.blake3));
    }
    diagnostics
}

fn validate_imported_non_claims(bundle: &ImportedProofBundle) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let joined = bundle.trust_boundary_notes.join(" ").to_lowercase();
    for required in [
        "blake3 identifies exact artifact bytes",
        "does not by itself prove proof correctness",
        "octet formal_proof requires rerunning",
    ] {
        if !joined.contains(required) {
            diagnostics
                .push(format!("imported proof bundle `{}` missing trust-boundary note `{required}`", bundle.bundle_id));
        }
    }
    for forbidden in [
        "proves whole-program correctness",
        "provides certification",
        "provides business-domain validation",
        "proves semantic equivalence",
        "guarantees full safety",
    ] {
        if joined.contains(forbidden) {
            diagnostics
                .push(format!("imported proof bundle `{}` contains unscoped claim `{forbidden}`", bundle.bundle_id));
        }
    }
    diagnostics
}

fn validate_imported_promotion_command(bundle: &ImportedProofBundle) -> Vec<String> {
    if bundle.promotion_command_argv.iter().any(|part| part == "octet")
        && bundle.promotion_command_argv.iter().any(|part| part == "prove")
    {
        Vec::new()
    } else {
        vec![format!(
            "imported proof bundle `{}` missing Octet prove promotion command",
            bundle.bundle_id
        )]
    }
}

fn push_required(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("imported proof bundle missing required field `{field}`"));
    }
}

fn push_required_collection(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("imported proof bundle missing required field `{field}`"));
    }
}

fn is_normalized_relative_path(path: &str) -> bool {
    !path.trim().is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|component| component == ".." || component.is_empty())
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const GENERATED_IMPORTS: &str = include_str!("../../standards/proof-imports/generated/trellis-proof-imports.json");
    const SOURCE_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference_source.rs");
    const PROOF_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference_proof.rs");
    const STDOUT_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.stdout");
    const STDERR_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.stderr");
    const RECEIPT_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.receipt.json");

    #[test]
    fn generated_trellis_import_fixture_validates() {
        let manifest = generated_manifest();
        let artifacts = [fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        let bundle = manifest.bundles.first().expect("bundle fixture");
        assert_eq!(bundle.repository_label, "../trellis");
        assert_eq!(bundle.hash_algorithm, HASH_ALGORITHM_BLAKE3);
        assert_eq!(bundle.intended_evidence_tier, EVIDENCE_TIER_MANUAL_REVIEW);
        assert_ne!(bundle.intended_evidence_tier, EVIDENCE_TIER_FORMAL_PROOF);
        assert!(bundle.promotion_command_argv.iter().any(|part| part == "prove"));
    }

    #[test]
    fn generated_trellis_import_json_is_deterministic() {
        let manifest = generated_manifest();
        let rendered = serde_json::to_value(&manifest).expect("render manifest value");
        let generated = serde_json::from_str::<serde_json::Value>(GENERATED_IMPORTS).expect("parse generated value");

        assert_eq!(rendered, generated);
        assert!(GENERATED_IMPORTS.contains("\"schema_version\": \"octet-imported-proof-bundles/v1\""));
    }

    #[test]
    fn trellis_import_file_shell_reads_fixture_artifacts() {
        let manifest = generated_manifest();

        let diagnostics =
            validate_imported_proof_manifest_files(&repo_root(), &manifest).expect("read fixture artifacts");

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn trellis_import_validation_rejects_stale_hash_and_formal_overclaim() {
        let mut manifest = generated_manifest();
        let bundle = manifest.bundles.first_mut().expect("bundle fixture");
        bundle.source_artifact.blake3 = "0".repeat(BLAKE3_HEX_LEN);
        bundle.intended_evidence_tier = EVIDENCE_TIER_FORMAL_PROOF.to_owned();
        bundle.trust_boundary_notes = vec!["provides certification".to_owned()];
        let artifacts = [fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("cannot claim formal_proof")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("provides certification")));
    }

    #[test]
    fn trellis_import_validation_rejects_missing_malformed_and_duplicate_fields() {
        let mut manifest = generated_manifest();
        let duplicate = manifest.bundles.first().expect("bundle fixture").clone();
        let bundle = manifest.bundles.first_mut().expect("bundle fixture");
        bundle.repository_revision.clear();
        bundle.hash_algorithm = "sha256".to_owned();
        bundle.proof_artifact.path = "../bad".to_owned();
        bundle.proof_artifact.blake3 = "bad-digest".to_owned();
        manifest.bundles.push(duplicate);
        let artifacts = [fixture_bytes(), fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("repository_revision")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported hash_algorithm")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed BLAKE3")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("not normalized relative")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate imported proof bundle_id")));
    }

    fn generated_manifest() -> ImportedProofBundleManifest {
        serde_json::from_str(GENERATED_IMPORTS).expect("generated imports JSON")
    }

    fn fixture_bytes() -> ImportedProofArtifactBytes<'static> {
        ImportedProofArtifactBytes {
            source: SOURCE_BYTES,
            proof: PROOF_BYTES,
            stdout: vec![STDOUT_BYTES],
            stderr: vec![STDERR_BYTES],
            receipts: vec![RECEIPT_BYTES],
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-octet has workspace parent")
            .to_path_buf()
    }
}
