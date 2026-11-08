use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use crate::determinism;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub enum RoadmapCommand {
    ArtifactStore {
        kind: String,
        input: PathBuf,
        store_dir: PathBuf,
    },
    ArtifactInspect {
        hash: String,
        store_dir: PathBuf,
    },
    ArtifactReplay {
        hash: String,
        store_dir: PathBuf,
    },
    CapsuleCapture {
        input: PathBuf,
        capsule: PathBuf,
    },
    CapsuleInspect {
        capsule: PathBuf,
    },
    CapsuleReplay {
        capsule: PathBuf,
        observed: PathBuf,
    },
    AbilityRegistry,
    AbilitySummary {
        findings: PathBuf,
    },
    AbilityEnforce {
        manifest: PathBuf,
        trace: PathBuf,
    },
    PointerSet {
        registry: PathBuf,
        name: String,
        kind: String,
        hash: String,
        receipt: PathBuf,
    },
    PointerInspect {
        registry: PathBuf,
        name: String,
    },
    PointerRollback {
        registry: PathBuf,
        receipt: PathBuf,
    },
    SessionCheck {
        input: PathBuf,
    },
    SessionMigrate {
        input: PathBuf,
        output: PathBuf,
    },
    SessionTodo {
        input: PathBuf,
    },
    PureCacheCheck {
        key: PathBuf,
        store_dir: PathBuf,
        output_hash: String,
    },
}

pub fn parse(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("artifact") => parse_artifact(&args[1..]),
        Some("capsule") => parse_capsule(&args[1..]),
        Some("ability") => parse_ability(&args[1..]),
        Some("pointer") => parse_pointer(&args[1..]),
        Some("session") => parse_session(&args[1..]),
        Some("pure-cache") => parse_pure_cache(&args[1..]),
        Some(other) => Err(format!("unknown roadmap subcommand `{other}`")),
        None => Err("missing roadmap subcommand".to_owned()),
    }
}

pub fn run(command: RoadmapCommand) -> Result<i32, String> {
    match command {
        RoadmapCommand::ArtifactStore { kind, input, store_dir } => artifact_store(&kind, &input, &store_dir),
        RoadmapCommand::ArtifactInspect { hash, store_dir } => artifact_inspect(&hash, &store_dir, "inspect"),
        RoadmapCommand::ArtifactReplay { hash, store_dir } => artifact_replay(&hash, &store_dir),
        RoadmapCommand::CapsuleCapture { input, capsule } => capsule_capture(&input, &capsule),
        RoadmapCommand::CapsuleInspect { capsule } => capsule_inspect(&capsule),
        RoadmapCommand::CapsuleReplay { capsule, observed } => capsule_replay(&capsule, &observed),
        RoadmapCommand::AbilityRegistry => ability_registry(),
        RoadmapCommand::AbilitySummary { findings } => ability_summary(&findings),
        RoadmapCommand::AbilityEnforce { manifest, trace } => ability_enforce(&manifest, &trace),
        RoadmapCommand::PointerSet {
            registry,
            name,
            kind,
            hash,
            receipt,
        } => pointer_set(&registry, &name, &kind, &hash, &receipt),
        RoadmapCommand::PointerInspect { registry, name } => pointer_inspect(&registry, &name),
        RoadmapCommand::PointerRollback { registry, receipt } => pointer_rollback(&registry, &receipt),
        RoadmapCommand::SessionCheck { input } => session_check(&input),
        RoadmapCommand::SessionMigrate { input, output } => session_migrate(&input, &output),
        RoadmapCommand::SessionTodo { input } => session_todo(&input),
        RoadmapCommand::PureCacheCheck {
            key,
            store_dir,
            output_hash,
        } => pure_cache_check(&key, &store_dir, &output_hash),
    }
}

fn parse_artifact(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("store") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--kind", "--input", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactStore {
                kind: flags.take("--kind")?,
                input: flags.take_path("--input")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--hash", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactInspect {
                hash: flags.take("--hash")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some("replay") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--hash", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactReplay {
                hash: flags.take("--hash")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some(other) => Err(format!("unknown artifact action `{other}`")),
        None => Err("missing artifact action".to_owned()),
    }
}

fn parse_capsule(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("capture") => {
            let mut flags = RoadmapFlags::parse("capsule", args, &["--input", "--capsule"])?;
            Ok(RoadmapCommand::CapsuleCapture {
                input: flags.take_path("--input")?,
                capsule: flags.take_path("--capsule")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("capsule", args, &["--capsule"])?;
            Ok(RoadmapCommand::CapsuleInspect {
                capsule: flags.take_path("--capsule")?,
            })
        }
        Some("replay") => {
            let mut flags = RoadmapFlags::parse("capsule", args, &["--capsule", "--observed"])?;
            Ok(RoadmapCommand::CapsuleReplay {
                capsule: flags.take_path("--capsule")?,
                observed: flags.take_path("--observed")?,
            })
        }
        Some(other) => Err(format!("unknown capsule action `{other}`")),
        None => Err("missing capsule action".to_owned()),
    }
}

fn parse_ability(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("registry") => {
            RoadmapFlags::parse("ability", args, &[])?;
            Ok(RoadmapCommand::AbilityRegistry)
        }
        Some("summary") => {
            let mut flags = RoadmapFlags::parse("ability", args, &["--findings"])?;
            Ok(RoadmapCommand::AbilitySummary {
                findings: flags.take_path("--findings")?,
            })
        }
        Some("enforce") => {
            let mut flags = RoadmapFlags::parse("ability", args, &["--manifest", "--trace"])?;
            Ok(RoadmapCommand::AbilityEnforce {
                manifest: flags.take_path("--manifest")?,
                trace: flags.take_path("--trace")?,
            })
        }
        Some(other) => Err(format!("unknown ability action `{other}`")),
        None => Err("missing ability action".to_owned()),
    }
}

fn parse_pointer(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("set") => {
            let mut flags =
                RoadmapFlags::parse("pointer", args, &["--registry", "--name", "--kind", "--hash", "--receipt"])?;
            Ok(RoadmapCommand::PointerSet {
                registry: flags.take_path("--registry")?,
                name: flags.take("--name")?,
                kind: flags.take("--kind")?,
                hash: flags.take("--hash")?,
                receipt: flags.take_path("--receipt")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("pointer", args, &["--registry", "--name"])?;
            Ok(RoadmapCommand::PointerInspect {
                registry: flags.take_path("--registry")?,
                name: flags.take("--name")?,
            })
        }
        Some("rollback") => {
            let mut flags = RoadmapFlags::parse("pointer", args, &["--registry", "--receipt"])?;
            Ok(RoadmapCommand::PointerRollback {
                registry: flags.take_path("--registry")?,
                receipt: flags.take_path("--receipt")?,
            })
        }
        Some(other) => Err(format!("unknown pointer action `{other}`")),
        None => Err("missing pointer action".to_owned()),
    }
}

fn parse_session(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("check") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input"])?;
            Ok(RoadmapCommand::SessionCheck {
                input: flags.take_path("--input")?,
            })
        }
        Some("migrate") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input", "--output"])?;
            Ok(RoadmapCommand::SessionMigrate {
                input: flags.take_path("--input")?,
                output: flags.take_path("--output")?,
            })
        }
        Some("todo") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input"])?;
            Ok(RoadmapCommand::SessionTodo {
                input: flags.take_path("--input")?,
            })
        }
        Some(other) => Err(format!("unknown session action `{other}`")),
        None => Err("missing session action".to_owned()),
    }
}

fn parse_pure_cache(args: &[String]) -> Result<RoadmapCommand, String> {
    if !matches!(args.first().map(String::as_str), Some("check")) {
        return Err("pure-cache expects `check`".to_owned());
    }
    let mut flags = RoadmapFlags::parse("pure-cache", args, &["--key", "--store-dir", "--output-hash"])?;
    Ok(RoadmapCommand::PureCacheCheck {
        key: flags.take_path("--key")?,
        store_dir: flags.take_path("--store-dir")?,
        output_hash: flags.take("--output-hash")?,
    })
}

/// `--flag VALUE` / `--flag=VALUE` options of one roadmap action, each given at most once.
struct RoadmapFlags {
    values: BTreeMap<&'static str, String>,
}

impl RoadmapFlags {
    /// Parses `args` (the action name followed by its options), rejecting flags outside `allowed`,
    /// stray positional arguments, repeated flags, and missing values (including a following
    /// `--flag`).
    fn parse(surface: &str, args: &[String], allowed: &[&'static str]) -> Result<Self, String> {
        let action = args.first().map_or("", String::as_str);
        let mut values = BTreeMap::new();
        let mut index = 1;
        while let Some(arg) = args.get(index) {
            let (name, inline_value) = match arg.split_once('=') {
                Some((name, value)) if name.starts_with("--") => (name, Some(value)),
                _ => (arg.as_str(), None),
            };
            let Some(flag) = allowed.iter().copied().find(|flag| *flag == name) else {
                return Err(if arg.starts_with('-') {
                    format!("unsupported roadmap {surface} {action} argument `{arg}`")
                } else {
                    format!("unexpected roadmap {surface} {action} argument `{arg}`")
                });
            };
            let value = if let Some(value) = inline_value {
                index += 1;
                value
            } else {
                let value = args.get(index + 1).map_or("", String::as_str);
                index += 2;
                if value.starts_with("--") { "" } else { value }
            };
            if value.is_empty() {
                return Err(format!("missing value after `{flag}`"));
            }
            if values.insert(flag, value.to_owned()).is_some() {
                return Err(format!("`{flag}` may be provided only once"));
            }
        }
        Ok(Self { values })
    }

    fn take(&mut self, flag: &str) -> Result<String, String> {
        self.values.remove(flag).ok_or_else(|| format!("missing `{flag}`"))
    }

    fn take_path(&mut self, flag: &str) -> Result<PathBuf, String> {
        self.take(flag).map(PathBuf::from)
    }
}

#[derive(Serialize)]
struct ArtifactRecord {
    schema_version: u32,
    artifact_kind: String,
    hash: String,
    normalized: Value,
    dependencies: Vec<String>,
    redaction_status: &'static str,
}

fn artifact_store(kind: &str, input: &Path, store_dir: &Path) -> Result<i32, String> {
    let raw = read_json(input)?;
    let normalized = normalize_value(raw);
    let canonical = canonical_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "artifact_kind": kind,
        "normalized": normalized,
    }))?;
    let hash = determinism::full_hash(&canonical);
    let dependencies = normalized
        .get("dependencies")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default();
    let record = ArtifactRecord {
        schema_version: SCHEMA_VERSION,
        artifact_kind: kind.to_owned(),
        hash: hash.clone(),
        normalized,
        dependencies,
        redaction_status: "redacted-local-and-secret-fields",
    };
    fs::create_dir_all(store_dir).map_err(|error| format!("create {}: {error}", store_dir.display()))?;
    write_json(&store_dir.join(hash_file_name(&hash)), &record)?;
    print_json(&record)?;
    Ok(0)
}

fn artifact_inspect(hash: &str, store_dir: &Path, operation: &'static str) -> Result<i32, String> {
    let record = read_json(&store_dir.join(hash_file_name(hash)))?;
    let report = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": operation,
        "artifact_kind": record["artifact_kind"],
        "hash": record["hash"],
        "dependencies": record["dependencies"],
        "replay_prerequisites": record["dependencies"],
    });
    print_json(&report)?;
    Ok(0)
}

fn artifact_replay(hash: &str, store_dir: &Path) -> Result<i32, String> {
    let record = read_json(&store_dir.join(hash_file_name(hash)))?;
    for dependency in record["dependencies"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        let dependency_path = store_dir.join(hash_file_name(dependency));
        if !dependency_path.exists() {
            let report = serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "operation": "replay",
                "hash": hash,
                "status": "failed-closed",
                "missing_dependency": dependency,
            });
            print_json(&report)?;
            return Ok(1);
        }
    }
    artifact_inspect(hash, store_dir, "replay")
}

fn capsule_capture(input: &Path, capsule_path: &Path) -> Result<i32, String> {
    let mut manifest = normalize_value(read_json(input)?);
    for field in [
        "command_argv",
        "run_kind",
        "config_hash",
        "input_hashes",
        "output_hashes",
        "expected_exit_code",
    ] {
        if manifest.get(field).is_none() {
            return Err(format!("capsule input missing {field}"));
        }
    }
    if manifest.get("expected_exit_code").and_then(Value::as_i64).is_none() {
        return Err("capsule input expected_exit_code must be an integer".to_owned());
    }
    if manifest.get("octet_binary_identity").is_none() {
        let Some(legacy_identity) = manifest.get("slotcar_binary_identity").cloned() else {
            return Err("capsule input missing octet_binary_identity".to_owned());
        };
        manifest["octet_binary_identity"] = legacy_identity;
    }
    if let Some(object) = manifest.as_object_mut() {
        object.remove("slotcar_binary_identity");
    }
    let canonical = canonical_json(&manifest)?;
    let capsule_hash = determinism::full_hash(&canonical);
    let capsule = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "capsule_hash": capsule_hash,
        "capsule_kind": "octet-replay-capsule",
        "manifest": manifest,
        "replay_contract": {
            "input_match": "required",
            "output_match": "required",
            "expected_failure_allowed": true,
            "drift_behavior": "fail-closed"
        },
        "redaction_status": "normalized-no-secret-values-or-local-paths"
    });
    write_json(capsule_path, &capsule)?;
    print_json(&capsule)?;
    Ok(0)
}

fn capsule_inspect(capsule_path: &Path) -> Result<i32, String> {
    let capsule = read_json(capsule_path)?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": "inspect",
        "capsule_hash": capsule["capsule_hash"],
        "run_kind": capsule["manifest"]["run_kind"],
        "command_argv": capsule["manifest"]["command_argv"],
        "input_hashes": capsule["manifest"]["input_hashes"],
        "output_hashes": capsule["manifest"]["output_hashes"],
        "expected_exit_code": capsule["manifest"]["expected_exit_code"],
        "redaction_status": capsule["redaction_status"],
    }))?;
    Ok(0)
}

fn capsule_replay(capsule_path: &Path, observed_path: &Path) -> Result<i32, String> {
    let capsule = read_json(capsule_path)?;
    let mut observed = normalize_value(read_json(observed_path)?);
    if observed.get("octet_binary_identity").is_none()
        && let Some(legacy_identity) = observed.get("slotcar_binary_identity").cloned()
    {
        observed["octet_binary_identity"] = legacy_identity;
    }
    if let Some(object) = observed.as_object_mut() {
        object.remove("slotcar_binary_identity");
    }
    let manifest = capsule
        .get("manifest")
        .filter(|manifest| manifest.is_object())
        .ok_or_else(|| format!("capsule {} manifest must be a JSON object", capsule_path.display()))?;
    if let Some(field) = CAPSULE_COMPARED_FIELDS.iter().find(|field| manifest.get(**field).is_none()) {
        return Err(format!("capsule manifest missing {field}"));
    }
    let expected_exit = manifest
        .get("expected_exit_code")
        .and_then(Value::as_i64)
        .ok_or_else(|| "capsule manifest expected_exit_code must be an integer".to_owned())?;
    let recorded_hash = capsule
        .get("capsule_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "capsule missing capsule_hash".to_owned())?;
    let recomputed_hash = determinism::full_hash(&canonical_json(manifest)?);
    if recorded_hash != recomputed_hash {
        return Err(format!(
            "capsule_hash mismatch: capsule records `{recorded_hash}` but its manifest hashes to `{recomputed_hash}`; the capsule was modified after capture"
        ));
    }
    let mismatches = capsule_mismatches(manifest, &observed);
    let observed_exit = observed["exit_code"].as_i64().unwrap_or(i64::MIN);
    let status = if mismatches.is_empty() && expected_exit == observed_exit {
        if expected_exit == 0 {
            "replayed"
        } else {
            "expected-failure-replayed"
        }
    } else {
        "drifted"
    };
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": "replay",
        "capsule_hash": capsule["capsule_hash"],
        "status": status,
        "expected_exit_code": expected_exit,
        "observed_exit_code": observed_exit,
        "mismatches": mismatches,
        "drift_behavior": "fail-closed",
    }))?;
    Ok(i32::from(status == "drifted"))
}

const CAPSULE_COMPARED_FIELDS: [&str; 6] = [
    "command_argv",
    "run_kind",
    "octet_binary_identity",
    "config_hash",
    "input_hashes",
    "output_hashes",
];

fn capsule_mismatches(manifest: &Value, observed: &Value) -> Vec<String> {
    let mut mismatches = Vec::new();
    for field in CAPSULE_COMPARED_FIELDS {
        match (manifest.get(field), observed.get(field)) {
            (Some(expected), Some(actual)) if expected == actual => {}
            _ => mismatches.push(field.to_owned()),
        }
    }
    mismatches
}

fn ability_registry() -> Result<i32, String> {
    let families = [
        ("time", "ambient time or clock authority"),
        ("randomness", "non-deterministic random data"),
        ("filesystem", "filesystem reads, writes, metadata, and paths"),
        ("network", "network sockets, DNS, and HTTP-like authority"),
        ("process", "process spawning and subprocess control"),
        ("environment", "environment variables and host configuration"),
        ("secrets", "secret material or credential access"),
        ("remote-execution", "remote machine or service execution"),
        ("unsafe-code", "unsafe Rust or unchecked low-level authority"),
        ("global-state", "mutable static or ambient global state"),
        ("concurrency", "threading, task, channel, or synchronization authority"),
        ("unknown", "explicit unmapped capability status"),
    ];
    let entries: Vec<_> = families
        .iter()
        .map(|(family, description)| {
            serde_json::json!({
                "capability_family": family,
                "description": description,
                "fcis_core_policy": if *family == "unknown" { "must-be-made-explicit" } else { "core-prohibited-unless-explicit-data-or-named-capability" },
                "shell_policy": "interpreter-responsibility",
            })
        })
        .collect();
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "approved-capability-vocabulary",
        "families": entries,
    }))?;
    Ok(0)
}

fn ability_summary(findings_path: &Path) -> Result<i32, String> {
    let findings = read_json(findings_path)?;
    let array = findings.as_array().ok_or_else(|| "findings must be a JSON array".to_owned())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut advisory: BTreeMap<String, usize> = BTreeMap::new();
    for finding in array {
        let family = finding.get("effect_family").and_then(Value::as_str).unwrap_or("unknown");
        let status = finding.get("capability_status").and_then(Value::as_str).unwrap_or(if family == "unknown" {
            "unknown"
        } else {
            "known"
        });
        let key = if status == "unknown" { "unknown" } else { family };
        *counts.entry(key.to_owned()).or_default() += 1;
        if finding.get("enforcement").and_then(Value::as_str) == Some("advisory") {
            *advisory.entry(key.to_owned()).or_default() += 1;
        }
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "capability-summary",
        "counts_by_capability": counts,
        "advisory_counts_by_capability": advisory,
        "unknown_capability_visible": counts.contains_key("unknown"),
    }))?;
    Ok(0)
}

fn ability_enforce(manifest_path: &Path, trace_path: &Path) -> Result<i32, String> {
    let manifest = read_json(manifest_path)?;
    let trace = read_json(trace_path)?;
    let allowed = string_array(&manifest, "allowed_effects", manifest_path)?;
    let effects = string_array(&trace, "effects", trace_path)?;
    let denied: Vec<String> = effects.iter().filter(|effect| !allowed.contains(effect)).cloned().collect();
    let status = if denied.is_empty() { "allowed" } else { "denied" };
    let report = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "allowed_effects": allowed,
        "observed_effects": effects,
        "denied_effects": denied,
        "redaction_status": "no-secret-values-or-absolute-paths",
    });
    print_json(&report)?;
    Ok(i32::from(status != "allowed"))
}

/// `document[field]` as strings; a missing field, a non-array, or a non-string entry is an error.
fn string_array(document: &Value, field: &str, path: &Path) -> Result<Vec<String>, String> {
    let items = document
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: {field} must be an array of strings", path.display()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{}: {field}[{index}] must be a string", path.display()))
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
struct PointerRegistry {
    pointers: BTreeMap<String, PointerTarget>,
    #[serde(default)]
    history: BTreeMap<String, Vec<PointerTarget>>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
struct PointerTarget {
    artifact_kind: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct PointerReceipt {
    schema_version: u32,
    name: String,
    target_kind: String,
    /// Artifact kind of the replaced target; absent in receipts written before it was recorded.
    #[serde(default)]
    old_kind: Option<String>,
    old_hash: Option<String>,
    new_hash: String,
    rollback_hint: String,
}

fn pointer_set(registry_path: &Path, name: &str, kind: &str, hash: &str, receipt_path: &Path) -> Result<i32, String> {
    let mut registry = read_registry(registry_path)?;
    let old_target = registry.pointers.get(name).cloned();
    let old_kind = old_target.as_ref().map(|target| target.artifact_kind.clone());
    let old_hash = old_target.as_ref().map(|target| target.hash.clone());
    if let Some(target) = old_target {
        registry.history.entry(name.to_owned()).or_default().push(target);
    }
    registry.pointers.insert(name.to_owned(), PointerTarget {
        artifact_kind: kind.to_owned(),
        hash: hash.to_owned(),
    });
    write_json(registry_path, &registry)?;
    let receipt = PointerReceipt {
        schema_version: SCHEMA_VERSION,
        name: name.to_owned(),
        target_kind: kind.to_owned(),
        old_kind,
        old_hash,
        new_hash: hash.to_owned(),
        rollback_hint: format!(
            "cargo octet roadmap pointer rollback --registry {} --receipt {}",
            registry_path.display(),
            receipt_path.display()
        ),
    };
    write_json(receipt_path, &receipt)?;
    print_json(&receipt)?;
    Ok(0)
}

fn pointer_inspect(registry_path: &Path, name: &str) -> Result<i32, String> {
    let registry = read_registry(registry_path)?;
    let target = registry.pointers.get(name).ok_or_else(|| format!("name `{name}` is not registered"))?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "name": name,
        "target_hash": target.hash,
        "target_artifact_kind": target.artifact_kind,
        "identity_source": "immutable-hash",
        "human_label": name,
        "previous_targets": registry.history.get(name).cloned().unwrap_or_default(),
        "review_identity_status": "matched-identity-not-correctness-proof",
        "correctness_claim": "none",
    }))?;
    Ok(0)
}

fn pointer_rollback(registry_path: &Path, receipt_path: &Path) -> Result<i32, String> {
    let receipt: PointerReceipt =
        serde_json::from_value(read_json(receipt_path)?).map_err(|error| format!("parse receipt: {error}"))?;
    let Some(old_hash) = receipt.old_hash.clone() else {
        return Err("receipt has no old_hash; cannot rollback initial pointer creation".to_owned());
    };
    let mut registry = read_registry(registry_path)?;
    let name = receipt.name.clone();
    let current = registry
        .pointers
        .get(&name)
        .cloned()
        .ok_or_else(|| format!("name `{name}` is not registered; cannot roll back receipt"))?;
    if current.hash != receipt.new_hash || current.artifact_kind != receipt.target_kind {
        return Err(format!(
            "pointer `{name}` currently targets {} `{}`, not the receipt's {} `{}`; refusing to roll back a receipt superseded by a later update",
            current.artifact_kind, current.hash, receipt.target_kind, receipt.new_hash
        ));
    }
    let old_kind = match receipt.old_kind {
        Some(old_kind) => old_kind,
        None => registry
            .history
            .get(&name)
            .and_then(|history| history.iter().rev().find(|target| target.hash == old_hash))
            .map(|target| target.artifact_kind.clone())
            .ok_or_else(|| {
                format!(
                    "receipt has no old_kind and history for `{name}` has no `{old_hash}` target; cannot restore the previous artifact kind"
                )
            })?,
    };
    registry.history.entry(name.clone()).or_default().push(current);
    registry.pointers.insert(name.clone(), PointerTarget {
        artifact_kind: old_kind.clone(),
        hash: old_hash.clone(),
    });
    write_json(registry_path, &registry)?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "rolled-back",
        "name": name,
        "restored_hash": old_hash,
        "restored_artifact_kind": old_kind,
    }))?;
    Ok(0)
}

fn session_check(input: &Path) -> Result<i32, String> {
    let session = read_json(input)?;
    let records = session["records"].as_array().ok_or_else(|| "session missing records array".to_owned())?;
    let mut kinds = Vec::new();
    for record in records {
        if record.get("kind").and_then(Value::as_str).is_none()
            || record.get("schema_version").and_then(Value::as_u64).is_none()
            || record.get("identity").and_then(Value::as_str).is_none()
            || record.get("redaction_status").and_then(Value::as_str).is_none()
        {
            return Err("session record missing kind, schema_version, identity, or redaction_status".to_owned());
        }
        kinds.push(record["kind"].as_str().unwrap().to_owned());
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "typed-versioned",
        "record_count": records.len(),
        "record_kinds": kinds,
    }))?;
    Ok(0)
}

fn session_migrate(input: &Path, output: &Path) -> Result<i32, String> {
    let mut session = read_json(input)?;
    let object = session.as_object_mut().ok_or_else(|| "session must be a JSON object".to_owned())?;
    let original_identity = object.get("identity").and_then(Value::as_str).unwrap_or("unknown").to_owned();
    let from_schema_version = object.get("original_schema_version").and_then(Value::as_u64).unwrap_or(0);
    let migration_receipt = serde_json::json!({
        "from_schema_version": from_schema_version,
        "to_schema_version": SCHEMA_VERSION,
        "original_identity": original_identity,
        "identity_change": false,
    });
    object.insert("schema_version".to_owned(), Value::from(SCHEMA_VERSION));
    object.insert("migration_receipt".to_owned(), migration_receipt.clone());
    write_json(output, &session)?;
    print_json(&migration_receipt)?;
    Ok(0)
}

fn session_todo(input: &Path) -> Result<i32, String> {
    let session = read_json(input)?;
    let todos = session["todos"].as_array().ok_or_else(|| "session missing todos array".to_owned())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut items = Vec::new();
    for todo in todos {
        let kind = todo["kind"].as_str().ok_or_else(|| "todo missing kind".to_owned())?;
        let status = todo["status"].as_str().ok_or_else(|| "todo missing status".to_owned())?;
        let source = todo["source"].as_str().unwrap_or("unknown");
        let next_action = todo["next_action"].as_str().unwrap_or("none");
        *counts.entry(status.to_owned()).or_default() += 1;
        items.push(serde_json::json!({"kind": kind, "status": status, "source": source, "next_action": next_action, "blocker": todo.get("blocker")}));
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "structured",
        "counts_by_status": counts,
        "items": items,
    }))?;
    Ok(0)
}

fn pure_cache_check(key_path: &Path, store_dir: &Path, output_hash: &str) -> Result<i32, String> {
    let key = normalize_value(read_json(key_path)?);
    let required = [
        "command_identity",
        "input_hashes",
        "config_hash",
        "tool_revision",
        "environment_allowlist",
    ];
    for field in required {
        if key.get(field).is_none() {
            return Err(format!("cache key missing {field}"));
        }
    }
    let canonical = canonical_json(&key)?;
    let cache_key = determinism::full_hash(&canonical);
    fs::create_dir_all(store_dir).map_err(|error| format!("create {}: {error}", store_dir.display()))?;
    let entry_path = store_dir.join(hash_file_name(&cache_key));
    if entry_path.exists() {
        let entry = read_json(&entry_path)?;
        print_json(&serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "status": "hit",
            "cache_key": cache_key,
            "stored_output_hash": entry["output_hash"],
            "key_components": key,
        }))?;
    } else {
        let entry = serde_json::json!({"cache_key": cache_key, "output_hash": output_hash, "key_components": key});
        write_json(&entry_path, &entry)?;
        print_json(&serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "status": "miss",
            "cache_key": cache_key,
            "changed_key_component": "new-or-changed-key",
        }))?;
    }
    Ok(0)
}

fn normalize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map {
                if is_redacted_key(&key) {
                    continue;
                }
                out.insert(key, normalize_value(value));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_value).collect()),
        other => other,
    }
}

fn is_redacted_key(key: &str) -> bool {
    matches!(key, "absolute_path" | "timestamp" | "username" | "env_dump" | "secret" | "raw_secret")
}

fn canonical_json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("serialize canonical json: {error}"))
}

fn hash_file_name(hash: &str) -> String {
    format!("{}.json", hash.replace(':', "_"))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn read_registry(path: &Path) -> Result<PointerRegistry, String> {
    if !path.exists() {
        return Ok(PointerRegistry::default());
    }
    serde_json::from_value(read_json(path)?).map_err(|error| format!("parse registry: {error}"))
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|error| format!("serialize json: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), String> {
    println!("{}", serde_json::to_string_pretty(value).map_err(|error| format!("serialize output: {error}"))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_mismatches_treat_absent_fields_as_mismatches() {
        let manifest = serde_json::json!({"command_argv": ["cargo", "octet"], "run_kind": "check"});
        let observed = serde_json::json!({"command_argv": ["cargo", "octet"], "run_kind": "check"});
        assert_eq!(capsule_mismatches(&manifest, &observed), vec![
            "octet_binary_identity",
            "config_hash",
            "input_hashes",
            "output_hashes",
        ]);
        assert_eq!(capsule_mismatches(&Value::Null, &Value::Null).len(), 6);
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn roadmap_parse_rejects_flag_in_value_position() {
        let error = parse(&args(&[
            "pointer",
            "set",
            "--registry",
            "registry.json",
            "--name",
            "--kind",
            "prompt",
            "--hash",
            "b3:x",
            "--receipt",
            "receipt.json",
        ]))
        .expect_err("flag in value position fails");
        assert!(error.contains("missing value after `--name`"), "{error}");

        let error = parse(&args(&["session", "check", "--input"])).expect_err("trailing flag fails");
        assert!(error.contains("missing value after `--input`"), "{error}");

        let error = parse(&args(&["session", "check", "--input="])).expect_err("empty inline value fails");
        assert!(error.contains("missing value after `--input`"), "{error}");
    }

    #[test]
    fn roadmap_parse_rejects_unknown_flags_and_stray_arguments() {
        let error = parse(&args(&[
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            "in.json",
            "--store-dir",
            "store",
            "--bogus",
            "x",
        ]))
        .expect_err("unknown flag fails");
        assert!(error.contains("unsupported roadmap artifact store argument `--bogus`"), "{error}");

        let error = parse(&args(&["ability", "registry", "--verbose"])).expect_err("registry takes no flags");
        assert!(error.contains("unsupported roadmap ability registry argument `--verbose`"), "{error}");

        let error = parse(&args(&["session", "check", "--input", "session.json", "extra"]))
            .expect_err("stray positional fails");
        assert!(error.contains("unexpected roadmap session check argument `extra`"), "{error}");

        let error = parse(&args(&["capsule", "inspect", "--capsule", "c.json", "--observed", "o.json"]))
            .expect_err("flag from another action fails");
        assert!(error.contains("unsupported roadmap capsule inspect argument `--observed`"), "{error}");
    }

    #[test]
    fn roadmap_parse_rejects_repeated_flags() {
        let error = parse(&args(&["session", "check", "--input", "a.json", "--input", "b.json"]))
            .expect_err("repeated flag fails");
        assert!(error.contains("`--input` may be provided only once"), "{error}");

        let error = parse(&args(&["session", "check", "--input=a.json", "--input", "b.json"]))
            .expect_err("repeated mixed-form flag fails");
        assert!(error.contains("`--input` may be provided only once"), "{error}");
    }

    #[test]
    fn roadmap_parse_accepts_split_and_inline_values() {
        let command = parse(&args(&["capsule", "replay", "--capsule=c.json", "--observed", "o.json"]))
            .expect("valid capsule replay args");
        assert!(
            matches!(&command, RoadmapCommand::CapsuleReplay { capsule, observed }
                if capsule == Path::new("c.json") && observed == Path::new("o.json")),
            "{command:?}"
        );
        let command = parse(&args(&[
            "pure-cache",
            "check",
            "--output-hash",
            "b3:out",
            "--store-dir",
            "cache",
            "--key",
            "key.json",
        ]))
        .expect("valid pure-cache args");
        assert!(
            matches!(&command, RoadmapCommand::PureCacheCheck { key, store_dir, output_hash }
                if key == Path::new("key.json") && store_dir == Path::new("cache") && output_hash == "b3:out"),
            "{command:?}"
        );
        assert!(matches!(parse(&args(&["ability", "registry"])), Ok(RoadmapCommand::AbilityRegistry)));
    }
}
