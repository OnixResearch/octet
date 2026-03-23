use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{self, Ty};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects `as u32`, `as u16`, `as u8`, `as i32`, `as i16`, `as i8`
    /// casts from a wider integer type without a prior bounds check.
    /// Unchecked narrowing casts silently truncate — use `TryFrom` or
    /// a checked conversion function.
    pub UNCHECKED_NARROWING,
    Warn,
    "narrowing integer cast without bounds check"
}

declare_lint_pass!(UncheckedNarrowing => [UNCHECKED_NARROWING]);

/// Returns the bit width of an integer type, or None for non-integer types.
fn int_width(ty: Ty<'_>) -> Option<u64> {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width().map(|w| w as u64),
        ty::Uint(uint_ty) => uint_ty.bit_width().map(|w| w as u64),
        _ => None,
    }
}

/// Returns a human-readable name for the type (e.g., "u64", "i32").
fn ty_name(ty: Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

impl<'tcx> LateLintPass<'tcx> for UncheckedNarrowing {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _cast_ty) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        let Some(source_width) = int_width(source_ty) else {
            return;
        };
        let Some(target_width) = int_width(target_ty) else {
            return;
        };

        // Only fire on narrowing (source wider than target)
        if source_width <= target_width {
            return;
        }

        let source_name = ty_name(source_ty);
        let target_name = ty_name(target_ty);

        cx.opt_span_lint(
            UNCHECKED_NARROWING,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "narrowing cast from `{source_name}` to `{target_name}` without bounds check",
                ));
                diag.help(format!(
                    "use `{target_name}::try_from(value)` or a checked narrowing function",
                ));
            }),
        );
    }
}
