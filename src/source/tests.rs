//! Existing compiler provenance and source-scope tests.

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

#[cfg(unix)]
#[test]
fn scopes_resolve_through_symlinks_and_parent_components() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time after the epoch")
        .as_nanos();
    let scratch = std::env::temp_dir().join(format!("octet-scope-resolution-{}-{unique}", std::process::id()));
    let root = scratch.join("workspace");
    std::fs::create_dir_all(root.join("src")).expect("source directory");
    std::fs::create_dir_all(root.join("crates/shell")).expect("crate directory");
    std::fs::create_dir_all(scratch.join("outside")).expect("outside directory");
    std::fs::write(root.join("src/lib.rs"), "").expect("source file");
    std::os::unix::fs::symlink(root.join("src"), root.join("linked")).expect("in-root symlink");
    std::os::unix::fs::symlink(scratch.join("outside"), root.join("escape")).expect("escaping symlink");
    let root = root.canonicalize().expect("canonical root");

    let cases = [
        ("src", super::ScopeResolution::Inside("src".to_owned())),
        ("src/lib.rs", super::ScopeResolution::Inside("src/lib.rs".to_owned())),
        ("linked", super::ScopeResolution::Inside("src".to_owned())),
        ("crates/shell/../../src", super::ScopeResolution::Inside("src".to_owned())),
        ("../outside", super::ScopeResolution::Outside),
        ("escape", super::ScopeResolution::Outside),
        ("crates/..", super::ScopeResolution::Root),
        ("missing/src", super::ScopeResolution::Missing),
    ];
    let resolved = cases.iter().map(|(scope, _expected)| super::resolve_scope(&root, scope)).collect::<Vec<_>>();
    std::fs::remove_dir_all(&scratch).expect("remove scratch directory");

    for ((scope, expected), actual) in cases.iter().zip(resolved) {
        assert_eq!(&actual, expected, "{scope}");
    }
}

#[test]
fn leading_comment_lines_cross_attributes_and_stop_at_gaps() {
    let prefix = "struct Before;\n\n// detached\n\n// first\n/// second\n#[inline]\n// between\n#[cfg_attr(\n    test,\n    derive(Debug)\n)]\n\n#[doc(hidden)]\n    ";
    let lines = super::leading_comment_lines(prefix).collect::<Vec<_>>();
    assert_eq!(lines, ["// between", "/// second", "// first"]);
}

#[test]
fn leading_comment_lines_stop_at_code_and_inner_attributes() {
    assert_eq!(super::leading_comment_lines("const A: [u8; 1] = [0];\n").count(), 0);
    assert_eq!(super::leading_comment_lines("#![allow(dead_code)]\n").count(), 0);
    assert_eq!(super::leading_comment_lines("// note\n\nfn f() {}\n").count(), 0);
    assert_eq!(super::leading_comment_lines("mod m { ").count(), 0);
    assert_eq!(super::leading_comment_lines("// note\n").collect::<Vec<_>>(), ["// note"]);
}
