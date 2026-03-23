use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::{ExprKind, PatKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty;
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects `match` over enum types where the last arm is a wildcard (`_`)
    /// returning a "deny" value: `false`, `None`, `Err(...)`, or `0`. When
    /// new variants are added, the wildcard silently handles them instead of
    /// forcing an explicit decision.
    pub CATCH_ALL_ON_ENUM,
    Warn,
    "catch-all wildcard on enum match returns deny value"
}

declare_lint_pass!(CatchAllOnEnum => [CATCH_ALL_ON_ENUM]);

fn is_deny_value(expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Lit(lit) => {
            matches!(lit.node, rustc_ast::LitKind::Bool(false))
                || matches!(lit.node, rustc_ast::LitKind::Int(v, _) if v.get() == 0)
        }
        ExprKind::Path(qpath) => {
            if let hir::QPath::Resolved(_, path) = qpath {
                let name = path
                    .segments
                    .last()
                    .map(|s| s.ident.name.as_str())
                    .unwrap_or("");
                name == "None"
            } else {
                false
            }
        }
        ExprKind::Call(func, _) => {
            // Check for Err(...)
            if let ExprKind::Path(hir::QPath::Resolved(_, path)) = &func.kind {
                path.segments
                    .last()
                    .map(|s| s.ident.name.as_str() == "Err")
                    .unwrap_or(false)
            } else {
                false
            }
        }
        _ => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for CatchAllOnEnum {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Match(scrutinee, arms, _) = &expr.kind else {
            return;
        };

        // Check if the scrutinee is an enum type
        let scrutinee_ty = cx.typeck_results().expr_ty(scrutinee);
        if !matches!(scrutinee_ty.kind(), ty::Adt(adt, _) if adt.is_enum()) {
            return;
        }

        // Check the last arm
        let Some(last_arm) = arms.last() else {
            return;
        };

        // Is it a wildcard?
        if !matches!(last_arm.pat.kind, PatKind::Wild) {
            return;
        }

        // Does it return a deny value?
        if !is_deny_value(last_arm.body) {
            return;
        }

        // Check if there's a justification comment nearby
        let source_map = cx.sess().source_map();
        if let Ok(snippet) = source_map.span_to_snippet(last_arm.span) {
            if snippet.contains("//") {
                return;
            }
        }

        cx.opt_span_lint(
            CATCH_ALL_ON_ENUM,
            Some(last_arm.span),
            DiagDecorator(|diag| {
                diag.primary_message(
                    "catch-all `_` on enum match returns deny value — \
                     new variants will be silently handled",
                );
                diag.help(
                    "list all variants explicitly so the compiler forces \
                     a decision when new variants are added",
                );
            }),
        );
    }
}
