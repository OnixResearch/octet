use std::fs;
use std::path::Path;
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

#[derive(Debug, Clone)]
pub struct MetadataInput<'a> {
    pub workspace_root: &'a Path,
    pub toolchain: &'a str,
    pub scope_args: &'a [String],
    pub cargo_check_args: &'a [String],
    pub output_format: &'a str,
}

pub fn build_run_metadata(input: &MetadataInput<'_>) -> RunMetadata {
    let config_hash = config_hash(input.workspace_root, input.scope_args, input.cargo_check_args);
    let profile_hash = profile_hash(input.scope_args, input.cargo_check_args, input.output_format, &config_hash);

    RunMetadata {
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        rustc_version: rustc_version(input.toolchain),
        toolchain: input.toolchain.to_owned(),
        profile_name: "workspace-metadata".to_owned(),
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

fn profile_hash(scope_args: &[String], cargo_check_args: &[String], output_format: &str, config_hash: &str) -> String {
    let payload = serde_json::json!({
        "scope_args": scope_args,
        "cargo_check_args": cargo_check_args,
        "output_format": output_format,
        "config_hash": config_hash,
    });
    full_hash(&payload.to_string())
}

fn config_hash(workspace_root: &Path, scope_args: &[String], cargo_check_args: &[String]) -> String {
    let mut files = Vec::new();
    push_file_hash(&workspace_root.join("Cargo.toml"), &mut files);
    push_file_hash(&workspace_root.join("dylint.toml"), &mut files);
    let payload = serde_json::json!({
        "files": files,
        "effective_scope_args": scope_args,
        "effective_cargo_check_args": cargo_check_args,
    });
    full_hash(&payload.to_string())
}

fn push_file_hash(path: &Path, files: &mut Vec<serde_json::Value>) {
    let relative = path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown");
    files.push(serde_json::json!({
        "path": relative,
        "hash": file_hash(path),
    }));
}

fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()))
}

fn rustc_version(toolchain: &str) -> String {
    Command::new("rustc")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .arg("-Vv")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.lines().next().unwrap_or("unknown").to_owned())
}
