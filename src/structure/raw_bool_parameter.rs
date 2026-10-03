use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Rejects direct boolean parameters in public crate-owned APIs and
    /// multiple direct boolean parameters in private callables.
    pub tigerstyle::RAW_BOOL_PARAMETER,
    Warn,
    "raw boolean parameters obscure the domain meaning of arguments"
}

declare_lint_pass!(RawBoolParameter => [RAW_BOOL_PARAMETER]);

const MIN_AMBIGUOUS_PRIVATE_BOOL_PARAMETERS: usize = 2;

fn direct_bool(ty: &hir::Ty<'_>) -> bool {
    matches!(
        ty.kind,
        hir::TyKind::Path(hir::QPath::Resolved(_, hir::Path {
            res: hir::def::Res::PrimTy(hir::PrimTy::Bool),
            ..
        }))
    )
}

fn lint_signature(cx: &LateContext<'_>, span: Span, name: &str, decl: &hir::FnDecl<'_>, is_public: bool) {
    if span.from_expansion() {
        return;
    }

    if is_public {
        for input in decl.inputs.iter().filter(|input| !input.span.from_expansion() && direct_bool(input)) {
            emit_tiger_style_lint(
                cx,
                RAW_BOOL_PARAMETER,
                input.span,
                format!(
                    "public callable `{name}` has a direct `bool` parameter; callers cannot see its domain meaning"
                ),
                "raw_bool_parameter",
            );
        }
    } else {
        let mut first = None;
        let mut count = 0;
        for input in decl.inputs.iter().filter(|input| !input.span.from_expansion() && direct_bool(input)) {
            first.get_or_insert(input.span);
            count += 1;
        }
        if count >= MIN_AMBIGUOUS_PRIVATE_BOOL_PARAMETERS
            && let Some(first_span) = first
        {
            emit_tiger_style_lint(
                cx,
                RAW_BOOL_PARAMETER,
                first_span,
                format!(
                    "private callable `{name}` has {count} direct `bool` parameters; positional flags are ambiguous"
                ),
                "raw_bool_parameter",
            );
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for RawBoolParameter {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, ident, .. } = &item.kind else {
            return;
        };
        lint_signature(
            cx,
            item.span,
            ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(item.owner_id.def_id),
        );
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &item.kind else {
            return;
        };
        let parent = cx.tcx.local_parent(item.owner_id.def_id);
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(cx.tcx.local_def_id_to_hir_id(parent)) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() {
            return;
        }
        lint_signature(
            cx,
            item.span,
            item.ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(item.owner_id.def_id),
        );
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(sig, _) = &item.kind else {
            return;
        };
        let trait_id = cx.tcx.local_parent(item.owner_id.def_id);
        lint_signature(
            cx,
            item.span,
            item.ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(trait_id),
        );
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let hir::ExprKind::Closure(closure) = &expr.kind else {
            return;
        };
        lint_signature(cx, expr.span, "closure", closure.fn_decl, false);
    }
}
