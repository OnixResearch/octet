use rustc_hir as hir;
use rustc_hir::BinOpKind;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_lint! {
    /// Detects unprotected `+`, `-`, `*` on unsigned integer types in
    /// production code where overflow is not checked. Verified-logic uses
    /// `checked_add`, `saturating_sub`, etc. for every arithmetic operation.
    pub RAW_ARITHMETIC_OVERFLOW,
    Allow,
    "unprotected arithmetic on unsigned integers"
}

declare_lint_pass!(RawArithmeticOverflow => [RAW_ARITHMETIC_OVERFLOW]);

fn is_unsigned_int(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Uint(_))
}

fn is_constant_expr(expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, ExprKind::Lit(_))
}

impl<'tcx> LateLintPass<'tcx> for RawArithmeticOverflow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Binary(op, lhs, rhs) = &expr.kind else {
            return;
        };

        if !matches!(op.node, BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul) {
            return;
        }

        let ty = cx.typeck_results().expr_ty(expr);
        if !is_unsigned_int(ty) {
            return;
        }

        // Skip if both operands are constants (compiler checks these)
        if is_constant_expr(lhs) && is_constant_expr(rhs) {
            return;
        }

        let op_name = match op.node {
            BinOpKind::Add => "addition",
            BinOpKind::Sub => "subtraction",
            BinOpKind::Mul => "multiplication",
            _ => "arithmetic",
        };

        emit_tiger_style_lint(
            cx,
            RAW_ARITHMETIC_OVERFLOW,
            expr.span,
            format!("unprotected unsigned {op_name} may overflow"),
            "raw_arithmetic_overflow",
        );
    }
}
