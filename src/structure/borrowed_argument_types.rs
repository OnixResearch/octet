use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects function parameters that borrow owned standard containers.
    /// Tiger Style prefers the most general borrowed argument type when a
    /// signature does not need ownership, capacity, or exact owned type identity.
    pub tigerstyle::BORROWED_ARGUMENT_TYPES,
    Warn,
    "borrowed owned-container argument type"
}

declare_lint_pass!(BorrowedArgumentTypes => [BORROWED_ARGUMENT_TYPES]);

struct BorrowedArgumentCandidate {
    span: rustc_span::Span,
    found: String,
    replacement: String,
    vector: Option<hir::def_id::DefId>,
}

fn path_ends_with(path: &[Symbol], suffix: &[&str]) -> bool {
    path.len() >= suffix.len()
        && path[path.len() - suffix.len()..]
            .iter()
            .copied()
            .eq(suffix.iter().map(|segment| Symbol::intern(segment)))
}

fn borrowed_replacement(cx: &LateContext<'_>, ty: &hir::Ty<'_>) -> Option<BorrowedArgumentCandidate> {
    let hir::TyKind::Ref(_, borrowed) = &ty.kind else {
        return None;
    };
    let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &borrowed.ty.kind else {
        return None;
    };
    let hir::def::Res::Def(_, def_id) = path.res else {
        return None;
    };

    let def_path = cx.get_def_path(def_id);
    if path_ends_with(&def_path, &["string", "String"]) {
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: "&String".to_owned(),
            replacement: "str".to_owned(),
            vector: None,
        });
    }
    if path_ends_with(&def_path, &["vec", "Vec"]) {
        let snippet = cx.sess().source_map().span_to_snippet(borrowed.ty.span).ok()?;
        let inner_start = snippet.find("Vec<")? + "Vec<".len();
        let inner = snippet[inner_start..].strip_suffix('>')?;
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: format!("&{snippet}"),
            replacement: format!("[{inner}]"),
            vector: Some(def_id),
        });
    }
    if path_ends_with(&def_path, &["path", "PathBuf"]) {
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: "&PathBuf".to_owned(),
            replacement: "Path".to_owned(),
            vector: None,
        });
    }

    None
}

fn signature_is_extern(cx: &LateContext<'_>, sig: &hir::FnSig<'_>) -> bool {
    cx.sess().source_map().span_to_snippet(sig.span).is_ok_and(|snippet| snippet.contains("extern \""))
}

// Recognize a required owning-container operation on this exact parameter.
// This deliberately does not infer aliases, captured uses, or arbitrary calls.
struct VectorPushFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    typeck: &'tcx ty::TypeckResults<'tcx>,
    parameter: hir::HirId,
    vector: hir::def_id::DefId,
    found: bool,
}

impl<'tcx> VectorPushFinder<'_, 'tcx> {
    fn is_parameter(&self, expr: &hir::Expr<'tcx>) -> bool {
        matches!(expr.kind, hir::ExprKind::Path(ref path)
            if self.typeck.qpath_res(path, expr.hir_id) == hir::def::Res::Local(self.parameter))
    }

    fn is_vector_push(&self, def_id: hir::def_id::DefId) -> bool {
        if self.cx.tcx.opt_item_name(def_id) != Some(Symbol::intern("push"))
            || self.cx.tcx.crate_name(def_id.krate) != Symbol::intern("alloc")
        {
            return false;
        }
        let assoc = self.cx.tcx.associated_item(def_id);
        if assoc.container != ty::AssocContainer::InherentImpl {
            return false;
        }
        let Some(owner) = assoc.impl_container(self.cx.tcx) else {
            return false;
        };
        matches!(self.cx.tcx.type_of(owner).instantiate_identity().kind(), ty::Adt(def, _) if def.did() == self.vector)
    }
}

impl<'tcx> Visitor<'tcx> for VectorPushFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found || matches!(expr.kind, hir::ExprKind::Closure(_)) {
            return;
        }
        match expr.kind {
            hir::ExprKind::MethodCall(_, receiver, _, _)
                if self.is_parameter(receiver)
                    && self.typeck.type_dependent_def_id(expr.hir_id).is_some_and(|id| self.is_vector_push(id)) =>
            {
                self.found = true;
            }
            hir::ExprKind::Call(callee, args) => {
                if let hir::ExprKind::Path(ref path) = callee.kind
                    && let hir::def::Res::Def(hir::def::DefKind::AssocFn, id) =
                        self.typeck.qpath_res(path, callee.hir_id)
                    && args.first().is_some_and(|arg| self.is_parameter(arg))
                    && self.is_vector_push(id)
                {
                    self.found = true;
                }
            }
            _ => {}
        }
        if !self.found {
            hir::intravisit::walk_expr(self, expr);
        }
    }
}

fn parameter_needs_vector<'tcx>(
    cx: &LateContext<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    index: usize,
    vector: hir::def_id::DefId,
) -> bool {
    let Some(parameter) = body.params.get(index) else {
        return false;
    };
    let hir::PatKind::Binding(_, binding, _, None) = parameter.pat.kind else {
        return false;
    };
    let mut visitor = VectorPushFinder {
        cx,
        typeck: cx.tcx.typeck(body.value.hir_id.owner.def_id),
        parameter: binding,
        vector,
        found: false,
    };
    visitor.visit_expr(body.value);
    visitor.found
}

fn lint_decl<'tcx>(
    cx: &LateContext<'tcx>,
    hir_id: hir::HirId,
    report_span: rustc_span::Span,
    sig: &hir::FnSig<'tcx>,
    body: &'tcx hir::Body<'tcx>,
) {
    if signature_is_extern(cx, sig) || test_context::late_hir_id_is_in_test_context(cx, hir_id, report_span) {
        return;
    }

    for (index, input) in sig.decl.inputs.iter().enumerate() {
        let Some(candidate) = borrowed_replacement(cx, input) else {
            continue;
        };
        if candidate.vector.is_some_and(|vector| parameter_needs_vector(cx, body, index, vector)) {
            continue;
        }
        let suggested_type = format!("&{}", candidate.replacement);
        let message = format!(
            "parameter uses `{}`; accept `{suggested_type}` unless ownership, capacity, or exact owned type identity is required",
            candidate.found
        );
        emit_tiger_style_lint_with_suggestion(
            cx,
            BORROWED_ARGUMENT_TYPES,
            report_span,
            message,
            "borrowed_argument_types",
            LintSuggestion::Single {
                span: candidate.span,
                message: format!("replace `{}` with `{suggested_type}`", candidate.found),
                replacement: candidate.replacement,
                applicability: Applicability::MaybeIncorrect,
            },
        );
        return;
    }
}

impl<'tcx> LateLintPass<'tcx> for BorrowedArgumentTypes {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, body, .. } = &item.kind else {
            return;
        };
        lint_decl(cx, item.hir_id(), item.span, sig, cx.tcx.hir_body(*body));
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, body) = &impl_item.kind else {
            return;
        };
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(impl_item.owner_id.def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() {
            return;
        }

        lint_decl(cx, impl_item.hir_id(), impl_item.span, sig, cx.tcx.hir_body(*body));
    }
}
