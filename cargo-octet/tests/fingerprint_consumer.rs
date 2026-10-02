use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_octet() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-octet")
}

fn write_consumer_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-consumer-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r#"// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/check.txt
pub fn checked_add_one(input: u32) -> u32 {
    input + 1
}

// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/macro.txt; macro=allow
pub fn rendered(input: u32) -> String {
    format!("value={input}")
}
"#,
    )
    .expect("write lib.rs");
}

fn write_selective_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-selective-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r#"// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/check.txt
#[inline]
pub const fn checked_age(current_ms: u64, started_ms: u64) -> u64 {
    current_ms.saturating_sub(started_ms)
}

#[cfg(test)]
mod tests {
    #[test]
    fn macro_heavy_unmarked_test_is_not_part_of_selective_check() {
        assert_eq!(format!("{}", 1), "1");
    }
}
"#,
    )
    .expect("write lib.rs");
}

fn write_macro_generated_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-macro-generated-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r"macro_rules! define_parametric {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input + 1
        }
    };
}

macro_rules! define_literal {
    () => {
        pub fn literal_generated() -> u32 {
            7
        }
    };
}

define_parametric!(generated_checked);
define_literal!();

// tigerstyle-fingerprint: b3:0; review=explicit-wrapper; evidence=evidence/wrapper; macro=allow
pub fn explicit_wrapper(input: u32) -> u32 {
    generated_checked(input)
}
",
    )
    .expect("write lib.rs");
}

fn write_directory_fixture(root: &Path) {
    let src_dir = root.join("src");
    let nested_dir = src_dir.join("nested");
    let tests_dir = root.join("tests");
    fs::create_dir_all(&nested_dir).expect("create nested src dir");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-directory-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r"pub mod nested;

macro_rules! define_generated {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input + 1
        }
    };
}

define_generated!(generated_from_root);

// tigerstyle-fingerprint: b3:0; review=directory-root-plain; evidence=evidence/root-plain
pub fn root_plain(input: u32) -> u32 {
    input.saturating_add(1)
}

// tigerstyle-fingerprint: b3:0; review=directory-root-macro; evidence=evidence/root-macro; macro=allow
#[inline]
pub fn root_with_macro(input: u32) -> u32 {
    assert!(input < 10);
    input + 2
}
",
    )
    .expect("write lib.rs");
    fs::write(
        nested_dir.join("mod.rs"),
        r#"trait Reader {
    fn read_value(&self) -> u32;
}

pub struct FixedReader;

impl Reader for FixedReader {
    // tigerstyle-fingerprint: b3:0; review=directory-trait-impl; evidence=evidence/trait-impl
    fn read_value(&self) -> u32 {
        7
    }
}

// tigerstyle-fingerprint: b3:0; review=directory-attributed; evidence=evidence/attributed
#[allow(
    dead_code,
    reason = "directory-wide fingerprint smoke"
)]
pub fn attributed_nested(input: u32) -> u32 {
    input * 2
}

#[cfg(test)]
mod tests {
    // tigerstyle-fingerprint: b3:0; review=directory-cfg-test; evidence=evidence/cfg-test; macro=allow
    #[test]
    fn nested_test_helper() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    )
    .expect("write nested mod.rs");
    fs::write(
        tests_dir.join("integration.rs"),
        r"macro_rules! literal_test_fn {
    () => {
        fn generated_test_literal() -> u32 {
            99
        }
    };
}

literal_test_fn!();

// tigerstyle-fingerprint: b3:0; review=directory-integration-helper; evidence=evidence/integration-helper
fn explicit_test_helper(input: u32) -> u32 {
    input + 3
}
",
    )
    .expect("write integration fixture");
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

#[test]
fn release_operator_smoke_transcript_and_help_cover_operator_surfaces() {
    let root = repo_root();
    let transcript_path = root.join("docs/examples/release-operator-smoke/transcript.md");
    let transcript = fs::read_to_string(&transcript_path).expect("read release smoke transcript");

    let transcript_output = Command::new(cargo_octet())
        .args(["transcript", transcript_path.to_str().expect("transcript path utf8")])
        .output()
        .expect("run release smoke transcript validation");
    assert!(
        transcript_output.status.success(),
        "release smoke transcript stderr: {}",
        String::from_utf8_lossy(&transcript_output.stderr)
    );
    let transcript_stdout = String::from_utf8_lossy(&transcript_output.stdout);
    assert!(transcript_stdout.contains("command_blocks=7"));
    assert!(transcript_stdout.contains("expected_output_blocks=8"));
    assert!(transcript_stdout.contains("status=passed"));

    for surface in [
        "artifact verify",
        "fingerprint",
        "migrate",
        "effect-summary",
        "lint-cache",
        "transcript",
        "roadmap",
    ] {
        assert!(transcript.contains(surface), "release smoke transcript missing {surface}");
    }

    let help_output = Command::new(cargo_octet()).arg("--help").output().expect("run root help");
    assert!(help_output.status.success(), "root help should succeed");
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    for surface in [
        "artifact verify",
        "fingerprint",
        "migrate",
        "effect-summary",
        "lint-cache",
        "transcript",
        "roadmap",
    ] {
        assert!(help_stdout.contains(surface), "root help missing {surface}");
    }
}

#[test]
fn fingerprint_review_manifest_is_stable_and_sorted() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let manifest = temp.path().join("review-manifest.json");

    let output = Command::new(cargo_octet())
        .args([
            "fingerprint",
            "--check",
            "--review-manifest",
            manifest.to_str().expect("manifest path utf8"),
            "src/lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint review manifest");
    assert!(!output.status.success(), "zero hashes should require review/update");

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).expect("read manifest")).expect("parse manifest");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["tool_name"], "cargo-octet");
    assert_eq!(json["mode"], "check");
    assert!(json["manifest_hash"].as_str().expect("manifest hash").starts_with("b3:"));

    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["name"], "checked_add_one");
    assert_eq!(records[1]["name"], "rendered");
    for record in records {
        assert!(record["stable_id"].as_str().expect("stable id").starts_with("b3:"));
        assert_eq!(record["requires_review"], true);
    }

    let provenance_path = manifest.with_extension("provenance.jsonl");
    let provenance_line = fs::read_to_string(&provenance_path).expect("read fingerprint provenance sidecar");
    let provenance: serde_json::Value = serde_json::from_str(provenance_line.trim()).expect("parse provenance sidecar");
    assert_eq!(provenance["checker_id"], "octet-fingerprint");
    assert_eq!(provenance["source_id"], "octet.fingerprint.review-manifest");
    assert_eq!(provenance["evidence_tier"], "manual_review_required");
    assert_eq!(provenance["artifact_locator"], manifest.display().to_string());
    assert!(provenance["human_action"].as_str().expect("human action").contains("review fingerprint manifest"));
}

#[test]
fn fingerprint_write_then_check_works_for_external_crate_file() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| record["status"] == "match"));
    assert!(records.iter().all(|record| record["review"] == "safety-critical"));
    assert!(records.iter().any(|record| record["macro_calls"] == true && record["macro_warning"] == false));
}

#[test]
fn fingerprint_update_then_check_only_tracks_marked_functions() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_selective_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "checked_age");
    assert_eq!(records[0]["status"], "match");
    assert_eq!(records[0]["macro_warning"], false);
}

#[test]
fn fingerprint_write_skips_functions_inside_macro_rules_definitions() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_macro_generated_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let json: serde_json::Value = serde_json::from_slice(&write.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_wrapper");
    assert_eq!(records[0]["status"], "written");

    let updated = fs::read_to_string(&fixture).expect("read updated fixture");
    assert!(!updated.contains("review=TODO"));
    assert_eq!(updated.matches("octet-fingerprint").count(), 1);

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn fingerprint_marker_detects_code_after_quoted_slashes() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    let reviewed = "pub fn fetch() -> u32 {\n    let _endpoint = \"https://example.invalid/api\"; 1\n}\n";
    fs::write(&fixture, reviewed).expect("write reviewed fixture");
    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));
    let marked = fs::read_to_string(&fixture)
        .expect("read marked fixture")
        .replace("review=TODO; evidence=TODO", "review=R-1; evidence=E-1");
    let check = |text: String| {
        fs::write(&fixture, text).expect("write edited fixture");
        let output = Command::new(cargo_octet())
            .args(["fingerprint", "--check", "--output-format=json"])
            .arg(&fixture)
            .output()
            .expect("run fingerprint check");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check json");
        (output.status.success(), json["records"][0]["status"].clone())
    };

    assert_eq!(check(marked.replace("; 1", "; 1 // still one")), (true, serde_json::json!("match")));
    assert_eq!(check(marked.replace("; 1", "; std::process::abort()")), (false, serde_json::json!("mismatch")));
}

#[test]
fn fingerprint_rejects_untagged_v1_markers() {
    // v1 marker hashes were untagged BLAKE3 over token text that split literal
    // contents into tokens, so `"batch=1"` hashed like `" batch = 1 "`.
    let legacy = |tokens: &str| blake3::hash(tokens.as_bytes()).to_hex().to_string();
    let capped = legacy("pub fn capped_batch_size ( requested : u32 ) - > u32 { requested . min ( 128 ) }");
    let label = legacy("pub fn label ( ) - > & ' static str { \" batch = 1 \" }");
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    let source = r#"// octet-fingerprint: b3:CAPPED; review=R-1; evidence=E-1
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(128)
}

// octet-fingerprint: b3:LABEL; review=R-1; evidence=E-1
pub fn label() -> &'static str {
    " batch = 1 "
}
"#;
    fs::write(&fixture, source.replace("CAPPED", &capped).replace("LABEL", &label)).expect("write legacy fixture");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "untagged v1 markers must fail closed");
    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check json");
    let statuses = json["records"]
        .as_array()
        .expect("records array")
        .iter()
        .map(|record| (record["name"].as_str().expect("name"), record["status"].as_str().expect("status")))
        .collect::<Vec<_>>();
    assert_eq!(statuses, [("capped_batch_size", "mismatch"), ("label", "mismatch")]);
}

#[test]
fn fingerprint_check_fails_on_review_warnings_even_when_hashes_match() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// octet-fingerprint: b3:0; review=macro-path; evidence=evidence/macro
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// octet-fingerprint: b3:0
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
"#,
    )
    .expect("write warning fixture");
    let run = |mode: &str| {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", mode, "reviewed.rs"])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let summary = stdout.lines().find(|line| line.starts_with("summary ")).expect("summary line").to_owned();
        (output.status.success(), summary)
    };

    let (updated, summary) = run("--update");
    assert!(updated);
    assert_eq!(
        summary,
        "summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1"
    );
    assert_eq!(
        run("--check"),
        (
            false,
            "summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1".to_owned()
        )
    );

    let reviewed = fs::read_to_string(&fixture)
        .expect("read updated fixture")
        .replace("macro=review-required", "macro=allow")
        .replace("review=TODO; evidence=TODO", "review=gate; evidence=evidence/gate");
    fs::write(&fixture, reviewed).expect("record review metadata");
    assert_eq!(
        run("--check"),
        (
            true,
            "summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0".to_owned()
        )
    );
}

#[test]
fn fingerprint_json_records_report_stored_and_computed_hashes_across_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        "// octet-fingerprint: b3:0; review=json-script; evidence=evidence/json\npub fn capped_retries(requested: u32) -> u32 {\n    requested.min(3)\n}\n",
    )
    .expect("write json fixture");
    let run = |mode: &str| {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", mode, "--output-format=json", "reviewed.rs"])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fingerprint json");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["mode"], mode.trim_start_matches('-'));
        assert_eq!(json["records"].as_array().map(Vec::len), Some(1));
        let field = |name: &str| json["records"][0][name].as_str().expect("string field").to_owned();
        (output.status.success(), field("status"), field("stored_hash"), field("computed_hash"))
    };

    let (updated, status, stored, reviewed) = run("--update");
    assert_eq!((updated, status.as_str(), stored.as_str()), (true, "written", "0"));
    assert_eq!(run("--check"), (true, "match".to_owned(), reviewed.clone(), reviewed.clone()));

    let source = fs::read_to_string(&fixture).expect("read reviewed fixture");
    fs::write(&fixture, source.replace("requested.min(3)", "requested.min(5)")).expect("mutate fixture");
    let (clean, status, stored, drifted) = run("--check");
    assert_eq!((clean, status.as_str(), stored.as_str()), (false, "mismatch", reviewed.as_str()));
    assert_ne!(drifted, reviewed);

    assert_eq!(run("--update"), (true, "written".to_owned(), reviewed, drifted.clone()));
    assert_eq!(run("--check"), (true, "match".to_owned(), drifted.clone(), drifted));
}

#[test]
fn fingerprint_update_check_and_diff_walk_directory_tree() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let expected = BTreeSet::from([
        "attributed_nested",
        "explicit_test_helper",
        "nested_test_helper",
        "read_value",
        "root_plain",
        "root_with_macro",
    ]);
    assert_directory_records(&update.stdout, &expected, "written");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
    assert_directory_records(&check.stdout, &expected, "match");

    let diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint diff");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    assert_directory_records(&diff.stdout, &expected, "match");
}

fn assert_directory_records(stdout: &[u8], expected: &BTreeSet<&str>, status: &str) {
    let json: serde_json::Value = serde_json::from_slice(stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    let names = records.iter().map(|record| record["name"].as_str().expect("record name")).collect::<BTreeSet<_>>();
    assert_eq!(&names, expected);
    assert!(!names.iter().any(|name| name.starts_with("generated")));
    assert!(records.iter().all(|record| record["status"] == status));
    assert!(records.iter().all(|record| !record["review"].is_null()));
    assert!(records.iter().all(|record| !record["evidence"].is_null()));
    assert!(records.iter().all(|record| record["macro_warning"] == false));
}

#[test]
fn fingerprint_check_reports_single_directory_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let lib_rs = temp.path().join("src/lib.rs");
    let source = fs::read_to_string(&lib_rs).expect("read lib.rs");
    fs::write(&lib_rs, source.replace("input.saturating_add(1)", "input.saturating_add(2)"))
        .expect("mutate marked function");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "drift check should fail");

    let human_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check"])
        .arg(temp.path())
        .output()
        .expect("run human fingerprint check");
    assert!(!human_check.status.success(), "human drift check should fail");
    assert_human_summary(&human_check.stdout, "match=5 mismatch=1 missing=0 written=0");
    assert_human_contains_record(&human_check.stdout, "root_plain", "mismatch");

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_single_directory_mismatch(records);

    let diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint drift diff");
    assert!(!diff.status.success(), "drift diff should fail");

    let diff_json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json output");
    let diff_records = diff_json["records"].as_array().expect("diff records array");
    assert_single_directory_mismatch(diff_records);

    let repair = Command::new(cargo_octet())
        .args(["fingerprint", "--update"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint repair update");
    assert!(repair.status.success(), "repair stderr: {}", String::from_utf8_lossy(&repair.stderr));
    assert_human_summary(&repair.stdout, "match=5 mismatch=0 missing=0 written=1");
    assert_human_contains_record(&repair.stdout, "root_plain", "written");

    let repaired_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run repaired fingerprint check");
    assert!(
        repaired_check.status.success(),
        "repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_directory_records(
        &repaired_check.stdout,
        &BTreeSet::from([
            "attributed_nested",
            "explicit_test_helper",
            "nested_test_helper",
            "read_value",
            "root_plain",
            "root_with_macro",
        ]),
        "match",
    );

    let repaired_diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run repaired fingerprint diff");
    assert!(
        repaired_diff.status.success(),
        "repaired diff stderr: {}",
        String::from_utf8_lossy(&repaired_diff.stderr)
    );
    assert_directory_records(
        &repaired_diff.stdout,
        &BTreeSet::from([
            "attributed_nested",
            "explicit_test_helper",
            "nested_test_helper",
            "read_value",
            "root_plain",
            "root_with_macro",
        ]),
        "match",
    );
}

fn assert_single_directory_mismatch(records: &[serde_json::Value]) {
    assert_eq!(records.len(), 6);
    let mismatches = records.iter().filter(|record| record["status"] == "mismatch").collect::<Vec<_>>();
    assert_eq!(mismatches.len(), 1);
    assert_eq!(mismatches[0]["name"], "root_plain");
    assert!(records.iter().all(|record| !record["review"].is_null()));
    assert!(records.iter().all(|record| !record["evidence"].is_null()));
    assert!(records.iter().all(|record| record["macro_warning"] == false));
}

fn assert_human_summary(stdout: &[u8], expected_counts: &str) {
    let text = String::from_utf8_lossy(stdout);
    let summary = text.lines().find(|line| line.starts_with("summary total=6 ")).expect("human summary line");
    assert!(summary.contains(expected_counts), "summary `{summary}` missing `{expected_counts}`");
    assert!(summary.contains("macro_warnings=0"));
    assert!(summary.contains("metadata_missing=0"));
}

fn assert_human_contains_record(stdout: &[u8], name: &str, status: &str) {
    let text = String::from_utf8_lossy(stdout);
    assert!(
        text.lines().any(|line| line.contains(name) && line.contains(status)),
        "human output did not include {name} as {status}:\n{text}"
    );
}

#[test]
fn object_support_does_not_require_fingerprint_markers() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fingerprint = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!fingerprint.status.success(), "zeroed fixture should report fingerprint mismatches");
    let json: serde_json::Value = serde_json::from_slice(&fingerprint.stdout).expect("fingerprint json");
    assert_eq!(json["records"].as_array().expect("records").len(), 2);
    assert!(json["records"].as_array().unwrap().iter().all(|record| record["stored_hash"] == "0"));
}

#[test]
fn object_pointer_set_records_move_and_validate_fails_closed() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let old_hash = blake3_hex(b"checked_value input + 1");
    let new_hash = blake3_hex(b"checked_value input + 2");
    let old_objects = temp.path().join("old-objects.json");
    let new_objects = temp.path().join("new-objects.json");
    write_object_records(&old_objects, &[("checked_value", &old_hash)]);
    write_object_records(&new_objects, &[("checked_value", &new_hash)]);

    let manifest = temp.path().join("pointers.json");
    let receipt1 = temp.path().join("receipt-old.json");
    let first_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&manifest)
        .args(["--name", "crate::checked_value", "--hash", &old_hash, "--objects"])
        .arg(&old_objects)
        .args(["--receipt"])
        .arg(&receipt1)
        .output()
        .expect("set old pointer");
    assert!(first_set.status.success(), "first pointer stderr: {}", String::from_utf8_lossy(&first_set.stderr));

    let receipt2 = temp.path().join("receipt-new.json");
    let second_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&manifest)
        .args(["--name", "crate::checked_value", "--hash", &new_hash, "--objects"])
        .arg(&new_objects)
        .args(["--receipt"])
        .arg(&receipt2)
        .output()
        .expect("set new pointer");
    assert!(
        second_set.status.success(),
        "second pointer stderr: {}",
        String::from_utf8_lossy(&second_set.stderr)
    );
    let receipt_json: serde_json::Value = serde_json::from_slice(&second_set.stdout).expect("receipt json");
    assert_eq!(receipt_json["old_hash"], old_hash);
    assert_eq!(receipt_json["new_hash"], new_hash);
    assert!(receipt_json["diff_summary"].as_str().unwrap().contains("pointer-moved"));

    let valid = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&manifest)
        .args(["--objects"])
        .arg(&new_objects)
        .output()
        .expect("validate pointer");
    assert!(valid.status.success(), "valid pointer stderr: {}", String::from_utf8_lossy(&valid.stderr));

    let invalid = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&manifest)
        .args(["--objects"])
        .arg(&old_objects)
        .output()
        .expect("validate stale pointer");
    assert!(!invalid.status.success(), "stale pointer unexpectedly resolved");
    let invalid_json: serde_json::Value = serde_json::from_slice(&invalid.stdout).expect("invalid validate json");
    assert_eq!(invalid_json["status"], "missing-object");
}

#[test]
fn object_diff_separates_body_signature_caveat_and_pointer_changes() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let old_path = temp.path().join("old.json");
    let new_path = temp.path().join("new.json");
    fs::write(
        &old_path,
        r#"{"object_hash":"old","name":"checked","body_hash":"body-a","signature_hash":"sig","dependency_caveats":["deps"],"effect_caveats":["none"],"macro_caveats":["none"]}"#,
    )
    .expect("write old object");
    fs::write(
        &new_path,
        r#"{"object_hash":"new","name":"checked","body_hash":"body-b","signature_hash":"sig","dependency_caveats":["deps"],"effect_caveats":["panic"],"macro_caveats":["none"]}"#,
    )
    .expect("write new object");
    let manifest = temp.path().join("pointers.json");
    fs::write(
        &manifest,
        r#"{"schema_version":1,"pointers":{"crate::checked":{"object_hash":"new","semantic_name":"crate::checked"}},"history":{}}"#,
    )
    .expect("write pointer manifest");

    let diff = Command::new(cargo_octet())
        .args(["object", "diff", "--old"])
        .arg(&old_path)
        .args(["--new"])
        .arg(&new_path)
        .args(["--pointer-manifest"])
        .arg(&manifest)
        .args(["--output-format=json"])
        .output()
        .expect("run object diff");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    let json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json");
    assert_eq!(json["body_changed"], true);
    assert_eq!(json["signature_changed"], false);
    assert_eq!(json["effect_caveats_changed"], true);
    assert_eq!(json["macro_caveats_changed"], false);
    assert_eq!(json["pointer_changes"].as_array().unwrap().len(), 1);
}

#[test]
fn object_bundle_export_is_deterministic_and_records_replay() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let objects_path = temp.path().join("objects.json");
    let alpha_hash = blake3_hex(b"alpha input + 1");
    write_object_records(&objects_path, &[("beta", &blake3_hex(b"beta input + 2")), ("alpha", &alpha_hash)]);
    let pointers = temp.path().join("pointers.json");
    let receipt = temp.path().join("receipt.json");
    let pointer_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&pointers)
        .args(["--name", "crate::alpha", "--hash", &alpha_hash, "--objects"])
        .arg(&objects_path)
        .args(["--receipt"])
        .arg(&receipt)
        .output()
        .expect("set pointer");
    assert!(pointer_set.status.success(), "pointer stderr: {}", String::from_utf8_lossy(&pointer_set.stderr));

    let one = temp.path().join("bundle-one.json");
    let two = temp.path().join("bundle-two.json");
    for output_path in [&one, &two] {
        let bundle = Command::new(cargo_octet())
            .args(["object", "bundle", "export", "--objects"])
            .arg(&objects_path)
            .args(["--pointers"])
            .arg(&pointers)
            .args(["--output"])
            .arg(output_path)
            .args([
                "--replay-command",
                "cargo octet object emit --output-format=json lib.rs",
            ])
            .output()
            .expect("export bundle");
        assert!(bundle.status.success(), "bundle stderr: {}", String::from_utf8_lossy(&bundle.stderr));
    }
    let first = fs::read_to_string(&one).expect("read first bundle");
    let second = fs::read_to_string(&two).expect("read second bundle");
    assert_eq!(first, second);
    let json: serde_json::Value = serde_json::from_str(&first).expect("bundle json");
    assert_eq!(json["schema"], "octet.function-object-bundle.v1");
    assert_eq!(json["object_count"], 2);
    assert_eq!(json["pointer_count"], 1);
    assert!(json["bundle_hash"].as_str().unwrap().starts_with("b3:"));
    assert_eq!(json["replay"]["command"], "cargo octet object emit --output-format=json lib.rs");
    assert!(json["replay"]["determinism_note"].as_str().unwrap().contains("not correctness"));

    let missing_pointer = temp.path().join("missing-pointer.json");
    let missing_hash = blake3_hex(b"no such object");
    fs::write(
        &missing_pointer,
        format!(
            r#"{{"schema_version":1,"pointers":{{"crate::missing":{{"object_hash":"{missing_hash}","semantic_name":"crate::missing"}}}},"history":{{}}}}"#
        ),
    )
    .expect("write missing pointer manifest");
    let unresolved = Command::new(cargo_octet())
        .args(["object", "bundle", "export", "--objects"])
        .arg(&objects_path)
        .args(["--pointers"])
        .arg(&missing_pointer)
        .args(["--output"])
        .arg(temp.path().join("unresolved-bundle.json"))
        .args([
            "--replay-command",
            "cargo octet object emit --output-format=json lib.rs",
        ])
        .output()
        .expect("export bundle with unresolved pointer");
    assert!(!unresolved.status.success(), "bundle with an unresolved pointer unexpectedly passed");
    assert!(
        String::from_utf8_lossy(&unresolved.stderr).contains("references missing object"),
        "unresolved pointer was not named: {}",
        String::from_utf8_lossy(&unresolved.stderr)
    );
    assert!(!temp.path().join("unresolved-bundle.json").exists());
}

const INTERNAL_OBJECT_CORPUS: &[&str] = &[
    "cargo-octet/src/baseline.rs",
    "cargo-octet/src/bin/cargo-slotcar.rs",
    "cargo-octet/src/bin/cargo-tigerstyle.rs",
    "cargo-octet/src/cli.rs",
    "cargo-octet/src/config.rs",
    "cargo-octet/src/determinism.rs",
    "cargo-octet/src/engine.rs",
    "cargo-octet/src/evidence.rs",
    "cargo-octet/src/evidence_rails.rs",
    "cargo-octet/src/fingerprint.rs",
    "cargo-octet/src/lib.rs",
    "cargo-octet/src/main.rs",
    "cargo-octet/src/migrate.rs",
    "cargo-octet/src/object.rs",
    "cargo-octet/src/output.rs",
    "cargo-octet/src/provider_manifest.rs",
    "cargo-octet/src/roadmap.rs",
    "cargo-octet/src/transcript.rs",
];

const INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/internal-corpus-receipt.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-taxonomy.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-caveat-taxonomy.json";
const OBJECT_CORPUS_COMPATIBILITY_DIR: &str = "cargo-octet/tests/fixtures/object-corpus/compatibility";
const OBJECT_CORPUS_COMPATIBILITY_SOURCE: &[&str] = &["src/lib.rs"];
const OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1: &str = "positive-v1-receipt.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MISSING_OBJECT_SET_HASH: &str = "negative-missing-object-set-hash.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_FUTURE_SCHEMA_VERSION: &str = "negative-future-schema-version.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_BOUNDARY_TAMPER: &str = "negative-boundary-tamper.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_DUPLICATE_SOURCE_PATH: &str = "negative-duplicate-source-path.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_EMPTY_SOURCE_SELECTION: &str = "negative-empty-source-selection.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MALFORMED_JSON: &str = "negative-malformed-json.json";
const OBJECT_CORPUS_COMPATIBILITY_NON_RUST_SOURCE: &str = "notes.txt";

fn check_corpus_receipt(root: &Path, receipt_path: &Path, paths: &[&str]) -> std::process::Output {
    Command::new(cargo_octet())
        .args(["object", "corpus", "check", "--receipt"])
        .arg(receipt_path)
        .args(["--output-format", "json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("check object corpus receipt")
}

fn check_corpus_receipt_fixture(root: &Path, receipt_name: &str) -> std::process::Output {
    check_corpus_receipt(
        &root.join(OBJECT_CORPUS_COMPATIBILITY_DIR),
        Path::new(receipt_name),
        OBJECT_CORPUS_COMPATIBILITY_SOURCE,
    )
}

fn read_json_fixture(root: &Path, relative: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join(relative)).unwrap_or_else(|error| panic!("read {relative}: {error}")))
        .unwrap_or_else(|error| panic!("parse {relative}: {error}"))
}

fn object_corpus_contract(root: &Path) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "contract", "--output-format", "json"])
        .current_dir(root)
        .output()
        .expect("object corpus contract metadata");
    assert!(
        output.status.success(),
        "contract stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("contract metadata json")
}

fn blake3_hex(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Writes `objects` in the `object emit` JSON shape. Commands that read
/// existing object evidence (`pointer`, `bundle export`) need only each
/// record's name and object hash.
fn write_object_records(path: &Path, objects: &[(&str, &str)]) {
    let objects = objects
        .iter()
        .map(|(name, hash)| serde_json::json!({ "schema": "octet.function-object.v1", "name": name, "object_hash": hash }))
        .collect::<Vec<_>>();
    let report = serde_json::json!({ "schema_version": 1, "mode": "emit", "objects": objects });
    fs::write(path, serde_json::to_vec_pretty(&report).expect("object records json")).expect("write object records");
}

/// Copies the checked-in v1 compatibility receipt to `dir/receipt.json` and its
/// source to `dir/lib.rs`, for receipt validation that runs before any object is
/// built.
fn copy_checked_in_receipt(dir: &Path) -> (std::path::PathBuf, serde_json::Value) {
    let compatibility = repo_root().join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    fs::copy(compatibility.join("src/lib.rs"), dir.join("lib.rs")).expect("copy compatibility source");
    let receipt_path = dir.join("receipt.json");
    fs::copy(compatibility.join(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1), &receipt_path)
        .expect("copy compatibility receipt");
    (receipt_path, read_json_fixture(&compatibility, OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1))
}

#[test]
fn object_v1_refusal_covers_every_source_backed_surface() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = "pub fn fetch() -> u32 {\n    let _endpoint = \"https://example.invalid/api\"; 1\n}\n";
    fs::write(temp.path().join("lib.rs"), source).expect("write source");
    let marked = format!("// octet-object: b3:{}; review=R-1; evidence=E-1\n{source}", "0".repeat(64));
    fs::write(temp.path().join("marked.rs"), &marked).expect("write marked source");

    for (surface, cwd, args, refused) in [
        ("emit", temp.path(), &["object", "emit", "--output-format=json", "lib.rs"][..], "lib.rs::fetch"),
        (
            "marker check",
            temp.path(),
            &["object", "check", "--output-format=json", "marked.rs"],
            "marked.rs::fetch",
        ),
        (
            "marker update",
            temp.path(),
            &["object", "update", "--output-format=json", "marked.rs"],
            "marked.rs::fetch",
        ),
        (
            "corpus receipt",
            temp.path(),
            &["object", "corpus", "receipt", "--output", "receipt.json", "lib.rs"],
            "lib.rs::fetch",
        ),
        (
            "corpus caveats",
            temp.path(),
            &["object", "corpus", "caveats", "--output-format", "json", "lib.rs"],
            "lib.rs::fetch",
        ),
        (
            "pilot supplement emit",
            temp.path(),
            &[
                "object",
                "supplement",
                "emit",
                "--family",
                "hir",
                "--output-format",
                "json",
                "lib.rs",
            ],
            "lib.rs::fetch",
        ),
        (
            "fixture supplement emit",
            &root,
            &[
                "object",
                "supplement",
                "emit",
                "--family",
                "mir",
                "--spike",
                "rustc-mir-fixture",
                "--fixture-shape",
                "checked-arithmetic",
                "--output-format",
                "json",
            ],
            "cargo-octet/tests/fixtures/mir-spike/checked.rs::checked",
        ),
    ] {
        let output = Command::new(cargo_octet()).args(args).current_dir(cwd).output().expect("run object command");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{surface} was not refused: {stderr}");
        assert!(output.stdout.is_empty(), "{surface} printed output: {}", String::from_utf8_lossy(&output.stdout));
        assert!(
            stderr.starts_with(&format!("error: function-object-v1-unavailable: {refused}: ")),
            "{surface}: {stderr}"
        );
        assert!(stderr.contains("`octet-source-token-v1`"), "{surface} did not name the v1 label: {stderr}");
    }
    assert_eq!(fs::read_to_string(temp.path().join("marked.rs")).expect("read marked source"), marked);
    assert!(!temp.path().join("receipt.json").exists(), "refused corpus receipt was written");

    let corpus = temp.path().join("corpus");
    fs::create_dir(&corpus).expect("create corpus dir");
    let (receipt_path, _) = copy_checked_in_receipt(&corpus);
    let check = check_corpus_receipt(&corpus, &receipt_path, &["lib.rs"]);
    assert_eq!(check.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("corpus check report json");
    assert_eq!(report["status"], "invalid");
    assert_eq!(report["errors"][0]["kind"], "function-object-v1-unavailable");
    assert!(
        report["errors"][0]["message"]
            .as_str()
            .expect("refusal message")
            .starts_with("function-object-v1-unavailable: lib.rs::compat_increment: "),
        "{report:#?}"
    );
    assert!(report["comparisons"].as_array().expect("comparisons").is_empty());
}

#[test]
fn object_v1_refusal_follows_supplement_receipt_validation() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let valid_shape = serde_json::json!({
        "schema": "octet.function-object-mir-supplement.v1",
        "schema_version": 1,
        "family": "mir",
        "status": "pilot-supported",
        "function_object_hash": blake3_hex(b"checked"),
        "source_path": "cargo-octet/tests/fixtures/mir-spike/checked.rs",
        "function_name": "checked",
        "rustc_version": "rustc fixture",
        "target_triple": "x86_64-unknown-linux-gnu",
        "extractor": {},
        "observations": {},
        "observation_hash": blake3_hex(b"observations"),
        "extraction_caveats": [],
        "boundary": {},
    });
    let check = |name: &str, receipt: &[u8]| {
        let path = temp.path().join(name);
        fs::write(&path, receipt).expect("write supplement receipt");
        Command::new(cargo_octet())
            .args([
                "object",
                "supplement",
                "check",
                "--family",
                "mir",
                "--spike",
                "rustc-mir-fixture",
            ])
            .args([
                "--fixture-shape",
                "checked-arithmetic",
                "--output-format",
                "json",
                "--receipt",
            ])
            .arg(&path)
            .current_dir(&root)
            .output()
            .expect("run supplement check")
    };

    let mut missing = valid_shape.clone();
    missing.as_object_mut().expect("receipt object").remove("function_object_hash");
    let mut future = valid_shape.clone();
    future["schema_version"] = serde_json::json!(99);
    let mut wrong_family = valid_shape.clone();
    wrong_family["family"] = serde_json::json!("thir");
    for (name, receipt, kind, field) in [
        ("malformed.json", b"{ not json".to_vec(), "invalid-json", "receipt"),
        ("missing.json", serde_json::to_vec(&missing).expect("json"), "missing-field", "function_object_hash"),
        (
            "future.json",
            serde_json::to_vec(&future).expect("json"),
            "unsupported-schema-version",
            "schema_version",
        ),
        ("family.json", serde_json::to_vec(&wrong_family).expect("json"), "family-mismatch", "family"),
    ] {
        let output = check(name, &receipt);
        assert_eq!(output.status.code(), Some(1), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("supplement report json");
        assert_eq!(report["status"], "invalid", "{name}: {report:#?}");
        assert_supplement_error(&report, kind, field);
    }

    let refused = check("valid-shape.json", &serde_json::to_vec(&valid_shape).expect("json"));
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(2), "{stderr}");
    assert!(refused.stdout.is_empty());
    assert!(
        stderr.starts_with(
            "error: function-object-v1-unavailable: cargo-octet/tests/fixtures/mir-spike/checked.rs::checked: "
        ),
        "{stderr}"
    );
}

#[test]
fn hir_mir_supplement_pilot_docs_name_commands_and_non_proofs() {
    let root = repo_root();
    let contributor = fs::read_to_string(root.join("docs/contributor-guide.md")).expect("read contributor guide");
    let boundaries = fs::read_to_string(root.join("docs/content-addressed-rust-evidence-boundaries.md"))
        .expect("read content-addressed boundaries doc");

    for needle in [
        "cargo octet object supplement emit --family hir",
        "cargo octet object supplement check --family hir",
        "cargo octet object supplement emit --family mir",
        "cargo octet object supplement check --family mir",
        "cargo octet object supplement emit --family thir",
        "cargo octet object supplement check --family thir",
        "cargo octet object corpus contract --output-format json",
        "cargo octet object supplement emit --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement check --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement emit --family mir --spike rustc-mir-fixture",
        "cargo octet object supplement check --family mir --spike rustc-mir-fixture",
        "--fixture-shape checked-arithmetic",
        "--fixture-shape branch-control-flow",
        "selected-fixture-backed/toolchain-scoped review",
        "checked.rs::checked",
        "clamp.rs::clamp_to_limit",
        "call_return.rs::call_then_return",
        "source-token-bounded review evidence",
        "selected-fixture-backed/toolchain-scoped review",
        "do not replace it as the",
        "The default MIR and THIR runtime seams",
        "correctness proof",
        "semantic-equivalence proof",
        "pure-cache eligibility proof",
        "cross-toolchain stability",
    ] {
        assert!(contributor.contains(needle), "contributor guide missing {needle}: {contributor}");
    }

    for needle in [
        "cargo octet object supplement emit --family hir",
        "cargo octet object supplement check --family hir",
        "cargo octet object supplement emit --family mir",
        "cargo octet object supplement check --family mir",
        "cargo octet object supplement emit --family thir",
        "cargo octet object supplement check --family thir",
        "cargo octet object supplement emit --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement check --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement emit --family mir --spike rustc-mir-fixture",
        "cargo octet object supplement check --family mir --spike rustc-mir-fixture",
        "--fixture-shape checked-arithmetic",
        "--fixture-shape branch-control-flow",
        "attached supplement to `octet.function-object.v1`",
        "selected-fixture-backed/toolchain-scoped",
        "arbitrary-path and non-fixture HIR remain",
        "Default THIR runtime receipts MUST keep status `unsupported-planned`",
        "does not prove THIR",
        "MUST NOT claim stable identity across compiler versions",
    ] {
        assert!(boundaries.contains(needle), "boundary doc missing {needle}: {boundaries}");
    }
}

fn assert_supplement_error(report: &serde_json::Value, expected_kind: &str, expected_field: &str) {
    assert!(
        report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["kind"] == expected_kind && error["field"] == expected_field }),
        "missing supplement error {expected_kind}/{expected_field}: {report:#?}"
    );
    assert_eq!(report["boundary"]["claims_correctness"], false);
    assert_eq!(report["boundary"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_supplement_runtime_smoke_covers_nested_help_and_inventory() {
    let root = repo_root();

    for args in [
        &["object", "supplement", "emit", "--help"][..],
        &["object", "supplement", "check", "--help"][..],
        &["object", "supplement", "fixtures", "--help"][..],
    ] {
        let output = Command::new(cargo_octet())
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run nested supplement help {args:?}: {error}"));
        assert!(
            output.status.success(),
            "nested supplement help {args:?} failed stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("cargo octet object"), "help missing object usage for {args:?}: {stdout}");
        assert!(
            stdout.contains("supplement fixtures --family mir --spike rustc-mir-fixture"),
            "help missing scoped fixture inventory command for {args:?}: {stdout}"
        );
        assert!(
            stdout.contains("--family hir|thir|mir"),
            "help missing THIR fail-closed supplement family for {args:?}: {stdout}"
        );
    }

    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "hir",
            "--spike",
            "rustc-hir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run HIR fixture inventory json smoke");
    assert!(
        inventory.status.success(),
        "inventory smoke failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("inventory smoke json");
    assert_eq!(json["schema"], "octet.hir-fixture-inventory.v1");
    assert_eq!(json["scope"], "rustc-hir-fixture-spike");
    assert_eq!(json["default_shape"], "checked-arithmetic");
    let shapes = json["shapes"].as_array().expect("fixture inventory shapes array");
    assert_eq!(shapes.len(), 2, "inventory should remain a closed two-shape fixture set: {json:#?}");
    for selector in ["checked-arithmetic", "branch-control-flow"] {
        let fixture = shapes
            .iter()
            .find(|fixture| fixture["shape"] == selector)
            .unwrap_or_else(|| panic!("missing fixture selector {selector}: {json:#?}"));
        assert_eq!(fixture["support_status"], "pilot-supported");
        assert!(fixture["emit_command"].as_str().unwrap().contains(selector));
        assert!(fixture["check_command"].as_str().unwrap().contains(selector));
    }
}

#[test]
fn thir_spike_fixture_listing_stays_closed_over_selected_shapes() {
    let root = repo_root();
    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "thir",
            "--spike",
            "rustc-thir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run aggregate THIR fixture inventory source");
    assert!(
        inventory.status.success(),
        "aggregate THIR inventory source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("aggregate THIR inventory json");
    let shapes = json["shapes"].as_array().expect("aggregate THIR fixture inventory shapes array");
    let listed: BTreeSet<_> =
        shapes.iter().map(|fixture| fixture["shape"].as_str().expect("fixture shape").to_owned()).collect();
    assert_eq!(
        listed,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned(),
        ]),
        "aggregate THIR inventory gate must stay closed over selected fixture shapes: {json:#?}"
    );
    let branch = shapes
        .iter()
        .find(|fixture| fixture["shape"] == "branch-control-flow")
        .expect("aggregate THIR branch-control-flow fixture entry");
    assert_eq!(branch["function"], "choose_branch");
    assert!(
        branch["fixture_path"]
            .as_str()
            .expect("branch fixture path")
            .ends_with("thir-spike/branch_control_flow.rs"),
        "branch-control-flow must be a THIR-specific selected fixture: {branch:#?}"
    );
    for fixture in shapes {
        assert_eq!(fixture["support_status"], "pilot-supported");
        assert!(
            fixture["emit_command"].as_str().expect("emit command").contains("--spike rustc-thir-fixture"),
            "aggregate THIR inventory must expose replayable emit commands: {fixture:#?}"
        );
        assert!(
            fixture["check_command"].as_str().expect("check command").contains("--spike rustc-thir-fixture"),
            "aggregate THIR inventory must expose replayable check commands: {fixture:#?}"
        );
        assert!(
            fixture["support_boundary"]
                .as_str()
                .expect("support boundary")
                .contains("selected checked-in rustc THIR fixture contract only"),
            "aggregate THIR inventory must remain selected-fixture-only: {fixture:#?}"
        );
    }

    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    let thir = supplements
        .iter()
        .find(|value| value["family"] == "thir")
        .unwrap_or_else(|| panic!("missing THIR supplement contract in {contract:#?}"));
    assert_eq!(thir["status"], "unsupported-planned");
    assert_eq!(thir["claims_correctness"], false);
    assert_eq!(thir["claims_semantic_equivalence"], false);
    assert_eq!(thir["claims_cross_toolchain_stability"], false);
    assert!(
        thir["unsupported_reason"]
            .as_str()
            .expect("THIR unsupported reason")
            .contains("default and non-fixture THIR requests remain unsupported-planned"),
        "aggregate THIR inventory must not broaden default/non-fixture THIR status: {thir:#?}"
    );
}

#[test]
fn mir_spike_fixture_listing_stays_closed_over_selected_shapes() {
    let root = repo_root();
    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run aggregate MIR fixture inventory source");
    assert!(
        inventory.status.success(),
        "aggregate MIR inventory source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("aggregate MIR inventory json");
    let shapes = json["shapes"].as_array().expect("aggregate MIR fixture inventory shapes array");
    let listed: BTreeSet<_> =
        shapes.iter().map(|fixture| fixture["shape"].as_str().expect("fixture shape").to_owned()).collect();
    assert_eq!(
        listed,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned()
        ]),
        "aggregate MIR inventory gate must stay closed over selected fixture shapes: {json:#?}"
    );
    for fixture in shapes {
        assert_eq!(fixture["support_status"], "pilot-supported");
        assert!(
            fixture["emit_command"].as_str().expect("emit command").contains("--spike rustc-mir-fixture"),
            "aggregate MIR inventory must expose replayable emit commands: {fixture:#?}"
        );
        assert!(
            fixture["check_command"].as_str().expect("check command").contains("--spike rustc-mir-fixture"),
            "aggregate MIR inventory must expose replayable check commands: {fixture:#?}"
        );
        assert!(
            fixture["support_boundary"].as_str().expect("support boundary").contains("selected-fixture-only"),
            "aggregate MIR inventory must remain selected-fixture-only: {fixture:#?}"
        );
    }

    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    let mir = supplements
        .iter()
        .find(|value| value["family"] == "mir")
        .unwrap_or_else(|| panic!("missing MIR supplement contract in {contract:#?}"));
    assert_eq!(mir["status"], "unsupported-planned");
    assert_eq!(mir["claims_correctness"], false);
    assert_eq!(mir["claims_semantic_equivalence"], false);
    assert_eq!(mir["claims_cross_toolchain_stability"], false);
    assert!(
        mir["unsupported_reason"]
            .as_str()
            .expect("MIR unsupported reason")
            .contains("default and non-fixture MIR requests remain unsupported-planned"),
        "aggregate MIR inventory must not broaden default/non-fixture MIR status: {mir:#?}"
    );
}
#[test]
fn object_corpus_contract_metadata_reports_receipt_and_check_shapes() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);

    assert_eq!(contract["schema"], "octet.object-corpus-contract-metadata.v1");
    assert_eq!(contract["schema_version"], 1);
    assert_eq!(contract["source_of_truth"], "rust-dto-and-constants");
    assert_eq!(contract["receipt"]["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(contract["receipt"]["schema_version"], 1);
    for field in [
        "schema",
        "schema_version",
        "mode",
        "object_count",
        "source_paths",
        "object_set_hash",
        "caveat_counts",
        "pure_cache_blocked_count",
        "replay",
        "boundary",
    ] {
        assert!(
            contract["receipt"]["required_fields"].as_array().unwrap().iter().any(|value| value == field),
            "missing receipt required field {field} in {contract:#?}"
        );
    }
    assert!(contract["receipt"]["compared_fields"].as_array().unwrap().iter().any(|value| value == "boundary"));
    assert_eq!(contract["check_report"]["schema"], "octet.function-object-corpus-check-report.v1");
    for field in [
        "schema",
        "schema_version",
        "mode",
        "status",
        "receipt_path",
        "comparisons",
        "errors",
    ] {
        assert!(
            contract["check_report"]["required_fields"].as_array().unwrap().iter().any(|value| value == field),
            "missing check-report required field {field} in {contract:#?}"
        );
    }
    for status in ["match", "mismatch", "invalid"] {
        assert!(contract["check_report"]["status_values"].as_array().unwrap().iter().any(|value| value == status));
    }
    assert_eq!(contract["boundaries"]["claims_correctness"], false);
    assert_eq!(contract["boundaries"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_corpus_contract_metadata_reports_pilot_and_planned_supplement_statuses() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    assert_eq!(supplements.len(), 3, "expected HIR, THIR, and MIR supplement contracts: {contract:#?}");

    let hir = supplements.iter().find(|value| value["family"] == "hir").expect("missing HIR supplement contract");
    assert!(
        hir["fixture_replay_command"].as_str().unwrap().contains("--spike rustc-hir-fixture"),
        "HIR replay command should name fixture spike scope: {hir:#?}"
    );
    for fixture_shape in ["checked-arithmetic", "branch-control-flow"] {
        assert!(
            hir["selected_fixture_shapes"].as_array().unwrap().iter().any(|value| value == fixture_shape),
            "HIR contract should name selected fixture shape {fixture_shape}: {hir:#?}"
        );
        assert!(
            hir["fixture_replay_command"].as_str().unwrap().contains(fixture_shape)
                || hir["extractor_identity_expectations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| { value.as_str().unwrap().contains(fixture_shape) }),
            "HIR contract should bind selected fixture shape {fixture_shape} to replay or extractor expectations: {hir:#?}"
        );
    }
    assert!(
        hir["unsupported_reason"]
            .as_str()
            .unwrap()
            .contains("non-fixture HIR requests remain unsupported-planned"),
        "HIR contract should reject broad/non-fixture HIR support claims: {hir:#?}"
    );
    assert!(
        hir["extractor_identity_expectations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("octet-rustc-hir-fixture-spike")),
        "HIR contract should name rustc HIR fixture spike extractor: {hir:#?}"
    );

    for (family, status) in [
        ("hir", "pilot-supported"),
        ("thir", "unsupported-planned"),
        ("mir", "unsupported-planned"),
    ] {
        let supplement = supplements
            .iter()
            .find(|value| value["family"] == family)
            .unwrap_or_else(|| panic!("missing {family} supplement contract in {contract:#?}"));
        assert_eq!(supplement["status"], status);
        assert!(
            supplement["relationship_to_v1"].as_str().unwrap().contains("preserves-function-object-v1-identity"),
            "{family} supplement must preserve v1 identity boundary: {supplement:#?}"
        );
        assert!(
            supplement["toolchain_scope"].as_str().unwrap().contains("rustc-version"),
            "{family} supplement must be rustc/toolchain scoped: {supplement:#?}"
        );
        for field in [
            "schema",
            "schema_version",
            "function_object_hash",
            "rustc_version",
            "target_triple",
            "extraction_caveats",
            "boundary",
        ] {
            assert!(
                supplement["required_fields"].as_array().unwrap().iter().any(|value| value == field),
                "missing required {family} supplement field {field}: {supplement:#?}"
            );
        }
        assert!(
            supplement["unsupported_reason"].as_str().unwrap().contains("unsupported")
                || supplement["unsupported_reason"].as_str().unwrap().contains("not implemented"),
            "{family} supplement should document unsupported/bounded scope: {supplement:#?}"
        );
        assert!(
            supplement["required_evidence_before_support"].as_array().unwrap().iter().any(|value| {
                value.as_str().unwrap().contains("fixture") || value.as_str().unwrap().contains("smoke")
            }),
            "{family} supplement should require deterministic fixture or smoke evidence: {supplement:#?}"
        );
        assert!(
            supplement["extractor_identity_expectations"].as_array().unwrap().iter().any(|value| {
                value.as_str().unwrap().contains("extractor") || value.as_str().unwrap().contains("rustc_version")
            }),
            "{family} supplement should name extractor/toolchain identity expectations: {supplement:#?}"
        );
        assert!(
            supplement["fixture_replay_command"].as_str().unwrap().contains("supplement")
                || supplement["fixture_replay_command"].as_str().unwrap().contains("unsupported-planned"),
            "{family} supplement should name fixture replay or unsupported-plan command: {supplement:#?}"
        );
        assert_eq!(supplement["claims_correctness"], false);
        assert_eq!(supplement["claims_semantic_equivalence"], false);
        assert_eq!(supplement["claims_cross_toolchain_stability"], false);
    }
}

#[test]
fn object_corpus_contract_human_reports_hir_mir_fail_closed_boundaries() {
    let root = repo_root();
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "contract", "--output-format", "human"])
        .current_dir(&root)
        .output()
        .expect("object corpus contract human metadata");
    assert!(
        output.status.success(),
        "contract human stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "supplemental_evidence family=hir",
        "supplemental_evidence family=thir",
        "supplemental_evidence family=mir",
        "status=pilot-supported",
        "unsupported-planned until general extractor evidence is added",
        "optional-supplement-preserves-function-object-v1-identity",
        "toolchain_scope=rustc-version",
        "extractor_identity_expectations=",
        "selected_fixture_shapes=checked-arithmetic,branch-control-flow",
        "selected_fixture_shapes=checked-arithmetic,call-return",
        "fixture_replay_command=",
        "claims_correctness=false",
        "claims_semantic_equivalence=false",
        "claims_cross_toolchain_stability=false",
    ] {
        assert!(stdout.contains(needle), "human contract output missing {needle}: {stdout}");
    }
}

#[test]
fn checked_in_internal_corpus_receipt_satisfies_contract_metadata() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);
    let fixture: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE)).expect("read checked-in receipt fixture"),
    )
    .expect("checked-in receipt fixture json");

    for field in contract["receipt"]["required_fields"].as_array().expect("required fields") {
        let field = field.as_str().expect("required field string");
        assert!(fixture.get(field).is_some(), "fixture missing contract-required field {field}: {fixture:#?}");
    }
    assert_eq!(fixture["schema"], contract["receipt"]["schema"]);
    assert_eq!(fixture["schema_version"], contract["receipt"]["schema_version"]);
    assert_eq!(fixture["boundary"]["identity_scope"], contract["boundaries"]["identity_scope"]);
    assert_eq!(fixture["boundary"]["claims_correctness"], false);
    assert_eq!(fixture["boundary"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_corpus_contract_missing_required_field_fails_closed() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, mut receipt) = copy_checked_in_receipt(temp.path());
    let contract = object_corpus_contract(temp.path());
    let required = contract["receipt"]["required_fields"]
        .as_array()
        .expect("required fields")
        .iter()
        .find_map(|value| value.as_str())
        .expect("at least one required field");
    receipt.as_object_mut().expect("receipt object").remove(required);
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write missing contract field receipt");

    let report =
        assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]), "missing-field");
    assert!(
        report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["kind"] == "missing-field" && error["field"] == required }),
        "missing contract field {required} was not named in {report:#?}"
    );
}

#[test]
fn checked_in_expanded_corpus_taxonomy_names_selected_source_buckets() {
    let root = repo_root();
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let receipt = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE);
    let taxonomy_paths = taxonomy["source_paths"].as_array().expect("taxonomy source paths");
    let receipt_paths = receipt["source_paths"].as_array().expect("receipt source paths");

    assert_eq!(taxonomy["schema"], "octet.expanded-internal-object-corpus-taxonomy.v1");
    assert_eq!(taxonomy["schema_version"], 1);
    assert_eq!(taxonomy_paths, receipt_paths);
    assert!(taxonomy_paths.len() > INTERNAL_OBJECT_CORPUS.len());
    for path in INTERNAL_OBJECT_CORPUS {
        assert!(taxonomy_paths.iter().any(|value| value == path), "missing core path {path}");
    }
    for bucket in ["cargo-octet", "octet-standards"] {
        assert!(
            taxonomy["buckets"]
                .as_array()
                .expect("taxonomy buckets")
                .iter()
                .any(|entry| entry["name"] == bucket && !entry["source_paths"].as_array().unwrap().is_empty()),
            "missing non-empty bucket {bucket}: {taxonomy:#?}"
        );
    }
    assert!(
        taxonomy_paths
            .iter()
            .any(|value| value.as_str().unwrap().starts_with("crates/octet-standards/src/")),
        "expanded taxonomy must include standards crate sources: {taxonomy:#?}"
    );
}

#[test]
fn checked_in_corpus_caveat_report_names_per_source_blockers() {
    let root = repo_root();
    let report = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE);
    assert_eq!(report["schema"], "octet.function-object-corpus-caveat-taxonomy.v1");
    assert_eq!(report["schema_version"], 1);
    let source_paths = report["source_paths"].as_array().expect("source paths");
    let source_buckets = report["source_buckets"].as_array().expect("source buckets");
    assert_eq!(source_paths.len(), source_buckets.len());
    let mut previous = "";
    for bucket in source_buckets {
        let source_path = bucket["source_path"].as_str().expect("bucket source path");
        assert!(source_path >= previous, "source buckets must be sorted");
        previous = source_path;
        assert!(bucket["object_count"].as_u64().expect("bucket object count") > 0);
        assert!(bucket["caveat_counts"].is_object(), "missing caveat counts for {source_path}");
        assert!(bucket["pure_cache_blocked_count"].as_u64().is_some());
        assert!(bucket["blocker_names"].is_array(), "missing blocker names for {source_path}");
    }
    assert!(
        source_buckets.iter().any(|bucket| {
            bucket["blocker_names"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == "dependency-identity-unresolved")
        }),
        "expected dependency blocker in caveat taxonomy: {report:#?}"
    );
    assert!(
        source_buckets.iter().any(|bucket| {
            bucket["blocker_names"].as_array().unwrap().iter().any(|name| name == "effect-summary-unresolved")
        }),
        "expected effect blocker in caveat taxonomy: {report:#?}"
    );
}

#[test]
fn checked_in_corpus_caveat_report_preserves_non_proof_boundaries() {
    let root = repo_root();
    let report = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE);
    assert_eq!(report["boundary"]["identity_scope"], "review-and-provenance-evidence");
    assert_eq!(report["boundary"]["claims_correctness"], false);
    assert_eq!(report["boundary"]["claims_semantic_equivalence"], false);
    assert!(
        report["boundary"]["detail"]
            .as_str()
            .expect("boundary detail")
            .contains("do not prove behavioral correctness")
    );
    assert!(
        report["pure_cache_blocked_count"].as_u64().expect("pure cache blocked") > 0,
        "taxonomy should explain blockers, not promote pure-cache eligibility"
    );
}

// Blocked, intentionally red: this checked-in receipt is an external Function
// Object v1 contract. It cannot replay while every source-backed object command
// refuses construction pending a reviewed valence-core normalization label.
#[test]
fn object_corpus_compatibility_positive_v1_fixture_replays_cleanly() {
    let root = repo_root();
    let compatibility_dir = root.join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    let fixture: serde_json::Value = serde_json::from_slice(
        &fs::read(compatibility_dir.join(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1))
            .expect("read positive compatibility fixture"),
    )
    .expect("positive compatibility fixture json");

    assert_eq!(fixture["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(fixture["schema_version"], 1);
    assert_eq!(fixture["source_paths"], serde_json::json!(["src/lib.rs"]));
    assert_eq!(fixture["boundary"]["claims_correctness"], false);
    assert_eq!(fixture["boundary"]["claims_semantic_equivalence"], false);

    let check = check_corpus_receipt_fixture(&root, OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1);
    assert!(
        check.status.success(),
        "positive compatibility fixture stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("positive check report json");
    assert_eq!(report["status"], "match");
    assert!(report["errors"].as_array().expect("errors array").is_empty());
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons array")
            .iter()
            .all(|comparison| comparison["status"] == "match"),
        "positive compatibility comparisons did not all match: {report:#?}"
    );
}

#[test]
fn object_corpus_compatibility_negative_fixtures_fail_closed() {
    let root = repo_root();
    for (fixture, expected_kind, expected_field) in [
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MISSING_OBJECT_SET_HASH,
            "missing-field",
            Some("object_set_hash"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_FUTURE_SCHEMA_VERSION,
            "unsupported-schema-version",
            Some("schema_version"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_DUPLICATE_SOURCE_PATH,
            "duplicate-source-path",
            Some("source_paths"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_EMPTY_SOURCE_SELECTION,
            "empty-source-selection",
            Some("source_paths"),
        ),
        (OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MALFORMED_JSON, "invalid-json", Some("receipt")),
    ] {
        let report = assert_invalid_check_error(&check_corpus_receipt_fixture(&root, fixture), expected_kind);
        if let Some(expected_field) = expected_field {
            assert!(
                report["errors"]
                    .as_array()
                    .expect("errors array")
                    .iter()
                    .any(|error| error["field"] == expected_field),
                "negative fixture {fixture} did not name expected field {expected_field}: {report:#?}"
            );
        }
    }
}

// Blocked, intentionally red for the same reason: the boundary comparison needs
// current objects, and their construction is refused.
#[test]
fn object_corpus_compatibility_boundary_tamper_fixture_fails_closed() {
    let root = repo_root();
    let check = check_corpus_receipt_fixture(&root, OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_BOUNDARY_TAMPER);
    assert!(
        !check.status.success(),
        "boundary tamper compatibility fixture unexpectedly passed stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("boundary tamper report json");
    assert_eq!(report["status"], "mismatch", "report={report:#?}");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons array")
            .iter()
            .any(|comparison| comparison["field"] == "boundary" && comparison["status"] == "mismatch"),
        "expected boundary mismatch for tamper fixture: {report:#?}"
    );
    assert!(report["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn object_corpus_compatibility_source_selection_fixtures_fail_closed() {
    let root = repo_root();
    let compatibility_dir = root.join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    let non_rust_report = assert_invalid_check_error(
        &check_corpus_receipt(&compatibility_dir, Path::new(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1), &[
            OBJECT_CORPUS_COMPATIBILITY_NON_RUST_SOURCE,
        ]),
        "unsupported-source-path",
    );
    assert!(
        non_rust_report["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["field"] == "paths"),
        "non-Rust selection should name paths field: {non_rust_report:#?}"
    );
}

fn assert_invalid_check_error(output: &std::process::Output, expected_kind: &str) -> serde_json::Value {
    assert!(
        !output.status.success(),
        "invalid check unexpectedly passed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid check report json");
    assert_eq!(report["status"], "invalid", "report={report:#?}");
    assert!(
        report["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["kind"] == expected_kind),
        "expected error kind {expected_kind} in {report:#?}"
    );
    assert!(report["comparisons"].as_array().expect("comparisons array").is_empty());
    report
}

#[test]
fn object_corpus_receipt_check_reports_malformed_receipts() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("bad.json");
    fs::write(&receipt_path, "{not json\n").expect("write malformed receipt");

    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    let report = assert_invalid_check_error(&check, "invalid-json");
    assert!(
        report["errors"][0]["message"].as_str().unwrap().contains("parse"),
        "malformed receipt should explain parse failure: {report:#?}"
    );
}

#[test]
fn object_corpus_receipt_check_rejects_unsupported_schema_and_future_version() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, mut receipt) = copy_checked_in_receipt(temp.path());

    receipt["schema"] = serde_json::Value::String("octet.future-corpus-receipt.v9".to_owned());
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write wrong schema receipt");
    assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]), "unsupported-schema");

    receipt["schema"] = serde_json::Value::String("octet.function-object-corpus-receipt.v1".to_owned());
    receipt["schema_version"] = serde_json::Value::Number(serde_json::Number::from(99));
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write future version receipt");
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]),
        "unsupported-schema-version",
    );
}

#[test]
fn object_corpus_receipt_check_rejects_invalid_source_selection() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, _) = copy_checked_in_receipt(temp.path());
    fs::write(temp.path().join("README.md"), "not rust\n").expect("write non-rust fixture");
    fs::write(temp.path().join("empty.rs"), "// no functions here\n").expect("write empty rust fixture");

    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["missing.rs"]),
        "source-path-missing",
    );
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs", "lib.rs"]),
        "duplicate-source-path",
    );
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["README.md"]),
        "unsupported-source-path",
    );
    assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["empty.rs"]), "empty-corpus");
}
