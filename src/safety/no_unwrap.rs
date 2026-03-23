use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty;
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Detects `.unwrap()` and `.expect()` calls on `Option` and `Result`
    /// in non-test code. These panic on failure, crashing the process
    /// instead of propagating the error.
    pub NO_UNWRAP,
    Warn,
    "`.unwrap()`/`.expect()` in production code"
}

declare_lint_pass!(NoUnwrap => [NO_UNWRAP]);

fn is_option_or_result(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    if let ty::Adt(adt, _) = ty.kind() {
        let did = adt.did();
        cx.tcx.is_diagnostic_item(sym::Option, did)
            || cx.tcx.is_diagnostic_item(sym::Result, did)
    } else {
        false
    }
}

fn is_in_test_context(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        if let hir::Node::Item(item) = node {
            let attrs = cx.tcx.hir_attrs(item.hir_id());
            // Check #[test]
            if attrs.iter().any(|a| a.has_name(sym::test)) {
                return true;
            }
            // Check #[cfg(test)] on mod
            if let hir::ItemKind::Mod(..) = &item.kind {
                for attr in attrs {
                    if attr.has_name(sym::cfg) {
                        let source = cx.sess().source_map()
                            .span_to_snippet(attr.span())
                            .unwrap_or_default();
                        if source.contains("test") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for NoUnwrap {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::MethodCall(segment, receiver, _args, _span) = &expr.kind else {
            return;
        };

        let name = segment.ident.name.as_str();
        if name != "unwrap" && name != "expect" {
            return;
        }

        // Only fire when the receiver is Option or Result
        let receiver_ty = cx.typeck_results().expr_ty(receiver);
        if !is_option_or_result(cx, receiver_ty) {
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
