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

        // Skip if inside a #[cfg(test)] or #[test] context.
        // EarlyLintPass can't walk parents easily, so check the span's
        // source file for test module patterns. This is a heuristic —
        // the Late pass (no_unwrap) does a more precise check.
        let source_map = cx.sess().source_map();
        if let Ok(lines) = source_map.span_to_lines(expr.span) {
            let filename = format!(
                "{}",
                lines.file.name.prefer_local_unconditionally()
            );
            // Skip files that are clearly test modules
            if filename.contains("/tests/") || filename.ends_with("_test.rs") {
                return;
            }
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
