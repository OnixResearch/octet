use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_lint! {
    /// Detects `as usize` and `as isize` casts from fixed-width integer
    /// types. On a 32-bit platform, `u64 as usize` silently truncates.
    /// Tiger Style mandates explicitly sized types for cross-platform
    /// consistency.
    pub PLATFORM_DEPENDENT_CAST,
    Warn,
    "`as usize`/`as isize` from fixed-width integer is platform-dependent"
}

declare_lint_pass!(PlatformDependentCast => [PLATFORM_DEPENDENT_CAST]);

fn is_fixed_width_int(ty: ty::Ty<'_>) -> bool {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width().is_some(),
        ty::Uint(uint_ty) => uint_ty.bit_width().is_some(),
        _ => false,
    }
}

fn is_platform_sized(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(rustc_middle::ty::IntTy::Isize) | ty::Uint(rustc_middle::ty::UintTy::Usize))
}

fn ty_name(ty: ty::Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

impl<'tcx> LateLintPass<'tcx> for PlatformDependentCast {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        // Fire when casting from fixed-width to platform-sized
        if is_fixed_width_int(source_ty) && is_platform_sized(target_ty) {
            let source_name = ty_name(source_ty);
            let target_name = ty_name(target_ty);

            emit_tiger_style_lint(
                cx,
                PLATFORM_DEPENDENT_CAST,
                expr.span,
                format!("`as {target_name}` from `{source_name}` is platform-dependent"),
                "platform_dependent_cast",
            );
        }
    }
}
