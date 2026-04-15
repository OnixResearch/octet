use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects negative predicate names on boolean-returning functions and
    /// bool-typed local bindings. Tiger Style prefers positive predicate names
    /// such as `is_ready` over names like `is_not_ready` or `no_capacity`.
    pub tigerstyle::NEGATED_PREDICATE,
    Warn,
    "negative predicate name"
}

declare_lint_pass!(NegatedPredicate => [NEGATED_PREDICATE]);

const NEGATIVE_PREFIXES: &[&str] = &[
    "is_not_",
    "has_no_",
    "has_not_",
    "lacks_",
    "cannot_",
    "cant_",
    "does_not_",
    "doesnt_",
    "isnt_",
    "wasnt_",
    "wont_",
    "no_",
];

fn has_negative_prefix(name: &str) -> bool {
    NEGATIVE_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

fn lint_name(cx: &LateContext<'_>, span: rustc_span::Span, name: &str) {
    emit_tiger_style_lint(
        cx,
        NEGATED_PREDICATE,
        span,
        format!("negated predicate name `{name}` — prefer a positive form like `is_ready`"),
        "negated_predicate",
    );
}

impl<'tcx> LateLintPass<'tcx> for NegatedPredicate {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { ident, .. } = &item.kind else {
            return;
        };

        let name = ident.name.as_str();
        if name == "not" && !cx.tcx.visibility(item.owner_id.def_id).is_public() {
            return;
        }
        if !has_negative_prefix(&name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, ident.span, &name);
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(_, _) = impl_item.kind else {
            return;
        };
        let name = impl_item.ident.name.as_str();
        if !has_negative_prefix(&name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(impl_item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, impl_item.ident.span, &name);
        }
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(_, _) = trait_item.kind else {
            return;
        };
        let name = trait_item.ident.name.as_str();
        if !has_negative_prefix(&name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(trait_item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, trait_item.ident.span, &name);
        }
    }

    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        let PatKind::Binding(_, _, ident, _) = &local.pat.kind else {
            return;
        };

        let name = ident.name.as_str();
        if !has_negative_prefix(&name) || !cx.typeck_results().pat_ty(local.pat).is_bool() {
            return;
        }

        lint_name(cx, ident.span, &name);
    }
}
