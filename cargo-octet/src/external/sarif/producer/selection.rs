/// Matches one path against a glob where `**` spans path segments and `*`/`?` stay within one
/// segment. The matcher is intentionally small: producer exclusions use segment globs only.
pub(super) fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    match_segments(&pattern, &path)
}

pub(super) fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
    let (mut pattern_index, mut path_index) = (0, 0);
    let (mut star, mut star_path) = (None, 0);
    while path_index < path.len() {
        match pattern.get(pattern_index) {
            Some(&"**") => {
                star = Some(pattern_index);
                star_path = path_index;
                pattern_index += 1;
            }
            Some(segment) if match_segment(segment, path[path_index]) => {
                pattern_index += 1;
                path_index += 1;
            }
            _ => match star {
                Some(index) => {
                    pattern_index = index + 1;
                    star_path += 1;
                    path_index = star_path;
                }
                None => return false,
            },
        }
    }
    pattern[pattern_index..].iter().all(|segment| *segment == "**")
}

pub(super) fn match_segment(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let (mut pattern_index, mut value_index) = (0, 0);
    let (mut star, mut star_value) = (None, 0);
    while value_index < value.len() {
        match pattern.get(pattern_index) {
            Some(&b'*') => {
                star = Some(pattern_index);
                star_value = value_index;
                pattern_index += 1;
            }
            Some(&b'?') => {
                pattern_index += 1;
                value_index += 1;
            }
            Some(&literal) if literal == value[value_index] => {
                pattern_index += 1;
                value_index += 1;
            }
            _ => match star {
                Some(star_index) => {
                    pattern_index = star_index + 1;
                    star_value += 1;
                    value_index = star_value;
                }
                None => return false,
            },
        }
    }
    pattern[pattern_index..].iter().all(|byte| *byte == b'*')
}

pub(crate) fn snapshot_sources(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
) -> Result<super::process::SourceSnapshot, String> {
    let mut files = std::collections::BTreeMap::new();
    let mut selected_paths = Vec::with_capacity(profile.scope.paths.len());
    let mut missing_paths = Vec::with_capacity(profile.scope.paths.len());
    let mut total_bytes = 0u64;
    for declared in &profile.scope.paths {
        let absolute = workspace_root.join(declared);
        match std::fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("SARIF producer scope path `{declared}` is a symlink, which is not allowed"));
            }
            Ok(metadata) if metadata.is_dir() => {
                selected_paths.push(declared.clone());
                walk_source_tree(
                    profile,
                    workspace_root,
                    std::path::Path::new(declared),
                    &mut files,
                    &mut total_bytes,
                )?;
            }
            Ok(metadata) if metadata.is_file() => {
                if scope_includes(profile, std::path::Path::new(declared), declared) {
                    record_source(profile, workspace_root, declared, &mut files, &mut total_bytes)?;
                    selected_paths.push(declared.clone());
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => missing_paths.push(declared.clone()),
            Err(error) => return Err(format!("inspect SARIF producer scope path `{declared}`: {error}")),
        }
    }
    if total_bytes > profile.scope.max_bytes {
        return Err(format!(
            "SARIF producer source snapshot exceeds the declared {} byte limit",
            profile.scope.max_bytes
        ));
    }
    if files.len() > profile.scope.max_files {
        return Err(format!(
            "SARIF producer source snapshot exceeds the declared {} file limit",
            profile.scope.max_files
        ));
    }
    let files = files.into_iter().map(|(path, identity)| super::SourceFile { path, identity }).collect::<Vec<_>>();
    Ok(super::process::SourceSnapshot {
        identity: super::recording::source_snapshot_identity(&files)?,
        files,
        selected_paths,
        missing_paths,
    })
}

pub(super) fn walk_source_tree(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
    relative: &std::path::Path,
    files: &mut std::collections::BTreeMap<String, String>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let names = sorted_directory_names(&workspace_root.join(relative), relative)?;
    let mut pending = Vec::with_capacity(1);
    pending.push((relative.to_path_buf(), names.into_iter()));
    while let Some((directory, names)) = pending.last_mut() {
        let Some(name) = names.next() else {
            pending.pop();
            continue;
        };
        let child = directory.join(name);
        let locator = child.to_string_lossy().replace('\\', "/");
        if profile.scope.exclude_globs.iter().any(|glob| glob_matches(glob, &locator)) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(workspace_root.join(&child))
            .map_err(|error| format!("inspect SARIF producer source `{locator}`: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("SARIF producer source `{locator}` is a symlink, which is not allowed"));
        }
        if metadata.is_dir() {
            let names = sorted_directory_names(&workspace_root.join(&child), &child)?;
            pending.push((child, names.into_iter()));
        } else if metadata.is_file() && scope_includes(profile, &child, &locator) {
            record_source(profile, workspace_root, &locator, files, total_bytes)?;
        }
    }
    Ok(())
}

fn sorted_directory_names(
    absolute: &std::path::Path,
    relative: &std::path::Path,
) -> Result<Vec<std::ffi::OsString>, String> {
    let mut names = std::fs::read_dir(absolute)
        .map_err(|error| format!("read SARIF producer scope directory {}: {error}", relative.display()))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read SARIF producer scope entry in {}: {error}", relative.display()))?;
    names.sort();
    Ok(names)
}

fn record_source(
    profile: &super::Profile,
    workspace_root: &std::path::Path,
    locator: &str,
    files: &mut std::collections::BTreeMap<String, String>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let bytes = read_bounded_file(&workspace_root.join(locator), profile.scope.max_bytes)?;
    let length = u64::try_from(bytes.len()).map_err(|error| format!("SARIF source length is out of range: {error}"))?;
    *total_bytes =
        total_bytes.checked_add(length).filter(|total| *total <= profile.scope.max_bytes).ok_or_else(|| {
            format!("SARIF producer source snapshot exceeds the declared {} byte limit", profile.scope.max_bytes)
        })?;
    if files.len() >= profile.scope.max_files && !files.contains_key(locator) {
        return Err(format!(
            "SARIF producer source snapshot exceeds the declared {} file limit",
            profile.scope.max_files
        ));
    }
    files.insert(locator.to_owned(), super::super::prefixed_identity(&bytes));
    Ok(())
}

pub(super) fn scope_includes(profile: &super::Profile, path: &std::path::Path, locator: &str) -> bool {
    profile.scope.extensions.iter().any(|extension| {
        path.extension()
            .is_some_and(|observed| observed.to_string_lossy() == extension.trim_start_matches('.'))
    }) && !profile.scope.exclude_globs.iter().any(|glob| glob_matches(glob, locator))
}

/// Read a regular file with one extra byte of bounded overflow detection. A path that is a symlink
/// or not a regular file is rejected, and an over-limit file fails closed instead of being
/// truncated.
pub(super) fn read_bounded_file(path: &std::path::Path, max_bytes: u64) -> Result<Vec<u8>, String> {
    use std::io::Read as _;

    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("inspect SARIF producer input {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("SARIF producer input {} must be a regular file", path.display()));
    }
    if metadata.len() > max_bytes {
        return Err(format!("SARIF producer input {} exceeds the {max_bytes} byte limit", path.display()));
    }
    let file =
        std::fs::File::open(path).map_err(|error| format!("open SARIF producer input {}: {error}", path.display()))?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len())
            .map_err(|error| format!("SARIF producer input length is out of range: {error}"))?,
    );
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read SARIF producer input {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).map_err(|error| format!("SARIF producer input length is out of range: {error}"))?
        > max_bytes
    {
        return Err(format!("SARIF producer input {} exceeds the {max_bytes} byte limit", path.display()));
    }
    Ok(bytes)
}

/// Read the report twice and require identical bytes. A scanner output swapped between the process
/// exit and the ingestion read is not evidence for this execution.
pub(super) fn read_stable_report_with(
    path: &std::path::Path,
    max_bytes: u64,
    between: impl FnOnce(),
) -> Result<Vec<u8>, super::process::StepFailure> {
    let first = read_bounded_file(path, max_bytes)
        .map_err(|error| super::process::StepFailure::new(super::process::FailureKind::Spawn, error))?;
    between();
    let second = read_bounded_file(path, max_bytes)
        .map_err(|error| super::process::StepFailure::new(super::process::FailureKind::Spawn, error))?;
    if first != second {
        return Err(super::process::StepFailure::new(
            super::process::FailureKind::Spawn,
            "SARIF report changed between execution and ingestion; the retained bytes are not evidence",
        ));
    }
    Ok(second)
}

pub(crate) fn read_stable_report(
    path: &std::path::Path,
    max_bytes: u64,
) -> Result<Vec<u8>, super::process::StepFailure> {
    read_stable_report_with(path, max_bytes, || {})
}

pub(super) fn remove_if_present(path: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove stale SARIF producer report {}: {error}", path.display())),
    }
}

pub(super) fn verify_rule_bytes(profile: &super::Profile, workspace_root: &std::path::Path) -> Result<(), String> {
    if profile.rules.kind != super::LOCAL_RULE_FILE {
        return Ok(());
    }
    let bytes = read_bounded_file(&workspace_root.join(&profile.rules.path), profile.limits.max_report_bytes)?;
    if super::super::prefixed_identity(&bytes) != profile.rules.identity {
        return Err(format!(
            "SARIF producer rule bundle `{}` no longer matches its checked identity",
            profile.rules.path
        ));
    }
    Ok(())
}
