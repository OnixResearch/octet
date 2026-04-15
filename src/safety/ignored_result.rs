use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;
use rustc_span::sym;

declare_lint! {
    /// Detects `let _ = expr` where `expr` returns `Result<_, _>`. Silently
    /// discarding a Result hides errors that could propagate or be logged.
    /// A study of catastrophic failures in distributed systems found that
    /// 92% had an error path that was explicitly ignored.
    pub IGNORED_RESULT,
    Warn,
    "Result value silently discarded with `let _ = ...`"
}

declare_lint_pass!(IgnoredResult => [IGNORED_RESULT]);

fn is_result_type(cx: &LateContext<'_>, t: ty::Ty<'_>) -> bool {
    if let ty::Adt(adt, _) = t.kind() {
        cx.tcx.is_diagnostic_item(sym::Result, adt.did())
    } else {
        false
    }
}

fn is_drop_impl_context(cx: &LateContext<'_>, hir_id: hir::HirId) -> bool {
    let mut saw_drop_method = false;

    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        match node {
            hir::Node::ImplItem(impl_item) => {
                saw_drop_method = impl_item.ident.name == sym::drop;
            }
            hir::Node::Item(item) => {
                let ItemKind::Impl(impl_block) = &item.kind else {
                    break;
                };

                if !saw_drop_method {
                    break;
                }

                let Some(of_trait) = impl_block.of_trait else {
                    break;
                };

                let Some(drop_trait) = cx.tcx.lang_items().drop_trait() else {
                    break;
                };

                return of_trait.trait_ref.trait_def_id() == Some(drop_trait);
            }
            hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    false
}

impl<'tcx> LateLintPass<'tcx> for IgnoredResult {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        // Only fire on wildcard pattern: `let _ = ...`
        if !matches!(local.pat.kind, PatKind::Wild) {
            return;
        }

        let Some(init) = local.init else {
            return;
        };

        let ty = cx.typeck_results().expr_ty(init);
        if !is_result_type(cx, ty) {
            return;
        }

        if is_drop_impl_context(cx, local.hir_id) {
            return;
        }

        cx.opt_span_lint(
            IGNORED_RESULT,
            Some(local.span),
            DiagDecorator(|diag| {
                diag.primary_message("`Result` value silently discarded with `let _ = ...`");
                diag.help(
                    "use `expr?` to propagate, `let _: () = expr?;` to assert infallibility, \
                     or handle the error explicitly",
                );
            }),
        );
    }
}
