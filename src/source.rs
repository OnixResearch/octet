//! Compiler-owned source provenance for file-level lint passes.

// r[impl octet.lint_signal_quality.source_ownership]
pub(crate) fn is_local(file: &rustc_span::SourceFile) -> bool {
    !file.is_imported()
}

pub(crate) fn source_file_path(file: &rustc_span::SourceFile) -> std::path::PathBuf {
    std::path::PathBuf::from(file.name.prefer_local_unconditionally().to_string_lossy().into_owned())
}

pub(crate) fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Cargo runs rustc from the workspace root. Files outside it, such as build
/// script output under a custom `CARGO_TARGET_DIR`, are not workspace source.
pub(crate) fn workspace_root() -> std::path::PathBuf {
    std::env::current_dir().map(|path| normalize_path(&path)).unwrap_or_default()
}

pub(crate) fn is_workspace_source_path(path: &std::path::Path, workspace_root: &std::path::Path) -> bool {
    if path.is_absolute() {
        return path.starts_with(workspace_root);
    }

    workspace_root.join(path).is_file()
}

#[cfg(test)]
mod tests {
    fn fixture() -> rustc_span::SourceFile {
        rustc_span::SourceFile::new(
            rustc_span::FileName::Real(rustc_span::RealFileName::from_virtual_path(std::path::Path::new(
                "/workspace/patches/example/bad_file.rs",
            ))),
            "pub struct Example;".to_owned(),
            rustc_span::SourceFileHashAlgorithm::Blake3,
            None,
        )
        .expect("small fixture source fits the compiler offset range")
    }

    // r[verify octet.lint_signal_quality.source_ownership]
    #[test]
    fn local_source_is_admitted_even_under_a_patch_directory() {
        rustc_span::create_default_session_globals_then(|| {
            let file = fixture();
            assert!(super::is_local(&file));
        });
    }

    #[test]
    fn imported_source_is_rejected_without_changing_its_path() {
        rustc_span::create_default_session_globals_then(|| {
            let mut file = fixture();
            assert!(super::is_local(&file));
            // Rustc records imported source separately and leaves `src` absent.
            file.src = None;
            assert!(!super::is_local(&file));
        });
    }
}
