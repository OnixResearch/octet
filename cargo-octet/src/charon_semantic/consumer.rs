#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WorkspaceProfile {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) provider: super::model::ProviderDependency,
    pub(crate) source: WorkspaceSource,
    pub(crate) extraction: ExtractionSettings,
    pub(crate) fact_vocabulary: Vec<String>,
    pub(crate) effect_candidate_vocabulary: Vec<String>,
    pub(crate) limits: super::model::SemanticLimits,
    pub(crate) artifact_roles: Vec<String>,
    pub(crate) references: Vec<super::model::ReferenceMetadata>,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub(crate) struct FileDigest {
    pub(crate) path: String,
    pub(crate) identity: String,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub(crate) struct WorkspaceSource {
    pub(crate) root: String,
    pub(crate) manifest: FileDigest,
    pub(crate) lockfile: FileDigest,
    pub(crate) crate_name: String,
    pub(crate) source_files: Vec<FileDigest>,
    pub(crate) selected_roots: Vec<super::model::SelectedItem>,
    pub(crate) broad_selection_allowed: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Nickel extraction contract declares explicit independent capability flags"
)]
pub(crate) struct ExtractionSettings {
    pub(crate) preset: String,
    pub(crate) has_errors: bool,
    pub(crate) skip_borrowck: bool,
    pub(crate) borrow_check_preserved: bool,
    pub(crate) target_triple: String,
    pub(crate) pointer_width_bytes: u64,
    pub(crate) endianness: String,
    pub(crate) serialization_format: String,
    pub(crate) no_default_features: bool,
    pub(crate) features: Vec<String>,
    pub(crate) cfg: Vec<String>,
    pub(crate) environment_allowlist: Vec<String>,
}

pub(crate) struct ValidatedWorkspace {
    pub(crate) profile_identity: String,
    pub(crate) profile: super::model::CharonSemanticProfile,
    pub(crate) source_files: Vec<FileDigest>,
    pub(crate) manifest: FileDigest,
    pub(crate) lockfile: FileDigest,
}

pub(crate) fn is_workspace_profile_json(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("schema_version").and_then(serde_json::Value::as_str).map(ToOwned::to_owned))
        .as_deref()
        == Some(super::model::CONSUMER_PROFILE_SCHEMA)
}

pub(crate) fn parse_and_validate(text: &str) -> Result<ValidatedWorkspace, Vec<String>> {
    let parsed: WorkspaceProfile =
        serde_json::from_str(text).map_err(|error| vec![format!("parse Charon workspace profile: {error}")])?;
    let mut diagnostics = Vec::new();
    if parsed.schema_version != super::model::CONSUMER_PROFILE_SCHEMA {
        diagnostics.push(format!("unsupported Charon workspace profile schema `{}`", parsed.schema_version));
    }
    if parsed.profile_id.trim().is_empty() || parsed.profile_id == super::model::PROFILE_ID {
        diagnostics.push("workspace profile_id must be non-empty and distinct from the canary profile".to_owned());
    }
    validate_source(&parsed.source, &mut diagnostics);
    let command_argv = derive_command_argv(&parsed, &mut diagnostics);
    let adapted = super::core::adapt_workspace_profile(&parsed, command_argv);
    diagnostics.extend(super::core::validate_provider_and_bounds(&adapted));
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(ValidatedWorkspace {
            profile_identity: super::prefixed_identity(text.as_bytes()),
            profile: adapted,
            source_files: parsed.source.source_files,
            manifest: parsed.source.manifest,
            lockfile: parsed.source.lockfile,
        })
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn verify_workspace_inputs(
    workspace_root: &std::path::Path,
    validated: &ValidatedWorkspace,
) -> Result<std::path::PathBuf, Vec<String>> {
    let canonical_workspace =
        workspace_root.canonicalize().map_err(|e| vec![format!("canonicalize workspace root: {e}")])?;
    let source_root = canonical_workspace
        .join(&validated.profile.source.root)
        .canonicalize()
        .map_err(|e| vec![format!("canonicalize workspace source root: {e}")])?;
    if !source_root.starts_with(&canonical_workspace) {
        return Err(vec!["workspace source root escapes the workspace root".to_owned()]);
    }
    let cap_root = crate::cap_fs::SourceRoot::open_existing(&source_root).map_err(|e| vec![e])?;
    let max_bytes = u64::try_from(validated.profile.limits.snapshot_max_bytes)
        .map_err(|_| vec!["workspace snapshot_max_bytes exceeds u64".to_owned()])?;
    let mut diagnostics = Vec::new();
    check_file_digest(&cap_root, &validated.manifest, max_bytes, "manifest", &mut diagnostics);
    check_file_digest(&cap_root, &validated.lockfile, max_bytes, "lockfile", &mut diagnostics);
    for source_file in &validated.source_files {
        check_file_digest(&cap_root, source_file, max_bytes, "source file", &mut diagnostics);
    }
    if diagnostics.is_empty() {
        Ok(source_root)
    } else {
        diagnostics.sort();
        diagnostics.dedup();
        Err(diagnostics)
    }
}

fn validate_source(source: &WorkspaceSource, diagnostics: &mut Vec<String>) {
    if !is_safe_relative_path(&source.root) || !is_safe_crate_name(&source.crate_name) {
        diagnostics.push("workspace source root or crate_name is unsafe".to_owned());
    }
    if source.broad_selection_allowed {
        diagnostics.push("workspace profile cannot enable broad_selection_allowed".to_owned());
    }
    if source.source_files.is_empty() || source.selected_roots.is_empty() {
        diagnostics.push("workspace profile requires bounded non-empty source_files and selected_roots".to_owned());
    }
    validate_file_digest(&source.manifest, "manifest", diagnostics);
    validate_file_digest(&source.lockfile, "lockfile", diagnostics);
    let mut file_map = std::collections::BTreeMap::new();
    for file in &source.source_files {
        validate_file_digest(file, "source file", diagnostics);
        if file_map.insert(file.path.as_str(), file.identity.as_str()).is_some() {
            diagnostics.push(format!("duplicate workspace source file `{}`", file.path));
        }
    }
    let mut roots = std::collections::BTreeSet::new();
    for root in &source.selected_roots {
        if !roots.insert(root.qualified_name.as_str()) {
            diagnostics.push(format!("duplicate selected root `{}`", root.qualified_name));
        }
        if root.kind != "function" || root.begin_line == 0 || root.end_line < root.begin_line {
            diagnostics.push(format!("selected root `{}` has invalid kind or span", root.qualified_name));
        }
        if file_map.get(root.source_file.as_str()).copied() != Some(root.source_identity.as_str()) {
            diagnostics
                .push(format!("selected root `{}` source identity mismatches source_files", root.qualified_name));
        }
    }
}

fn derive_command_argv(profile: &WorkspaceProfile, diagnostics: &mut Vec<String>) -> Vec<String> {
    const POINTER_WIDTH_BYTES: u64 = 8;
    let ext = &profile.extraction;
    if ext.has_errors || ext.skip_borrowck || !ext.borrow_check_preserved {
        diagnostics.push("workspace profile must preserve borrow checking without extraction errors".to_owned());
    }
    if ext.preset != "Aeneas"
        || ext.serialization_format != "llbc-json-v1"
        || ext.target_triple != "x86_64-unknown-linux-gnu"
        || ext.pointer_width_bytes != POINTER_WIDTH_BYTES
        || ext.endianness != "little"
    {
        diagnostics.push("workspace profile extraction target or preset is unsupported".to_owned());
    }
    if ext.environment_allowlist.iter().map(String::as_str).collect::<Vec<_>>()
        != ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME"]
    {
        diagnostics.push("workspace profile environment allowlist drifted".to_owned());
    }
    let mut argv = vec![
        "charon".into(),
        "cargo".into(),
        "--preset".into(),
        "aeneas".into(),
        "--dest-file".into(),
        "snapshot.llbc".into(),
    ];
    for root in &profile.source.selected_roots {
        if !is_safe_item_path(&root.qualified_name) {
            diagnostics.push(format!("selected root `{}` contains unsafe characters", root.qualified_name));
        }
        argv.extend(["--start-from".into(), root.qualified_name.clone()]);
    }
    for cfg in &ext.cfg {
        if !is_safe_cfg(cfg) {
            diagnostics.push(format!("extraction cfg `{cfg}` is unsafe"));
        }
        argv.extend(["--rustc-arg".into(), format!("--cfg={cfg}")]);
    }
    argv.extend([
        "--".into(),
        "--manifest-path".into(),
        profile.source.manifest.path.clone(),
        "--locked".into(),
        "--offline".into(),
    ]);
    if ext.no_default_features {
        argv.push("--no-default-features".into());
    }
    if !ext.features.is_empty() {
        for feature in &ext.features {
            if !is_safe_feature(feature) {
                diagnostics.push(format!("extraction feature `{feature}` is unsafe"));
            }
        }
        argv.extend(["--features".into(), ext.features.join(",")]);
    }
    argv
}

fn check_file_digest(
    root: &crate::cap_fs::SourceRoot,
    digest: &FileDigest,
    max_bytes: u64,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let Ok(relative) = crate::cap_fs::RootRelativePath::new(&digest.path) else {
        diagnostics.push(format!("unsafe workspace {label} path `{}`", digest.path));
        return;
    };
    match root.read_bounded(&relative, max_bytes) {
        Ok(bytes) => {
            let actual = super::prefixed_identity(&bytes);
            if actual != digest.identity {
                diagnostics.push(format!(
                    "workspace {label} `{}` identity mismatch: expected {}, actual {actual}",
                    digest.path, digest.identity
                ));
            }
        }
        Err(error) => diagnostics.push(format!("workspace {label} `{}` is unreadable: {error}", digest.path)),
    }
}

fn validate_file_digest(digest: &FileDigest, label: &str, diagnostics: &mut Vec<String>) {
    if !is_safe_relative_path(&digest.path) {
        diagnostics.push(format!("workspace {label} path `{}` is unsafe", digest.path));
    }
    if !super::is_b3_identity(&digest.identity) {
        diagnostics.push(format!("workspace {label} `{}` has malformed identity", digest.path));
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !std::path::Path::new(path).is_absolute()
        && path.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
}

fn is_safe_token(text: &str, extra: &str) -> bool {
    !text.is_empty()
        && !text.starts_with('-')
        && text.bytes().all(|b| b.is_ascii_alphanumeric() || extra.as_bytes().contains(&b))
}

fn is_safe_crate_name(name: &str) -> bool {
    is_safe_token(name, "_-")
}
fn is_safe_feature(feature: &str) -> bool {
    is_safe_token(feature, "_-/:")
}
fn is_safe_cfg(cfg: &str) -> bool {
    is_safe_token(cfg, "_=\"")
}
fn is_safe_item_path(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('-') && name.split("::").all(|p| is_safe_token(p, "_"))
}
