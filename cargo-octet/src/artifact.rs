use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cli::ArtifactArgs;
use crate::cli::OutputFormat;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;

const REPORT_SCHEMA_VERSION: u32 = 1;
const EXIT_VALID: i32 = 0;
const EXIT_INVALID: i32 = 1;
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const JSON_RESULTS_ARTIFACT_NAME: &str = "results.jsonl";
const SARIF_RESULTS_ARTIFACT_NAME: &str = "results.sarif";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const PROOF_CARDS_ARTIFACT_NAME: &str = "proof-cards.json";
const RUNNER_PROVENANCE_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const RUNNER_STATIC_FINDINGS_SOURCE_ID: &str = "octet.runner.static-findings";
const RUNNER_CHECKER_ID: &str = "cargo-octet";
const STATUS_VALID: &str = "valid";
const STATUS_INVALID: &str = "invalid";
const CHECKED_SCOPE_PREFIX: &str = "profile:";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_EVIDENCE_TIER: &str = "evidence_tier";
const FIELD_RECORD: &str = "record";
const STATUS_BUNDLE_KIND_PROVE: &str = "prove";
const PROOF_ARTIFACT_PREFIX: &str = "proofs/";
const PROOF_SOURCE_ID: &str = "octet.standards.formal-proof";
const CHECKER_OCTET_STANDARDS: &str = "octet-standards";

#[derive(Debug, Clone)]
pub struct BundleFile {
    pub name: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ArtifactBundleInput {
    pub artifact_dir: String,
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct VerifiedArtifactRow {
    pub artifact: String,
    pub source_id: String,
    pub checker_id: String,
    pub evidence_tier: String,
    pub blake3: String,
    pub checked_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_action: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ArtifactVerificationReport {
    pub schema_version: u32,
    pub artifact_dir: String,
    pub output_format: String,
    pub status: &'static str,
    pub artifacts: Vec<VerifiedArtifactRow>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BundleStatus {
    output_format: Option<OutputFormat>,
    bundle_kind: BundleKind,
    checked_scope: Option<String>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BundleKind {
    Check,
    Prove,
}

#[derive(Clone, Copy)]
struct ExpectedRecordShape {
    source_id: &'static str,
    checker_id: &'static str,
    evidence_tier: EvidenceTier,
}

pub fn run(args: ArtifactArgs) -> Result<i32, String> {
    if args.output_format == OutputFormat::Sarif {
        return Err("artifact verification does not support SARIF output; use human or json".to_owned());
    }

    let files = read_bundle_files(&args.artifact_dir)?;
    let report = verify_bundle(ArtifactBundleInput {
        artifact_dir: args.artifact_dir.display().to_string(),
        files,
    });

    match args.output_format {
        OutputFormat::Human => print_human_report(&report),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize artifact verification report: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF rejected before rendering"),
    }

    Ok(if report.diagnostics.is_empty() {
        EXIT_VALID
    } else {
        EXIT_INVALID
    })
}

pub fn verify_bundle(input: ArtifactBundleInput) -> ArtifactVerificationReport {
    let file_map = collect_file_map(input.files);
    let status = inspect_status(&file_map);
    let output_format = status.output_format.unwrap_or_else(|| infer_output_format(&file_map));
    let base_expected_files = expected_artifact_files(output_format, status.bundle_kind);
    let mut diagnostics = status.diagnostics;

    let provenance_records = parse_provenance_records(&file_map, &mut diagnostics);
    let expected_files = expected_files_with_proofs(&base_expected_files, &provenance_records, status.bundle_kind);
    let expected_non_provenance = expected_files
        .iter()
        .map(String::as_str)
        .filter(|name| *name != PROVENANCE_ARTIFACT_NAME)
        .collect::<Vec<_>>();

    validate_file_set(&file_map, &expected_files, &mut diagnostics);

    let expected_hashes = expected_artifact_hashes(&file_map, &expected_non_provenance);
    validate_provenance_records(
        &provenance_records,
        &expected_hashes,
        &expected_non_provenance,
        status.checked_scope.as_deref(),
        output_format,
        status.bundle_kind,
        &mut diagnostics,
    );

    let artifacts = verified_artifact_rows(&provenance_records, &expected_non_provenance);
    diagnostics.sort();
    diagnostics.dedup();

    ArtifactVerificationReport {
        schema_version: REPORT_SCHEMA_VERSION,
        artifact_dir: input.artifact_dir,
        output_format: output_format.as_str().to_owned(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        },
        artifacts,
        diagnostics,
    }
}

fn read_bundle_files(artifact_dir: &Path) -> Result<Vec<BundleFile>, String> {
    read_bundle_files_from_directory(artifact_dir, artifact_dir)
}

fn read_bundle_files_from_directory(root: &Path, directory: &Path) -> Result<Vec<BundleFile>, String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("read artifact directory {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read artifact directory {} entry: {error}", directory.display()))?;
    entries.sort_by_key(std::fs::DirEntry::path);

    let mut files = Vec::new();
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            files.extend(read_bundle_files_from_directory(root, &path)?);
            continue;
        }
        let name = path
            .strip_prefix(root)
            .map_err(|error| format!("derive relative artifact path for {}: {error}", path.display()))?
            .to_string_lossy()
            .into_owned();
        let contents = fs::read(&path).map_err(|error| format!("read artifact {}: {error}", path.display()))?;
        files.push(BundleFile { name, contents });
    }
    Ok(files)
}

fn print_human_report(report: &ArtifactVerificationReport) {
    println!(
        "octet artifact verification artifact_dir={} output_format={} status={}",
        report.artifact_dir, report.output_format, report.status
    );
    for artifact in &report.artifacts {
        let manual_action = artifact.manual_action.as_deref().unwrap_or("");
        println!(
            "artifact={} source_id={} checker_id={} evidence_tier={} blake3={} checked_scope={} manual_action={}",
            artifact.artifact,
            artifact.source_id,
            artifact.checker_id,
            artifact.evidence_tier,
            artifact.blake3,
            artifact.checked_scope,
            manual_action
        );
    }
    for diagnostic in &report.diagnostics {
        println!("diagnostic: {diagnostic}");
    }
}

fn collect_file_map(files: Vec<BundleFile>) -> BTreeMap<String, Vec<u8>> {
    let mut file_map = BTreeMap::new();
    for file in files {
        file_map.insert(file.name, file.contents);
    }
    file_map
}

fn inspect_status(file_map: &BTreeMap<String, Vec<u8>>) -> BundleStatus {
    let Some(status_bytes) = file_map.get(STATUS_ARTIFACT_NAME) else {
        return BundleStatus {
            output_format: None,
            bundle_kind: BundleKind::Check,
            checked_scope: None,
            diagnostics: vec![format!("missing expected artifact `{STATUS_ARTIFACT_NAME}`")],
        };
    };

    let status_json: serde_json::Value = match serde_json::from_slice(status_bytes) {
        Ok(value) => value,
        Err(error) => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("malformed {STATUS_ARTIFACT_NAME}: {error}")],
            };
        }
    };

    let output_format = match status_json.get("output_format").and_then(serde_json::Value::as_str) {
        Some("human") => Some(OutputFormat::Human),
        Some("json") => Some(OutputFormat::Json),
        Some("sarif") => Some(OutputFormat::Sarif),
        Some(other) => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("unsupported {STATUS_ARTIFACT_NAME} output_format `{other}`")],
            };
        }
        None => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("missing {STATUS_ARTIFACT_NAME} output_format")],
            };
        }
    };

    let bundle_kind = match status_json.get("bundle_kind").and_then(serde_json::Value::as_str) {
        Some(STATUS_BUNDLE_KIND_PROVE) => BundleKind::Prove,
        Some(_) | None => BundleKind::Check,
    };

    let checked_scope = status_json
        .get("metadata")
        .and_then(|metadata| metadata.get("profile_name"))
        .and_then(serde_json::Value::as_str)
        .filter(|profile| !profile.trim().is_empty())
        .map(|profile| format!("{CHECKED_SCOPE_PREFIX}{profile}"));

    let diagnostics = if checked_scope.is_some() {
        Vec::new()
    } else {
        vec![format!("missing {STATUS_ARTIFACT_NAME} metadata.profile_name")]
    };

    BundleStatus {
        output_format,
        bundle_kind,
        checked_scope,
        diagnostics,
    }
}

fn infer_output_format(file_map: &BTreeMap<String, Vec<u8>>) -> OutputFormat {
    if file_map.contains_key(SARIF_RESULTS_ARTIFACT_NAME) {
        OutputFormat::Sarif
    } else if file_map.contains_key(JSON_RESULTS_ARTIFACT_NAME) {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}

fn expected_artifact_files(output_format: OutputFormat, bundle_kind: BundleKind) -> Vec<String> {
    let mut files = vec![
        COMMAND_ARTIFACT_NAME.to_owned(),
        PROVENANCE_ARTIFACT_NAME.to_owned(),
        STATUS_ARTIFACT_NAME.to_owned(),
        SUMMARY_ARTIFACT_NAME.to_owned(),
    ];
    if bundle_kind == BundleKind::Check {
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => files.push(JSON_RESULTS_ARTIFACT_NAME.to_owned()),
            OutputFormat::Sarif => files.push(SARIF_RESULTS_ARTIFACT_NAME.to_owned()),
        }
    } else {
        files.push(PROOF_CARDS_ARTIFACT_NAME.to_owned());
    }
    files.sort();
    files
}

fn expected_files_with_proofs(
    base_expected_files: &[String],
    records: &[ProvenanceRecord],
    bundle_kind: BundleKind,
) -> Vec<String> {
    let mut files = base_expected_files.to_vec();
    if bundle_kind == BundleKind::Prove {
        files.extend(records.iter().filter_map(expected_proof_artifact));
    }
    files.sort();
    files.dedup();
    files
}

fn expected_proof_artifact(record: &ProvenanceRecord) -> Option<String> {
    (record.evidence_tier == EvidenceTier::FormalProof && record.artifact_locator.starts_with(PROOF_ARTIFACT_PREFIX))
        .then(|| record.artifact_locator.clone())
}

fn validate_file_set(file_map: &BTreeMap<String, Vec<u8>>, expected_files: &[String], diagnostics: &mut Vec<String>) {
    for expected in expected_files {
        if !file_map.contains_key(expected) {
            diagnostics.push(format!("missing expected artifact `{expected}`"));
        }
    }

    for actual in file_map.keys() {
        if !expected_files.iter().any(|expected| expected == actual) {
            diagnostics.push(format!("unexpected artifact `{actual}`"));
        }
    }
}

fn parse_provenance_records(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Vec<ProvenanceRecord> {
    let Some(provenance_bytes) = file_map.get(PROVENANCE_ARTIFACT_NAME) else {
        return Vec::new();
    };
    let provenance_text = String::from_utf8_lossy(provenance_bytes);
    let mut records = Vec::new();
    for (offset, line) in provenance_text.lines().enumerate() {
        let line_number = offset + 1;
        match serde_json::from_str::<ProvenanceRecord>(line) {
            Ok(record) => records.push(record),
            Err(error) => diagnostics.push(format!(
                "malformed provenance line {line_number} field `{}`: {error}",
                malformed_provenance_field(line)
            )),
        }
    }
    records
}

fn malformed_provenance_field(line: &str) -> &'static str {
    if line.contains(FIELD_EVIDENCE_TIER) {
        FIELD_EVIDENCE_TIER
    } else {
        FIELD_RECORD
    }
}

fn expected_artifact_hashes(
    file_map: &BTreeMap<String, Vec<u8>>,
    expected_non_provenance: &[&str],
) -> BTreeMap<String, String> {
    let mut hashes = BTreeMap::new();
    for artifact in expected_non_provenance {
        if let Some(bytes) = file_map.get(*artifact) {
            hashes.insert((*artifact).to_owned(), crate::evidence::artifact_blake3_hex(bytes));
        }
    }
    hashes
}

fn validate_provenance_records(
    records: &[ProvenanceRecord],
    expected_hashes: &BTreeMap<String, String>,
    expected_non_provenance: &[&str],
    expected_checked_scope: Option<&str>,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.extend(crate::evidence::validate_provenance_chain(records, expected_hashes));
    for record in records {
        validate_expected_record_shape(record, output_format, bundle_kind, expected_checked_scope, diagnostics);
    }
    validate_exact_provenance_coverage(records, expected_non_provenance, diagnostics);
}

fn validate_expected_record_shape(
    record: &ProvenanceRecord,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
    expected_checked_scope: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    let Some(expected) = expected_shape_for_artifact(&record.artifact_locator, output_format, bundle_kind) else {
        diagnostics.push(format!("unmatched provenance record for artifact `{}`", record.artifact_locator));
        return;
    };

    if record.source_id != expected.source_id {
        diagnostics.push(format!(
            "unexpected source_id for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator, expected.source_id, record.source_id
        ));
    }
    if record.checker_id != expected.checker_id {
        diagnostics.push(format!(
            "unexpected checker_id for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator, expected.checker_id, record.checker_id
        ));
    }
    if record.evidence_tier != expected.evidence_tier {
        diagnostics.push(format!(
            "unexpected evidence_tier for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator,
            expected.evidence_tier.as_str(),
            record.evidence_tier.as_str()
        ));
    }

    if bundle_kind == BundleKind::Prove && record.evidence_tier == EvidenceTier::FormalProof {
        if record.checked_scope.trim().is_empty() {
            diagnostics.push(format!("missing required provenance field `{FIELD_CHECKED_SCOPE}`"));
        }
        return;
    }

    match expected_checked_scope {
        Some(scope) if record.checked_scope == scope => {}
        Some(scope) => diagnostics.push(format!(
            "unexpected checked_scope for artifact `{}`; expected `{scope}`, got `{}`",
            record.artifact_locator, record.checked_scope
        )),
        None if !record.checked_scope.trim().is_empty() => {}
        None => diagnostics.push(format!("missing required provenance field `{FIELD_CHECKED_SCOPE}`")),
    }
}

fn expected_shape_for_artifact(
    artifact: &str,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
) -> Option<ExpectedRecordShape> {
    let manual_review_shape = ExpectedRecordShape {
        source_id: RUNNER_PROVENANCE_SOURCE_ID,
        checker_id: RUNNER_CHECKER_ID,
        evidence_tier: EvidenceTier::ManualReviewRequired,
    };
    match artifact {
        COMMAND_ARTIFACT_NAME | STATUS_ARTIFACT_NAME | SUMMARY_ARTIFACT_NAME => Some(manual_review_shape),
        PROOF_CARDS_ARTIFACT_NAME if bundle_kind == BundleKind::Prove => Some(manual_review_shape),
        JSON_RESULTS_ARTIFACT_NAME if output_format == OutputFormat::Json => Some(ExpectedRecordShape {
            source_id: RUNNER_STATIC_FINDINGS_SOURCE_ID,
            checker_id: RUNNER_CHECKER_ID,
            evidence_tier: EvidenceTier::StaticLint,
        }),
        SARIF_RESULTS_ARTIFACT_NAME if output_format == OutputFormat::Sarif => Some(ExpectedRecordShape {
            source_id: RUNNER_STATIC_FINDINGS_SOURCE_ID,
            checker_id: RUNNER_CHECKER_ID,
            evidence_tier: EvidenceTier::StaticLint,
        }),
        _ if bundle_kind == BundleKind::Prove && artifact.starts_with(PROOF_ARTIFACT_PREFIX) => {
            Some(ExpectedRecordShape {
                source_id: PROOF_SOURCE_ID,
                checker_id: CHECKER_OCTET_STANDARDS,
                evidence_tier: EvidenceTier::FormalProof,
            })
        }
        _ => None,
    }
}

fn validate_exact_provenance_coverage(
    records: &[ProvenanceRecord],
    expected_non_provenance: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let mut counts = BTreeMap::<String, usize>::new();
    for record in records {
        *counts.entry(record.artifact_locator.clone()).or_default() += 1;
    }

    for artifact in expected_non_provenance {
        match counts.get(*artifact).copied().unwrap_or_default() {
            0 => diagnostics.push(format!("missing provenance record for artifact `{artifact}`")),
            1 => {}
            count => diagnostics.push(format!("duplicate provenance records for artifact `{artifact}`; found {count}")),
        }
    }
}

fn verified_artifact_rows(records: &[ProvenanceRecord], expected_non_provenance: &[&str]) -> Vec<VerifiedArtifactRow> {
    let mut rows = Vec::new();
    for artifact in expected_non_provenance {
        if let Some(record) = records.iter().find(|record| record.artifact_locator == *artifact) {
            rows.push(VerifiedArtifactRow {
                artifact: (*artifact).to_owned(),
                source_id: record.source_id.clone(),
                checker_id: record.checker_id.clone(),
                evidence_tier: record.evidence_tier.as_str().to_owned(),
                blake3: record.blake3.clone(),
                checked_scope: record.checked_scope.clone(),
                manual_action: record.human_action.clone(),
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_DIR: &str = "artifacts/octet";
    const STATUS_HUMAN: &str = "human";
    const STATUS_JSON: &str = "json";
    const STATUS_SARIF: &str = "sarif";
    const CHECKED_SCOPE: &str = "profile:workspace-metadata";
    const PROFILE_NAME: &str = "workspace-metadata";
    const COMMAND_CONTENTS: &[u8] = b"cargo octet check\n";
    const STATUS_CONTENTS: &[u8] = br#"{
  "output_format": "human",
  "metadata": { "profile_name": "workspace-metadata" }
}
"#;
    const SUMMARY_CONTENTS: &[u8] = b"--- octet summary ---\nStatus: clean\n---\n";
    const JSON_CONTENTS: &[u8] = b"{\"lint\":\"no_unwrap\"}\n";
    const SARIF_CONTENTS: &[u8] = br#"{"version":"2.1.0"}
"#;
    const HUMAN_ACTION: &str = "review runner status before release";
    const COMMAND_RECORD_INDEX: usize = 0;
    const STATUS_RECORD_INDEX: usize = 1;
    const SUMMARY_RECORD_INDEX: usize = 2;

    #[test]
    fn complete_human_json_and_sarif_bundles_pass() {
        for output_format in [OutputFormat::Human, OutputFormat::Json, OutputFormat::Sarif] {
            let report = verify_bundle(valid_input(output_format));

            assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);
            assert!(report.diagnostics.is_empty());
            assert_eq!(report.output_format, output_format.as_str());
        }
    }

    #[test]
    fn missing_expected_artifact_fails_closed() {
        let mut input = valid_input(OutputFormat::Json);
        input.files.retain(|file| file.name != JSON_RESULTS_ARTIFACT_NAME);

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing expected artifact `results.jsonl`"))
        );
    }

    #[test]
    fn stale_extra_artifact_fails_closed() {
        let mut input = valid_input(OutputFormat::Human);
        input.files.push(BundleFile {
            name: "stale.txt".to_owned(),
            contents: b"stale\n".to_vec(),
        });

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic == "unexpected artifact `stale.txt`"));
    }

    #[test]
    fn malformed_jsonl_fails_closed() {
        let mut input = valid_input(OutputFormat::Human);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, b"not-json\n".to_vec());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed provenance line")));
    }

    #[test]
    fn hash_mismatch_unknown_checker_invalid_tier_and_missing_manual_action_fail_closed() {
        let mut records = valid_records(OutputFormat::Json);
        records[COMMAND_RECORD_INDEX].blake3 = crate::evidence::artifact_blake3_hex(b"different bytes");
        records[STATUS_RECORD_INDEX].checker_id = "mystery-checker".to_owned();
        records[STATUS_RECORD_INDEX].human_action = None;
        records[SUMMARY_RECORD_INDEX].evidence_tier = EvidenceTier::FormalProof;
        let mut input = valid_input(OutputFormat::Json);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("hash mismatch for artifact locator `command.txt`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unknown checker_id `mystery-checker`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing required provenance field `human_action`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unexpected evidence_tier for artifact `summary.txt`"))
        );
    }

    #[test]
    fn exact_coverage_rejects_duplicate_and_unmatched_records() {
        let mut records = valid_records(OutputFormat::Human);
        records.push(records[COMMAND_RECORD_INDEX].clone());
        let mut extra = records[COMMAND_RECORD_INDEX].clone();
        extra.artifact_locator = "ghost.txt".to_owned();
        records.push(extra);
        let mut input = valid_input(OutputFormat::Human);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate provenance records for artifact `command.txt`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unmatched provenance record for artifact `ghost.txt`"))
        );
    }

    fn valid_input(output_format: OutputFormat) -> ArtifactBundleInput {
        let mut files = vec![
            BundleFile {
                name: COMMAND_ARTIFACT_NAME.to_owned(),
                contents: COMMAND_CONTENTS.to_vec(),
            },
            BundleFile {
                name: STATUS_ARTIFACT_NAME.to_owned(),
                contents: status_contents(output_format).into_bytes(),
            },
            BundleFile {
                name: SUMMARY_ARTIFACT_NAME.to_owned(),
                contents: SUMMARY_CONTENTS.to_vec(),
            },
        ];
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => files.push(BundleFile {
                name: JSON_RESULTS_ARTIFACT_NAME.to_owned(),
                contents: JSON_CONTENTS.to_vec(),
            }),
            OutputFormat::Sarif => files.push(BundleFile {
                name: SARIF_RESULTS_ARTIFACT_NAME.to_owned(),
                contents: SARIF_CONTENTS.to_vec(),
            }),
        }
        files.push(BundleFile {
            name: PROVENANCE_ARTIFACT_NAME.to_owned(),
            contents: render_records(&valid_records(output_format)).into_bytes(),
        });
        ArtifactBundleInput {
            artifact_dir: ARTIFACT_DIR.to_owned(),
            files,
        }
    }

    fn valid_records(output_format: OutputFormat) -> Vec<ProvenanceRecord> {
        let mut records = vec![
            record(COMMAND_ARTIFACT_NAME, COMMAND_CONTENTS, EvidenceTier::ManualReviewRequired),
            record(STATUS_ARTIFACT_NAME, status_contents(output_format).as_bytes(), EvidenceTier::ManualReviewRequired),
            record(SUMMARY_ARTIFACT_NAME, SUMMARY_CONTENTS, EvidenceTier::ManualReviewRequired),
        ];
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => {
                records.push(record(JSON_RESULTS_ARTIFACT_NAME, JSON_CONTENTS, EvidenceTier::StaticLint))
            }
            OutputFormat::Sarif => {
                records.push(record(SARIF_RESULTS_ARTIFACT_NAME, SARIF_CONTENTS, EvidenceTier::StaticLint))
            }
        }
        records
    }

    fn record(artifact: &str, contents: &[u8], evidence_tier: EvidenceTier) -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: if evidence_tier == EvidenceTier::StaticLint {
                RUNNER_STATIC_FINDINGS_SOURCE_ID.to_owned()
            } else {
                RUNNER_PROVENANCE_SOURCE_ID.to_owned()
            },
            checker_id: RUNNER_CHECKER_ID.to_owned(),
            evidence_tier,
            artifact_locator: artifact.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(contents),
            checked_scope: CHECKED_SCOPE.to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: (evidence_tier == EvidenceTier::ManualReviewRequired).then(|| HUMAN_ACTION.to_owned()),
            parent_artifacts: Vec::new(),
        }
    }

    fn status_contents(output_format: OutputFormat) -> String {
        let name = match output_format {
            OutputFormat::Human => STATUS_HUMAN,
            OutputFormat::Json => STATUS_JSON,
            OutputFormat::Sarif => STATUS_SARIF,
        };
        String::from_utf8(STATUS_CONTENTS.to_vec())
            .expect("status fixture utf8")
            .replace(STATUS_HUMAN, name)
            .replace(PROFILE_NAME, PROFILE_NAME)
    }

    fn render_records(records: &[ProvenanceRecord]) -> String {
        let mut rendered = String::new();
        for record in records {
            rendered.push_str(&serde_json::to_string(record).expect("serialize provenance fixture"));
            rendered.push('\n');
        }
        rendered
    }

    fn replace_file(input: &mut ArtifactBundleInput, file_name: &str, contents: Vec<u8>) {
        let file = input
            .files
            .iter_mut()
            .find(|file| file.name == file_name)
            .unwrap_or_else(|| panic!("missing fixture file {file_name}"));
        file.contents = contents;
    }
}
