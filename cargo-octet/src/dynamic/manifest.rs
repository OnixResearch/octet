const MAX_SOURCE_FILE_BYTES: u64 = 1_048_576;

pub fn collect_source_digest(
    workspace_root: &std::path::Path,
    limits: &super::Limits,
) -> Result<(String, Option<String>), String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let files = collect_rust_files_iterative(workspace_root, limits.max_source_files)?;
    let mut manifest = String::with_capacity(files.len() * 80);
    for rel in &files {
        let bytes = root.read_bounded(&crate::cap_fs::RootRelativePath::new(rel)?, MAX_SOURCE_FILE_BYTES)?;
        manifest.push_str(rel);
        manifest.push('\t');
        manifest.push_str(&super::prefixed_identity(&bytes));
        manifest.push('\n');
    }
    let lock_path = workspace_root.join("Cargo.lock");
    let lockfile_identity = if lock_path.is_file() {
        let lock_bytes =
            root.read_bounded(&crate::cap_fs::RootRelativePath::new("Cargo.lock")?, MAX_SOURCE_FILE_BYTES)?;
        Some(super::prefixed_identity(&lock_bytes))
    } else {
        None
    };
    Ok((super::prefixed_identity(manifest.as_bytes()), lockfile_identity))
}

fn collect_rust_files_iterative(base: &std::path::Path, max_files: usize) -> Result<Vec<String>, String> {
    let mut files = Vec::with_capacity(max_files.min(64));
    let mut pending = Vec::with_capacity(16);
    pending.push(base.to_path_buf());
    while let Some(current) = pending.pop() {
        let entries =
            std::fs::read_dir(&current).map_err(|error| format!("read dir {}: {error}", current.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("read dir entry: {error}"))?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else if matches!(name_str.as_ref(), "Cargo.toml" | "Cargo.lock") || name_str.ends_with(".rs") {
                if files.len() >= max_files {
                    return Err(format!("source file count exceeds profile limit {max_files}"));
                }
                let rel = path.strip_prefix(base).map_err(|error| format!("strip workspace prefix: {error}"))?;
                files.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    Ok(files)
}
