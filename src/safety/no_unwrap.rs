use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Detects `.unwrap()` and `.expect()` calls in non-test code. These
    /// panic on failure, crashing the process instead of propagating the
    /// error. A study of 198 catastrophic failures in distributed systems
    /// found 92% had an error path that was explicitly ignored or
    /// inadequately handled.
    pub NO_UNWRAP,
    Warn,
    "`.unwrap()`/`.expect()` in production code"
}

declare_lint_pass!(NoUnwrap => [NO_UNWRAP]);

fn is_in_test_context(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    // Walk up the HIR to check if we're inside a #[test] function
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        if let hir::Node::Item(item) = node {
            let attrs = cx.tcx.hir_attrs(item.hir_id());
            if attrs.iter().any(|a| a.has_name(sym::test)) {
                return true;
            }
            // Check for mod tests / #[cfg(test)]
            if let hir::ItemKind::Mod(..) = &item.kind {
                if attrs.iter().any(|a| {
                    let s = format!("{a:?}");
                    s.contains("test")
                }) {
                    return true;
                }
            }
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for NoUnwrap {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::MethodCall(segment, _receiver, _args, _span) = &expr.kind else {
            return;
        };

        let name = segment.ident.name.as_str();
        if name != "unwrap" && name != "expect" {
            return;
        }

        if is_in_test_context(cx, expr) {
            return;
        }

        let suggestion = if name == "unwrap" {
            "use `?` to propagate, or match/map the error explicitly"
        } else {
            "use `?` with `.context()` (anyhow) or `.whatever_context()` (snafu) instead"
        };

        cx.opt_span_lint(
            NO_UNWRAP,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "`.{name}()` in production code will panic on failure",
                ));
                diag.help(suggestion);
            }),
        );
    }
}
