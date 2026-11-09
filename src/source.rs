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

/// Normalizes one configured workspace-relative source scope the way
/// `cargo octet` does: trims whitespace, strips leading `./` and trailing `/`.
pub(crate) fn normalize_scope(raw: &str) -> String {
    let mut scope = raw.trim();
    while let Some(rest) = scope.strip_prefix("./") {
        scope = rest.trim_start_matches('/');
    }
    scope.trim_end_matches('/').to_owned()
}

/// Whether a workspace-relative source path is one of `scopes` or lies below
/// one. Matching is by whole path components, so `crates/core` covers
/// `crates/core/src/lib.rs` but not `crates/core-shell/src/lib.rs`.
pub(crate) fn path_in_scopes(path: &std::path::Path, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| !scope.is_empty() && path.starts_with(scope))
}

/// Whether `span` comes from a file inside one of the normalized `scopes`.
/// Cargo runs rustc from the workspace root, so an absolute file name is made
/// relative to rustc's working directory before matching.
pub(crate) fn span_in_scopes(sess: &rustc_session::Session, span: rustc_span::Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let file = sess.source_map().lookup_source_file(span.lo());
    let path = normalize_path(&source_file_path(&file));
    let relative = sess
        .opts
        .working_dir
        .local_path()
        .map(normalize_path)
        .and_then(|root| path.strip_prefix(root).ok().map(std::path::Path::to_path_buf));
    path_in_scopes(relative.as_deref().unwrap_or(&path), scopes)
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

    #[test]
    fn scopes_normalize_like_the_runner() {
        for raw in [
            "crates/domain/src",
            "./crates/domain/src",
            " crates/domain/src/ ",
            "././crates/domain/src//",
        ] {
            assert_eq!(super::normalize_scope(raw), "crates/domain/src", "{raw:?}");
        }
        assert_eq!(super::normalize_scope(" ./ "), "");
    }

    #[test]
    fn scopes_match_whole_path_components() {
        let scopes = vec![
            super::normalize_scope("./crates/core/"),
            super::normalize_scope("src/lib.rs"),
        ];
        let inside = ["crates/core/src/lib.rs", "crates/core", "src/lib.rs"];
        let outside = [
            "crates/core-shell/src/lib.rs",
            "vendor/crates/core/src/lib.rs",
            "src/lib.rs.bak",
            "src/main.rs",
        ];

        for path in inside {
            assert!(super::path_in_scopes(std::path::Path::new(path), &scopes), "{path} is in scope");
        }
        for path in outside {
            assert!(!super::path_in_scopes(std::path::Path::new(path), &scopes), "{path} is out of scope");
        }
        assert!(!super::path_in_scopes(std::path::Path::new("src/lib.rs"), &[String::new()]));
    }
}
