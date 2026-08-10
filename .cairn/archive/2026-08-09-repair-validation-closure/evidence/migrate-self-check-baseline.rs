#!/usr/bin/env -S nix develop . -c cargo -q -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.3"
serde = { version = "=1.0.228", features = ["derive"] }
serde_json = "=1.0.149"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use serde_json::Value;

const EXECUTE_ARG: &str = "--execute";
const SELF_TEST_ARG: &str = "--self-test";
const COMPARE_OCCURRENCES_ARG: &str = "--compare-occurrences";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const SUCCESS_EXIT_CODE: u8 = 0;
const FAILURE_EXIT_CODE: u8 = 1;
const SELF_TEST_ARG_COUNT: usize = 2;
const DRY_RUN_ARG_COUNT: usize = 5;
const EXECUTE_ARG_COUNT: usize = 6;
const COMPARE_OCCURRENCES_ARG_COUNT: usize = 4;
const SELF_TEST_OLD_TOTAL: usize = 2;
const EXPECTED_OLD_TOTAL: usize = 1_815;
const EXPECTED_CANDIDATE_TOTAL: usize = 1_813;
const EXPECTED_REKEY_TOTAL: usize = 13;
const EXPECTED_REMOVAL_TOTAL: usize = 2;
const REMOVED_FINDING_IDS: &[&str] = &["b3:5cda7b996474af06", "b3:a7bc26bfc639e5fe"];
const EXCESSIVE_FILE_LENGTH_LINT: &str = "excessive_file_length";
const MODULE_FILE_COUNT_LINT: &str = "module_file_count";
const NORMALIZED_FILE_LENGTH_MESSAGE: &str = "source file exceeds configured line limit";
const NORMALIZED_MODULE_COUNT: &str = "<count>";
const BASELINE_FIELDS: &[&str] = &[
    "stable_id",
    "semantic_id",
    "identity_status",
    "lint",
    "group",
    "severity",
    "file",
    "line",
    "column",
    "message",
    "crate_name",
];

#[derive(Clone, Copy)]
struct Limits {
    old_total: usize,
    candidate_total: usize,
    rekey_total: usize,
    removal_total: usize,
}

struct Transition {
    candidate_findings: Vec<Value>,
    rekeys: Vec<(String, String)>,
    removals: Vec<String>,
}

#[derive(serde::Serialize)]
struct OutputMetadata<'a> {
    tool_name: &'a str,
    tool_version: &'a str,
    rustc_version: &'a str,
    toolchain: &'a str,
    profile_name: &'a str,
    profile_hash: &'a str,
    config_hash: &'a str,
}

#[derive(serde::Serialize)]
struct OutputFinding<'a> {
    stable_id: &'a str,
    semantic_id: &'a str,
    identity_status: &'a str,
    lint: &'a str,
    group: &'a str,
    severity: &'a str,
    file: &'a str,
    line: u64,
    column: u64,
    message: &'a str,
    crate_name: &'a str,
}

#[derive(serde::Serialize)]
struct OutputBaseline<'a> {
    schema_version: u64,
    finding_schema_version: u64,
    metadata: OutputMetadata<'a>,
    baseline_hash: String,
    findings: Vec<OutputFinding<'a>>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::from(SUCCESS_EXIT_CODE),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == SELF_TEST_ARG_COUNT && args[1] == SELF_TEST_ARG {
        self_test()?;
        println!("self-check baseline migration self-test passed");
        return Ok(());
    }
    if args.len() == COMPARE_OCCURRENCES_ARG_COUNT && args[1] == COMPARE_OCCURRENCES_ARG {
        let old_results = read_json_lines(Path::new(&args[2]))?;
        let candidate_results = read_json_lines(Path::new(&args[3]))?;
        compare_occurrences(&old_results, &candidate_results)?;
        return Ok(());
    }
    if args.len() != DRY_RUN_ARG_COUNT && args.len() != EXECUTE_ARG_COUNT {
        return Err(format!(
            "usage: {} OLD_BASELINE RESULTS_JSONL STATUS_JSON OUTPUT [{EXECUTE_ARG}] | {SELF_TEST_ARG}",
            args[0]
        ));
    }
    let execute = args.len() == EXECUTE_ARG_COUNT && args[5] == EXECUTE_ARG;
    if args.len() == EXECUTE_ARG_COUNT && !execute {
        return Err(format!("unsupported argument: {}", args[5]));
    }

    let old = read_json(Path::new(&args[1]))?;
    let results = read_json_lines(Path::new(&args[2]))?;
    let status = read_json(Path::new(&args[3]))?;
    let limits = Limits {
        old_total: EXPECTED_OLD_TOTAL,
        candidate_total: EXPECTED_CANDIDATE_TOTAL,
        rekey_total: EXPECTED_REKEY_TOTAL,
        removal_total: EXPECTED_REMOVAL_TOTAL,
    };
    let transition = validate_transition(&old, results, &status, limits, REMOVED_FINDING_IDS)?;
    print_summary(&transition);
    if execute {
        write_baseline(Path::new(&args[4]), &old, &status, transition.candidate_findings)?;
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn read_json_lines(path: &Path) -> Result<Vec<Value>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("parse {}: {error}", path.display())))
        .collect()
}

fn findings(value: &Value) -> Result<&Vec<Value>, String> {
    value
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline is missing findings".to_owned())
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("finding is missing string field {field}"))
}

fn required_usize(value: &Value, field: &str) -> Result<usize, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
        .ok_or_else(|| format!("value is missing bounded integer field {field}"))
}

fn finding_map(values: &[Value]) -> Result<BTreeMap<String, Value>, String> {
    let mut mapped = BTreeMap::new();
    for value in values {
        let stable_id = required_string(value, "stable_id")?.to_owned();
        if !mapped.contains_key(&stable_id) {
            mapped.insert(stable_id, project_finding(value)?);
        }
    }
    Ok(mapped)
}

fn project_finding(value: &Value) -> Result<Value, String> {
    let mut object = serde_json::Map::new();
    for field in BASELINE_FIELDS {
        let projected = value.get(*field).cloned().ok_or_else(|| format!("finding is missing field {field}"))?;
        object.insert((*field).to_owned(), projected);
    }
    Ok(Value::Object(object))
}

fn semantic_key(finding: &Value) -> Result<String, String> {
    let lint = required_string(finding, "lint")?;
    let message = normalized_message(lint, required_string(finding, "message")?);
    Ok([
        required_string(finding, "identity_status")?.to_owned(),
        lint.to_owned(),
        required_string(finding, "group")?.to_owned(),
        required_string(finding, "severity")?.to_owned(),
        required_string(finding, "file")?.to_owned(),
        required_string(finding, "crate_name")?.to_owned(),
        message,
    ]
    .join("\u{1f}"))
}

fn normalized_message(lint: &str, message: &str) -> String {
    if lint == EXCESSIVE_FILE_LENGTH_LINT {
        return NORMALIZED_FILE_LENGTH_MESSAGE.to_owned();
    }
    if lint == MODULE_FILE_COUNT_LINT
        && let Some((prefix, rest)) = message.split_once(" contains ")
        && let Some((_count, suffix)) = rest.split_once(" Rust source files")
    {
        return format!("{prefix} contains {NORMALIZED_MODULE_COUNT} Rust source files{suffix}");
    }
    message.to_owned()
}

fn status_ids(status: &Value, field: &str) -> Result<BTreeSet<String>, String> {
    status
        .get("baseline")
        .and_then(|baseline| baseline.get(field))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("status is missing baseline.{field}"))?
        .iter()
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(|| format!("baseline.{field} contains a non-string")))
        .collect()
}

fn ensure_new_ids_are_consistent(results: &[Value], new_ids: &BTreeSet<String>) -> Result<(), String> {
    let mut semantics = BTreeMap::<String, BTreeSet<String>>::new();
    for finding in results {
        let stable_id = required_string(finding, "stable_id")?;
        if new_ids.contains(stable_id) {
            semantics.entry(stable_id.to_owned()).or_default().insert(semantic_key(finding)?);
        }
    }
    for (stable_id, keys) in semantics {
        if keys.len() != 1 {
            return Err(format!("new stable ID {stable_id} has conflicting semantic findings"));
        }
    }
    Ok(())
}

fn occurrence_counts(findings: &[Value]) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for finding in findings {
        let key = semantic_key(finding)?;
        *counts.entry(key).or_insert(0) += 1;
    }
    Ok(counts)
}

fn compare_occurrences(old: &[Value], candidate: &[Value]) -> Result<(), String> {
    let old_counts = occurrence_counts(old)?;
    let candidate_counts = occurrence_counts(candidate)?;
    let mut growth = Vec::new();
    let mut reductions = Vec::new();
    let keys = old_counts.keys().chain(candidate_counts.keys()).cloned().collect::<BTreeSet<_>>();
    for key in keys {
        let old_count = old_counts.get(&key).copied().unwrap_or_default();
        let candidate_count = candidate_counts.get(&key).copied().unwrap_or_default();
        if candidate_count > old_count {
            growth.push(format!("old={old_count} candidate={candidate_count} {}", key.replace('\u{1f}', " | ")));
        } else if candidate_count < old_count {
            reductions.push(format!("old={old_count} candidate={candidate_count} {}", key.replace('\u{1f}', " | ")));
        }
    }
    println!(
        "old_occurrences={} candidate_occurrences={} growth_groups={} reduction_groups={}",
        old.len(),
        candidate.len(),
        growth.len(),
        reductions.len()
    );
    for diagnostic in &growth {
        println!("GROWTH {diagnostic}");
    }
    for diagnostic in &reductions {
        println!("REDUCTION {diagnostic}");
    }
    if growth.is_empty() {
        Ok(())
    } else {
        Err("candidate occurrence counts exceed the accepted run".to_owned())
    }
}

fn validate_transition(
    old: &Value,
    results: Vec<Value>,
    status: &Value,
    limits: Limits,
    allowed_removals: &[&str],
) -> Result<Transition, String> {
    let old_map = finding_map(findings(old)?)?;
    let candidate_map = finding_map(&results)?;
    if old_map.len() != limits.old_total || candidate_map.len() != limits.candidate_total {
        return Err(format!(
            "unexpected identity totals: old={} candidate={} expected_old={} expected_candidate={}",
            old_map.len(),
            candidate_map.len(),
            limits.old_total,
            limits.candidate_total
        ));
    }
    if required_usize(status.get("baseline").ok_or_else(|| "status is missing baseline".to_owned())?, "current_total")?
        != candidate_map.len()
    {
        return Err("status current_total does not match candidate identities".to_owned());
    }

    let new_ids = status_ids(status, "new_finding_ids")?;
    let stale_ids = status_ids(status, "stale_finding_ids")?;
    ensure_new_ids_are_consistent(&results, &new_ids)?;
    let allowed = allowed_removals.iter().map(|id| (*id).to_owned()).collect::<BTreeSet<_>>();
    let removals = stale_ids.intersection(&allowed).cloned().collect::<Vec<_>>();
    if removals.len() != limits.removal_total {
        return Err(format!("reviewed removal mismatch: {removals:?}"));
    }

    let mut candidate_by_key = BTreeMap::<String, String>::new();
    for new_id in &new_ids {
        let finding = candidate_map.get(new_id).ok_or_else(|| format!("new ID {new_id} is absent from results"))?;
        let key = semantic_key(finding)?;
        if candidate_by_key.insert(key, new_id.clone()).is_some() {
            return Err("multiple new identities share one migration semantic key".to_owned());
        }
    }

    let mut rekeys = Vec::new();
    let mut paired_new = BTreeSet::new();
    for stale_id in &stale_ids {
        if allowed.contains(stale_id) {
            continue;
        }
        let stale = old_map.get(stale_id).ok_or_else(|| format!("stale ID {stale_id} is absent from baseline"))?;
        let key = semantic_key(stale)?;
        let new_id = candidate_by_key
            .get(&key)
            .ok_or_else(|| format!("stale ID {stale_id} has no exact semantic replacement"))?;
        if !paired_new.insert(new_id.clone()) {
            return Err(format!("new ID {new_id} maps from multiple stale identities"));
        }
        rekeys.push((stale_id.clone(), new_id.clone()));
    }
    if rekeys.len() != limits.rekey_total || paired_new != new_ids {
        let unpaired = new_ids.difference(&paired_new).cloned().collect::<Vec<_>>();
        return Err(format!("semantic rekey mismatch: rekeys={} unpaired_new={unpaired:?}", rekeys.len()));
    }

    Ok(Transition {
        candidate_findings: candidate_map.into_values().collect(),
        rekeys,
        removals,
    })
}

fn baseline_hash(stable_ids: &[String]) -> String {
    let payload = serde_json::json!({
        "schema_version": BASELINE_SCHEMA_VERSION,
        "stable_ids": stable_ids,
    });
    format!("b3:{}", blake3::hash(payload.to_string().as_bytes()).to_hex())
}

fn required_u64(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("value is missing integer field {field}"))
}

fn output_metadata(status: &Value) -> Result<OutputMetadata<'_>, String> {
    let metadata = status.get("metadata").ok_or_else(|| "status lacks metadata".to_owned())?;
    Ok(OutputMetadata {
        tool_name: required_string(metadata, "tool_name")?,
        tool_version: required_string(metadata, "tool_version")?,
        rustc_version: required_string(metadata, "rustc_version")?,
        toolchain: required_string(metadata, "toolchain")?,
        profile_name: required_string(metadata, "profile_name")?,
        profile_hash: required_string(metadata, "profile_hash")?,
        config_hash: required_string(metadata, "config_hash")?,
    })
}

fn output_finding(finding: &Value) -> Result<OutputFinding<'_>, String> {
    Ok(OutputFinding {
        stable_id: required_string(finding, "stable_id")?,
        semantic_id: required_string(finding, "semantic_id")?,
        identity_status: required_string(finding, "identity_status")?,
        lint: required_string(finding, "lint")?,
        group: required_string(finding, "group")?,
        severity: required_string(finding, "severity")?,
        file: required_string(finding, "file")?,
        line: required_u64(finding, "line")?,
        column: required_u64(finding, "column")?,
        message: required_string(finding, "message")?,
        crate_name: required_string(finding, "crate_name")?,
    })
}

fn write_baseline(path: &Path, old: &Value, status: &Value, findings: Vec<Value>) -> Result<(), String> {
    let stable_ids = findings
        .iter()
        .map(|finding| required_string(finding, "stable_id").map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    let output_findings = findings.iter().map(output_finding).collect::<Result<Vec<_>, _>>()?;
    let output = OutputBaseline {
        schema_version: required_u64(old, "schema_version")?,
        finding_schema_version: required_u64(old, "finding_schema_version")?,
        metadata: output_metadata(status)?,
        baseline_hash: baseline_hash(&stable_ids),
        findings: output_findings,
    };
    let mut text = serde_json::to_string_pretty(&output).map_err(|error| format!("serialize baseline: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn print_summary(transition: &Transition) {
    println!(
        "candidate={} rekeys={} removals={}",
        transition.candidate_findings.len(),
        transition.rekeys.len(),
        transition.removals.len()
    );
    for (old_id, new_id) in &transition.rekeys {
        println!("rekey {old_id} -> {new_id}");
    }
    for removed_id in &transition.removals {
        println!("removed {removed_id}");
    }
}

fn fixture_finding(stable_id: &str, lint: &str, message: &str) -> Value {
    serde_json::json!({
        "stable_id": stable_id,
        "semantic_id": stable_id,
        "identity_status": "computed",
        "lint": lint,
        "group": "tigerstyle::structure",
        "severity": "warning",
        "file": "src/lib.rs",
        "line": 1,
        "column": 1,
        "message": message,
        "crate_name": "fixture",
    })
}

fn self_test() -> Result<(), String> {
    let removed_id = REMOVED_FINDING_IDS[0];
    let old = serde_json::json!({
        "findings": [
            fixture_finding("b3:old", EXCESSIVE_FILE_LENGTH_LINT, "source file is 10 lines (limit: 5)"),
            fixture_finding(removed_id, "non_trait_imports", "resolved import"),
        ]
    });
    let candidate = vec![fixture_finding(
        "b3:new",
        EXCESSIVE_FILE_LENGTH_LINT,
        "source file is 12 lines (limit: 5)",
    )];
    let status = serde_json::json!({
        "baseline": {
            "current_total": 1,
            "new_finding_ids": ["b3:new"],
            "stale_finding_ids": ["b3:old", removed_id],
        }
    });
    let limits = Limits {
        old_total: SELF_TEST_OLD_TOTAL,
        candidate_total: 1,
        rekey_total: 1,
        removal_total: 1,
    };
    validate_transition(&old, candidate.clone(), &status, limits, &[removed_id])?;

    let mut drift = candidate;
    drift.push(fixture_finding("b3:unknown", "new_lint", "unreviewed"));
    let drift_status = serde_json::json!({
        "baseline": {
            "current_total": SELF_TEST_OLD_TOTAL,
            "new_finding_ids": ["b3:new", "b3:unknown"],
            "stale_finding_ids": ["b3:old", removed_id],
        }
    });
    let drift_limits = Limits {
        candidate_total: SELF_TEST_OLD_TOTAL,
        ..limits
    };
    if validate_transition(&old, drift, &drift_status, drift_limits, &[removed_id]).is_ok() {
        return Err("negative unreviewed candidate fixture was accepted".to_owned());
    }

    let occurrence = fixture_finding("b3:occurrence", "demo", "same");
    compare_occurrences(&[occurrence.clone(), occurrence.clone()], std::slice::from_ref(&occurrence))?;
    if compare_occurrences(std::slice::from_ref(&occurrence), &[occurrence.clone(), occurrence.clone()]).is_ok() {
        return Err("negative occurrence-growth fixture was accepted".to_owned());
    }
    Ok(())
}
