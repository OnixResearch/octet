use rustc_ast::Crate;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::SourceFile;
use rustc_span::Span;
use rustc_span::SyntaxContext;

use crate::diagnostics::emit_tiger_style_lint;

const IGNORED_SOURCE_COMPONENTS: &[&str] = &[
    "target",
    "ui_tests",
    "generated",
    "OUT_DIR",
    "rustlib",
    "registry",
    ".cargo",
];
const VENDOR_CARGO_DEPS_MARKER: &str = "vendor-cargo-deps";

declare_tool_lint! {
    /// Detects production source files whose whole-file physical line count
    /// exceeds a configurable project limit (default: 300).
    pub tigerstyle::EXCESSIVE_FILE_LENGTH,
    Warn,
    "source file exceeds line limit"
}

pub struct ExcessiveFileLength {
    max_lines: usize,
}

impl ExcessiveFileLength {
    pub fn new(max_lines: usize) -> Self {
        Self { max_lines }
    }
}

impl_lint_pass!(ExcessiveFileLength => [EXCESSIVE_FILE_LENGTH]);

fn normalized_components(path: &str) -> Vec<String> {
    let mut components = Vec::new();
    for component in path.replace('\\', "/").split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            other => components.push(other.to_owned()),
        }
    }
    components
}

fn is_ignored_source_component(component: &str) -> bool {
    IGNORED_SOURCE_COMPONENTS.contains(&component) || component.contains(VENDOR_CARGO_DEPS_MARKER)
}

fn is_generated_or_fixture_path(path: &str) -> bool {
    let components = normalized_components(path);
    components.iter().any(|component| is_ignored_source_component(component))
        || components.windows(2).any(|window| window[0] == "tests" && window[1] == "fixtures")
        || path.ends_with(".rs.in")
}

fn is_checked_source_path(path: &std::path::Path, workspace_root: &std::path::Path) -> bool {
    crate::source::is_workspace_source_path(path, workspace_root)
        && !is_generated_or_fixture_path(&path.to_string_lossy())
}

fn line_count(source_file: &SourceFile) -> Option<usize> {
    source_file.src.as_ref().map(|src| src.lines().count())
}

impl EarlyLintPass for ExcessiveFileLength {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _krate: &Crate) {
        let workspace_root = crate::source::workspace_root();
        for source_file in cx.sess().source_map().files().iter() {
            if !crate::source::is_local(source_file) {
                continue;
            }
            let path = crate::source::normalize_path(&crate::source::source_file_path(source_file));
            if !is_checked_source_path(&path, &workspace_root) {
                continue;
            }

            let Some(lines) = line_count(source_file) else {
                continue;
            };
            let max = self.max_lines;
            if lines <= max {
                continue;
            }

            emit_tiger_style_lint(
                cx,
                EXCESSIVE_FILE_LENGTH,
                Span::new(source_file.start_pos, source_file.end_position(), SyntaxContext::root(), None),
                format!("source file is {lines} lines (limit: {max})"),
                "excessive_file_length",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_checked_source_path;
    use super::is_generated_or_fixture_path;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn checks_existing_workspace_source_files() {
        assert!(is_checked_source_path(Path::new("src/lib.rs"), workspace_root()));
        assert!(is_checked_source_path(&workspace_root().join("src/lib.rs"), workspace_root()));
    }

    // A custom `CARGO_TARGET_DIR` has no `target` component, so only the
    // workspace-root check keeps `include!`d build-script output out.
    #[test]
    fn skips_build_script_output_under_a_custom_target_dir() {
        assert!(!is_checked_source_path(
            Path::new("/home/user/.cargo-target/debug/build/cargo-octet-0123/out/embedded_source.rs"),
            workspace_root()
        ));
    }

    #[test]
    fn accepts_workspace_source_files() {
        assert!(!is_generated_or_fixture_path("crates/example/src/lib.rs"));
    }

    #[test]
    fn skips_vendored_dependency_source_files() {
        assert!(is_generated_or_fixture_path(
            "/nix/store/hash-vendor-cargo-deps/serde_core-1.0.228/src/crate_root.rs"
        ));
    }
}
