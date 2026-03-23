use rustc_ast::{BinOpKind, Expr, ExprKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects `if` conditions containing 3 or more `&&`/`||` operators
    /// at the same nesting level. Complex conditions should be decomposed
    /// into early returns, guard clauses, or named boolean bindings.
    pub COMPOUND_CONDITION,
    Allow,
    "if condition has 3+ boolean operators"
}

declare_lint_pass!(CompoundCondition => [COMPOUND_CONDITION]);

fn count_logical_ops(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs)
            if matches!(op.node, BinOpKind::And | BinOpKind::Or) =>
        {
            1 + count_logical_ops(lhs) + count_logical_ops(rhs)
        }
        _ => 0,
    }
}

impl EarlyLintPass for CompoundCondition {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::If(cond, _, _) = &expr.kind else {
            return;
        };

        // Don't fire on macro-expanded if expressions (e.g., from assert!)
        if expr.span.from_expansion() {
            return;
        }

        let ops = count_logical_ops(cond);
        if ops >= 3 {
            cx.opt_span_lint(
                COMPOUND_CONDITION,
                Some(cond.span),
                DiagDecorator(|diag| {
                    diag.primary_message(format!(
                        "condition has {ops} boolean operators",
                    ));
                    diag.help(
                        "decompose into early returns, guard clauses, \
                         or named boolean bindings",
                    );
                }),
            );
        }
    }
}
