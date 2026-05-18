use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

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

const GENERATED_COMPONENTS: &[&str] = &["target", "generated", "OUT_DIR"];

declare_tool_lint! {
    /// Detects module directories with too many Rust source files
    /// (default: 7).
    pub tigerstyle::MODULE_FILE_COUNT,
    Warn,
    "module directory contains too many Rust source files"
}

pub struct ModuleFileCount {
    max_files: usize,
}

impl ModuleFileCount {
    pub fn new(max_files: usize) -> Self {
        Self { max_files }
    }
}

impl_lint_pass!(ModuleFileCount => [MODULE_FILE_COUNT]);

#[derive(Clone)]
struct ModuleFile {
    path: PathBuf,
    span: Span,
}

fn source_file_path(source_file: &SourceFile) -> PathBuf {
    PathBuf::from(source_file.name.prefer_local_unconditionally().to_string_lossy().into_owned())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn has_generated_component(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        GENERATED_COMPONENTS.iter().any(|generated| value == *generated)
    })
}

fn is_countable_rust_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "rs") && !has_generated_component(path)
}

fn directory_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl EarlyLintPass for ModuleFileCount {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _krate: &Crate) {
        let mut by_directory: BTreeMap<PathBuf, Vec<ModuleFile>> = BTreeMap::new();

        for source_file in cx.sess().source_map().files().iter() {
            let path = normalize_path(&source_file_path(source_file));
            if !is_countable_rust_source(&path) {
                continue;
            }
            let Some(parent) = path.parent() else {
                continue;
            };

            by_directory.entry(parent.to_path_buf()).or_default().push(ModuleFile {
                path,
                span: Span::new(source_file.start_pos, source_file.end_position(), SyntaxContext::root(), None),
            });
        }

        for (directory, mut files) in by_directory {
            let count = files.len();
            let max = self.max_files;
            if count <= max {
                continue;
            }

            files.sort_by(|left, right| left.path.cmp(&right.path));
            emit_tiger_style_lint(
                cx,
                MODULE_FILE_COUNT,
                files[0].span,
                format!(
                    "module directory `{}` contains {count} Rust source files (limit: {max})",
                    directory_display(&directory)
                ),
                "module_file_count",
            );
        }
    }
}
