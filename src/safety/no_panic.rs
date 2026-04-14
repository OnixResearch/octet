use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::DiagDecorator;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

declare_lint! {
    /// Detects `panic!()`, `todo!()`, `unimplemented!()`, and
    /// `unreachable!()` in non-test code. These abort the process
    /// instead of returning an error. Production code should return
    /// `Result` and let the caller decide how to handle failure.
    pub NO_PANIC,
    Warn,
    "panic macro in production code"
}

declare_lint_pass!(NoPanic => [NO_PANIC]);

const PANIC_MACROS: &[&str] = &["panic!", "todo!", "unimplemented!", "unreachable!"];

/// Heuristic test detection for EarlyLintPass (no HIR parent chain).
/// Checks source context around the span for test indicators.
fn is_in_test_context(cx: &EarlyContext<'_>, expr: &Expr) -> bool {
    let source_map = cx.sess().source_map();

    let filename = source_map.span_to_filename(expr.span);
    let display = filename.prefer_local_unconditionally();
    let path = format!("{display}");
    if path.contains("/tests/") || path.ends_with("_test.rs") {
        return true;
    }

    if let Ok(lines) = source_map.span_to_lines(expr.span) {
        let line_idx = lines.lines.first().map(|l| l.line_index).unwrap_or(0);
        let start = line_idx.saturating_sub(50);
        let mut in_test_mod = false;
        for i in start..line_idx {
            if let Some(line) = lines.file.get_line(i) {
                let trimmed = line.trim();
                if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[test]") {
                    in_test_mod = true;
                }
                if trimmed.starts_with("mod ") && !in_test_mod {
                    in_test_mod = false;
                }
            }
        }
        if in_test_mod {
            return true;
        }
    }

    false
}

fn panic_macro_name_from_callsite(snippet: &str) -> Option<&'static str> {
    let trimmed = snippet.trim();
    PANIC_MACROS.iter().copied().find(|name| trimmed.starts_with(name))
}

fn suggestion_for(name: &str) -> &'static str {
    match name {
        "todo!" => "return an error or use a tracking issue instead of `todo!()`",
        "unimplemented!" => "return `Err(...)` explaining what's missing",
        "unreachable!" => "if truly unreachable, use `debug_assert!` or prove it with types",
        _ => "return `Err(...)` instead of panicking",
    }
}

impl EarlyLintPass for NoPanic {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Panic macros are expanded before EarlyLintPass runs. We recover the
        // original macro invocation from the expansion callsite snippet.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded block expressions to avoid duplicate diagnostics on
        // nested nodes that share the same callsite.
        if !matches!(expr.kind, ExprKind::Block(..)) {
            return;
        }

        if is_in_test_context(cx, expr) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };

        let Some(name) = panic_macro_name_from_callsite(&snippet) else {
            return;
        };

        let suggestion = suggestion_for(name);

        cx.opt_span_lint(
            NO_PANIC,
            Some(callsite),
            DiagDecorator(|diag| {
                diag.primary_message(format!("`{name}` will abort the process on execution"));
                diag.help(suggestion);
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::panic_macro_name_from_callsite;

    #[test]
    fn finds_panic_macro_name() {
        assert_eq!(panic_macro_name_from_callsite("panic!(\"oops\")"), Some("panic!"));
        assert_eq!(panic_macro_name_from_callsite("todo!(\"later\")"), Some("todo!"));
        assert_eq!(panic_macro_name_from_callsite("unimplemented!(\"x\")"), Some("unimplemented!"));
        assert_eq!(panic_macro_name_from_callsite("unreachable!(\"x\")"), Some("unreachable!"));
        assert_eq!(panic_macro_name_from_callsite("assert!(x > 0)"), None);
    }
}
