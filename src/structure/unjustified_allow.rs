use rustc_ast::AttrId;
use rustc_ast::Attribute;
use rustc_ast::MetaItemInner;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `#[allow(...)]` and `#[expect(...)]` attributes that suppress
    /// Clippy or Octet lints without a justification. Governed lint paths are
    /// `clippy::*`, `tigerstyle::*` (including Octet groups such as
    /// `tigerstyle::safety`), and Octet's bare compatibility aliases such as
    /// `no_unwrap`. rustc's own lints stay out of scope, and `no_todo` allows
    /// belong to `unjustified_no_todo_allow`. Only attributes written in source
    /// (directly or through `cfg_attr`) are governed, not macro output.
    ///
    /// A non-doc `//` comment on the attribute's lines or on the line directly
    /// above it, or a non-empty `reason = "..."` argument inside the attribute,
    /// counts as the justification. A comment or reason that starts with the
    /// `TODO(octet)` placeholder does not. Unjustified suppressions hide real
    /// bugs — verified-logic documents every `#[allow(...)]` with a rationale.
    pub tigerstyle::UNJUSTIFIED_ALLOW,
    Allow,
    "`#[allow]` attribute without justification comment or reason"
}

/// Early pass state for [`UNJUSTIFIED_ALLOW`].
#[derive(Default)]
pub struct UnjustifiedAllow {
    /// rustc reports a statement's attributes once for the statement and again
    /// for the `let`, item, or expression it wraps; this remembers the first
    /// attribute of the last checked list so each list is checked once.
    last_checked: Option<AttrId>,
}

impl_lint_pass!(UnjustifiedAllow => [UNJUSTIFIED_ALLOW]);

/// Placeholder this lint's own suggestion inserts; it never justifies.
const PLACEHOLDER_PREFIX: &str = "TODO(octet)";
/// Lint tools whose suppressions this lint governs.
const GOVERNED_TOOLS: &[&str] = &["clippy", octet_lint_metadata::CANONICAL_NAMESPACE];
/// Octet lint whose allows `unjustified_no_todo_allow` audits instead.
const DEDICATED_AUDIT_LINT: &str = "no_todo";

/// Whether a lint path inside `allow(...)` / `expect(...)` names a Clippy
/// lint, an Octet lint or group, or an Octet bare compatibility alias.
fn is_governed_lint_path<'a>(mut segments: impl Iterator<Item = &'a str>) -> bool {
    match (segments.next(), segments.next()) {
        (Some(tool), Some(lint)) => {
            GOVERNED_TOOLS.contains(&tool)
                && !(tool == octet_lint_metadata::CANONICAL_NAMESPACE && lint == DEDICATED_AUDIT_LINT)
        }
        (Some(name), None) => {
            name != DEDICATED_AUDIT_LINT && crate::LINT_INVENTORY.iter().any(|entry| entry.name == name)
        }
        _ => false,
    }
}

/// Whether comment or reason text explains the suppression: non-empty and not
/// the unedited placeholder.
fn is_justification_text(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && !text.starts_with(PLACEHOLDER_PREFIX)
}

/// Whether `comment`, which starts at `//`, is a justifying non-doc line
/// comment. `///` and `//!` doc comments document items, not suppressions.
fn is_justifying_line_comment(comment: &str) -> bool {
    comment
        .trim_start()
        .strip_prefix("//")
        .is_some_and(|body| !body.starts_with('/') && !body.starts_with('!') && is_justification_text(body))
}

/// Whether a source line holds a justifying `//` comment outside string
/// literals, wherever it starts on the line.
fn line_has_justifying_comment(line: &str) -> bool {
    let mut chars = line.char_indices().peekable();
    let mut is_in_string = false;
    while let Some((offset, character)) = chars.next() {
        if is_in_string {
            match character {
                '\\' => {
                    chars.next();
                }
                '"' => is_in_string = false,
                _ => {}
            }
        } else if character == '"' {
            is_in_string = true;
        } else if character == '/' && chars.peek().is_some_and(|&(_, next)| next == '/') {
            return is_justifying_line_comment(&line[offset..]);
        }
    }
    false
}

fn suppression_name(attr: &Attribute) -> Option<&'static str> {
    if attr.has_name(sym::allow) {
        Some("allow")
    } else if attr.has_name(sym::expect) {
        Some("expect")
    } else {
        None
    }
}

fn suppresses_governed_lint(items: &[MetaItemInner]) -> bool {
    items
        .iter()
        .filter_map(MetaItemInner::meta_item)
        .any(|meta| is_governed_lint_path(meta.path.segments.iter().map(|segment| segment.ident.as_str())))
}

fn has_reason_argument(items: &[MetaItemInner]) -> bool {
    items.iter().any(|item| {
        item.meta_item()
            .filter(|meta| meta.has_name(sym::reason))
            .and_then(rustc_ast::MetaItem::value_str)
            .is_some_and(|reason| is_justification_text(reason.as_str()))
    })
}

/// Whether `snippet`, the source text at an attribute's span, spells that
/// attribute: `#[name(`, `#![name(`, or `name(` for one expanded from
/// `cfg_attr(..., name(...))`. Proc macros such as `#[tokio::main]` re-span
/// generated attributes onto user tokens, whose text spells something else.
fn snippet_spells_attribute(snippet: &str, name: &str) -> bool {
    let body = match snippet.strip_prefix('#') {
        Some(rest) => {
            let rest = rest.trim_start();
            let rest = rest.strip_prefix('!').unwrap_or(rest).trim_start();
            let Some(rest) = rest.strip_prefix('[') else {
                return false;
            };
            rest.trim_start()
        }
        None => snippet,
    };
    body.strip_prefix(name).is_some_and(|rest| rest.trim_start().starts_with('('))
}

/// Whether the user wrote this attribute in source, rather than a macro
/// generating it.
fn is_user_written(cx: &EarlyContext<'_>, attr: &Attribute, name: &str) -> bool {
    !attr.span.from_expansion()
        && cx
            .sess()
            .source_map()
            .span_to_snippet(attr.span)
            .is_ok_and(|snippet| snippet_spells_attribute(&snippet, name))
}

fn justification_stub(cx: &EarlyContext<'_>, attr: &Attribute) -> String {
    let source_map = cx.sess().source_map();
    let indent_width = source_map.lookup_char_pos(attr.span.lo()).col.0;
    let indent = " ".repeat(indent_width);
    format!("// {PLACEHOLDER_PREFIX}: explain why this allow is safe here\n{indent}")
}

/// Whether a justifying comment sits on the attribute's own source lines or on
/// the line directly above them.
fn has_comment_nearby(cx: &EarlyContext<'_>, attr: &Attribute) -> bool {
    let Ok(lines) = cx.sess().source_map().span_to_lines(attr.span) else {
        return true; // assume justified if we can't read source
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return true;
    };

    let has_attribute_line_comment = lines
        .lines
        .iter()
        .filter_map(|line| lines.file.get_line(line.line_index))
        .any(|line| line_has_justifying_comment(&line));
    let has_previous_line_comment = first_line_idx
        .checked_sub(1)
        .and_then(|line_idx| lines.file.get_line(line_idx))
        .is_some_and(|line| is_justifying_line_comment(&line));

    has_attribute_line_comment || has_previous_line_comment
}

fn check_attrs(cx: &EarlyContext<'_>, attrs: &[Attribute]) {
    // Skip items in #[cfg(test)] blocks
    if attrs.iter().any(|a| a.has_name(sym::test)) {
        return;
    }

    for attr in attrs {
        let Some(name) = suppression_name(attr) else {
            continue;
        };
        let Some(items) = attr.meta_item_list() else {
            continue;
        };
        let is_user_governed = suppresses_governed_lint(&items) && is_user_written(cx, attr, name);
        if is_user_governed && !has_reason_argument(&items) && !has_comment_nearby(cx, attr) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                UNJUSTIFIED_ALLOW,
                attr.span,
                format!("`#[{name}(...)]` without justification comment"),
                "unjustified_allow",
                LintSuggestion::Single {
                    span: attr.span.shrink_to_lo(),
                    message: "add a local justification comment".to_owned(),
                    replacement: justification_stub(cx, attr),
                    applicability: Applicability::HasPlaceholders,
                },
            );
        }
    }
}

impl EarlyLintPass for UnjustifiedAllow {
    // rustc calls this for the attributes of every lint-level node: the crate
    // root, items, associated and foreign items, statements, fields,
    // variants, expressions, arms, parameters, and generic parameters.
    fn check_attributes(&mut self, cx: &EarlyContext<'_>, attrs: &[Attribute]) {
        let Some(first) = attrs.first() else {
            return;
        };
        if self.last_checked == Some(first.id) {
            return;
        }
        self.last_checked = Some(first.id);
        check_attrs(cx, attrs);
    }
}

#[cfg(test)]
mod tests {
    use super::is_governed_lint_path;
    use super::is_justifying_line_comment;
    use super::line_has_justifying_comment;
    use super::snippet_spells_attribute;

    fn governed(path: &str) -> bool {
        is_governed_lint_path(path.split("::"))
    }

    #[test]
    fn governs_clippy_octet_lints_groups_and_bare_aliases() {
        assert!(governed("clippy::needless_return"));
        assert!(governed("clippy::all"));
        assert!(governed("tigerstyle::no_unwrap"));
        assert!(governed("tigerstyle::zero_alloc_attr_contract"));
        assert!(governed("tigerstyle::safety"));
        assert!(governed("no_unwrap"));
        assert!(governed("zero_alloc_attr_contract"));
    }

    #[test]
    fn leaves_rustc_lints_and_no_todo_allows_alone() {
        assert!(!governed("dead_code"));
        assert!(!governed("rustdoc::broken_intra_doc_links"));
        assert!(!governed("reason"));
        assert!(!governed("no_todo"));
        assert!(!governed("tigerstyle::no_todo"));
    }

    #[test]
    fn previous_line_justification_excludes_doc_comments_and_placeholders() {
        assert!(is_justifying_line_comment("    // Verified casts: Verus checks every narrowing."));
        assert!(!is_justifying_line_comment("/// Returns a constant."));
        assert!(!is_justifying_line_comment("//! Crate docs."));
        assert!(!is_justifying_line_comment("//"));
        assert!(!is_justifying_line_comment("// TODO(octet): explain why this allow is safe here"));
        assert!(!is_justifying_line_comment("let value = 1; // trailing"));
    }

    #[test]
    fn attribute_line_justification_reads_the_whole_line() {
        assert!(line_has_justifying_comment("#[allow(clippy::needless_return)] // Mirrors the state table."));
        assert!(line_has_justifying_comment("    clippy::needless_return, // Mirrors the state table."));
        assert!(!line_has_justifying_comment("#[allow(clippy::needless_return)]"));
        assert!(!line_has_justifying_comment(
            "#[allow(clippy::needless_return)] // TODO(octet): explain why this allow is safe here"
        ));
        assert!(!line_has_justifying_comment("#[allow(clippy::x, reason = \"see https://example.com\")]"));
        assert!(!line_has_justifying_comment("#[allow(clippy::x)] /// Doc for the item."));
    }

    #[test]
    fn only_source_text_that_spells_the_attribute_is_user_written() {
        assert!(snippet_spells_attribute("#[allow(clippy::needless_return)]", "allow"));
        assert!(snippet_spells_attribute("#![allow(clippy::all)]", "allow"));
        assert!(snippet_spells_attribute("# [ expect (clippy::x)]", "expect"));
        assert!(snippet_spells_attribute("allow(clippy::needless_return)", "allow"));
        assert!(!snippet_spells_attribute(";", "allow"));
        assert!(!snippet_spells_attribute("run().await", "allow"));
        assert!(!snippet_spells_attribute("{ #[allow(clippy::x)] let value = 1; }", "allow"));
        assert!(!snippet_spells_attribute("#[expect(clippy::x)]", "allow"));
        assert!(!snippet_spells_attribute("allowed(value)", "allow"));
    }
}
