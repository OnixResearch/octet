use rustc_hir as hir;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::find_attr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects explicit `drop(...)` calls that consume `#[must_use]` values.
    /// Calling `drop` on a value whose type or producer is marked `#[must_use]`
    /// acknowledges the value only syntactically while still discarding the
    /// success/error or obligation it represents.
    pub tigerstyle::DROP_MUST_USE,
    Warn,
    "explicitly dropping a must-use value hides the required handling"
}

declare_lint_pass!(DropMustUse => [DROP_MUST_USE]);

#[allow(deprecated)]
fn has_must_use_attr(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> bool {
    find_attr!(cx.tcx.get_all_attrs(def_id), AttributeKind::MustUse { .. })
}

fn type_is_must_use(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty = ty.peel_refs();
    let ty::Adt(adt, _) = ty.kind() else {
        return false;
    };

    cx.tcx.is_diagnostic_item(sym::Result, adt.did())
        || cx.tcx.is_diagnostic_item(sym::Option, adt.did())
        || has_must_use_attr(cx, adt.did())
}

fn expr_is_must_use_call(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        hir::ExprKind::Call(callee, _) => {
            let hir::ExprKind::Path(qpath) = &callee.kind else {
                return false;
            };
            cx.qpath_res(qpath, callee.hir_id).opt_def_id().is_some_and(|def_id| has_must_use_attr(cx, def_id))
        }
        hir::ExprKind::MethodCall(_, _, _, _) => cx
            .typeck_results()
            .type_dependent_def_id(expr.hir_id)
            .is_some_and(|def_id| has_must_use_attr(cx, def_id)),
        _ => false,
    }
}

fn is_std_mem_drop(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Path(qpath) = &callee.kind else {
        return false;
    };

    cx.qpath_res(qpath, callee.hir_id)
        .opt_def_id()
        .and_then(|def_id| cx.tcx.get_diagnostic_name(def_id))
        .is_some_and(|name| name == sym::mem_drop)
}

impl<'tcx> LateLintPass<'tcx> for DropMustUse {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let hir::ExprKind::Call(callee, [arg]) = &expr.kind else {
            return;
        };

        if !is_std_mem_drop(cx, callee) {
            return;
        }

        if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let arg_ty = cx.typeck_results().expr_ty(arg);
        if !type_is_must_use(cx, arg_ty) && !expr_is_must_use_call(cx, arg) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            DROP_MUST_USE,
            expr.span,
            "explicitly dropping a must-use value hides the required handling",
            "drop_must_use",
        );
    }
}
