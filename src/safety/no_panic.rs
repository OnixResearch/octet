use rustc_ast::{Expr, ExprKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

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

const PANIC_MACROS: &[&str] = &[
    "panic",
    "todo",
    "unimplemented",
    "unreachable",
];

/// Heuristic test detection for EarlyLintPass (no HIR parent chain).
/// Checks source context around the span for test indicators.
fn is_in_test_context(cx: &EarlyContext<'_>, expr: &Expr) -> bool {
    let source_map = cx.sess().source_map();

    // Check filename
    let filename = source_map.span_to_filename(expr.span);
    let display = filename.prefer_local_unconditionally();
    let path = format!("{display}");
    if path.contains("/tests/") || path.ends_with("_test.rs") {
        return true;
    }

    // Check if we're inside a #[cfg(test)] module by looking at
    // surrounding source lines. Walk backwards from the span to find
    // `#[cfg(test)]` or `#[test]`.
    if let Ok(lines) = source_map.span_to_lines(expr.span) {
        let line_idx = lines.lines.first().map(|l| l.line_index).unwrap_or(0);
        // Scan up to 50 lines back for test context
        let start = line_idx.saturating_sub(50);
        let mut in_test_mod = false;
        for i in start..line_idx {
            if let Some(line) = lines.file.get_line(i) {
                let trimmed = line.trim();
                if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[test]") {
                    in_test_mod = true;
                }
                // If we hit a non-test module boundary, reset
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

impl EarlyLintPass for NoPanic {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::MacCall(mac) = &expr.kind else {
            return;
        };

        let Some(last) = mac.path.segments.last() else {
            return;
        };

        let name = last.ident.name.as_str();
        if !PANIC_MACROS.contains(&name) {
            return;
        }

        if is_in_test_context(cx, expr) {
            return;
        }

        let suggestion = match name {
            "todo" => "return an error or use a tracking issue instead of `todo!()`",
            "unimplemented" => "return `Err(...)` explaining what's missing",
            "unreachable" => "if truly unreachable, use `debug_assert!` or prove it with types",
            _ => "return `Err(...)` instead of panicking",
        };

        cx.opt_span_lint(
            NO_PANIC,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "`{name}!()` will abort the process on execution",
                ));
                diag.help(suggestion);
            }),
        );
    }
}
