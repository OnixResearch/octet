//! Compiler-owned source provenance for file-level lint passes.

// r[impl octet.lint_signal_quality.source_ownership]
pub(crate) fn is_local(file: &rustc_span::SourceFile) -> bool {
    !file.is_imported()
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
