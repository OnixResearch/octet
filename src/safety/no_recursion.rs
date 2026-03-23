use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::Span;

declare_lint! {
    /// Detects direct recursion (a function calling itself). Tiger Style
    /// avoids recursion because it makes execution depth unpredictable
    /// and can overflow the stack. Use iteration with explicit bounds
    /// instead.
    pub NO_RECURSION,
    Warn,
    "function calls itself recursively"
}

declare_lint_pass!(NoRecursion => [NO_RECURSION]);

struct RecursionFinder {
    target_def_id: hir::def_id::DefId,
    found: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for RecursionFinder {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        match &expr.kind {
            ExprKind::Call(func, _) => {
                if let ExprKind::Path(qpath) = &func.kind {
                    if let hir::QPath::Resolved(_, path) = qpath {
                        if let hir::def::Res::Def(_, def_id) = path.res {
                            if def_id == self.target_def_id {
                                self.found = Some(expr.span);
                                return;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for NoRecursion {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        _span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        let mut finder = RecursionFinder {
            target_def_id: def_id.to_def_id(),
            found: None,
        };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
            cx.opt_span_lint(
                NO_RECURSION,
                Some(call_span),
                DiagDecorator(|diag| {
                    diag.primary_message("function calls itself recursively");
                    diag.help(
                        "replace with iteration using an explicit loop and \
                         a stack/queue data structure if needed",
                    );
                }),
            );
        }
    }
}
