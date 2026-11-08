use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects direct recursion (a function calling itself). Tiger Style
    /// avoids recursion because it makes execution depth unpredictable
    /// and can overflow the stack. Use iteration with explicit bounds
    /// instead.
    pub tigerstyle::NO_RECURSION,
    Warn,
    "function calls itself recursively"
}

declare_lint_pass!(NoRecursion => [NO_RECURSION]);

fn resolved_call_is_recursive(target_def_id: hir::def_id::DefId, candidate_def_id: Option<hir::def_id::DefId>) -> bool {
    candidate_def_id == Some(target_def_id)
}

struct RecursionFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    target_def_id: hir::def_id::DefId,
    found: Option<Span>,
}

impl<'tcx> RecursionFinder<'_, 'tcx> {
    /// Resolved callee of a call or method call, plus the HIR id that carries its generic args.
    fn callee(&self, expr: &'tcx hir::Expr<'tcx>) -> Option<(hir::def_id::DefId, hir::HirId)> {
        match &expr.kind {
            ExprKind::Call(func, _) => {
                let ExprKind::Path(qpath) = &func.kind else {
                    return None;
                };
                // `qpath_res` also resolves type-relative paths like `Self::f(..)` / `Type::f(..)`.
                let hir::def::Res::Def(_, def_id) = self.cx.qpath_res(qpath, func.hir_id) else {
                    return None;
                };
                Some((def_id, func.hir_id))
            }
            ExprKind::MethodCall(..) => {
                self.cx.typeck_results().type_dependent_def_id(expr.hir_id).map(|def_id| (def_id, expr.hir_id))
            }
            _ => None,
        }
    }

    fn is_recursive_call(&self, callee: hir::def_id::DefId, args_hir_id: hir::HirId) -> bool {
        if resolved_call_is_recursive(self.target_def_id, Some(callee)) {
            return true;
        }
        // Calls through a trait (`self.required()`, `Trait::required(self)`) resolve to the
        // trait item; resolve them to the impl item selected for the concrete receiver so a
        // trait impl method calling itself counts, while calls into other impls do not.
        let tcx = self.cx.tcx;
        if !matches!(tcx.def_kind(callee), hir::def::DefKind::AssocFn) || tcx.trait_of_assoc(callee).is_none() {
            return false;
        }
        let args = self.cx.typeck_results().node_args(args_hir_id);
        // Desugared lang-item calls (e.g. `for` loop `Iterator::next`) carry no recorded args.
        if args.len() != tcx.generics_of(callee).count() {
            return false;
        }
        let resolved = rustc_middle::ty::Instance::try_resolve(tcx, self.cx.typing_env(), callee, args)
            .ok()
            .flatten()
            .map(|instance| instance.def_id());
        resolved_call_is_recursive(self.target_def_id, resolved)
    }
}

impl<'tcx> Visitor<'tcx> for RecursionFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let Some((callee, args_hir_id)) = self.callee(expr)
            && self.is_recursive_call(callee, args_hir_id)
        {
            self.found = Some(expr.span);
            return;
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
            cx,
            target_def_id: def_id.to_def_id(),
            found: None,
        };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
            emit_tiger_style_lint(cx, NO_RECURSION, call_span, "function calls itself recursively", "no_recursion");
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_hir::def_id::DefId;
    use rustc_hir::def_id::LOCAL_CRATE;

    use super::resolved_call_is_recursive;

    fn local_def(index: u32) -> DefId {
        DefId {
            krate: LOCAL_CRATE,
            index: rustc_hir::def_id::DefIndex::from_u32(index),
        }
    }

    #[test]
    fn classifies_same_resolved_call_as_recursive() {
        let target = local_def(1);

        assert!(resolved_call_is_recursive(target, Some(target)));
    }

    #[test]
    fn skips_different_or_unresolved_calls() {
        let target = local_def(1);
        let other = local_def(2);

        assert!(!resolved_call_is_recursive(target, Some(other)));
        assert!(!resolved_call_is_recursive(target, None));
    }
}
