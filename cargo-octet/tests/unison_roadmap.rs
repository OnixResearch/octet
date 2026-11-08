use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_octet() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-octet")
}

#[allow(clippy::needless_pass_by_value)]
fn write_json(path: &Path, value: serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn run(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(cargo_octet()).args(args).current_dir(cwd).output().unwrap()
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
fn replay_capsules_capture_inspect_replay_and_drift() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");
    let drifted = temp.path().join("drifted.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "expected_exit_code": 0,
            "absolute_path": "/private/worktree",
            "raw_secret": "do-not-store"
        }),
    );
    write_json(
        &observed,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "exit_code": 0
        }),
    );
    let captured = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(captured["capsule_kind"], "octet-replay-capsule");
    assert_eq!(captured["manifest"]["octet_binary_identity"], "b3:bin");
    assert!(captured["manifest"].get("slotcar_binary_identity").is_none());
    assert!(captured["capsule_hash"].as_str().unwrap().starts_with("b3:"));
    assert!(!captured.to_string().contains("do-not-store"));

    let inspected =
        stdout_json(&run(&["roadmap", "capsule", "inspect", "--capsule", capsule.to_str().unwrap()], temp.path()));
    assert_eq!(inspected["run_kind"], "check");
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            observed.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["status"], "replayed");

    write_json(
        &drifted,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:changed",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "exit_code": 0
        }),
    );
    let drift = run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            drifted.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!drift.status.success());
    let drift_report: serde_json::Value = serde_json::from_slice(&drift.stdout).unwrap();
    assert_eq!(drift_report["status"], "drifted");
    assert_eq!(drift_report["mismatches"], serde_json::json!(["config_hash"]));
}

#[test]
fn replay_capsules_normalize_legacy_slotcar_identity_field() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("legacy-capsule-input.json");
    let capsule = temp.path().join("legacy-capsule.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "slotcar", "check"],
            "run_kind": "check",
            "slotcar_binary_identity": "b3:legacy-bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "expected_exit_code": 0
        }),
    );

    let captured = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));

    assert_eq!(captured["manifest"]["octet_binary_identity"], "b3:legacy-bin");
    assert!(captured["manifest"].get("slotcar_binary_identity").is_none());
}

#[test]
fn replay_capsules_accept_expected_failures() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("fingerprint-input.json");
    let capsule = temp.path().join("capsule.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "fingerprint", "--check", "src/lib.rs"],
            "run_kind": "fingerprint",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:diff"],
            "expected_exit_code": 1
        }),
    );
    stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let mut observed: serde_json::Value = serde_json::from_str(&fs::read_to_string(&input).unwrap()).unwrap();
    observed["exit_code"] = serde_json::json!(1);
    write_json(&input, observed);
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            input.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["status"], "expected-failure-replayed");
}

fn capsule_manifest_input() -> serde_json::Value {
    serde_json::json!({
        "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
        "run_kind": "check",
        "octet_binary_identity": "b3:bin",
        "config_hash": "b3:cfg",
        "input_hashes": ["b3:src"],
        "output_hashes": ["b3:findings"],
        "expected_exit_code": 0
    })
}

fn capsule_replay(capsule: &Path, observed: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            observed.to_str().unwrap(),
        ],
        cwd,
    )
}

#[test]
fn replay_capsules_fail_closed_on_malformed_capsule_manifests() {
    let temp = tempfile::tempdir().unwrap();
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");

    write_json(&observed, serde_json::json!({"exit_code": 0}));
    write_json(&capsule, serde_json::json!({"capsule_hash": "b3:anything", "capsule_kind": "octet-replay-capsule"}));
    let missing_manifest = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(
        missing_manifest.status.code(),
        Some(2),
        "stdout={}",
        String::from_utf8_lossy(&missing_manifest.stdout)
    );
    assert!(stderr_text(&missing_manifest).contains("manifest must be a JSON object"));

    let mut partial_manifest = capsule_manifest_input();
    let partial_object = partial_manifest.as_object_mut().unwrap();
    partial_object.remove("octet_binary_identity");
    partial_object.remove("expected_exit_code");
    let mut partial_observed = partial_manifest.clone();
    partial_observed["exit_code"] = serde_json::json!(0);
    write_json(&observed, partial_observed);
    write_json(&capsule, serde_json::json!({"capsule_hash": "b3:anything", "manifest": partial_manifest}));
    let missing_field = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(missing_field.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&missing_field.stdout));
    assert!(stderr_text(&missing_field).contains("capsule manifest missing octet_binary_identity"));
}

#[test]
fn replay_capsules_capture_requires_integer_expected_exit_code() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    for expected_exit_code in [serde_json::json!("1"), serde_json::json!(1.5), serde_json::Value::Null] {
        let mut manifest = capsule_manifest_input();
        manifest["expected_exit_code"] = expected_exit_code.clone();
        write_json(&input, manifest);
        let output = run(
            &[
                "roadmap",
                "capsule",
                "capture",
                "--input",
                input.to_str().unwrap(),
                "--capsule",
                capsule.to_str().unwrap(),
            ],
            temp.path(),
        );
        assert_eq!(output.status.code(), Some(2), "expected_exit_code={expected_exit_code}");
        assert!(
            stderr_text(&output).contains("expected_exit_code must be an integer"),
            "expected_exit_code={expected_exit_code} stderr={}",
            stderr_text(&output)
        );
        assert!(!capsule.exists(), "expected_exit_code={expected_exit_code} wrote a capsule");
    }
}

#[test]
fn replay_capsules_detect_manifest_tampering_via_capsule_hash() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");
    write_json(&input, capsule_manifest_input());
    stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));

    let mut tampered: serde_json::Value = serde_json::from_str(&fs::read_to_string(&capsule).unwrap()).unwrap();
    tampered["manifest"]["output_hashes"] = serde_json::json!(["b3:forged"]);
    write_json(&capsule, tampered);
    let mut forged_run = capsule_manifest_input();
    forged_run["output_hashes"] = serde_json::json!(["b3:forged"]);
    forged_run["exit_code"] = serde_json::json!(0);
    write_json(&observed, forged_run);

    let replay = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(replay.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&replay.stdout));
    assert!(stderr_text(&replay).contains("capsule_hash mismatch"), "stderr={}", stderr_text(&replay));
}

#[test]
fn capability_registry_and_summary_report_known_unknown_and_advisory() {
    let temp = tempfile::tempdir().unwrap();
    let registry = stdout_json(&run(&["roadmap", "ability", "registry"], temp.path()));
    assert_eq!(registry["status"], "approved-capability-vocabulary");
    assert!(registry["families"].to_string().contains("filesystem"));
    assert!(
        registry["families"]
            .to_string()
            .contains("core-prohibited-unless-explicit-data-or-named-capability")
    );

    let findings = temp.path().join("findings.json");
    write_json(
        &findings,
        serde_json::json!([
            {"lint": "no_system_time", "effect_family": "time", "capability_status": "known", "enforcement": "hard"},
            {"lint": "no_absolute_path", "effect_family": "filesystem", "capability_status": "known", "enforcement": "advisory"},
            {"lint": "new_lint", "effect_family": "unknown", "capability_status": "unknown", "enforcement": "advisory"}
        ]),
    );
    let summary = stdout_json(&run(
        &[
            "roadmap",
            "ability",
            "summary",
            "--findings",
            findings.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(summary["counts_by_capability"]["time"], 1);
    assert_eq!(summary["counts_by_capability"]["filesystem"], 1);
    assert_eq!(summary["counts_by_capability"]["unknown"], 1);
    assert_eq!(summary["advisory_counts_by_capability"]["filesystem"], 1);
    assert_eq!(summary["unknown_capability_visible"], true);
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
    assert_eq!(inspected["human_label"], "done-review");
    assert_eq!(inspected["previous_targets"][0]["hash"], "b3:old");
    assert_eq!(inspected["review_identity_status"], "matched-identity-not-correctness-proof");
    assert_eq!(inspected["correctness_claim"], "none");
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

fn pointer_set(registry: &Path, kind: &str, hash: &str, receipt: &Path, cwd: &Path) -> serde_json::Value {
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
            kind,
            "--hash",
            hash,
            "--receipt",
            receipt.to_str().unwrap(),
        ],
        cwd,
    ))
}

fn pointer_rollback(registry: &Path, receipt: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "pointer",
            "rollback",
            "--registry",
            registry.to_str().unwrap(),
            "--receipt",
            receipt.to_str().unwrap(),
        ],
        cwd,
    )
}

fn pointer_inspect(registry: &Path, cwd: &Path) -> serde_json::Value {
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        cwd,
    ))
}

#[test]
fn pointer_rollback_restores_previous_artifact_kind() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    let update = pointer_set(&registry, "policy", "b3:new", &receipt2, temp.path());
    assert_eq!(update["old_kind"], "prompt");

    stdout_json(&pointer_rollback(&registry, &receipt2, temp.path()));
    let rolled_back = pointer_inspect(&registry, temp.path());
    assert_eq!(rolled_back["target_hash"], "b3:old");
    assert_eq!(rolled_back["target_artifact_kind"], "prompt");
}

#[test]
fn pointer_rollback_recovers_previous_kind_for_receipts_without_old_kind() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    pointer_set(&registry, "policy", "b3:new", &receipt2, temp.path());
    let mut legacy_receipt: serde_json::Value = serde_json::from_str(&fs::read_to_string(&receipt2).unwrap()).unwrap();
    legacy_receipt.as_object_mut().unwrap().remove("old_kind");
    write_json(&receipt2, legacy_receipt);

    stdout_json(&pointer_rollback(&registry, &receipt2, temp.path()));
    let rolled_back = pointer_inspect(&registry, temp.path());
    assert_eq!(rolled_back["target_hash"], "b3:old");
    assert_eq!(rolled_back["target_artifact_kind"], "prompt");
}

#[test]
fn pointer_rollback_rejects_receipts_superseded_by_later_updates() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    let receipt3 = temp.path().join("receipt3.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    pointer_set(&registry, "prompt", "b3:new", &receipt2, temp.path());
    pointer_set(&registry, "prompt", "b3:newer", &receipt3, temp.path());
    let registry_before = fs::read_to_string(&registry).unwrap();

    let stale = pointer_rollback(&registry, &receipt2, temp.path());
    assert_eq!(stale.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&stale.stdout));
    assert!(stderr_text(&stale).contains("b3:newer"), "stderr={}", stderr_text(&stale));
    assert!(stderr_text(&stale).contains("b3:new`"), "stderr={}", stderr_text(&stale));
    assert_eq!(fs::read_to_string(&registry).unwrap(), registry_before);
    assert_eq!(pointer_inspect(&registry, temp.path())["target_hash"], "b3:newer");

    stdout_json(&pointer_rollback(&registry, &receipt3, temp.path()));
    assert_eq!(pointer_inspect(&registry, temp.path())["target_hash"], "b3:new");
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

#[test]
fn session_migrate_rejects_non_object_sessions_without_panicking() {
    let temp = tempfile::tempdir().unwrap();
    let migrated = temp.path().join("migrated.json");
    for (index, session_json) in ["[]", "1", "\"x\"", "true", "null"].iter().enumerate() {
        let session = temp.path().join(format!("session-{index}.json"));
        fs::write(&session, session_json).unwrap();
        let output = run(
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
        );
        assert_eq!(output.status.code(), Some(2), "session={session_json} stderr={}", stderr_text(&output));
        assert!(
            stderr_text(&output).contains("session must be a JSON object"),
            "session={session_json} stderr={}",
            stderr_text(&output)
        );
        assert!(!migrated.exists(), "session={session_json} wrote migrated output");
    }
}

fn enforce(manifest: serde_json::Value, trace: serde_json::Value, cwd: &Path) -> std::process::Output {
    let manifest_path = cwd.join("enforce-manifest.json");
    let trace_path = cwd.join("enforce-trace.json");
    write_json(&manifest_path, manifest);
    write_json(&trace_path, trace);
    run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--trace",
            trace_path.to_str().unwrap(),
        ],
        cwd,
    )
}

#[test]
fn ability_enforce_requires_string_arrays_for_manifest_and_trace() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = serde_json::json!({"allowed_effects": ["FileRead"]});
    for trace in [
        serde_json::json!({"effects": [{"family": "network"}]}),
        serde_json::json!({"effects": "network"}),
        serde_json::json!({"effect": ["network"]}),
        serde_json::json!({"effects": ["FileRead", null]}),
    ] {
        let output = enforce(manifest.clone(), trace.clone(), temp.path());
        assert_eq!(output.status.code(), Some(2), "trace={trace} stdout={}", String::from_utf8_lossy(&output.stdout));
        assert!(output.stdout.is_empty(), "trace={trace} printed a verdict");
    }
    for bad_manifest in [
        serde_json::json!({"allowed": ["FileRead"]}),
        serde_json::json!({"allowed_effects": "FileRead"}),
        serde_json::json!({"allowed_effects": [1]}),
    ] {
        let output = enforce(bad_manifest.clone(), serde_json::json!({"effects": []}), temp.path());
        assert_eq!(output.status.code(), Some(2), "manifest={bad_manifest}");
    }

    let allowed = stdout_json(&enforce(manifest.clone(), serde_json::json!({"effects": ["FileRead"]}), temp.path()));
    assert_eq!(allowed["status"], "allowed");
    let empty = stdout_json(&enforce(manifest, serde_json::json!({"effects": []}), temp.path()));
    assert_eq!(empty["status"], "allowed");
}
