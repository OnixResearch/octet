use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::DiagDecorator;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

declare_lint! {
    /// Detects `loop { }` blocks that appear to lack an iteration bound.
    /// Unbounded loops are a class of liveness bug — a retry loop without
    /// a max-attempts counter can stall a system indefinitely.
    pub UNBOUNDED_LOOP,
    Warn,
    "loop without visible iteration bound"
}

declare_lint_pass!(UnboundedLoop => [UNBOUNDED_LOOP]);

impl EarlyLintPass for UnboundedLoop {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Only check `loop { ... }` — `for` and `while` have inherent bounds
        let ExprKind::Loop(_body, _, _) = &expr.kind else {
            return;
        };

        // Heuristic: if the loop body source contains a counter pattern
        // or `select!` macro, it's probably bounded
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(expr.span) else {
            return;
        };

        // Skip event loops using tokio::select! or similar
        if snippet.contains("select!") {
            return;
        }

        // Skip loops with visible counter patterns
        if snippet.contains("MAX_RETRIES")
            || snippet.contains("max_retries")
            || snippet.contains("max_attempts")
            || snippet.contains("attempt")
            || snippet.contains("remaining")
        {
            return;
        }

        cx.opt_span_lint(
            UNBOUNDED_LOOP,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message("loop without visible iteration bound");
                diag.help(
                    "add a counter or max-iterations guard, or use \
                     `for _ in 0..MAX_RETRIES` instead",
                );
            }),
        );
    }
}
