use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RunMetadata {
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub rustc_version: String,
    pub toolchain: String,
    pub profile_name: String,
    pub profile_hash: String,
    pub config_hash: String,
}

#[derive(Debug, Clone, Copy)]
pub struct StrictProfileIdentity<'a> {
    pub profile_id: &'a str,
    pub policy_hash: &'a str,
}

#[derive(Debug, Clone)]
pub struct MetadataInput<'a> {
    pub workspace_root: &'a Path,
    pub member_dirs: Option<&'a [PathBuf]>,
    pub toolchain: &'a str,
    pub scope_args: &'a [String],
    pub cargo_check_args: &'a [String],
    pub output_format: &'a str,
    pub strict_profile: Option<StrictProfileIdentity<'a>>,
}

pub fn build_run_metadata(input: &MetadataInput<'_>) -> RunMetadata {
    let config_hash = config_hash(
        input.workspace_root,
        input.member_dirs,
        input.scope_args,
        input.cargo_check_args,
        input.strict_profile,
    );
    let profile_hash =
        profile_hash(input.scope_args, input.cargo_check_args, input.output_format, &config_hash, input.strict_profile);

    RunMetadata {
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        rustc_version: rustc_version(input.toolchain),
        toolchain: input.toolchain.to_owned(),
        profile_name: input
            .strict_profile
            .map_or_else(|| "workspace-metadata".to_owned(), |profile| profile.profile_id.to_owned()),
        profile_hash,
        config_hash,
    }
}

pub fn semantic_finding_id(
    lint: &str,
    crate_name: &str,
    message: &str,
    suggestion: Option<&str>,
    normalized_snippet_hash: &str,
    config_hash: Option<&str>,
) -> String {
    let canonical = format!(
        "lint={lint}\ncrate={crate_name}\nmessage={message}\nsuggestion={}\nsnippet={normalized_snippet_hash}\nconfig={}\ntool=cargo-octet@{}\n",
        suggestion.unwrap_or(""),
        config_hash.unwrap_or("unknown"),
        env!("CARGO_PKG_VERSION"),
    );
    short_hash(&canonical)
}

pub fn normalized_snippet_hash(snippet: &str) -> String {
    let normalized = snippet.split_whitespace().collect::<Vec<_>>().join(" ");
    full_hash(&normalized)
}

pub fn crate_graph_hash(workspace_root: &Path) -> String {
    let payload = serde_json::json!({
        "cargo_toml": file_hash(&workspace_root.join("Cargo.toml")),
        "cargo_lock": file_hash(&workspace_root.join("Cargo.lock")),
    });
    full_hash(&payload.to_string())
}

pub fn short_hash(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes()).to_hex().to_string();
    format!("b3:{}", &hash[..16])
}

pub fn full_hash(input: &str) -> String {
    format!("b3:{}", blake3::hash(input.as_bytes()).to_hex())
}

pub fn normalize_path(path: &Path) -> String {
    normalize_path_text(&path.display().to_string())
}

pub fn normalize_path_text(path: &str) -> String {
    path.replace('\\', "/")
}

fn profile_hash(
    scope_args: &[String],
    cargo_check_args: &[String],
    output_format: &str,
    config_hash: &str,
    strict_profile: Option<StrictProfileIdentity<'_>>,
) -> String {
    let payload = match strict_profile {
        Some(profile) => serde_json::json!({
            "scope_args": scope_args,
            "cargo_check_args": cargo_check_args,
            "output_format": output_format,
            "config_hash": config_hash,
            "strict_profile": {
                "profile_id": profile.profile_id,
                "policy_hash": profile.policy_hash,
            },
        }),
        None => serde_json::json!({
            "scope_args": scope_args,
            "cargo_check_args": cargo_check_args,
            "output_format": output_format,
            "config_hash": config_hash,
        }),
    };
    full_hash(&payload.to_string())
}

/// `[workspace.metadata.<name>]` tables the runner reads from the root manifest (current table
/// first, then the legacy fallbacks in `config::workspace_metadata_table`). Every other manifest
/// key stays out of finding identity.
const RUNNER_METADATA_TABLES: [&str; 3] = ["octet", "slotcar", "tigerstyle"];

fn config_hash(
    workspace_root: &Path,
    member_dirs: Option<&[PathBuf]>,
    scope_args: &[String],
    cargo_check_args: &[String],
    strict_profile: Option<StrictProfileIdentity<'_>>,
) -> String {
    let runner_manifest_config = runner_manifest_config(workspace_root, member_dirs);
    let dylint_toml = file_hash(&workspace_root.join("dylint.toml"));
    let payload = match strict_profile {
        Some(profile) => serde_json::json!({
            "runner_manifest_config": runner_manifest_config,
            "dylint_toml": dylint_toml,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
            "strict_profile": {
                "profile_id": profile.profile_id,
                "policy_hash": profile.policy_hash,
            },
        }),
        None => serde_json::json!({
            "runner_manifest_config": runner_manifest_config,
            "dylint_toml": dylint_toml,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
        }),
    };
    full_hash(&payload.to_string())
}

/// Canonical JSON of the runner metadata tables in the root manifest. A missing manifest and an
/// unparsable one hash differently from each other and from any parsed manifest; the unparsable
/// case keeps the raw bytes in identity.
fn runner_manifest_config(workspace_root: &Path, member_dirs: Option<&[PathBuf]>) -> serde_json::Value {
    let Ok(bytes) = fs::read(workspace_root.join("Cargo.toml")) else {
        return serde_json::json!({ "status": "missing" });
    };
    let document = std::str::from_utf8(&bytes).ok().and_then(|text| text.parse::<toml::Table>().ok());
    let Some(document) = document else {
        return serde_json::json!({
            "status": "unparsable",
            "hash": format!("b3:{}", blake3::hash(&bytes).to_hex()),
        });
    };
    let metadata = document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("metadata"))
        .and_then(toml::Value::as_table);
    let tables = RUNNER_METADATA_TABLES
        .iter()
        .filter_map(|name| {
            let value = metadata?.get(*name)?;
            Some(((*name).to_owned(), canonical_toml_json(value)))
        })
        .collect::<serde_json::Map<_, _>>();
    let member_lints = collect_member_lints(workspace_root, &document, member_dirs);
    let mut root_obj = serde_json::Map::new();
    root_obj.insert("status".to_owned(), serde_json::json!("parsed"));
    root_obj.insert("workspace_metadata".to_owned(), serde_json::Value::Object(tables));
    if let Some(cargo_lints) =
        crate::cargo::lints::manifest_lint_table_json(&document, crate::cargo::lints::WorkspaceInheritanceMode::Ignore)
    {
        root_obj.insert("cargo_lints".to_owned(), cargo_lints);
    }
    if !member_lints.is_empty() {
        root_obj.insert("member_cargo_lints".to_owned(), serde_json::Value::Object(member_lints));
    }
    serde_json::Value::Object(root_obj)
}

fn collect_member_lints(
    workspace_root: &Path,
    document: &toml::Table,
    member_dirs: Option<&[PathBuf]>,
) -> serde_json::Map<String, serde_json::Value> {
    let inheritance_mode = if crate::cargo::lints::has_workspace_lint_table(document) {
        crate::cargo::lints::WorkspaceInheritanceMode::Track
    } else {
        crate::cargo::lints::WorkspaceInheritanceMode::Ignore
    };
    let discovered;
    let dirs = if let Some(dirs) = member_dirs {
        dirs
    } else {
        discovered = discover_workspace_member_dirs(workspace_root, document);
        &discovered
    };
    let mut unique_dirs: Vec<PathBuf> =
        dirs.iter().filter(|rel_dir| !rel_dir.as_os_str().is_empty()).cloned().collect();
    unique_dirs.sort();
    unique_dirs.dedup();
    unique_dirs
        .into_iter()
        .filter_map(|rel_dir| {
            let text = fs::read_to_string(workspace_root.join(&rel_dir).join("Cargo.toml")).ok()?;
            let member_doc = text.parse::<toml::Table>().ok()?;
            let lints = crate::cargo::lints::manifest_lint_table_json(&member_doc, inheritance_mode)?;
            Some((format!("{}/Cargo.toml", normalize_path(&rel_dir)), lints))
        })
        .collect()
}

fn discover_workspace_member_dirs(workspace_root: &Path, document: &toml::Table) -> Vec<PathBuf> {
    let Some(workspace) = document.get("workspace").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    let Some(members) = workspace.get("members").and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    let excludes: Vec<PathBuf> = workspace
        .get("exclude")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .filter_map(normalize_workspace_relative_path)
        .collect();
    let has_root_exclusion = excludes.iter().any(|excl| excl.as_os_str().is_empty());
    members
        .iter()
        .filter_map(toml::Value::as_str)
        .flat_map(|pattern| expand_member_pattern(workspace_root, pattern, &excludes, has_root_exclusion))
        .collect()
}

fn expand_member_pattern(
    workspace_root: &Path,
    pattern: &str,
    excludes: &[PathBuf],
    has_root_exclusion: bool,
) -> Vec<PathBuf> {
    let Some(normalized) = normalize_workspace_relative_path(pattern) else {
        return Vec::new();
    };
    if normalized.as_os_str().is_empty() || has_root_exclusion {
        return Vec::new();
    }
    if !normalize_path(&normalized).contains(['*', '?']) {
        return vec![normalized];
    }
    expand_glob_member_pattern(workspace_root, &normalized)
        .into_iter()
        .filter(|rel_dir| !is_excluded_member_dir(rel_dir, excludes))
        .collect()
}

fn expand_glob_member_pattern(workspace_root: &Path, normalized_pattern: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::new()];
    for component in normalized_pattern.components() {
        let std::path::Component::Normal(part_os) = component else {
            return Vec::new();
        };
        let Some(part) = part_os.to_str() else {
            return Vec::new();
        };
        dirs = dirs.into_iter().flat_map(|dir| expand_glob_component(workspace_root, &dir, part)).collect();
        if dirs.is_empty() {
            break;
        }
    }
    dirs.into_iter().filter(|dir| !dir.as_os_str().is_empty()).collect()
}

fn expand_glob_component(workspace_root: &Path, dir: &Path, part: &str) -> Vec<PathBuf> {
    if part == "**" {
        collect_recursive_subdirs(workspace_root, dir)
    } else if part.contains(['*', '?']) {
        read_dir_subdirs(&workspace_root.join(dir), dir)
            .into_iter()
            .filter(|sub| {
                sub.file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .is_some_and(|name| glob_segment_matches(part, name))
            })
            .collect()
    } else {
        let candidate = dir.join(part);
        if workspace_root.join(&candidate).is_dir() {
            vec![candidate]
        } else {
            Vec::new()
        }
    }
}

fn collect_recursive_subdirs(workspace_root: &Path, start: &Path) -> Vec<PathBuf> {
    const MAX_RECURSIVE_DIRS: usize = 4096;
    let mut out = Vec::with_capacity(64);
    let mut stack = Vec::with_capacity(64);
    stack.push(start.to_path_buf());
    for _ in 0..MAX_RECURSIVE_DIRS {
        let Some(dir) = stack.pop() else {
            break;
        };
        if !dir.as_os_str().is_empty() {
            out.push(dir.clone());
        }
        let mut children = read_dir_subdirs(&workspace_root.join(&dir), &dir);
        children.sort();
        for child in children.into_iter().rev() {
            if stack.len() >= MAX_RECURSIVE_DIRS {
                break;
            }
            stack.push(child);
        }
    }
    out
}

fn normalize_workspace_relative_path(raw: &str) -> Option<PathBuf> {
    let slash_normalized = normalize_path_text(raw);
    let mut normalized = PathBuf::new();
    for component in Path::new(&slash_normalized).components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return None;
            }
        }
    }
    Some(normalized)
}

fn is_excluded_member_dir(rel_dir: &Path, excludes: &[PathBuf]) -> bool {
    if rel_dir.as_os_str().is_empty() {
        return false;
    }
    excludes.iter().any(|excl| excl.as_os_str().is_empty() || rel_dir.starts_with(excl))
}

fn glob_segment_matches(pattern: &str, candidate: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let candidate_chars: Vec<char> = candidate.chars().collect();
    let mut pattern_idx = 0;
    let mut candidate_idx = 0;
    let mut star_idx = None;
    let mut retry_idx = 0;

    while candidate_idx < candidate_chars.len() {
        if pattern_idx < pattern_chars.len()
            && (pattern_chars[pattern_idx] == '?' || pattern_chars[pattern_idx] == candidate_chars[candidate_idx])
        {
            pattern_idx += 1;
            candidate_idx += 1;
        } else if pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
            star_idx = Some(pattern_idx);
            pattern_idx += 1;
            retry_idx = candidate_idx;
        } else if let Some(star) = star_idx {
            pattern_idx = star + 1;
            retry_idx += 1;
            candidate_idx = retry_idx;
        } else {
            return false;
        }
    }

    while pattern_idx < pattern_chars.len() && pattern_chars[pattern_idx] == '*' {
        pattern_idx += 1;
    }
    pattern_idx == pattern_chars.len()
}

fn read_dir_subdirs(dir: &Path, prefix: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| {
            if prefix.as_os_str().is_empty() {
                PathBuf::from(entry.file_name())
            } else {
                prefix.join(entry.file_name())
            }
        })
        .collect()
}

/// Converts TOML to JSON with object keys inserted in sorted order, so the rendered text does not
/// depend on manifest key order or on whether `serde_json` preserves insertion order.
pub(crate) fn canonical_toml_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(text) => serde_json::Value::String(text.clone()),
        toml::Value::Integer(number) => serde_json::Value::from(*number),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map_or_else(|| serde_json::Value::String(number.to_string()), serde_json::Value::Number),
        toml::Value::Boolean(flag) => serde_json::Value::Bool(*flag),
        toml::Value::Datetime(datetime) => serde_json::Value::String(datetime.to_string()),
        toml::Value::Array(items) => serde_json::Value::Array(items.iter().map(canonical_toml_json).collect()),
        toml::Value::Table(table) => {
            let mut keys = table.keys().collect::<Vec<_>>();
            keys.sort();
            serde_json::Value::Object(
                keys.into_iter().map(|key| (key.clone(), canonical_toml_json(&table[key.as_str()]))).collect(),
            )
        }
    }
}

fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()))
}

/// `RunMetadata::rustc_version` when `rustc -Vv` cannot be run or read.
pub const UNKNOWN_RUSTC_VERSION: &str = "unknown";

fn rustc_version(toolchain: &str) -> String {
    Command::new("rustc")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .arg("-Vv")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.lines().next().map(ToOwned::to_owned))
        .unwrap_or_else(|| UNKNOWN_RUSTC_VERSION.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_HASH: &str = "b3:config";
    const ROBUST_POLICY_HASH: &str = "b3:robust";
    const BOUNDED_POLICY_HASH: &str = "b3:bounded";

    #[test]
    fn strict_profile_identity_changes_config_hash_deterministically() {
        let root = tempfile::tempdir().expect("profile hash tempdir");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("write Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
        let scope_args = vec!["-p".to_owned(), "domain".to_owned()];
        let cargo_args = vec!["--lib".to_owned()];
        let robust = StrictProfileIdentity {
            profile_id: "robust",
            policy_hash: ROBUST_POLICY_HASH,
        };
        let bounded = StrictProfileIdentity {
            profile_id: "bounded-systems",
            policy_hash: BOUNDED_POLICY_HASH,
        };

        let default_hash = config_hash(root.path(), None, &scope_args, &cargo_args, None);
        let expected_default_hash = full_hash(
            &serde_json::json!({
                "runner_manifest_config": { "status": "parsed", "workspace_metadata": {} },
                "dylint_toml": file_hash(&root.path().join("dylint.toml")),
                "effective_scope_args": scope_args,
                "effective_cargo_check_args": cargo_args,
            })
            .to_string(),
        );
        let robust_hash = config_hash(root.path(), None, &scope_args, &cargo_args, Some(robust));
        let repeated_robust_hash = config_hash(root.path(), None, &scope_args, &cargo_args, Some(robust));
        let bounded_hash = config_hash(root.path(), None, &scope_args, &cargo_args, Some(bounded));

        assert_eq!(default_hash, expected_default_hash);
        assert_eq!(robust_hash, repeated_robust_hash);
        assert_ne!(default_hash, robust_hash);
        assert_ne!(robust_hash, bounded_hash);
    }

    const BASE_MANIFEST: &str = r#"[workspace]
members = ["core"]
resolver = "2"

[workspace.dependencies]
serde = "1.0.200"

[workspace.metadata.octet]
default_scope = ["--workspace"]
cargo_check_args = ["--lib"]

[package]
name = "root"
version = "0.1.0"
"#;

    fn config_hash_for(manifest: &str, dylint: &str) -> String {
        let root = tempfile::tempdir().expect("config hash tempdir");
        std::fs::write(root.path().join("Cargo.toml"), manifest).expect("write Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), dylint).expect("write dylint.toml");
        config_hash(root.path(), None, &["--workspace".to_owned()], &["--lib".to_owned()], None)
    }

    #[test]
    fn config_hash_ignores_manifest_edits_outside_runner_metadata() {
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        let dependency_bump = BASE_MANIFEST.replace("serde = \"1.0.200\"", "serde = \"1.0.228\"");
        let new_member = BASE_MANIFEST.replace("members = [\"core\"]", "members = [\"core\", \"cli\"]");
        let package_edit = BASE_MANIFEST.replace("version = \"0.1.0\"", "version = \"0.2.0\"\nedition = \"2024\"");
        let reordered_metadata = BASE_MANIFEST.replace(
            "default_scope = [\"--workspace\"]\ncargo_check_args = [\"--lib\"]",
            "cargo_check_args = [\"--lib\"]\ndefault_scope = [\"--workspace\"]",
        );

        assert!(baseline.starts_with("b3:") && baseline.len() == "b3:".len() + 64);
        for edited in [dependency_bump, new_member, package_edit, reordered_metadata] {
            assert_ne!(edited, BASE_MANIFEST);
            assert_eq!(config_hash_for(&edited, "[octet]\n"), baseline, "manifest edit changed config hash:\n{edited}");
        }
    }

    #[test]
    fn config_hash_tracks_runner_metadata_and_dylint_config() {
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        let octet_edit = BASE_MANIFEST.replace("cargo_check_args = [\"--lib\"]", "cargo_check_args = [\"--tests\"]");
        let legacy_table =
            format!("{BASE_MANIFEST}\n[workspace.metadata.slotcar]\ndefault_scope = [\"-p\", \"core\"]\n");
        let dylint_edit = config_hash_for(BASE_MANIFEST, "[octet]\ndisabled_lints = [\"no_unwrap\"]\n");
        let missing_manifest = {
            let root = tempfile::tempdir().expect("missing manifest tempdir");
            config_hash(root.path(), None, &["--workspace".to_owned()], &["--lib".to_owned()], None)
        };
        let unparsable = config_hash_for("[workspace\n", "[octet]\n");

        assert_ne!(config_hash_for(&octet_edit, "[octet]\n"), baseline);
        assert_ne!(config_hash_for(&legacy_table, "[octet]\n"), baseline);
        assert_ne!(dylint_edit, baseline);
        assert_eq!(unparsable, config_hash_for("[workspace\n", "[octet]\n"));
        assert_ne!(unparsable, config_hash_for("[workspace\n\n", "[octet]\n"));
        assert_ne!(unparsable, baseline);
        assert_ne!(missing_manifest, baseline);
    }

    #[test]
    fn config_hash_tracks_cargo_lints_tables_deterministically() {
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        let with_warn = format!("{BASE_MANIFEST}\n[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n");
        let with_deny = format!("{BASE_MANIFEST}\n[workspace.lints.cargo]\nunused_dependencies = \"deny\"\n");

        let warn_hash = config_hash_for(&with_warn, "[octet]\n");
        let deny_hash = config_hash_for(&with_deny, "[octet]\n");
        assert_ne!(warn_hash, baseline);
        assert_ne!(deny_hash, warn_hash);
        assert_eq!(warn_hash, config_hash_for(&with_warn, "[octet]\n"));
    }

    #[test]
    fn config_hash_tracks_member_cargo_lints_without_mutating_plain_workspaces() {
        let root = tempfile::tempdir().expect("member lint tempdir");
        std::fs::write(root.path().join("Cargo.toml"), BASE_MANIFEST).expect("write root Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
        std::fs::create_dir_all(root.path().join("core")).expect("create core dir");
        std::fs::write(root.path().join("core/Cargo.toml"), "[package]\nname = \"core\"\nversion = \"0.1.0\"\n")
            .expect("write plain member");

        let scope = ["--workspace".to_owned()];
        let check = ["--lib".to_owned()];
        let member_dirs = [PathBuf::from("core")];
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        assert_eq!(config_hash(root.path(), Some(&member_dirs), &scope, &check, None), baseline);

        std::fs::write(
            root.path().join("core/Cargo.toml"),
            "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"warn\"\n",
        )
        .expect("write member warn");
        let member_warn = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
        let discovered_warn = config_hash(root.path(), None, &scope, &check, None);
        assert_ne!(member_warn, baseline);
        assert_eq!(member_warn, discovered_warn);

        std::fs::write(
            root.path().join("core/Cargo.toml"),
            "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"deny\"\n",
        )
        .expect("write member deny");
        let member_deny = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
        assert_ne!(member_deny, member_warn);
    }

    #[test]
    fn config_hash_tracks_member_workspace_lint_inheritance_only_with_workspace_cargo_lints() {
        let scope = ["--workspace".to_owned()];
        let check = ["--lib".to_owned()];
        let member_dirs = [PathBuf::from("core")];

        for root_manifest in [
            BASE_MANIFEST.to_owned(),
            format!("{BASE_MANIFEST}\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n"),
        ] {
            let root = tempfile::tempdir().expect("plain workspace tempdir");
            std::fs::write(root.path().join("Cargo.toml"), &root_manifest).expect("write root manifest");
            std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
            std::fs::create_dir_all(root.path().join("core")).expect("create core dir");
            std::fs::write(root.path().join("core/Cargo.toml"), "[package]\nname = \"core\"\nversion = \"0.1.0\"\n")
                .expect("write plain member");

            let plain_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_eq!(plain_cfg, config_hash(root.path(), None, &scope, &check, None));
            let plain_profile = profile_hash(&scope, &check, "json", &plain_cfg, None);
            let plain_finding =
                semantic_finding_id("cargo::unused_dependencies", "core", "msg", None, "b3:snip", Some(&plain_cfg));

            std::fs::write(
                root.path().join("core/Cargo.toml"),
                "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = true\n",
            )
            .expect("write inheriting member");
            let inherited_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_eq!(inherited_cfg, plain_cfg);
            assert_eq!(config_hash(root.path(), None, &scope, &check, None), plain_cfg);
            assert_eq!(profile_hash(&scope, &check, "json", &inherited_cfg, None), plain_profile);
            assert_eq!(
                semantic_finding_id("cargo::unused_dependencies", "core", "msg", None, "b3:snip", Some(&inherited_cfg)),
                plain_finding
            );

            std::fs::write(root.path().join("Cargo.toml"), format!("{root_manifest}\n[lints]\nworkspace = true\n"))
                .expect("write root workspace = true without cargo lints");
            assert_eq!(config_hash(root.path(), Some(&member_dirs), &scope, &check, None), plain_cfg);
        }

        for workspace_lints_section in [
            "[workspace.lints.cargo]\nunused_dependencies = \"warn\"\n",
            "[workspace.lints.cargo]\n",
        ] {
            let root_manifest = format!("{BASE_MANIFEST}\n{workspace_lints_section}");
            let root = tempfile::tempdir().expect("cargo lint workspace tempdir");
            std::fs::write(root.path().join("Cargo.toml"), &root_manifest).expect("write root manifest");
            std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
            std::fs::create_dir_all(root.path().join("core")).expect("create core dir");

            std::fs::write(root.path().join("core/Cargo.toml"), "[package]\nname = \"core\"\nversion = \"0.1.0\"\n")
                .expect("write non-inheriting member");
            let no_inherit_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_eq!(no_inherit_cfg, config_hash(root.path(), None, &scope, &check, None));
            let no_inherit_profile = profile_hash(&scope, &check, "json", &no_inherit_cfg, None);
            let no_inherit_finding = semantic_finding_id(
                "cargo::unused_dependencies",
                "core",
                "msg",
                None,
                "b3:snip",
                Some(&no_inherit_cfg),
            );

            std::fs::write(
                root.path().join("core/Cargo.toml"),
                "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = false\n",
            )
            .expect("write workspace = false member");
            assert_eq!(config_hash(root.path(), Some(&member_dirs), &scope, &check, None), no_inherit_cfg);

            std::fs::write(
                root.path().join("core/Cargo.toml"),
                "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints]\nworkspace = true\n",
            )
            .expect("write workspace = true member");
            let inherit_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_eq!(inherit_cfg, config_hash(root.path(), None, &scope, &check, None));
            assert_ne!(inherit_cfg, no_inherit_cfg);
            assert_ne!(profile_hash(&scope, &check, "json", &inherit_cfg, None), no_inherit_profile);
            assert_ne!(
                semantic_finding_id("cargo::unused_dependencies", "core", "msg", None, "b3:snip", Some(&inherit_cfg)),
                no_inherit_finding
            );

            std::fs::write(
                root.path().join("core/Cargo.toml"),
                "[package]\nname = \"core\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"warn\"\n",
            )
            .expect("write explicit member lints");
            let explicit_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_ne!(explicit_cfg, inherit_cfg);
            assert_ne!(explicit_cfg, no_inherit_cfg);

            std::fs::write(root.path().join("core/Cargo.toml"), "[package]\nname = \"core\"\nversion = \"0.1.0\"\n")
                .expect("restore plain member");
            assert_eq!(config_hash(root.path(), Some(&member_dirs), &scope, &check, None), no_inherit_cfg);

            std::fs::write(root.path().join("Cargo.toml"), format!("{root_manifest}\n[lints]\nworkspace = true\n"))
                .expect("enable root package inheritance");
            let root_inherit_cfg = config_hash(root.path(), Some(&member_dirs), &scope, &check, None);
            assert_ne!(root_inherit_cfg, no_inherit_cfg);
        }
    }

    #[test]
    fn config_hash_respects_workspace_exclude_when_member_dirs_omitted() {
        let root = tempfile::tempdir().expect("workspace exclude tempdir");
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\", \"crates/wip/*\"]\nexclude = [\"crates/excluded\", \"crates/wip\"]\nresolver = \"2\"\n\n[workspace.metadata.octet]\ndefault_scope = [\"--workspace\"]\ncargo_check_args = [\"--lib\"]\n",
        )
        .expect("write root Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
        for (dir, name) in [
            ("crates/included", "included"),
            ("crates/excluded", "excluded"),
            ("crates/wip/demo", "demo"),
        ] {
            std::fs::create_dir_all(root.path().join(dir)).expect("create member dir");
            std::fs::write(
                root.path().join(dir).join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"warn\"\n"),
            )
            .expect("write member manifest");
        }

        let scope = ["--workspace".to_owned()];
        let check = ["--lib".to_owned()];
        let explicit_members = [PathBuf::from("crates/included")];
        let discovered = config_hash(root.path(), None, &scope, &check, None);
        let explicit = config_hash(root.path(), Some(&explicit_members), &scope, &check, None);
        assert_eq!(discovered, explicit);

        for excluded in ["crates/excluded/Cargo.toml", "crates/wip/demo/Cargo.toml"] {
            std::fs::write(
                root.path().join(excluded),
                "[package]\nname = \"excluded\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"deny\"\n",
            )
            .expect("mutate excluded manifest");
            assert_eq!(config_hash(root.path(), None, &scope, &check, None), discovered);
        }

        std::fs::write(
            root.path().join("crates/included/Cargo.toml"),
            "[package]\nname = \"included\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"deny\"\n",
        )
        .expect("mutate included manifest");
        assert_ne!(config_hash(root.path(), None, &scope, &check, None), discovered);
    }

    #[test]
    fn config_hash_omitted_member_dirs_matches_cargo_exclude_and_path_normalization() {
        let root = tempfile::tempdir().expect("parity tempdir");
        std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
        for (dir, name) in [
            ("crates/included", "included"),
            ("crates/ex", "ex"),
            ("crates/excluded", "excluded"),
            ("wip/demo", "demo"),
            ("deep/a/sub", "sub"),
        ] {
            std::fs::create_dir_all(root.path().join(dir)).expect("create crate dir");
            std::fs::write(
                root.path().join(dir).join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"warn\"\n"),
            )
            .expect("write crate manifest");
        }

        let scope = ["--workspace".to_owned()];
        let check = ["--lib".to_owned()];

        // 1. `exclude` does NOT expand globs (`wip/*`, `*`, `**`, `deep/**`), and explicit members ignore
        //    non-`.` excludes.
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"./crates/included/\", \"./crates/*/\", \"./wip/*\", \"deep/*/*\"]\nexclude = [\"./crates/ex/\", \"wip/*\", \"*\", \"**\", \"deep/**\"]\nresolver = \"2\"\n",
        )
        .expect("write Cargo.toml case 1");
        let expected_case_1 = [
            PathBuf::from("crates/included"),
            PathBuf::from("crates/excluded"),
            PathBuf::from("wip/demo"),
            PathBuf::from("deep/a/sub"),
        ];
        let hash_none_1 = config_hash(root.path(), None, &scope, &check, None);
        let hash_some_1 = config_hash(root.path(), Some(&expected_case_1), &scope, &check, None);
        assert_eq!(hash_none_1, hash_some_1);

        // Mutating `wip/demo` MUST change `config_hash` because `exclude = ["wip/*"]` does not exclude it.
        std::fs::write(
            root.path().join("wip/demo/Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"deny\"\n",
        )
        .expect("mutate wip/demo");
        assert_ne!(config_hash(root.path(), None, &scope, &check, None), hash_none_1);

        // Mutating `crates/ex` MUST NOT change `config_hash`, while `crates/excluded` remains included
        // (component boundary).
        let base_after_wip = config_hash(root.path(), None, &scope, &check, None);
        std::fs::write(
            root.path().join("crates/ex/Cargo.toml"),
            "[package]\nname = \"ex\"\nversion = \"0.1.0\"\n\n[lints.cargo]\nunused_dependencies = \"deny\"\n",
        )
        .expect("mutate crates/ex");
        assert_eq!(config_hash(root.path(), None, &scope, &check, None), base_after_wip);

        // 2. Explicit non-glob member is kept even if listed under `exclude` (except `exclude = ["."]`).
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"./crates/included/\", \"wip/demo/\"]\nexclude = [\"wip\"]\nresolver = \"2\"\n",
        )
        .expect("write Cargo.toml case 2");
        assert_eq!(
            config_hash(root.path(), None, &scope, &check, None),
            config_hash(
                root.path(),
                Some(&[PathBuf::from("crates/included"), PathBuf::from("wip/demo")]),
                &scope,
                &check,
                None
            )
        );

        // 3. `exclude = ["."]` excludes all sub-members.
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/included\", \"wip/demo\"]\nexclude = [\".\"]\nresolver = \"2\"\n",
        )
        .expect("write Cargo.toml case 3");
        assert_eq!(
            config_hash(root.path(), None, &scope, &check, None),
            config_hash(root.path(), Some(&[]), &scope, &check, None)
        );
    }

    #[test]
    fn strict_profile_identity_changes_profile_hash_deterministically() {
        let scope_args = vec!["-p".to_owned(), "domain".to_owned()];
        let cargo_args = vec!["--lib".to_owned()];
        let robust = StrictProfileIdentity {
            profile_id: "robust",
            policy_hash: ROBUST_POLICY_HASH,
        };
        let bounded = StrictProfileIdentity {
            profile_id: "bounded-systems",
            policy_hash: BOUNDED_POLICY_HASH,
        };

        let default_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, None);
        let expected_default_hash = full_hash(
            &serde_json::json!({
                "scope_args": scope_args,
                "cargo_check_args": cargo_args,
                "output_format": "json",
                "config_hash": CONFIG_HASH,
            })
            .to_string(),
        );
        let robust_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(robust));
        let repeated_robust_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(robust));
        let bounded_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(bounded));

        assert_eq!(default_hash, expected_default_hash);
        assert_eq!(robust_hash, repeated_robust_hash);
        assert_ne!(default_hash, robust_hash);
        assert_ne!(robust_hash, bounded_hash);
    }
}
