use std::collections::BTreeSet;

use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Ident;

use crate::config::IndexableLintExceptions;
use crate::diagnostics::emit_tiger_style_lint;
use crate::naming::indexable_exceptions::is_generated_source_span;
use crate::naming::indexable_exceptions::is_lint_identity_name;

const COMPATIBILITY_WORDS: &[&str] = &["compat", "compatibility", "external", "mandated", "api"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Repetition {
    repeated_word: String,
    path_segment: String,
}

declare_tool_lint! {
    /// Detects item names that repeat words already supplied by the crate or
    /// ancestor module path. Prefer `guest::spawn_vm` over
    /// `guest::spawn_guest_vm`.
    pub tigerstyle::PATH_SEGMENT_REPETITION,
    Warn,
    "item name repeats an ancestor path segment"
}

pub struct PathSegmentRepetition {
    exceptions: IndexableLintExceptions,
    workspace_root: std::path::PathBuf,
}

impl PathSegmentRepetition {
    pub fn new(exceptions: IndexableLintExceptions) -> Self {
        Self {
            exceptions,
            workspace_root: crate::source::workspace_root(),
        }
    }
}

impl_lint_pass!(PathSegmentRepetition => [PATH_SEGMENT_REPETITION]);

fn identifier_for_item(item: &hir::Item<'_>) -> Option<Ident> {
    match item.kind {
        hir::ItemKind::Fn { ident, .. }
        | hir::ItemKind::Struct(ident, ..)
        | hir::ItemKind::Union(ident, ..)
        | hir::ItemKind::Enum(ident, ..)
        | hir::ItemKind::TyAlias(ident, ..)
        | hir::ItemKind::Const(ident, ..) => Some(ident),
        _ => None,
    }
}

fn normalized_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' || ch == '-' || ch == ':' || ch == '<' || ch == '>' {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
        } else if ch.is_ascii_uppercase() {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
    }

    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }

    words.into_iter().filter(|word| word.len() > 1).collect()
}

fn path_word_set(path_segments: &[&str]) -> BTreeSet<String> {
    path_segments.iter().flat_map(|segment| normalized_words(segment)).collect()
}

fn repeated_path_word(item_name: &str, path_segments: &[&str]) -> Option<Repetition> {
    let path_words = path_word_set(path_segments);
    normalized_words(item_name).into_iter().find_map(|item_word| {
        if !path_words.contains(&item_word) {
            return None;
        }

        let path_segment = path_segments
            .iter()
            .copied()
            .find(|segment| normalized_words(segment).contains(&item_word))
            .unwrap_or("<path>")
            .to_owned();

        Some(Repetition {
            repeated_word: item_word,
            path_segment,
        })
    })
}

/// The crate name followed by the ancestor module path of `item`.
/// `def_path_str` omits the local crate name, so it is prepended here.
fn item_path_segments(cx: &LateContext<'_>, item: &hir::Item<'_>) -> Vec<String> {
    let path = cx.tcx.def_path_str(item.owner_id.def_id.to_def_id());
    let mut segments = std::iter::once(cx.tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE).as_str().to_owned())
        .chain(path.split("::").map(str::to_owned))
        .collect::<Vec<_>>();
    segments.pop();
    segments
}

fn has_compatibility_documentation(cx: &LateContext<'_>, item: &hir::Item<'_>) -> bool {
    let source_map = cx.sess().source_map();

    let Ok(lines) = source_map.span_to_lines(item.span) else {
        return false;
    };

    lines.lines.first().is_some_and(|first_line| {
        let line_idx = first_line.line_index;
        line_idx > 0
            && lines.file.get_line(line_idx - 1).is_some_and(|line| {
                let lower = line.trim().to_ascii_lowercase();
                (lower.starts_with("//") || lower.starts_with("///"))
                    && COMPATIBILITY_WORDS.iter().any(|word| lower.contains(word))
            })
    })
}

fn is_unrelated_fixture(cx: &LateContext<'_>, item: &hir::Item<'_>) -> bool {
    let filename = cx.sess().source_map().span_to_filename(item.span);
    let path = filename.prefer_local_unconditionally().to_string();
    path.contains("tests/fixtures") || (path.contains("ui_tests/") && !path.contains("path_segment_repetition"))
}

impl<'tcx> LateLintPass<'tcx> for PathSegmentRepetition {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if item.span.from_expansion() || is_unrelated_fixture(cx, item) {
            return;
        }

        let Some(ident) = identifier_for_item(item) else {
            return;
        };

        if is_generated_source_span(cx.sess().source_map(), item.span, &self.workspace_root, self.exceptions) {
            return;
        }

        if has_compatibility_documentation(cx, item) {
            return;
        }

        let segments = item_path_segments(cx, item);
        if is_lint_identity_name(ident.name.as_str(), &segments, self.exceptions) {
            return;
        }
        let segment_refs = segments.iter().map(String::as_str).collect::<Vec<_>>();
        let Some(repetition) = repeated_path_word(ident.name.as_str(), &segment_refs) else {
            return;
        };

        emit_tiger_style_lint(
            cx,
            PATH_SEGMENT_REPETITION,
            item.span,
            format!(
                "item name `{}` repeats `{}` from ancestor path segment `{}`",
                ident.name.as_str(),
                repetition.repeated_word,
                repetition.path_segment
            ),
            "path_segment_repetition",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::repeated_path_word;

    #[test]
    fn crate_name_hyphens_separate_words() {
        let repetition = repeated_path_word("vm_config", &["octet-vm", "guest"]).expect("`vm` repeats the crate name");
        assert_eq!(repetition.repeated_word, "vm");
        assert_eq!(repetition.path_segment, "octet-vm");
        assert!(repeated_path_word("octetvm_config", &["octet-vm"]).is_none());
    }
}
