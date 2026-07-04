#![allow(clippy::module_name_repetitions)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub const INDEX_SCHEMA_VERSION: &str = "octet-evidence-codebase-index/v1";
pub const NORMALIZATION_VERSION: &str = "octet-evidence-codebase-normalization/v1";
pub const INDEX_BOUNDARY: &str = "Evidence codebase index proves artifact identity and linkage only; it does not claim whole-program correctness, semantic equivalence, complete dependency closure, distributed execution correctness, remote code mobility, or formal proof unless a linked formal-proof receipt scopes that claim.";

const PROVENANCE_SCHEMA_VERSION: &str = "octet-provenance-record/v1";
const PROOF_CARDS_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION: &str = "octet-evidence-log-manifest/v1";
const EVIDENCE_RAIL_RECEIPT_SCHEMA_VERSION: &str = "octet-evidence-rail-receipt/v1";
const REPLAY_CAPSULE_KIND: &str = "octet-replay-capsule";
const MANUAL_REVIEW_TIER: &str = "manual_review_required";
const TRANSCRIPT_ASSERTION_TIER: &str = "transcript_assertion";
const STATIC_LINT_TIER: &str = "static_lint";
const POLICY_CHECK_TIER: &str = "policy_check";
const UNSUPPORTED_FAMILY_CODE: &str = "unsupported_family";
const MALFORMED_JSON_CODE: &str = "malformed_json";
const HASH_PREFIX: &str = "b3:";
const COMMAND_PREFIX: &str = "$ ";
const EXPECTED_OUTPUT_PREFIX: &str = "=> ";

const LOCATOR_STABLE_MARKERS: &[&str] = &[
    "artifacts/",
    "cargo-octet/tests/fixtures/",
    "cairn/",
    "docs/",
    "openspec/",
    "standards/",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceArtifactInput {
    pub locator: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceIndexBuildMode {
    Strict,
    Advisory,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EvidenceCodebaseIndex {
    pub schema_version: String,
    pub normalization_version: String,
    pub index_hash: String,
    pub boundary: String,
    pub source_artifact_count: usize,
    pub record_count: usize,
    pub records: Vec<EvidenceIndexRecord>,
    pub issues: Vec<EvidenceIndexIssue>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EvidenceIndexRecord {
    pub record_id: String,
    pub artifact_family: String,
    pub stable_id: String,
    pub artifact_locator: String,
    pub artifact_blake3: String,
    pub evidence_tier: String,
    pub source_ids: Vec<String>,
    pub requirement_ids: Vec<String>,
    pub lint_ids: Vec<String>,
    pub policy_ids: Vec<String>,
    pub fingerprints: Vec<String>,
    pub artifact_hashes: Vec<String>,
    pub provenance_source_ids: Vec<String>,
    pub replay_commands: Vec<String>,
    pub parent_links: Vec<String>,
    pub caveats: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EvidenceIndexIssue {
    pub severity: EvidenceIndexIssueSeverity,
    pub code: String,
    pub artifact_locator: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceIndexIssueSeverity {
    Error,
    Caveat,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct EvidenceIndexQueryReport {
    pub schema_version: String,
    pub query: String,
    pub match_count: usize,
    pub boundary: String,
    pub records: Vec<EvidenceIndexRecord>,
}

#[must_use]
pub fn build_evidence_codebase_index(
    inputs: &[EvidenceArtifactInput],
    mode: EvidenceIndexBuildMode,
) -> EvidenceCodebaseIndex {
    let sources = normalized_sources(inputs, mode);
    let source_hashes = source_hashes(&sources);
    let mut builder = IndexBuilder::new(mode);

    for source in &sources {
        ingest_source(source, &source_hashes, &mut builder);
    }

    builder.records.sort_by(|left, right| {
        left.artifact_family
            .cmp(&right.artifact_family)
            .then(left.stable_id.cmp(&right.stable_id))
            .then(left.artifact_locator.cmp(&right.artifact_locator))
            .then(left.record_id.cmp(&right.record_id))
    });
    builder.issues.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then(left.code.cmp(&right.code))
            .then(left.artifact_locator.cmp(&right.artifact_locator))
            .then(left.detail.cmp(&right.detail))
    });
    builder.issues.dedup();

    let record_count = builder.records.len();
    let mut index = EvidenceCodebaseIndex {
        schema_version: INDEX_SCHEMA_VERSION.to_owned(),
        normalization_version: NORMALIZATION_VERSION.to_owned(),
        index_hash: String::new(),
        boundary: INDEX_BOUNDARY.to_owned(),
        source_artifact_count: sources.len(),
        record_count,
        records: builder.records,
        issues: builder.issues,
    };
    index.index_hash = index_hash(&index);
    index
}

#[must_use]
pub fn query_evidence_index(index: &EvidenceCodebaseIndex, query: &str) -> EvidenceIndexQueryReport {
    let mut selected_ids = BTreeSet::<String>::new();
    for record in &index.records {
        if record_matches(record, query) {
            selected_ids.insert(record.record_id.clone());
            for linked in linked_records(index, record) {
                selected_ids.insert(linked.record_id.clone());
            }
        }
    }

    let records = index
        .records
        .iter()
        .filter(|record| selected_ids.contains(&record.record_id))
        .cloned()
        .collect::<Vec<_>>();

    EvidenceIndexQueryReport {
        schema_version: INDEX_SCHEMA_VERSION.to_owned(),
        query: query.to_owned(),
        match_count: records.len(),
        boundary: INDEX_BOUNDARY.to_owned(),
        records,
    }
}

#[must_use]
pub fn render_evidence_index_html(index: &EvidenceCodebaseIndex) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<title>Octet evidence codebase index</title>\n");
    html.push_str("<style>body{font-family:sans-serif;max-width:72rem;margin:2rem auto;line-height:1.4}code{background:#f2f2f2;padding:0.1rem 0.25rem}.issue{color:#8a4b00}.record{border-top:1px solid #ddd;padding-top:1rem;margin-top:1rem}</style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("<h1>Octet evidence codebase index</h1>\n");
    let _ = writeln!(&mut html, "<p><strong>Index hash:</strong> <code>{}</code></p>", escape_html(&index.index_hash));
    let _ = writeln!(&mut html, "<p>{}</p>", escape_html(&index.boundary));
    let _ = writeln!(
        &mut html,
        "<p>Records: {}. Source artifacts: {}. Issues: {}.</p>",
        index.record_count,
        index.source_artifact_count,
        index.issues.len()
    );

    html.push_str("<h2>Records</h2>\n<ul>\n");
    for record in &index.records {
        let _ = writeln!(
            &mut html,
            "<li><a href=\"#{}\"><code>{}</code></a> {} <small>{}</small></li>",
            html_id(&record.record_id),
            escape_html(&record.stable_id),
            escape_html(&record.artifact_family),
            escape_html(&record.summary)
        );
    }
    html.push_str("</ul>\n");

    for record in &index.records {
        let _ = writeln!(&mut html, "<section class=\"record\" id=\"{}\">", html_id(&record.record_id));
        let _ = writeln!(&mut html, "<h3>{}</h3>", escape_html(&record.stable_id));
        let _ = writeln!(
            &mut html,
            "<p><strong>Family:</strong> {}<br><strong>Tier:</strong> {}<br><strong>Artifact:</strong> <code>{}</code><br><strong>BLAKE3:</strong> <code>{}</code></p>",
            escape_html(&record.artifact_family),
            escape_html(&record.evidence_tier),
            escape_html(&record.artifact_locator),
            escape_html(&record.artifact_blake3)
        );
        render_list(&mut html, "Requirement IDs", &record.requirement_ids);
        render_list(&mut html, "Lint IDs", &record.lint_ids);
        render_list(&mut html, "Policy IDs", &record.policy_ids);
        render_list(&mut html, "Fingerprints", &record.fingerprints);
        render_list(&mut html, "Provenance source IDs", &record.provenance_source_ids);
        render_list(&mut html, "Replay commands", &record.replay_commands);
        render_list(&mut html, "Parent links", &record.parent_links);
        render_list(&mut html, "Caveats", &record.caveats);
        html.push_str("</section>\n");
    }

    if !index.issues.is_empty() {
        html.push_str("<h2>Issues and caveats</h2>\n<ul>\n");
        for issue in &index.issues {
            let _ = writeln!(
                &mut html,
                "<li class=\"issue\"><code>{:?}</code> <code>{}</code> {}: {}</li>",
                issue.severity,
                escape_html(&issue.code),
                escape_html(&issue.artifact_locator),
                escape_html(&issue.detail)
            );
        }
        html.push_str("</ul>\n");
    }

    html.push_str("</body>\n</html>\n");
    html
}

#[must_use]
pub fn output_has_required_boundary(text: &str) -> bool {
    text.contains(INDEX_BOUNDARY)
        && !text.contains("proves whole-program correctness")
        && !text.contains("proves semantic equivalence")
        && !text.contains("complete dependency closure proof")
        && !text.contains("remote code mobility guarantee")
}

fn normalized_sources(inputs: &[EvidenceArtifactInput], mode: EvidenceIndexBuildMode) -> Vec<NormalizedSource> {
    let mut sources = inputs
        .iter()
        .map(|input| NormalizedSource {
            locator: normalize_locator(&input.locator),
            bytes: input.bytes.clone(),
            blake3: blake3_hex(&input.bytes),
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.locator.cmp(&right.locator).then(left.blake3.cmp(&right.blake3)));

    if mode == EvidenceIndexBuildMode::Advisory {
        sources.dedup_by(|left, right| left.locator == right.locator && left.blake3 == right.blake3);
    }
    sources
}

fn source_hashes(sources: &[NormalizedSource]) -> BTreeMap<String, String> {
    sources
        .iter()
        .map(|source| (source.locator.clone(), source.blake3.clone()))
        .collect::<BTreeMap<_, _>>()
}

fn ingest_source(source: &NormalizedSource, source_hashes: &BTreeMap<String, String>, builder: &mut IndexBuilder) {
    if let Some(objects) = parse_jsonl_objects(source, builder) {
        if objects.iter().all(is_provenance_record) {
            ingest_provenance_records(source, &objects, source_hashes, builder);
            return;
        }
        if objects.iter().all(is_finding_record) {
            ingest_finding_records(source, &objects, builder);
            return;
        }
        builder.issue(source, UNSUPPORTED_FAMILY_CODE, "JSONL artifact has no supported Octet evidence family");
        return;
    }

    let text = String::from_utf8_lossy(&source.bytes);
    if let Ok(value) = serde_json::from_slice::<Value>(&source.bytes) {
        if is_proof_card_deck(&value) {
            ingest_proof_cards(source, &value, builder);
            return;
        }
        if is_fingerprint_review_manifest(&value) {
            ingest_fingerprint_review_manifest(source, &value, builder);
            return;
        }
        if is_replay_capsule(&value) {
            ingest_replay_capsule(source, &value, builder);
            return;
        }
        if is_transcript_report(&value) {
            ingest_transcript_report(source, &value, builder);
            return;
        }
        if is_evidence_log_manifest(&value) {
            ingest_evidence_log_manifest(source, &value, builder);
            return;
        }
        if is_evidence_rail_receipt(&value) {
            ingest_evidence_rail_receipt(source, &value, builder);
            return;
        }
        if is_single_finding_record(&value) || looks_like_finding_record(&value) {
            ingest_finding_records(source, &[value], builder);
            return;
        }
        if is_provenance_record(&value) {
            ingest_provenance_records(source, &[value], source_hashes, builder);
            return;
        }
        builder.issue(source, UNSUPPORTED_FAMILY_CODE, "JSON artifact has no supported Octet evidence family");
        return;
    }

    if text.contains(COMMAND_PREFIX) && text.contains(EXPECTED_OUTPUT_PREFIX) {
        ingest_transcript_markdown(source, &text, builder);
    } else if looks_like_json(&text) {
        builder.issue(source, MALFORMED_JSON_CODE, "artifact is not parseable JSON or JSONL");
    } else {
        builder.issue(source, UNSUPPORTED_FAMILY_CODE, "artifact has no supported Octet evidence family");
    }
}

fn parse_jsonl_objects(source: &NormalizedSource, builder: &mut IndexBuilder) -> Option<Vec<Value>> {
    let text = String::from_utf8_lossy(&source.bytes);
    let lines = text.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();
    if lines.is_empty() || lines.len() == usize::from(looks_like_json(&text)) {
        return None;
    }

    let mut objects = Vec::new();
    for line in lines {
        match serde_json::from_str::<Value>(line) {
            Ok(Value::Object(_)) => objects.push(serde_json::from_str::<Value>(line).unwrap_or(Value::Null)),
            Ok(_) => {
                builder.issue(source, MALFORMED_JSON_CODE, "JSONL line is not an object");
                return Some(Vec::new());
            }
            Err(_) => return None,
        }
    }
    Some(objects)
}

fn ingest_provenance_records(
    source: &NormalizedSource,
    objects: &[Value],
    source_hashes: &BTreeMap<String, String>,
    builder: &mut IndexBuilder,
) {
    for object in objects {
        let mut required = RequiredFields::new(source, builder, "provenance");
        let schema_version = required.string(object, "schema_version");
        let source_id = required.string(object, "source_id");
        let checker_id = required.string(object, "checker_id");
        let evidence_tier = required.string(object, "evidence_tier");
        let artifact_locator = required.string(object, "artifact_locator");
        let blake3 = required.string(object, "blake3");
        let Some(stable_id) = source_id.clone().or_else(|| artifact_locator.clone()) else {
            continue;
        };
        if schema_version.as_deref() != Some(PROVENANCE_SCHEMA_VERSION) {
            builder.issue(source, "provenance.schema_version", "unsupported or missing provenance schema_version");
        }
        if let (Some(locator), Some(record_hash)) = (&artifact_locator, &blake3) {
            let normalized_locator = normalize_locator(locator);
            if let Some(expected_hash) = source_hashes.get(&normalized_locator) {
                if expected_hash != record_hash {
                    builder.issue(
                        source,
                        "provenance.blake3_mismatch",
                        "provenance record blake3 does not match indexed artifact bytes",
                    );
                }
            } else {
                builder.caveat(
                    source,
                    "provenance.referenced_artifact_missing",
                    "referenced artifact bytes were not supplied to this index build",
                );
            }
        }

        let record = EvidenceIndexRecordBuilder::new("octet-provenance", &stable_id, source)
            .evidence_tier(evidence_tier.as_deref().unwrap_or(MANUAL_REVIEW_TIER))
            .source_id(source_id)
            .source_id(checker_id)
            .artifact_hash(blake3)
            .provenance_source_id(object_string(object, "source_id"))
            .parent_links(object_strings(object, "parent_artifacts"))
            .summary(format!("provenance for {}", artifact_locator.as_deref().unwrap_or("unknown artifact")))
            .build();
        builder.record(record);
    }
}

fn ingest_finding_records(source: &NormalizedSource, objects: &[Value], builder: &mut IndexBuilder) {
    for object in objects {
        let mut required = RequiredFields::new(source, builder, "finding");
        let stable_id = required.string(object, "stable_id");
        let lint = required.string(object, "lint");
        let Some(stable_id) = stable_id else {
            continue;
        };
        let summary = format!(
            "finding {} at {}",
            lint.as_deref().unwrap_or("unknown-lint"),
            object_string(object, "file").unwrap_or_else(|| "unknown source".to_owned())
        );
        let record = EvidenceIndexRecordBuilder::new("octet-finding", &stable_id, source)
            .evidence_tier(object_string(object, "evidence_tier").as_deref().unwrap_or(STATIC_LINT_TIER))
            .source_id(object_string(object, "file"))
            .lint_id(lint)
            .fingerprint(object_string(object, "semantic_id"))
            .fingerprint(object_string(object, "normalized_snippet_hash"))
            .fingerprint(object_string(object, "config_hash"))
            .requirement_ids(strings_with_requirement_ids(object))
            .summary(summary)
            .build();
        builder.record(record);
    }
}

fn ingest_proof_cards(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let Some(cards) = value.get("cards").and_then(Value::as_array) else {
        builder.issue(source, "proof_cards.cards", "proof card deck missing cards array");
        return;
    };
    if value.get("schema_version").and_then(Value::as_str) != Some(PROOF_CARDS_SCHEMA_VERSION) {
        builder.issue(source, "proof_cards.schema_version", "unsupported proof card deck schema_version");
    }
    for card in cards {
        let mut required = RequiredFields::new(source, builder, "proof_card");
        let proof_id = required.string(card, "proof_or_property_id");
        let policy_id = required.string(card, "policy_id");
        let evidence_tier = required.string(card, "evidence_tier");
        let Some(proof_id) = proof_id else {
            continue;
        };
        let artifact_hashes = card
            .get("artifact_blake3_identities")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|identity| object_string(identity, "blake3"))
            .collect::<Vec<_>>();
        let parent_links = object_string(card, "receipt_path")
            .into_iter()
            .chain(object_string(card, "provenance_artifact"))
            .chain(
                card.get("artifact_blake3_identities")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|identity| object_string(identity, "artifact")),
            )
            .collect::<Vec<_>>();
        let record = EvidenceIndexRecordBuilder::new("octet-proof-card", &proof_id, source)
            .evidence_tier(evidence_tier.as_deref().unwrap_or(MANUAL_REVIEW_TIER))
            .source_id(object_string(card, "checked_rust_source_scope"))
            .requirement_id(object_string(card, "source_requirement_or_policy_id"))
            .policy_id(policy_id)
            .artifact_hashes(artifact_hashes)
            .replay_command(object_string(card, "artifact_verifier_command"))
            .parent_links(parent_links)
            .caveats(object_strings(card, "non_claims"))
            .summary("proof card linkage".to_owned())
            .build();
        builder.record(record);
    }
}

fn ingest_fingerprint_review_manifest(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let Some(records) = value.get("records").and_then(Value::as_array) else {
        builder.issue(source, "fingerprint.records", "fingerprint review manifest missing records array");
        return;
    };
    for item in records {
        let mut required = RequiredFields::new(source, builder, "fingerprint");
        let stable_id = required.string(item, "stable_id");
        let Some(stable_id) = stable_id else {
            continue;
        };
        let mut caveats = Vec::new();
        if item.get("requires_review").and_then(Value::as_bool).unwrap_or(false) {
            caveats.push("requires_review".to_owned());
        }
        if item.get("macro_warning").and_then(Value::as_bool).unwrap_or(false) {
            caveats.push("macro_warning".to_owned());
        }
        let source_label = match (object_string(item, "path"), object_string(item, "name")) {
            (Some(path), Some(name)) => Some(format!("{path}::{name}")),
            (Some(path), None) => Some(path),
            (None, Some(name)) => Some(name),
            (None, None) => None,
        };
        let record = EvidenceIndexRecordBuilder::new("octet-fingerprint-review", &stable_id, source)
            .evidence_tier(MANUAL_REVIEW_TIER)
            .source_id(source_label)
            .fingerprint(object_string(item, "computed_hash"))
            .fingerprint(object_string(item, "normalized_body_hash"))
            .fingerprint(object_string(item, "semantic_id"))
            .fingerprint(object_string(item, "dependency_set_hash"))
            .requirement_ids(strings_with_requirement_ids(item))
            .caveats(caveats)
            .summary("function fingerprint review record".to_owned())
            .build();
        builder.record(record);
    }
}

fn ingest_replay_capsule(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let mut required = RequiredFields::new(source, builder, "replay_capsule");
    let capsule_hash = required.string(value, "capsule_hash");
    let Some(capsule_hash) = capsule_hash else {
        return;
    };
    let manifest = value.get("manifest").unwrap_or(&Value::Null);
    let replay_command =
        manifest.get("command_argv").and_then(Value::as_array).map(|items| command_from_json_array(items));
    let artifact_hashes = hash_values_from(manifest.get("input_hashes"))
        .into_iter()
        .chain(hash_values_from(manifest.get("output_hashes")))
        .collect::<Vec<_>>();
    let record = EvidenceIndexRecordBuilder::new("octet-replay-capsule", &capsule_hash, source)
        .evidence_tier(TRANSCRIPT_ASSERTION_TIER)
        .source_id(object_string(manifest, "run_kind"))
        .fingerprint(object_string(manifest, "config_hash"))
        .artifact_hashes(artifact_hashes)
        .replay_command(replay_command)
        .caveat(Some("replay identity only; not distributed execution correctness".to_owned()))
        .summary("replay capsule".to_owned())
        .build();
    builder.record(record);
}

fn ingest_transcript_report(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let mut required = RequiredFields::new(source, builder, "transcript");
    let transcript_hash = required.string(value, "transcript_hash");
    let Some(transcript_hash) = transcript_hash else {
        return;
    };
    let record = EvidenceIndexRecordBuilder::new("octet-transcript-report", &transcript_hash, source)
        .evidence_tier(TRANSCRIPT_ASSERTION_TIER)
        .source_id(object_string(value, "path"))
        .provenance_source_id(value.get("provenance").and_then(|item| object_string(item, "source_id")))
        .parent_link(value.get("provenance").and_then(|item| object_string(item, "artifact_locator")))
        .artifact_hash(value.get("provenance").and_then(|item| object_string(item, "blake3")))
        .summary("transcript assertion report".to_owned())
        .build();
    builder.record(record);
}

fn ingest_transcript_markdown(source: &NormalizedSource, text: &str, builder: &mut IndexBuilder) {
    let commands = text
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix(COMMAND_PREFIX).map(str::to_owned))
        .collect::<Vec<_>>();
    let stable_id = prefixed_hash(text.as_bytes());
    let record = EvidenceIndexRecordBuilder::new("octet-transcript-markdown", &stable_id, source)
        .evidence_tier(TRANSCRIPT_ASSERTION_TIER)
        .source_id(Some(source.locator.clone()))
        .replay_commands(commands)
        .summary("markdown transcript assertions".to_owned())
        .build();
    builder.record(record);
}

fn ingest_evidence_log_manifest(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let Some(logs) = value.get("logs").and_then(Value::as_array) else {
        builder.issue(source, "evidence_log.logs", "evidence log manifest missing logs array");
        return;
    };
    for log in logs {
        let mut required = RequiredFields::new(source, builder, "evidence_log");
        let locator = required.string(log, "artifact_locator");
        let blake3 = required.string(log, "blake3");
        let Some(locator) = locator else {
            continue;
        };
        let record = EvidenceIndexRecordBuilder::new("octet-evidence-log", &locator, source)
            .evidence_tier(TRANSCRIPT_ASSERTION_TIER)
            .source_id(object_string(log, "capture_source"))
            .source_id(object_string(log, "checked_scope"))
            .artifact_hash(blake3)
            .replay_command(object_string(log, "replay_command"))
            .summary("bounded evidence log".to_owned())
            .build();
        builder.record(record);
    }
}

fn ingest_evidence_rail_receipt(source: &NormalizedSource, value: &Value, builder: &mut IndexBuilder) {
    let mut required = RequiredFields::new(source, builder, "evidence_rail");
    let rail_id = required.string(value, "rail_id");
    let checker_id = required.string(value, "checker_id");
    let Some(rail_id) = rail_id else {
        return;
    };
    let artifact_hashes = [
        object_string(value, "bounded_log_identity"),
        object_string(value, "imported_input_identity"),
        object_string(value, "baseline_identity"),
        object_string(value, "lockfile_identity"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let record = EvidenceIndexRecordBuilder::new("octet-evidence-rail-receipt", &rail_id, source)
        .evidence_tier(POLICY_CHECK_TIER)
        .source_id(checker_id)
        .policy_id(Some(rail_id.clone()))
        .artifact_hashes(artifact_hashes)
        .replay_command(value.get("command_argv").and_then(Value::as_array).map(|items| command_from_json_array(items)))
        .parent_links(object_strings(value, "parent_artifacts"))
        .caveats(object_strings(value, "non_claims"))
        .summary("external evidence rail receipt".to_owned())
        .build();
    builder.record(record);

    for finding in value.get("findings").and_then(Value::as_array).into_iter().flatten() {
        if let Some(family) = object_string(finding, "family") {
            let finding_id = format!("{rail_id}#{family}");
            let finding_record = EvidenceIndexRecordBuilder::new("octet-evidence-rail-finding", &finding_id, source)
                .evidence_tier(POLICY_CHECK_TIER)
                .policy_id(Some(rail_id.clone()))
                .lint_id(Some(family.clone()))
                .source_id(object_string(finding, "file"))
                .parent_link(Some(rail_id.clone()))
                .summary(format!("evidence rail finding {family}"))
                .build();
            builder.record(finding_record);
        }
    }
}

fn is_provenance_record(value: &Value) -> bool {
    value.get("schema_version").and_then(Value::as_str) == Some(PROVENANCE_SCHEMA_VERSION)
        || (value.get("source_id").is_some()
            && value.get("checker_id").is_some()
            && value.get("artifact_locator").is_some()
            && value.get("blake3").is_some())
}

fn is_finding_record(value: &Value) -> bool {
    value.get("stable_id").is_some() && value.get("lint").is_some() && value.get("evidence_tier").is_some()
}

fn is_single_finding_record(value: &Value) -> bool {
    value.get("stable_id").is_some() && value.get("lint").is_some()
}

fn looks_like_finding_record(value: &Value) -> bool {
    value.get("lint").is_some() && value.get("evidence_tier").is_some()
}

fn is_proof_card_deck(value: &Value) -> bool {
    value.get("schema_version").and_then(Value::as_str) == Some(PROOF_CARDS_SCHEMA_VERSION)
        || value.get("cards").is_some()
}

fn is_fingerprint_review_manifest(value: &Value) -> bool {
    value.get("manifest_hash").is_some()
        && value.get("records").is_some()
        && value.get("tool_name").and_then(Value::as_str) == Some("cargo-octet")
}

fn is_replay_capsule(value: &Value) -> bool {
    value.get("capsule_kind").and_then(Value::as_str) == Some(REPLAY_CAPSULE_KIND)
        || value.get("capsule_hash").is_some() && value.get("manifest").is_some()
}

fn is_transcript_report(value: &Value) -> bool {
    value.get("transcript_hash").is_some() && value.get("command_blocks").is_some()
}

fn is_evidence_log_manifest(value: &Value) -> bool {
    value.get("schema_version").and_then(Value::as_str) == Some(EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION)
        || value.get("logs").is_some()
}

fn is_evidence_rail_receipt(value: &Value) -> bool {
    value.get("schema_version").and_then(Value::as_str) == Some(EVIDENCE_RAIL_RECEIPT_SCHEMA_VERSION)
        || value.get("rail_id").is_some() && value.get("bounded_log_identity").is_some()
}

fn record_matches(record: &EvidenceIndexRecord, query: &str) -> bool {
    record.record_id == query
        || record.stable_id == query
        || record.artifact_locator == query
        || record.artifact_blake3 == query
        || record.artifact_blake3 == query.trim_start_matches(HASH_PREFIX)
        || contains_query(&record.source_ids, query)
        || contains_query(&record.requirement_ids, query)
        || contains_query(&record.lint_ids, query)
        || contains_query(&record.policy_ids, query)
        || contains_query(&record.fingerprints, query)
        || contains_query(&record.artifact_hashes, query)
        || contains_query(&record.provenance_source_ids, query)
        || contains_query(&record.parent_links, query)
}

fn linked_records<'a>(index: &'a EvidenceCodebaseIndex, record: &EvidenceIndexRecord) -> Vec<&'a EvidenceIndexRecord> {
    index
        .records
        .iter()
        .filter(|candidate| candidate.record_id != record.record_id)
        .filter(|candidate| {
            record.parent_links.iter().any(|link| link_matches_record(link, candidate))
                || candidate.parent_links.iter().any(|link| link_matches_record(link, record))
                || shared_identity(&record.policy_ids, &candidate.policy_ids)
                || shared_identity(&record.requirement_ids, &candidate.requirement_ids)
                || shared_identity(&record.lint_ids, &candidate.lint_ids)
                || cross_record_hash_link(record, candidate)
        })
        .collect()
}

fn link_matches_record(link: &str, record: &EvidenceIndexRecord) -> bool {
    link == record.artifact_locator || link == record.stable_id || link == record.record_id
}

fn shared_identity(left: &[String], right: &[String]) -> bool {
    left.iter().any(|value| right.contains(value))
}

fn cross_record_hash_link(left: &EvidenceIndexRecord, right: &EvidenceIndexRecord) -> bool {
    right.artifact_hashes.contains(&left.stable_id)
        || left.artifact_hashes.contains(&right.stable_id)
        || right.fingerprints.contains(&left.stable_id)
        || left.fingerprints.contains(&right.stable_id)
        || shared_identity(&left.artifact_hashes, &right.artifact_hashes)
        || shared_identity(&left.fingerprints, &right.fingerprints)
        || shared_identity(&left.artifact_hashes, &right.fingerprints)
        || shared_identity(&left.fingerprints, &right.artifact_hashes)
}

fn contains_query(values: &[String], query: &str) -> bool {
    values.iter().any(|value| value == query || value == query.trim_start_matches(HASH_PREFIX))
}

fn index_hash(index: &EvidenceCodebaseIndex) -> String {
    let payload = HashableIndex {
        schema_version: &index.schema_version,
        normalization_version: &index.normalization_version,
        boundary: &index.boundary,
        source_artifact_count: index.source_artifact_count,
        record_count: index.record_count,
        records: &index.records,
        issues: &index.issues,
    };
    let rendered = serde_json::to_string(&payload).unwrap_or_else(|error| format!("serialization-error:{error}"));
    blake3_hex(rendered.as_bytes())
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn prefixed_hash(bytes: &[u8]) -> String {
    format!("{HASH_PREFIX}{}", blake3_hex(bytes))
}

fn normalize_locator(locator: &str) -> String {
    let replaced = locator.replace('\\', "/");
    let trimmed = replaced.trim_start_matches("./");
    for marker in LOCATOR_STABLE_MARKERS {
        if let Some(offset) = trimmed.find(marker) {
            return trimmed[offset..].to_owned();
        }
    }
    if trimmed.starts_with('/') {
        let mut components = trimmed.rsplit('/').filter(|component| !component.is_empty());
        let Some(file) = components.next() else {
            return "absolute-path".to_owned();
        };
        if let Some(parent) = components.next() {
            return format!("{parent}/{file}");
        }
        return file.to_owned();
    }
    trimmed.to_owned()
}

fn object_string(value: &Value, field: &str) -> Option<String> {
    value.get(field).and_then(Value::as_str).map(str::to_owned).filter(|item| !item.trim().is_empty())
}

fn object_strings(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

fn strings_with_requirement_ids(value: &Value) -> Vec<String> {
    let mut ids = BTreeSet::new();
    collect_requirement_ids(value, &mut ids);
    ids.into_iter().collect()
}

fn collect_requirement_ids(value: &Value, ids: &mut BTreeSet<String>) {
    match value {
        Value::String(text) => {
            let mut remainder = text.as_str();
            while let Some(start) = remainder.find("r[") {
                let after_start = &remainder[start..];
                let Some(end) = after_start.find(']') else {
                    break;
                };
                ids.insert(after_start[..=end].to_owned());
                remainder = &after_start[end + 1..];
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_requirement_ids(item, ids);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_requirement_ids(item, ids);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn hash_values_from(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
        Some(Value::Object(map)) => map.values().filter_map(Value::as_str).map(str::to_owned).collect(),
        Some(Value::String(text)) => vec![text.clone()],
        Some(Value::Null | Value::Bool(_) | Value::Number(_)) | None => Vec::new(),
    }
}

fn command_from_json_array(items: &[Value]) -> String {
    items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" ")
}

fn looks_like_json(text: &str) -> bool {
    let trimmed = text.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn render_list(html: &mut String, heading: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let _ = writeln!(html, "<p><strong>{}</strong></p><ul>", escape_html(heading));
    for value in values {
        let _ = writeln!(html, "<li><code>{}</code></li>", escape_html(value));
    }
    html.push_str("</ul>\n");
}

fn html_id(value: &str) -> String {
    let mut id = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        } else if !id.ends_with('-') {
            id.push('-');
        }
    }
    id.trim_matches('-').to_owned()
}

fn escape_html(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[derive(Clone, Debug)]
struct NormalizedSource {
    locator: String,
    bytes: Vec<u8>,
    blake3: String,
}

struct IndexBuilder {
    mode: EvidenceIndexBuildMode,
    records: Vec<EvidenceIndexRecord>,
    issues: Vec<EvidenceIndexIssue>,
}

impl IndexBuilder {
    fn new(mode: EvidenceIndexBuildMode) -> Self {
        Self {
            mode,
            records: Vec::new(),
            issues: Vec::new(),
        }
    }

    fn record(&mut self, record: EvidenceIndexRecord) {
        self.records.push(record);
    }

    fn issue(&mut self, source: &NormalizedSource, code: &str, detail: &str) {
        self.issues.push(EvidenceIndexIssue {
            severity: if self.mode == EvidenceIndexBuildMode::Strict {
                EvidenceIndexIssueSeverity::Error
            } else {
                EvidenceIndexIssueSeverity::Caveat
            },
            code: code.to_owned(),
            artifact_locator: source.locator.clone(),
            detail: detail.to_owned(),
        });
    }

    fn caveat(&mut self, source: &NormalizedSource, code: &str, detail: &str) {
        self.issues.push(EvidenceIndexIssue {
            severity: EvidenceIndexIssueSeverity::Caveat,
            code: code.to_owned(),
            artifact_locator: source.locator.clone(),
            detail: detail.to_owned(),
        });
    }
}

struct RequiredFields<'a, 'b> {
    source: &'a NormalizedSource,
    builder: &'b mut IndexBuilder,
    family: &'static str,
}

impl<'a, 'b> RequiredFields<'a, 'b> {
    fn new(source: &'a NormalizedSource, builder: &'b mut IndexBuilder, family: &'static str) -> Self {
        Self {
            source,
            builder,
            family,
        }
    }

    fn string(&mut self, value: &Value, field: &str) -> Option<String> {
        let result = object_string(value, field);
        if result.is_none() {
            self.builder.issue(
                self.source,
                &format!("{}.missing_{}", self.family, field),
                &format!("missing required {field} field"),
            );
        }
        result
    }
}

struct EvidenceIndexRecordBuilder<'a> {
    artifact_family: &'static str,
    stable_id: &'a str,
    source: &'a NormalizedSource,
    evidence_tier: String,
    source_ids: BTreeSet<String>,
    requirement_ids: BTreeSet<String>,
    lint_ids: BTreeSet<String>,
    policy_ids: BTreeSet<String>,
    fingerprints: BTreeSet<String>,
    artifact_hashes: BTreeSet<String>,
    provenance_source_ids: BTreeSet<String>,
    replay_commands: BTreeSet<String>,
    parent_links: BTreeSet<String>,
    caveats: BTreeSet<String>,
    summary: String,
}

impl<'a> EvidenceIndexRecordBuilder<'a> {
    fn new(artifact_family: &'static str, stable_id: &'a str, source: &'a NormalizedSource) -> Self {
        let mut artifact_hashes = BTreeSet::new();
        artifact_hashes.insert(source.blake3.clone());
        Self {
            artifact_family,
            stable_id,
            source,
            evidence_tier: MANUAL_REVIEW_TIER.to_owned(),
            source_ids: BTreeSet::new(),
            requirement_ids: BTreeSet::new(),
            lint_ids: BTreeSet::new(),
            policy_ids: BTreeSet::new(),
            fingerprints: BTreeSet::new(),
            artifact_hashes,
            provenance_source_ids: BTreeSet::new(),
            replay_commands: BTreeSet::new(),
            parent_links: BTreeSet::new(),
            caveats: BTreeSet::new(),
            summary: String::new(),
        }
    }

    fn evidence_tier(mut self, value: &str) -> Self {
        value.clone_into(&mut self.evidence_tier);
        self
    }

    fn source_id(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.source_ids, value);
        self
    }

    fn requirement_id(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.requirement_ids, value);
        self
    }

    fn requirement_ids(mut self, values: Vec<String>) -> Self {
        self.requirement_ids.extend(values);
        self
    }

    fn lint_id(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.lint_ids, value);
        self
    }

    fn policy_id(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.policy_ids, value);
        self
    }

    fn fingerprint(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.fingerprints, value);
        self
    }

    fn artifact_hash(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.artifact_hashes, value);
        self
    }

    fn artifact_hashes(mut self, values: Vec<String>) -> Self {
        self.artifact_hashes.extend(values.into_iter().filter(|value| !value.trim().is_empty()));
        self
    }

    fn provenance_source_id(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.provenance_source_ids, value);
        self
    }

    fn replay_command(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.replay_commands, value);
        self
    }

    fn replay_commands(mut self, values: Vec<String>) -> Self {
        self.replay_commands.extend(values.into_iter().filter(|value| !value.trim().is_empty()));
        self
    }

    fn parent_link(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.parent_links, value.map(|item| normalize_locator(&item)));
        self
    }

    fn parent_links(mut self, values: Vec<String>) -> Self {
        self.parent_links.extend(values.into_iter().map(|item| normalize_locator(&item)));
        self
    }

    fn caveat(mut self, value: Option<String>) -> Self {
        insert_optional(&mut self.caveats, value);
        self
    }

    fn caveats(mut self, values: Vec<String>) -> Self {
        self.caveats.extend(values.into_iter().filter(|value| !value.trim().is_empty()));
        self
    }

    fn summary(mut self, value: String) -> Self {
        self.summary = value;
        self
    }

    fn build(self) -> EvidenceIndexRecord {
        let record_seed = format!(
            "family={}\nstable_id={}\nlocator={}\nartifact_blake3={}\n",
            self.artifact_family, self.stable_id, self.source.locator, self.source.blake3
        );
        EvidenceIndexRecord {
            record_id: format!("{}:{}", self.artifact_family, blake3_hex(record_seed.as_bytes())),
            artifact_family: self.artifact_family.to_owned(),
            stable_id: self.stable_id.to_owned(),
            artifact_locator: self.source.locator.clone(),
            artifact_blake3: self.source.blake3.clone(),
            evidence_tier: self.evidence_tier,
            source_ids: self.source_ids.into_iter().collect(),
            requirement_ids: self.requirement_ids.into_iter().collect(),
            lint_ids: self.lint_ids.into_iter().collect(),
            policy_ids: self.policy_ids.into_iter().collect(),
            fingerprints: self.fingerprints.into_iter().collect(),
            artifact_hashes: self.artifact_hashes.into_iter().collect(),
            provenance_source_ids: self.provenance_source_ids.into_iter().collect(),
            replay_commands: self.replay_commands.into_iter().collect(),
            parent_links: self.parent_links.into_iter().collect(),
            caveats: self.caveats.into_iter().collect(),
            summary: self.summary,
        }
    }
}

fn insert_optional(values: &mut BTreeSet<String>, value: Option<String>) {
    if let Some(value) = value.filter(|item| !item.trim().is_empty()) {
        values.insert(value);
    }
}

#[derive(Serialize)]
struct HashableIndex<'a> {
    schema_version: &'a str,
    normalization_version: &'a str,
    boundary: &'a str,
    source_artifact_count: usize,
    record_count: usize,
    records: &'a [EvidenceIndexRecord],
    issues: &'a [EvidenceIndexIssue],
}

impl PartialOrd for EvidenceIndexIssueSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EvidenceIndexIssueSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        issue_severity_rank(*self).cmp(&issue_severity_rank(*other))
    }
}

fn issue_severity_rank(severity: EvidenceIndexIssueSeverity) -> u8 {
    match severity {
        EvidenceIndexIssueSeverity::Error => 0,
        EvidenceIndexIssueSeverity::Caveat => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FINDING_LOCATOR: &str = "artifacts/octet/findings.jsonl";
    const PROVENANCE_LOCATOR: &str = "artifacts/octet/provenance.jsonl";
    const PROOF_CARDS_LOCATOR: &str = "artifacts/octet/proof-cards.json";
    const CAPSULE_LOCATOR: &str = "artifacts/octet/check-capsule.json";
    const TRANSCRIPT_LOCATOR: &str = "docs/examples/release-operator-smoke/transcript.md";
    const FINGERPRINT_LOCATOR: &str = "artifacts/octet/fingerprint-review.json";
    const EVIDENCE_LOG_LOCATOR: &str = "artifacts/octet/evidence-log-manifest.json";
    const RAIL_LOCATOR: &str = "artifacts/octet/evidence-rail-receipt.json";
    const REQUIREMENT_ID: &str = "r[octet.evidence_codebase_index.query.lookup]";
    const LINT_ID: &str = "ambient_clock";
    const POLICY_ID: &str = "octet.policy.capability-boundary";
    const FINDING_STABLE_ID: &str = "finding-stable-id";
    const FINGERPRINT_STABLE_ID: &str = "fingerprint-stable-id";
    const CAPSULE_HASH: &str = "capsule-hash-fixture";

    #[test]
    fn deterministic_build_has_stable_records_and_hash() {
        let inputs = fixture_inputs();
        let first = build_evidence_codebase_index(&inputs, EvidenceIndexBuildMode::Strict);
        let second = build_evidence_codebase_index(&inputs, EvidenceIndexBuildMode::Strict);

        assert_eq!(first, second);
        assert!(first.issues.iter().all(|issue| issue.severity == EvidenceIndexIssueSeverity::Caveat));
        assert!(first.records.iter().any(|record| record.artifact_family == "octet-finding"));
        assert!(first.records.iter().any(|record| record.artifact_family == "octet-proof-card"));
        assert!(first.records.iter().any(|record| record.artifact_family == "octet-replay-capsule"));
        assert!(first.records.iter().any(|record| record.artifact_family == "octet-transcript-markdown"));
        assert!(first.index_hash.len() > INDEX_SCHEMA_VERSION.len());
    }

    #[test]
    fn absolute_root_prefixes_do_not_change_index_hash() {
        let mut left = fixture_inputs();
        let mut right = fixture_inputs();
        for input in &mut left {
            input.locator = format!("/tmp/review-a/{}", input.locator);
        }
        for input in &mut right {
            input.locator = format!("/home/review-b/work/{}", input.locator);
        }

        let left_index = build_evidence_codebase_index(&left, EvidenceIndexBuildMode::Strict);
        let right_index = build_evidence_codebase_index(&right, EvidenceIndexBuildMode::Strict);

        assert_eq!(left_index.index_hash, right_index.index_hash);
        assert_eq!(left_index.records, right_index.records);
    }

    #[test]
    fn malformed_and_missing_identity_artifacts_fail_closed_in_strict_mode() {
        let inputs = vec![
            EvidenceArtifactInput {
                locator: "artifacts/octet/malformed.json".to_owned(),
                bytes: b"{".to_vec(),
            },
            EvidenceArtifactInput {
                locator: FINDING_LOCATOR.to_owned(),
                bytes: br#"{"version":2,"lint":"ambient_clock","evidence_tier":"static_lint"}"#.to_vec(),
            },
            EvidenceArtifactInput {
                locator: "artifacts/octet/unsupported.json".to_owned(),
                bytes: br#"{"hello":"world"}"#.to_vec(),
            },
        ];

        let index = build_evidence_codebase_index(&inputs, EvidenceIndexBuildMode::Strict);

        assert!(index.issues.iter().any(|issue| issue.code == MALFORMED_JSON_CODE));
        assert!(index.issues.iter().any(|issue| issue.code == "finding.missing_stable_id"));
        assert!(index.issues.iter().any(|issue| issue.code == UNSUPPORTED_FAMILY_CODE));
        assert!(index.issues.iter().any(|issue| issue.severity == EvidenceIndexIssueSeverity::Error));
    }

    #[test]
    fn advisory_mode_keeps_partial_caveats_without_panic() {
        let inputs = vec![EvidenceArtifactInput {
            locator: "artifacts/octet/unsupported.json".to_owned(),
            bytes: br#"{"hello":"world"}"#.to_vec(),
        }];

        let index = build_evidence_codebase_index(&inputs, EvidenceIndexBuildMode::Advisory);

        assert_eq!(index.record_count, 0);
        assert!(index.issues.iter().all(|issue| issue.severity == EvidenceIndexIssueSeverity::Caveat));
    }

    #[test]
    fn stale_provenance_hash_is_reported() {
        let artifact = EvidenceArtifactInput {
            locator: "artifacts/octet/status.json".to_owned(),
            bytes: br#"{"status":"clean"}"#.to_vec(),
        };
        let provenance = EvidenceArtifactInput {
            locator: PROVENANCE_LOCATOR.to_owned(),
            bytes: br#"{"schema_version":"octet-provenance-record/v1","source_id":"octet.runner.artifact-bundle","checker_id":"cargo-octet","evidence_tier":"manual_review_required","artifact_locator":"artifacts/octet/status.json","blake3":"stale","checked_scope":"fixture","human_action":"review"}
"#.to_vec(),
        };

        let index = build_evidence_codebase_index(&[artifact, provenance], EvidenceIndexBuildMode::Strict);

        assert!(index.issues.iter().any(|issue| issue.code == "provenance.blake3_mismatch"));
    }

    #[test]
    fn query_returns_direct_and_linked_records() {
        let index = build_evidence_codebase_index(&fixture_inputs(), EvidenceIndexBuildMode::Strict);
        let report = query_evidence_index(&index, REQUIREMENT_ID);

        assert!(report.records.iter().any(|record| record.stable_id == FINDING_STABLE_ID));
        assert!(report.records.iter().any(|record| record.policy_ids.contains(&POLICY_ID.to_owned())));
        assert!(report.records.iter().any(|record| record.lint_ids.contains(&LINT_ID.to_owned())));
        assert!(report.records.iter().any(|record| record.stable_id == CAPSULE_HASH));
        assert!(report.boundary.contains("identity and linkage only"));
    }

    #[test]
    fn html_surface_has_links_and_non_claim_boundary() {
        let index = build_evidence_codebase_index(&fixture_inputs(), EvidenceIndexBuildMode::Strict);
        let html = render_evidence_index_html(&index);

        assert!(html.contains("Octet evidence codebase index"));
        assert!(html.contains("href=\"#octet-finding-"));
        assert!(html.contains("identity and linkage only"));
        assert!(output_has_required_boundary(&html));
    }

    fn fixture_inputs() -> Vec<EvidenceArtifactInput> {
        let finding = format!(
            "{{\"version\":2,\"index\":\"F1\",\"stable_id\":\"{FINDING_STABLE_ID}\",\"semantic_id\":\"semantic-fixture\",\"identity_status\":\"computed\",\"normalized_snippet_hash\":\"snippet-fixture\",\"config_hash\":\"config-fixture\",\"cache_status\":\"disabled\",\"lint\":\"{LINT_ID}\",\"group\":\"tigerstyle::purity\",\"severity\":\"warning\",\"file\":\"src/lib.rs\",\"line\":1,\"column\":1,\"message\":\"{REQUIREMENT_ID}\",\"principle\":\"Functional Core\",\"effect_family\":\"time\",\"evidence_tier\":\"static_lint\",\"crate_name\":\"fixture\"}}\n"
        );
        let proof_cards = format!(
            "{{\"schema_version\":\"octet-proof-cards/v1\",\"cards\":[{{\"proof_or_property_id\":\"{POLICY_ID}#rail-0\",\"source_requirement_or_policy_id\":\"{REQUIREMENT_ID}\",\"policy_id\":\"{POLICY_ID}\",\"rail_index\":0,\"checked_rust_source_scope\":\"src/lib.rs\",\"source_paths\":[\"src/lib.rs\"],\"proof_target_paths\":[\"proof.rs\"],\"verifier_mode\":\"fixture\",\"verifier_identity_summary\":\"fixture\",\"solver_identity_summary\":\"fixture\",\"toolchain_identity_summary\":\"fixture\",\"receipt_path\":\"artifacts/octet/proofs/policy.receipt\",\"artifact_blake3_identities\":[{{\"artifact\":\"artifacts/octet/proofs/policy.receipt\",\"blake3\":\"proof-hash\"}}],\"evidence_tier\":\"formal_proof\",\"evidence_origin\":\"octet-rerun\",\"status\":\"passed\",\"trust_boundaries\":[],\"non_claims\":[\"not whole-program correctness\"],\"artifact_verifier_command\":\"cargo octet artifact verify --artifact-dir artifacts/octet\",\"provenance_artifact\":\"artifacts/octet/provenance.jsonl\",\"diagnostics\":[]}}]}}"
        );
        let capsule = format!(
            "{{\"schema_version\":1,\"capsule_hash\":\"{CAPSULE_HASH}\",\"capsule_kind\":\"octet-replay-capsule\",\"manifest\":{{\"command_argv\":[\"cargo\",\"octet\",\"check\"],\"run_kind\":\"check\",\"octet_binary_identity\":\"cargo-octet\",\"config_hash\":\"config-fixture\",\"input_hashes\":[\"input-hash\"],\"output_hashes\":[\"{FINDING_STABLE_ID}\"],\"expected_exit_code\":0}},\"replay_contract\":{{}},\"redaction_status\":\"normalized\"}}"
        );
        let transcript = "$ cargo octet check\n=> pass\n".to_owned();
        let fingerprint = format!(
            "{{\"schema_version\":1,\"tool_name\":\"cargo-octet\",\"tool_version\":\"0.1.0\",\"mode\":\"check\",\"input_paths\":[\"src/lib.rs\"],\"manifest_hash\":\"manifest-fixture\",\"records\":[{{\"stable_id\":\"{FINGERPRINT_STABLE_ID}\",\"path\":\"src/lib.rs\",\"name\":\"checked\",\"status\":\"match\",\"computed_hash\":\"fingerprint-hash\",\"normalized_body_hash\":\"body-hash\",\"semantic_id\":\"semantic-fixture\",\"dependency_set_hash\":\"dependency-hash\",\"dependency_identity_status\":\"source-scanner-token-set\",\"tool_version\":\"0.1.0\",\"review\":\"{REQUIREMENT_ID}\",\"evidence\":\"{POLICY_ID}\",\"requires_review\":false,\"macro_warning\":false}}]}}"
        );
        let evidence_log = "{\"schema_version\":\"octet-evidence-log-manifest/v1\",\"logs\":[{\"artifact_locator\":\"run.txt.gz\",\"blake3\":\"log-hash\",\"bytes\":10,\"replay_command\":\"cargo octet check\",\"checked_scope\":\"fixture\",\"capture_source\":\"test\",\"retention\":\"compressed\"}]}".to_owned();
        let rail = format!(
            "{{\"schema_version\":\"octet-evidence-rail-receipt/v1\",\"rail_id\":\"{POLICY_ID}\",\"checker_id\":\"cargo-deny\",\"mode\":\"import\",\"command_argv\":[\"cargo\",\"deny\",\"check\"],\"workspace_scope\":\".\",\"checked_rule_families\":[\"licenses\"],\"status\":\"passed\",\"finding_counts\":{{}},\"bounded_log_identity\":\"rail-log-hash\",\"imported_input_identity\":\"rail-input-hash\",\"non_claims\":[\"not correctness\"],\"diagnostics\":[],\"findings\":[{{\"family\":\"{LINT_ID}\",\"severity\":\"warning\",\"message\":\"fixture\",\"file\":\"src/lib.rs\"}}],\"parent_artifacts\":[\"{CAPSULE_HASH}\"]}}"
        );
        let provenance = format!(
            "{{\"schema_version\":\"octet-provenance-record/v1\",\"source_id\":\"octet.transcript.assertion\",\"checker_id\":\"octet-transcript\",\"evidence_tier\":\"transcript_assertion\",\"artifact_locator\":\"{TRANSCRIPT_LOCATOR}\",\"blake3\":\"{}\",\"checked_scope\":\"transcript\",\"parent_artifacts\":[\"{CAPSULE_HASH}\"]}}\n",
            blake3_hex(transcript.as_bytes())
        );

        vec![
            input(FINDING_LOCATOR, finding),
            input(PROVENANCE_LOCATOR, provenance),
            input(PROOF_CARDS_LOCATOR, proof_cards),
            input(CAPSULE_LOCATOR, capsule),
            input(TRANSCRIPT_LOCATOR, transcript),
            input(FINGERPRINT_LOCATOR, fingerprint),
            input(EVIDENCE_LOG_LOCATOR, evidence_log),
            input(RAIL_LOCATOR, rail),
        ]
    }

    fn input(locator: &str, text: String) -> EvidenceArtifactInput {
        EvidenceArtifactInput {
            locator: locator.to_owned(),
            bytes: text.into_bytes(),
        }
    }
}
