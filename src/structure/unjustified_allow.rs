use rustc_ast::{Attribute, Item};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Detects `#[allow(clippy::...)]` attributes without an accompanying
    /// comment explaining why the lint is suppressed. Unjustified suppressions
    /// hide real bugs — verified-logic documents every `#[allow(...)]` with
    /// a rationale.
    pub UNJUSTIFIED_ALLOW,
    Allow,
    "`#[allow]` attribute without justification comment"
}

declare_lint_pass!(UnjustifiedAllow => [UNJUSTIFIED_ALLOW]);

fn is_allow_attr(attr: &Attribute) -> bool {
    attr.has_name(sym::allow)
}

fn has_comment_nearby(cx: &EarlyContext<'_>, attr: &Attribute) -> bool {
    let source_map = cx.sess().source_map();

    // Get the source line of the attribute and the line before it
    let Ok(snippet) = source_map.span_to_snippet(attr.span) else {
        return true; // assume justified if we can't read source
    };

    // Check if the attribute itself has an inline comment
    if snippet.contains("//") {
        return true;
    }

    // Check the line before the attribute for a comment.
    // Use span_to_lines to get the line number, then look at the previous line.
    let Ok(lines) = source_map.span_to_lines(attr.span) else {
        return true;
    };

    if let Some(first_line) = lines.lines.first() {
        let line_idx = first_line.line_index;
        if line_idx > 0 {
            if let Some(prev_line) = lines.file.get_line(line_idx - 1) {
                let prev = prev_line.trim();
                if prev.starts_with("//") {
                    return true;
                }
            }
        }
    }

    false
}

impl EarlyLintPass for UnjustifiedAllow {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Skip items in #[cfg(test)] blocks
        if item.attrs.iter().any(|a| a.has_name(sym::test)) {
            return;
        }

        for attr in &item.attrs {
            if is_allow_attr(attr) && !has_comment_nearby(cx, attr) {
                cx.opt_span_lint(
                    UNJUSTIFIED_ALLOW,
                    Some(attr.span),
                    DiagDecorator(|diag| {
                        diag.primary_message(
                            "`#[allow(...)]` without justification comment",
                        );
                        diag.help(
                            "add a comment on the preceding line or same line \
                             explaining why this lint is suppressed",
                        );
                    }),
                );
            }
        }
    }
}
