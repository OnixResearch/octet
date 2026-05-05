use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_tigerstyle() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-tigerstyle")
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
        r#"macro_rules! define_parametric {
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
"#,
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
        r#"pub mod nested;

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
"#,
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
        r#"macro_rules! literal_test_fn {
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
"#,
    )
    .expect("write integration fixture");
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

#[test]
fn fingerprint_lifecycle_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-lifecycle");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("lifecycle-transcript.md")).expect("read lifecycle transcript");

    let final_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run docs final check");
    assert!(
        final_check.status.success(),
        "docs final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("bounded_retry_delay_ms match"));
    assert!(final_stdout.contains("summary total=1 match=1 mismatch=0 missing=0 written=0"));
    assert!(transcript.contains(final_stdout.as_ref()), "transcript missing final check output:\n{final_stdout}");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"#[rustfmt::skip]
// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    50_u32.saturating_mul(attempt.saturating_add(1))
}
"#,
    )
    .expect("write docs lifecycle initial fixture");

    let initial_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run initial docs update");
    assert!(
        initial_update.status.success(),
        "initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "initial update");

    let clean_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run initial docs check");
    assert!(clean_check.status.success(), "clean check stderr: {}", String::from_utf8_lossy(&clean_check.stderr));
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "clean check");

    let source = fs::read_to_string(&fixture).expect("read lifecycle fixture");
    fs::write(
        &fixture,
        source.replace(
            "50_u32.saturating_mul(attempt.saturating_add(1))",
            "75_u32.saturating_mul(attempt.saturating_add(1))",
        ),
    )
    .expect("mutate lifecycle fixture");

    let drift_diff = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--diff", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs drift diff");
    assert!(!drift_diff.status.success(), "drift diff should fail");
    assert_transcript_contains_stdout(&transcript, &drift_diff.stdout, "drift diff");

    let repair_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs repair update");
    assert!(
        repair_update.status.success(),
        "repair update stderr: {}",
        String::from_utf8_lossy(&repair_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repair_update.stdout, "repair update");

    let repaired_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs repaired check");
    assert!(
        repaired_check.status.success(),
        "repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repaired_check.stdout, "repaired check");

    let repaired_source = fs::read_to_string(&fixture).expect("read repaired lifecycle fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final docs fixture");
    assert_eq!(repaired_source, checked_in_source);
}

fn assert_transcript_contains_stdout(transcript: &str, stdout: &[u8], label: &str) {
    let text = String::from_utf8_lossy(stdout);
    assert!(transcript.contains(text.as_ref()), "transcript missing {label} output:\n{text}");
}

#[test]
fn fingerprint_human_summary_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-human-summary");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read human summary transcript");

    let final_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run human summary final check");
    assert!(
        final_check.status.success(),
        "human summary final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("capped_batch_size match"));
    assert!(final_stdout.contains("describe_batch match"));
    assert!(final_stdout.contains("summary total=2 match=2 mismatch=0 missing=0 written=0"));
    assert!(
        transcript.contains(final_stdout.as_ref()),
        "transcript missing final human summary check output:\n{final_stdout}"
    );

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// tigerstyle-fingerprint: b3:0; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(128)
}

// tigerstyle-fingerprint: b3:0; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md; macro=allow
pub fn describe_batch(size: u32) -> String {
    format!("batch={size}")
}
"#,
    )
    .expect("write human summary initial fixture");

    let initial_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary initial update");
    assert!(
        initial_update.status.success(),
        "human summary initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "human summary initial update");

    let clean_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary clean check");
    assert!(
        clean_check.status.success(),
        "human summary clean check stderr: {}",
        String::from_utf8_lossy(&clean_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "human summary clean check");

    let source = fs::read_to_string(&fixture).expect("read human summary fixture");
    fs::write(&fixture, source.replace("requested.min(128)", "requested.min(256)"))
        .expect("mutate human summary fixture");

    let drift_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary drift check");
    assert!(!drift_check.status.success(), "human summary drift check should fail");
    let drift_stdout = String::from_utf8_lossy(&drift_check.stdout);
    assert!(drift_stdout.contains("summary total=2 match=1 mismatch=1 missing=0 written=0"));
    assert_transcript_contains_stdout(&transcript, &drift_check.stdout, "human summary drift check");

    let repair_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary repair update");
    assert!(
        repair_update.status.success(),
        "human summary repair update stderr: {}",
        String::from_utf8_lossy(&repair_update.stderr)
    );
    let repair_stdout = String::from_utf8_lossy(&repair_update.stdout);
    assert!(repair_stdout.contains("summary total=2 match=1 mismatch=0 missing=0 written=1"));
    assert_transcript_contains_stdout(&transcript, &repair_update.stdout, "human summary repair update");

    let repaired_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary repaired check");
    assert!(
        repaired_check.status.success(),
        "human summary repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repaired_check.stdout, "human summary repaired check");

    let repaired_source = fs::read_to_string(&fixture).expect("read repaired human summary fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final human summary fixture");
    assert_eq!(repaired_source, checked_in_source);
}

#[test]
fn fingerprint_warning_summary_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-warning-summary");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read warning summary transcript");

    let final_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run warning summary final check");
    assert!(!final_check.status.success(), "warning summary final check should fail on warning counts");
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("unreviewed_macro_path match"));
    assert!(final_stdout.contains("missing_gate_metadata match"));
    assert!(
        final_stdout
            .contains("summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1")
    );
    assert!(
        transcript.contains(final_stdout.as_ref()),
        "transcript missing final warning summary check output:\n{final_stdout}"
    );

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// tigerstyle-fingerprint: b3:0; review=macro-warning-example; evidence=docs/examples/function-fingerprint-warning-summary/transcript.md
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// tigerstyle-fingerprint: b3:0
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
"#,
    )
    .expect("write warning summary initial fixture");

    let initial_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run warning summary initial update");
    assert!(
        initial_update.status.success(),
        "warning summary initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    let initial_stdout = String::from_utf8_lossy(&initial_update.stdout);
    assert!(
        initial_stdout
            .contains("summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1")
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "warning summary initial update");

    let clean_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run warning summary clean check");
    assert!(!clean_check.status.success(), "warning summary clean check should fail on warning counts");
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "warning summary clean check");

    let updated_source = fs::read_to_string(&fixture).expect("read updated warning summary fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final warning summary fixture");
    assert_eq!(updated_source, checked_in_source);
}

fn assert_fingerprint_json_record(output: &[u8], mode: &str, status: &str, stored_hash: &str) {
    let json: serde_json::Value = serde_json::from_slice(output).expect("fingerprint json output");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["mode"], mode);
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["path"], "reviewed.rs");
    assert_eq!(record["name"], "capped_retries");
    assert_eq!(record["status"], status);
    assert_eq!(record["stored_hash"], stored_hash);
    assert_eq!(record["review"], "json-script-example");
    assert_eq!(record["evidence"], "docs/examples/function-fingerprint-json-output/transcript.md");
    assert_eq!(record["macro_calls"], false);
    assert_eq!(record["macro_warning"], false);
}

#[test]
fn fingerprint_json_output_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-json-output");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read json output transcript");

    let final_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run json output final check");
    assert!(
        final_check.status.success(),
        "json output final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    assert_fingerprint_json_record(
        &final_check.stdout,
        "check",
        "match",
        "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
    );
    assert_transcript_contains_stdout(&transcript, &final_check.stdout, "json output final check");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// tigerstyle-fingerprint: b3:0; review=json-script-example; evidence=docs/examples/function-fingerprint-json-output/transcript.md; macro=allow
pub fn capped_retries(requested: u32) -> u32 {
    requested.min(3)
}
"#,
    )
    .expect("write json output initial fixture");

    let initial_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output initial update");
    assert!(initial_update.status.success());
    assert_fingerprint_json_record(&initial_update.stdout, "update", "written", "0");
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "json output initial update");

    let clean_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output clean check");
    assert!(clean_check.status.success());
    assert_fingerprint_json_record(
        &clean_check.stdout,
        "check",
        "match",
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
    );
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "json output clean check");

    let source = fs::read_to_string(&fixture).expect("read json output fixture");
    fs::write(&fixture, source.replace("requested.min(3)", "requested.min(5)")).expect("mutate json output fixture");

    let drift_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output drift check");
    assert!(!drift_check.status.success(), "json output drift check should fail");
    assert_fingerprint_json_record(
        &drift_check.stdout,
        "check",
        "mismatch",
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
    );
    assert_transcript_contains_stdout(&transcript, &drift_check.stdout, "json output drift check");

    let accepted_update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output accepted update");
    assert!(accepted_update.status.success());
    assert_fingerprint_json_record(
        &accepted_update.stdout,
        "update",
        "written",
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
    );
    assert_transcript_contains_stdout(&transcript, &accepted_update.stdout, "json output accepted update");

    let accepted_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output accepted check");
    assert!(accepted_check.status.success());
    assert_fingerprint_json_record(
        &accepted_check.stdout,
        "check",
        "match",
        "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
    );
    assert_transcript_contains_stdout(&transcript, &accepted_check.stdout, "json output accepted check");

    let accepted_source = fs::read_to_string(&fixture).expect("read accepted json output fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final json output fixture");
    assert_eq!(accepted_source, checked_in_source);
}

#[test]
fn fingerprint_source_scanner_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-source-scanner");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read source scanner transcript");

    let final_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run source scanner final check");
    assert!(
        final_check.status.success(),
        "source scanner final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&final_check.stdout).expect("source scanner json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_generated_boundary");
    assert_ne!(records[0]["name"], "generated_boundary");
    assert_eq!(records[0]["status"], "match");
    assert_eq!(records[0]["review"], "source-scanner-boundary");
    assert_transcript_contains_stdout(&transcript, &final_check.stdout, "source scanner final json check");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"macro_rules! define_generated_boundary {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input.saturating_add(1)
        }
    };
}

define_generated_boundary!(generated_boundary);

// tigerstyle-fingerprint: b3:0; review=source-scanner-boundary; evidence=docs/examples/function-fingerprint-source-scanner/transcript.md
pub fn explicit_generated_boundary(input: u32) -> u32 {
    generated_boundary(input).min(64)
}
"#,
    )
    .expect("write source scanner initial fixture");

    let update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner update");
    assert!(update.status.success());
    let update_stdout = String::from_utf8_lossy(&update.stdout);
    assert!(update_stdout.contains("summary total=1 match=0 mismatch=0 missing=0 written=1"));
    assert!(update_stdout.contains("explicit_generated_boundary written"));
    assert!(!update_stdout.contains("reviewed.rs generated_boundary "));
    assert_transcript_contains_stdout(&transcript, &update.stdout, "source scanner update");

    let json_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner json check");
    assert!(json_check.status.success());
    assert_transcript_contains_stdout(&transcript, &json_check.stdout, "source scanner json check");
    let json: serde_json::Value = serde_json::from_slice(&json_check.stdout).expect("source scanner json check");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_generated_boundary");

    let human_check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner human check");
    assert!(human_check.status.success());
    assert_transcript_contains_stdout(&transcript, &human_check.stdout, "source scanner human check");

    let updated_source = fs::read_to_string(&fixture).expect("read source scanner fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final source scanner fixture");
    assert_eq!(updated_source, checked_in_source);
}

#[test]
fn fingerprint_write_then_check_works_for_external_crate_file() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let check = Command::new(cargo_tigerstyle())
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

    let update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let check = Command::new(cargo_tigerstyle())
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

    let write = Command::new(cargo_tigerstyle())
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
    assert_eq!(updated.matches("tigerstyle-fingerprint").count(), 1);

    let check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn fingerprint_update_check_and_diff_walk_directory_tree() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_tigerstyle())
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

    let check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
    assert_directory_records(&check.stdout, &expected, "match");

    let diff = Command::new(cargo_tigerstyle())
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

    let update = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let lib_rs = temp.path().join("src/lib.rs");
    let source = fs::read_to_string(&lib_rs).expect("read lib.rs");
    fs::write(&lib_rs, source.replace("input.saturating_add(1)", "input.saturating_add(2)"))
        .expect("mutate marked function");

    let check = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "drift check should fail");

    let human_check = Command::new(cargo_tigerstyle())
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

    let diff = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint drift diff");
    assert!(!diff.status.success(), "drift diff should fail");

    let diff_json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json output");
    let diff_records = diff_json["records"].as_array().expect("diff records array");
    assert_single_directory_mismatch(diff_records);

    let repair = Command::new(cargo_tigerstyle())
        .args(["fingerprint", "--update"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint repair update");
    assert!(repair.status.success(), "repair stderr: {}", String::from_utf8_lossy(&repair.stderr));
    assert_human_summary(&repair.stdout, "match=5 mismatch=0 missing=0 written=1");
    assert_human_contains_record(&repair.stdout, "root_plain", "written");

    let repaired_check = Command::new(cargo_tigerstyle())
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

    let repaired_diff = Command::new(cargo_tigerstyle())
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
