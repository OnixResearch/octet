use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const ENVELOPE_POLICY_SCHEMA_VERSION: &str = "octet-effect-closure-envelope-policy/v1";
pub(crate) const ENVELOPE_POLICY_JSON: &str =
    include_str!("../../../standards/effect-closure/generated/envelope-policy.json");
const BUILD_CONTEXT_DOMAIN: &str = "octet-effect-closure-build-context/v1";
const RELATIVE_PATH_HEADROOM: usize = 4;

#[derive(Clone, Debug)]
pub(crate) struct BuildContextInput<'a> {
    pub(crate) workspace_root: &'a Path,
    pub(crate) target: &'a str,
    pub(crate) toolchain: &'a str,
    pub(crate) rustc_version: &'a str,
    pub(crate) cargo_lock_blake3: &'a str,
    pub(crate) metadata_cargo_args: &'a [String],
    pub(crate) rustflags: &'a str,
}

#[derive(Clone, Debug)]
pub(crate) struct BuildContextOutcome {
    pub(crate) blake3: String,
    pub(crate) incomplete_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) schema_version: String,
    pub(crate) default_binding_profile: String,
    pub(crate) supported_binding_profiles: Vec<String>,
    pub(crate) relocation_preserving: bool,
    pub(crate) included_extensions: Vec<String>,
    pub(crate) included_manifests: Vec<String>,
    pub(crate) build_script: String,
    pub(crate) workspace_build_inputs: Vec<String>,
    pub(crate) excluded_root_dirs: Vec<String>,
    pub(crate) symlinks: String,
    pub(crate) unreadable_inputs: String,
    pub(crate) generated_inputs_policy: String,
    pub(crate) max_files_per_package: usize,
    pub(crate) max_bytes_per_package: usize,
    pub(crate) max_resolve_depth: usize,
    pub(crate) non_claims: Vec<String>,
}

pub(crate) fn embedded_json_blake3() -> String {
    format!("b3:{}", blake3::hash(ENVELOPE_POLICY_JSON.as_bytes()).to_hex())
}

pub(crate) fn embedded_settings() -> Result<Settings, String> {
    let settings = serde_json::from_str::<Settings>(ENVELOPE_POLICY_JSON)
        .map_err(|error| format!("parse embedded source-envelope policy: {error}"))?;
    validate_settings(&settings)?;
    Ok(settings)
}

pub(crate) fn validate_settings(settings: &Settings) -> Result<(), String> {
    if settings.schema_version != ENVELOPE_POLICY_SCHEMA_VERSION {
        return Err(format!("unsupported policy schema `{}`", settings.schema_version));
    }
    if settings.symlinks != "reject" || settings.unreadable_inputs != "incomplete" {
        return Err("source-envelope policy must reject symlinks and mark unreadable inputs incomplete".to_owned());
    }
    if settings.generated_inputs_policy != "static-literal-only-else-incomplete" {
        return Err(format!("unsupported generated_inputs_policy `{}`", settings.generated_inputs_policy));
    }
    if settings.max_files_per_package == 0 || settings.max_bytes_per_package == 0 || settings.max_resolve_depth == 0 {
        return Err("source-envelope policy bounds must be positive".to_owned());
    }
    if settings.non_claims.is_empty() || settings.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        return Err("source-envelope policy must declare non-empty non_claims".to_owned());
    }
    validate_binding_profiles(settings)?;
    validate_scan_lists(settings)
}

fn validate_binding_profiles(settings: &Settings) -> Result<(), String> {
    let expected = super::BindingProfile::ImplementationBoundV1.as_str();
    if settings.default_binding_profile != expected {
        return Err(format!("unsupported default_binding_profile `{}`", settings.default_binding_profile));
    }
    let mut seen = std::collections::BTreeSet::new();
    for profile in &settings.supported_binding_profiles {
        if profile != expected && profile != super::BindingProfile::LockfileOnlyV1.as_str() {
            return Err(format!("unsupported binding profile `{profile}` in policy"));
        }
        if !seen.insert(profile.as_str()) {
            return Err(format!("duplicate binding profile `{profile}` in policy"));
        }
    }
    if !seen.contains(settings.default_binding_profile.as_str()) {
        return Err("default_binding_profile must appear in supported_binding_profiles".to_owned());
    }
    Ok(())
}

fn validate_scan_lists(settings: &Settings) -> Result<(), String> {
    if settings.build_script.trim().is_empty() {
        return Err("source-envelope policy build_script must not be empty".to_owned());
    }
    for (label, items) in [
        ("included_extensions", &settings.included_extensions),
        ("included_manifests", &settings.included_manifests),
        ("workspace_build_inputs", &settings.workspace_build_inputs),
        ("excluded_root_dirs", &settings.excluded_root_dirs),
    ] {
        if items.is_empty() || items.iter().any(|item| item.trim().is_empty()) {
            return Err(format!("source-envelope policy `{label}` must contain non-empty entries"));
        }
    }
    if let Some(bad) = settings.workspace_build_inputs.iter().find(|rel| !is_pure_normal_relative(rel)) {
        return Err(format!("workspace_build_inputs entry `{bad}` must be a clean relative path"));
    }
    Ok(())
}

fn is_pure_normal_relative(relative: &str) -> bool {
    !relative.is_empty()
        && !relative.contains('\\')
        && Path::new(relative).components().all(|comp| matches!(comp, std::path::Component::Normal(_)))
}

pub(crate) fn compute_build_context(input: &BuildContextInput<'_>, settings: &Settings) -> BuildContextOutcome {
    let mut incomplete_reason = None;
    let file_hashes = settings
        .workspace_build_inputs
        .iter()
        .map(|relative| {
            let (value, reason) = inspect_workspace_build_input(input.workspace_root, relative);
            if incomplete_reason.is_none() {
                incomplete_reason = reason;
            }
            (relative.clone(), value)
        })
        .collect::<BTreeMap<_, _>>();
    let payload = serde_json::json!({
        "target": input.target, "toolchain": input.toolchain,
        "rustc_version": input.rustc_version,
        "cargo_lock_blake3": super::canonical_blake3_identity(input.cargo_lock_blake3),
        "metadata_cargo_args": input.metadata_cargo_args, "rustflags": input.rustflags,
        "workspace_build_inputs": file_hashes, "relocation_preserving": settings.relocation_preserving,
    });
    BuildContextOutcome {
        blake3: super::framed_blake3(BUILD_CONTEXT_DOMAIN, &[payload.to_string().as_bytes()]),
        incomplete_reason,
    }
}

fn inspect_workspace_build_input(workspace_root: &Path, relative: &str) -> (String, Option<String>) {
    if !is_pure_normal_relative(relative) {
        return ("invalid-path".to_owned(), Some(format!("invalid workspace build input `{relative}`")));
    }
    let Ok(segments) = resolve_relative_stack(&[], relative) else {
        return ("invalid-path".to_owned(), Some(format!("invalid workspace build input `{relative}`")));
    };
    if segments.is_empty() {
        return ("invalid-path".to_owned(), Some(format!("invalid workspace build input `{relative}`")));
    }
    let mut cur = workspace_root.to_path_buf();
    for (idx, seg) in segments.iter().enumerate() {
        cur.push(seg);
        let is_last = idx.saturating_add(1) == segments.len();
        match std::fs::symlink_metadata(&cur) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return ("symlink".to_owned(), Some(format!("workspace build input `{relative}` traverses a symlink")));
            }
            Ok(meta) if (!is_last && !meta.file_type().is_dir()) || (is_last && !meta.file_type().is_file()) => {
                return (
                    "non-regular".to_owned(),
                    Some(format!("workspace build input `{relative}` is not a regular file")),
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return ("missing".to_owned(), None),
            Err(error) => {
                return ("error".to_owned(), Some(format!("inspect workspace build input `{relative}`: {error}")));
            }
        }
    }
    match std::fs::read(&cur) {
        Ok(bytes) => (super::framed_blake3("octet-build-input/v1", &[&bytes]), None),
        Err(error) => {
            ("unreadable".to_owned(), Some(format!("unreadable workspace build input `{relative}`: {error}")))
        }
    }
}

pub(crate) fn check_existing_prefix_no_symlinks(root: &Path, segments: &[String]) -> Result<(), String> {
    let mut cur = root.to_path_buf();
    for seg in segments {
        cur.push(seg);
        match std::fs::symlink_metadata(&cur) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("path `{}` traverses a symlink", cur.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(format!("inspect `{}`: {error}", cur.display())),
        }
    }
    Ok(())
}

pub(crate) fn resolve_relative_stack(base: &[String], rel_str: &str) -> Result<Vec<String>, String> {
    if rel_str.is_empty() || rel_str.starts_with('/') || rel_str.contains('\\') {
        return Err(format!("invalid relative path `{rel_str}`"));
    }
    let mut stack = Vec::with_capacity(base.len().saturating_add(RELATIVE_PATH_HEADROOM));
    stack.extend_from_slice(base);
    for seg in rel_str.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            if stack.pop().is_none() {
                return Err(format!("path `{rel_str}` escapes package root"));
            }
        } else {
            stack.push(seg.to_owned());
        }
    }
    Ok(stack)
}

pub(crate) fn check_relative_segments(
    root: &Path,
    segments: &[String],
    is_file_required: bool,
) -> Result<std::path::PathBuf, String> {
    if is_file_required && segments.is_empty() {
        return Err("empty relative path".to_owned());
    }
    let mut cur = root.to_path_buf();
    for (idx, seg) in segments.iter().enumerate() {
        cur.push(seg);
        let meta = std::fs::symlink_metadata(&cur).map_err(|error| format!("inspect `{}`: {error}", cur.display()))?;
        if meta.file_type().is_symlink() {
            return Err(format!("path `{}` traverses a symlink", cur.display()));
        }
        let is_last = idx.saturating_add(1) == segments.len();
        if !is_last && !meta.file_type().is_dir() {
            return Err(format!("parent `{}` is not a directory", cur.display()));
        }
        if is_last && is_file_required && !meta.file_type().is_file() {
            return Err(format!("path `{}` is not a regular file", cur.display()));
        }
    }
    Ok(cur)
}

fn lexical_normalize_abs(path: &Path) -> Result<std::path::PathBuf, String> {
    let mut out = std::path::PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            std::path::Component::RootDir => out.push(comp.as_os_str()),
            std::path::Component::ParentDir if !out.pop() => {
                return Err(format!("path `{}` escapes root", path.display()));
            }
            std::path::Component::CurDir | std::path::Component::ParentDir => {}
            std::path::Component::Normal(seg) => out.push(seg),
        }
    }
    Ok(out)
}

pub(crate) fn resolve_target_relative_stack(package_root: &Path, src_path: &str) -> Result<Vec<String>, String> {
    if src_path.is_empty() {
        return Err("target has empty src_path".to_owned());
    }
    let src = Path::new(src_path);
    if !src.is_absolute() {
        return resolve_relative_stack(&[], src_path);
    }
    let abs_root = if package_root.is_absolute() {
        package_root.to_path_buf()
    } else {
        std::path::absolute(package_root).map_err(|error| format!("resolve package root: {error}"))?
    };
    let norm_root = lexical_normalize_abs(&abs_root)?;
    let norm_src = lexical_normalize_abs(src)?;
    let rel = norm_src
        .strip_prefix(&norm_root)
        .map_err(|_| format!("target `{src_path}` escapes root `{}`", package_root.display()))?;
    rel.components()
        .map(|comp| match comp {
            std::path::Component::Normal(s) => {
                s.to_str().map(str::to_owned).ok_or_else(|| format!("non-UTF-8 `{src_path}`"))
            }
            std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::CurDir
            | std::path::Component::ParentDir => Err(format!("invalid target component in `{src_path}`")),
        })
        .collect()
}
