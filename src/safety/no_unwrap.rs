use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;
use rustc_span::Span;
use rustc_span::sym;

use crate::test_context;

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
        cx.tcx.is_diagnostic_item(sym::Option, did) || cx.tcx.is_diagnostic_item(sym::Result, did)
    } else {
        false
    }
}

fn is_known_constructor_call(receiver: &hir::Expr<'_>) -> bool {
    let ExprKind::Call(callee, [_]) = &receiver.kind else {
        return false;
    };

    let ExprKind::Path(hir::QPath::Resolved(_, path)) = &callee.kind else {
        return false;
    };

    matches!(path.segments.last().map(|segment| segment.ident.name.as_str().as_ref()), Some("Some") | Some("Ok"))
}

fn is_string_literal_parse(receiver: &hir::Expr<'_>) -> bool {
    let ExprKind::MethodCall(segment, target, args, _) = &receiver.kind else {
        return false;
    };

    segment.ident.name.as_str() == "parse"
        && args.is_empty()
        && matches!(target.kind, ExprKind::Lit(lit) if matches!(lit.node, rustc_ast::LitKind::Str(..)))
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.lo() <= inner.lo() && inner.hi() <= outer.hi()
}

fn same_receiver(lhs: &hir::Expr<'_>, rhs: &hir::Expr<'_>) -> bool {
    match (&lhs.kind, &rhs.kind) {
        (ExprKind::Path(hir::QPath::Resolved(_, lhs_path)), ExprKind::Path(hir::QPath::Resolved(_, rhs_path))) => {
            lhs_path.res == rhs_path.res
        }
        (ExprKind::Field(lhs_base, lhs_ident), ExprKind::Field(rhs_base, rhs_ident)) => {
            lhs_ident.name == rhs_ident.name && same_receiver(lhs_base, rhs_base)
        }
        _ => false,
    }
}

fn guard_method_name(cx: &LateContext<'_>, receiver: &hir::Expr<'_>) -> Option<&'static str> {
    let receiver_ty = cx.typeck_results().expr_ty(receiver);
    if let ty::Adt(adt, _) = receiver_ty.kind() {
        let did = adt.did();
        if cx.tcx.is_diagnostic_item(sym::Option, did) {
            return Some("is_some");
        }
        if cx.tcx.is_diagnostic_item(sym::Result, did) {
            return Some("is_ok");
        }
    }

    None
}

fn condition_requires_guard(cond: &hir::Expr<'_>, receiver: &hir::Expr<'_>, guard_name: &str) -> bool {
    match &cond.kind {
        ExprKind::MethodCall(segment, guard_receiver, args, _) => {
            args.is_empty() && segment.ident.name.as_str() == guard_name && same_receiver(receiver, guard_receiver)
        }
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, hir::BinOpKind::And) => {
            condition_requires_guard(lhs, receiver, guard_name) || condition_requires_guard(rhs, receiver, guard_name)
        }
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, hir::BinOpKind::Or) => {
            condition_requires_guard(lhs, receiver, guard_name) && condition_requires_guard(rhs, receiver, guard_name)
        }
        ExprKind::Block(block, _) => {
            block.expr.is_some_and(|tail| condition_requires_guard(tail, receiver, guard_name))
        }
        _ => false,
    }
}

fn has_same_branch_guard(cx: &LateContext<'_>, expr: &hir::Expr<'_>, receiver: &hir::Expr<'_>) -> bool {
    let Some(guard_name) = guard_method_name(cx, receiver) else {
        return false;
    };

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        let hir::Node::Expr(parent) = node else {
            continue;
        };

        let ExprKind::If(cond, then_expr, _) = &parent.kind else {
            continue;
        };

        if span_contains(then_expr.span, expr.span) && condition_requires_guard(cond, receiver, guard_name) {
            return true;
        }
    }

    false
}

fn should_skip_unwrap(cx: &LateContext<'_>, expr: &hir::Expr<'_>, receiver: &hir::Expr<'_>, name: &str) -> bool {
    if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
        return true;
    }

    if name != "unwrap" {
        return false;
    }

    is_known_constructor_call(receiver)
        || is_string_literal_parse(receiver)
        || has_same_branch_guard(cx, expr, receiver)
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

        if should_skip_unwrap(cx, expr, receiver, &name) {
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
                diag.primary_message(format!("`.{name}()` in production code will panic on failure",));
                diag.help(suggestion);
            }),
        );
    }
}
