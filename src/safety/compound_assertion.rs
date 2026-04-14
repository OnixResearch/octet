use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::DiagDecorator;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

declare_lint! {
    /// Detects `assert!(a && b)` and `assert!(a || b)` where the macro
    /// argument contains a top-level `&&` or `||`. Compound assertions
    /// produce a single "assertion failed" message on failure, hiding
    /// which condition actually broke.
    pub COMPOUND_ASSERTION,
    Warn,
    "compound assertion should be split into separate assertions"
}

declare_lint_pass!(CompoundAssertion => [COMPOUND_ASSERTION]);

/// Scan a macro-call snippet for top-level `&&` or `||` inside `assert!(...)`.
///
/// We work from the callsite source text because Dylint does not reliably run
/// pre-expansion passes. The expanded AST still carries the original macro
/// callsite span, which lets us re-read the user's source.
fn has_top_level_compound_assertion(snippet: &str) -> bool {
    let Some(rest) = snippet.strip_prefix("assert!(") else {
        return false;
    };

    let mut depth = 1usize;
    let mut chars = rest.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            '&' if depth == 1 && chars.peek() == Some(&'&') => return true,
            '|' if depth == 1 && chars.peek() == Some(&'|') => return true,
            _ => {}
        }
    }

    false
}

impl EarlyLintPass for CompoundAssertion {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Only inspect expressions originating from macro expansion.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded `if` nodes from `assert!` to avoid duplicate
        // diagnostics on children that share the same callsite.
        if !matches!(expr.kind, ExprKind::If(..)) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };
        let trimmed = snippet.trim();

        if !trimmed.starts_with("assert!(") {
            return;
        }

        if !has_top_level_compound_assertion(trimmed) {
            return;
        }

        cx.opt_span_lint(
            COMPOUND_ASSERTION,
            Some(callsite),
            DiagDecorator(|diag| {
                diag.primary_message("compound assertion should be split into separate assertions");
                diag.help(
                    "split into separate `assert!()` calls so each condition \
                     reports independently on failure",
                );
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::has_top_level_compound_assertion;

    #[test]
    fn detects_top_level_compound_assertion() {
        assert!(has_top_level_compound_assertion("assert!(a && b)"));
        assert!(has_top_level_compound_assertion("assert!(a || b)"));
        assert!(!has_top_level_compound_assertion("assert!(a)"));
        assert!(!has_top_level_compound_assertion("assert!(items.iter().all(|x| x > 0 && x < 10))"));
        assert!(!has_top_level_compound_assertion("debug_assert!(a && b)"));
    }
}
