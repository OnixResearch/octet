use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::{BinOpKind, ExprKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty;
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects `a / b` or `a % b` where `b` is a function parameter or
    /// local variable (not a constant) and there is no prior zero-guard
    /// in the function. Every division in verified-logic has an explicit
    /// `requires b > 0` precondition.
    pub UNCHECKED_DIVISION,
    Warn,
    "division by variable without zero check"
}

declare_lint_pass!(UncheckedDivision => [UNCHECKED_DIVISION]);

fn is_integer_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

fn is_literal_or_const(expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Lit(_) => true,
        ExprKind::Path(hir::QPath::Resolved(_, path)) => {
            // Constants are resolved paths with Def::Const or Def::AssocConst
            matches!(
                path.res,
                hir::def::Res::Def(hir::def::DefKind::Const { .. } | hir::def::DefKind::AssocConst { .. }, _)
            )
        }
        _ => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for UncheckedDivision {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Binary(op, _lhs, rhs) = &expr.kind else {
            return;
        };

        // Only check / and %
        if !matches!(op.node, BinOpKind::Div | BinOpKind::Rem) {
            return;
        }

        // Only integer division (float division doesn't panic on zero)
        let rhs_ty = cx.typeck_results().expr_ty(rhs);
        if !is_integer_ty(rhs_ty) {
            return;
        }

        // Skip if divisor is a literal or named constant
        if is_literal_or_const(rhs) {
            return;
        }

        // Get the divisor name for the diagnostic
        let divisor_name = match &rhs.kind {
            ExprKind::Path(hir::QPath::Resolved(_, path)) => {
                path.segments.last().map(|s| s.ident.name.as_str().to_string())
            }
            _ => None,
        };

        let name = divisor_name.unwrap_or_else(|| "expression".into());

        cx.opt_span_lint(
            UNCHECKED_DIVISION,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "division by `{name}` without zero check",
                ));
                diag.help(
                    "add `assert!(divisor > 0)` or an early return for the zero case",
                );
            }),
        );
    }
}
