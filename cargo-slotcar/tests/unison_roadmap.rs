use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_slotcar() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-slotcar")
}

fn write_json(path: &Path, value: serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn run(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(cargo_slotcar()).args(args).current_dir(cwd).output().unwrap()
}

fn stdout_json(output: &std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn stderr_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn artifact_hash_inspect_and_replay_are_content_addressed() {
    let temp = tempfile::tempdir().unwrap();
    let dto_a = temp.path().join("a.json");
    let dto_b = temp.path().join("b.json");
    let store = temp.path().join("store");
    write_json(
        &dto_a,
        serde_json::json!({
            "name": "review-prompt",
            "body": "check this",
            "timestamp": "2026-01-01T00:00:00Z",
            "absolute_path": "/home/brittonr/secret/path",
            "raw_secret": "do-not-store"
        }),
    );
    write_json(
        &dto_b,
        serde_json::json!({
            "absolute_path": "/tmp/other",
            "raw_secret": "different-secret",
            "body": "check this",
            "name": "review-prompt",
            "timestamp": "2027-01-01T00:00:00Z"
        }),
    );

    let a = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            dto_a.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let b = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            dto_b.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(a["hash"], b["hash"]);
    assert!(a["hash"].as_str().unwrap().starts_with("b3:"));
    assert!(a.to_string().contains("redacted-local-and-secret-fields"));
    assert!(!a.to_string().contains("do-not-store"));

    let hash = a["hash"].as_str().unwrap();
    let inspected = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "inspect",
            "--hash",
            hash,
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(inspected["artifact_kind"], "prompt");
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "replay",
            "--hash",
            hash,
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["operation"], "replay");
}

#[test]
fn ability_manifest_denies_undeclared_effects() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("manifest.json");
    let trace = temp.path().join("trace.json");
    write_json(&manifest, serde_json::json!({"allowed_effects": ["FileRead"]}));
    write_json(&trace, serde_json::json!({"effects": ["FileRead", "Network"]}));
    let output = run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            manifest.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "denied");
    assert_eq!(report["denied_effects"], serde_json::json!(["Network"]));
}

#[test]
fn name_pointer_updates_are_rollbackable() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "set",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
            "--kind",
            "prompt",
            "--hash",
            "b3:old",
            "--receipt",
            receipt1.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let update = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "set",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
            "--kind",
            "prompt",
            "--hash",
            "b3:new",
            "--receipt",
            receipt2.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(update["old_hash"], "b3:old");
    let inspected = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        temp.path(),
    ));
    assert_eq!(inspected["target_hash"], "b3:new");
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "rollback",
            "--registry",
            registry.to_str().unwrap(),
            "--receipt",
            receipt2.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let rolled_back = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        temp.path(),
    ));
    assert_eq!(rolled_back["target_hash"], "b3:old");
}

#[test]
fn session_records_todos_and_migrations_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let session = temp.path().join("session.json");
    let migrated = temp.path().join("migrated.json");
    write_json(
        &session,
        serde_json::json!({
            "identity": "b3:session",
            "original_schema_version": 0,
            "records": [
                {"kind": "model_request", "schema_version": 1, "identity": "b3:req", "redaction_status": "redacted"},
                {"kind": "tool_call", "schema_version": 1, "identity": "b3:tool", "redaction_status": "redacted"}
            ],
            "todos": [
                {"kind": "failing_check", "status": "blocked", "source": "V1", "next_action": "rerun", "blocker": "fixture missing"},
                {"kind": "pending_edit", "status": "completed", "source": "I1", "next_action": "none"},
                {"kind": "review_findings", "status": "deferred", "source": "H1", "next_action": "human review"}
            ]
        }),
    );
    let check = stdout_json(&run(&["roadmap", "session", "check", "--input", session.to_str().unwrap()], temp.path()));
    assert_eq!(check["status"], "typed-versioned");
    let migration = stdout_json(&run(
        &[
            "roadmap",
            "session",
            "migrate",
            "--input",
            session.to_str().unwrap(),
            "--output",
            migrated.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(migration["original_identity"], "b3:session");
    assert_eq!(migration["identity_change"], false);
    let todos = stdout_json(&run(&["roadmap", "session", "todo", "--input", session.to_str().unwrap()], temp.path()));
    assert_eq!(todos["counts_by_status"]["blocked"], 1);
    assert_eq!(todos["counts_by_status"]["completed"], 1);
    assert_eq!(todos["counts_by_status"]["deferred"], 1);
}

#[test]
fn pure_cache_hits_only_matching_key_components() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let key1 = temp.path().join("key1.json");
    let key2 = temp.path().join("key2.json");
    write_json(
        &key1,
        serde_json::json!({
            "command_identity": "cargo-check",
            "input_hashes": ["b3:src"],
            "config_hash": "b3:cfg",
            "tool_revision": "rev1",
            "environment_allowlist": {"RUSTFLAGS": ""}
        }),
    );
    write_json(
        &key2,
        serde_json::json!({
            "command_identity": "cargo-check",
            "input_hashes": ["b3:src"],
            "config_hash": "b3:cfg",
            "tool_revision": "rev2",
            "environment_allowlist": {"RUSTFLAGS": ""}
        }),
    );
    let miss = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key1.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    ));
    assert_eq!(miss["status"], "miss");
    let hit = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key1.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    ));
    assert_eq!(hit["status"], "hit");
    assert_eq!(hit["stored_output_hash"], "b3:out");
    let drift = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key2.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out2",
        ],
        temp.path(),
    ));
    assert_eq!(drift["status"], "miss");
    assert_eq!(drift["changed_key_component"], "new-or-changed-key");
}

#[test]
fn roadmap_docs_transcript_covers_agent_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let transcript_path = root.join("docs/examples/roadmap-agent-surfaces/transcript.md");
    let transcript = fs::read_to_string(&transcript_path).expect("read roadmap transcript");

    let output = run(&["transcript", transcript_path.to_str().unwrap()], root);
    assert!(output.status.success(), "stderr={}", stderr_text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_blocks=10"));
    assert!(stdout.contains("expected_output_blocks=10"));
    assert!(stdout.contains("status=passed"));

    for surface in ["artifact", "ability", "pointer", "session", "todo", "pure-cache"] {
        assert!(transcript.contains(surface), "roadmap transcript missing {surface}");
    }
}

#[test]
fn roadmap_error_paths_fail_closed_with_operator_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing_artifact = run(
        &[
            "roadmap",
            "artifact",
            "inspect",
            "--hash",
            "b3:missing",
            "--store-dir",
            temp.path().join("store").to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!missing_artifact.status.success());
    assert!(stderr_text(&missing_artifact).contains("read"));
    assert!(stderr_text(&missing_artifact).contains("b3_missing.json"));

    let malformed_manifest = temp.path().join("manifest.json");
    let trace = temp.path().join("trace.json");
    fs::write(&malformed_manifest, "not json").unwrap();
    write_json(&trace, serde_json::json!({"effects": ["Network"]}));
    let malformed = run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            malformed_manifest.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!malformed.status.success());
    assert!(stderr_text(&malformed).contains("parse"));
    assert!(stderr_text(&malformed).contains("manifest.json"));

    let bad_session = temp.path().join("session.json");
    write_json(&bad_session, serde_json::json!({"records": [{"kind": "tool_call"}]}));
    let session = run(&["roadmap", "session", "check", "--input", bad_session.to_str().unwrap()], temp.path());
    assert!(!session.status.success());
    assert!(
        stderr_text(&session).contains("session record missing kind, schema_version, identity, or redaction_status")
    );

    let bad_key = temp.path().join("key.json");
    write_json(&bad_key, serde_json::json!({"command_identity": "cargo-check"}));
    let cache = run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            bad_key.to_str().unwrap(),
            "--store-dir",
            temp.path().join("cache").to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    );
    assert!(!cache.status.success());
    assert!(stderr_text(&cache).contains("cache key missing input_hashes"));
}
