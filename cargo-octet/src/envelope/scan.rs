use std::collections::BTreeMap;
use std::path::Path;

const PACKAGE_ENVELOPE_DOMAIN: &str = "octet-effect-closure-package-envelope/v1";
const PACKAGE_INCOMPLETE_DOMAIN: &str = "octet-effect-closure-package-envelope-incomplete/v1";
const INITIAL_DIR_QUEUE_CAP: usize = 64;

#[derive(Clone, Debug)]
pub(crate) struct PackageTargetMeta {
    pub(crate) kind: Vec<String>,
    pub(crate) src_path: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PackageSnapshot {
    pub(crate) completeness: super::CompletenessStatus,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) package_envelope_blake3: String,
    pub(crate) generated_inputs: Vec<super::GeneratedInputBinding>,
}

struct WalkBudget {
    max_files: usize,
    max_bytes: usize,
    files_seen: usize,
    bytes_seen: usize,
}

pub(crate) fn inspect_local_package(
    manifest_path: &Path,
    targets: &[PackageTargetMeta],
    settings: &super::policy::Settings,
) -> PackageSnapshot {
    let Some(manifest_name) = manifest_path.file_name().and_then(|name| name.to_str()) else {
        return incomplete_package(vec![format!("manifest `{}` has invalid file name", manifest_path.display())]);
    };
    if !settings.included_manifests.iter().any(|allowed| allowed == manifest_name) {
        return incomplete_package(vec![format!(
            "manifest `{}` is not in policy included_manifests",
            manifest_path.display()
        )]);
    }
    let manifest_bytes = match super::read_regular_non_symlink_file(manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => return incomplete_package(vec![format!("manifest `{}`: {error}", manifest_path.display())]),
    };
    let Some(package_root) = manifest_path.parent() else {
        return incomplete_package(vec![format!(
            "manifest `{}` has no parent directory",
            manifest_path.display()
        )]);
    };
    match std::fs::symlink_metadata(package_root) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return incomplete_package(vec![format!("package root `{}` is a symlink", package_root.display())]);
        }
        Ok(meta) if meta.file_type().is_dir() => {}
        Ok(_) => return incomplete_package(vec![format!("package root `{}` is not a dir", package_root.display())]),
        Err(error) => {
            return incomplete_package(vec![format!("inspect package root `{}`: {error}", package_root.display())]);
        }
    }
    let mut files = BTreeMap::new();
    let mut budget = WalkBudget {
        max_files: settings.max_files_per_package,
        max_bytes: settings.max_bytes_per_package,
        files_seen: 1,
        bytes_seen: manifest_bytes.len(),
    };
    files.insert(manifest_name.to_owned(), manifest_bytes);
    if let Err(error) = collect_package_tree(package_root, manifest_name, settings, &mut budget, &mut files) {
        return incomplete_package(vec![error]);
    }
    analyze_collected_files(package_root, targets, settings, &mut files)
}

fn validate_package_targets(
    package_root: &Path,
    targets: &[PackageTargetMeta],
    settings: &super::policy::Settings,
    files: &BTreeMap<String, Vec<u8>>,
    incomplete: &mut Vec<String>,
) {
    for target in targets {
        let stack = match super::policy::resolve_target_relative_stack(package_root, &target.src_path) {
            Ok(stack) => stack,
            Err(error) => {
                incomplete.push(error);
                continue;
            }
        };
        if stack.is_empty() || settings.excluded_root_dirs.iter().any(|ex| ex == &stack[0]) {
            incomplete.push(format!("package target `{}` enters excluded or empty path", target.src_path));
            continue;
        }
        if let Err(error) = super::policy::check_relative_segments(package_root, &stack, true) {
            incomplete.push(format!("package target `{}`: {error}", target.src_path));
            continue;
        }
        let locator = stack.join("/");
        let is_custom_build = target.kind.iter().any(|kind| kind == "custom-build");
        if !is_custom_build && !files.contains_key(&locator) {
            incomplete
                .push(format!("package target `{}` (`{locator}`) is not included in source envelope", target.src_path));
        }
    }
}

fn analyze_collected_files(
    package_root: &Path,
    targets: &[PackageTargetMeta],
    settings: &super::policy::Settings,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> PackageSnapshot {
    let mut unsupported = Vec::with_capacity(files.len().saturating_add(1));
    let has_custom_build = targets.iter().any(|target| {
        target.kind.iter().any(|kind| kind == "custom-build")
            || Path::new(&target.src_path).file_name().is_some_and(|n| n == settings.build_script.as_str())
    });
    if has_custom_build || files.contains_key(&settings.build_script) {
        unsupported.push(format!("package contains unadmitted build script `{}`", settings.build_script));
    }
    let mut incomplete = Vec::with_capacity(targets.len().saturating_add(files.len()));
    validate_package_targets(package_root, targets, settings, files, &mut incomplete);
    let per_file = files
        .iter()
        .filter(|(locator, _)| {
            Path::new(locator)
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| settings.included_extensions.iter().any(|e| e == ext))
        })
        .map(|(locator, bytes)| super::ast::scan_rust_source(package_root, locator, bytes, settings))
        .collect::<Vec<_>>();
    let generated = per_file.iter().flat_map(|out| out.generated.clone()).collect::<BTreeMap<_, _>>();
    unsupported.extend(per_file.iter().flat_map(|out| out.unsupported.iter().cloned()));
    incomplete.extend(per_file.iter().flat_map(|out| out.incomplete.iter().cloned()));
    incomplete.extend(
        per_file
            .iter()
            .flat_map(|out| out.module_locators.iter())
            .filter(|mod_loc| !files.contains_key(*mod_loc))
            .map(|mod_loc| format!("module path `{mod_loc}` is not included in source envelope")),
    );
    let generated_inputs = generated
        .iter()
        .map(|(locator, bytes)| super::GeneratedInputBinding {
            locator: locator.clone(),
            blake3: super::framed_blake3("octet-generated-input/v1", &[bytes]),
        })
        .collect::<Vec<_>>();
    files.extend(generated);
    finalize_package(files, generated_inputs, unsupported, incomplete)
}

fn finalize_package(
    files: &BTreeMap<String, Vec<u8>>,
    generated_inputs: Vec<super::GeneratedInputBinding>,
    mut unsupported: Vec<String>,
    mut incomplete: Vec<String>,
) -> PackageSnapshot {
    if !incomplete.is_empty() {
        incomplete.extend(unsupported);
        return incomplete_package(incomplete);
    }
    if !unsupported.is_empty() {
        unsupported.sort();
        unsupported.dedup();
        let joined = unsupported.join("\n");
        return PackageSnapshot {
            completeness: super::CompletenessStatus::UnsupportedGeneratedInput,
            diagnostics: unsupported,
            package_envelope_blake3: super::framed_blake3(PACKAGE_INCOMPLETE_DOMAIN, &[
                b"unsupported",
                joined.as_bytes(),
            ]),
            generated_inputs,
        };
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(PACKAGE_ENVELOPE_DOMAIN.as_bytes());
    hasher.update(&[0]);
    for (locator, bytes) in files {
        if let Ok(len) = u64::try_from(locator.len()) {
            hasher.update(&len.to_le_bytes());
        }
        hasher.update(locator.as_bytes());
        if let Ok(blen) = u64::try_from(bytes.len()) {
            hasher.update(&blen.to_le_bytes());
        }
        hasher.update(bytes);
    }
    PackageSnapshot {
        completeness: super::CompletenessStatus::Complete,
        diagnostics: Vec::new(),
        package_envelope_blake3: format!("b3:{}", hasher.finalize().to_hex()),
        generated_inputs,
    }
}

fn incomplete_package(mut diagnostics: Vec<String>) -> PackageSnapshot {
    diagnostics.sort();
    diagnostics.dedup();
    let joined = diagnostics.join("\n");
    PackageSnapshot {
        completeness: super::CompletenessStatus::Incomplete,
        diagnostics,
        package_envelope_blake3: super::framed_blake3(PACKAGE_INCOMPLETE_DOMAIN, &[b"incomplete", joined.as_bytes()]),
        generated_inputs: Vec::new(),
    }
}

fn collect_package_tree(
    root: &Path,
    primary_manifest: &str,
    settings: &super::policy::Settings,
    budget: &mut WalkBudget,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let mut pending_dirs = Vec::with_capacity(budget.max_files.min(INITIAL_DIR_QUEUE_CAP));
    pending_dirs.push(root.to_path_buf());
    while let Some(dir) = pending_dirs.pop() {
        let has_manifest = settings.included_manifests.iter().any(|manifest| dir.join(manifest).is_file());
        let mut entries = std::fs::read_dir(&dir)
            .map_err(|error| format!("read package directory `{}`: {error}", dir.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("read package directory entry in `{}`: {error}", dir.display()))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| format!("non-UTF-8 entry inside package directory `{}`", dir.display()))?;
            if (dir == root && name == primary_manifest)
                || (has_manifest && settings.excluded_root_dirs.iter().any(|excluded| excluded == &name))
            {
                continue;
            }
            let path = entry.path();
            let meta =
                std::fs::symlink_metadata(&path).map_err(|error| format!("inspect `{}`: {error}", path.display()))?;
            if meta.file_type().is_symlink() {
                return Err(format!("symlink `{}` is rejected by source-envelope policy", path.display()));
            }
            if meta.file_type().is_dir() {
                if pending_dirs.len() >= budget.max_files {
                    return Err(format!("package directory queue exceeded {}", budget.max_files));
                }
                pending_dirs.push(path);
                continue;
            }
            if !meta.file_type().is_file() {
                return Err(format!("non-regular file `{}` is rejected", path.display()));
            }
            if !is_included_file(&name, settings) {
                continue;
            }
            budget.files_seen = budget.files_seen.saturating_add(1);
            if budget.files_seen > budget.max_files {
                return Err(format!("package exceeded max_files_per_package ({})", budget.max_files));
            }
            let bytes = super::read_regular_non_symlink_file(&path)?;
            budget.bytes_seen = budget.bytes_seen.saturating_add(bytes.len());
            if budget.bytes_seen > budget.max_bytes {
                return Err(format!("package exceeded max_bytes_per_package ({})", budget.max_bytes));
            }
            let locator = super::relative_locator(root, &path)?;
            files.insert(locator, bytes);
        }
    }
    Ok(())
}

fn is_included_file(name: &str, settings: &super::policy::Settings) -> bool {
    name == settings.build_script
        || settings.included_manifests.iter().any(|manifest| manifest == name)
        || Path::new(name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| settings.included_extensions.iter().any(|allowed| allowed == ext))
}
