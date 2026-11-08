use rustc_hir as hir;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_span::Span;

struct UseSiteCollector<'cx, 'tcx> {
    cx: &'cx LateContext<'tcx>,
    target: hir::HirId,
    replacement: &'cx str,
    use_sites: Vec<(Span, String)>,
    has_expansion: bool,
}

impl UseSiteCollector<'_, '_> {
    fn is_target_path(&self, expr: &hir::Expr<'_>) -> bool {
        matches!(
            expr.kind,
            hir::ExprKind::Path(hir::QPath::Resolved(None, path))
                if path.res == hir::def::Res::Local(self.target)
        )
    }

    fn record(&mut self, span: Span, replacement: String) {
        if span.from_expansion() {
            self.has_expansion = true;
        }
        self.use_sites.push((span, replacement));
    }
}

impl<'tcx> Visitor<'tcx> for UseSiteCollector<'_, 'tcx> {
    // Closures and async blocks are nested bodies; uses inside them must be
    // renamed too or the rewritten code no longer resolves.
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.is_target_path(expr) {
            self.record(expr.span, self.replacement.to_owned());
        }
        intravisit::walk_expr(self, expr);
    }

    fn visit_expr_field(&mut self, field: &'tcx hir::ExprField<'tcx>) {
        // `S { name }` must become `S { name: new_name }`, not `S { new_name }`.
        if field.is_shorthand && self.is_target_path(field.expr) {
            self.record(field.expr.span, format!("{}: {}", field.ident, self.replacement));
            return;
        }
        intravisit::walk_expr_field(self, field);
    }
}

pub(crate) fn enclosing_body<'tcx>(cx: &LateContext<'tcx>, hir_id: hir::HirId) -> &'tcx hir::Body<'tcx> {
    let body_owner = cx.tcx.hir_enclosing_body_owner(hir_id);
    cx.tcx.hir_body_owned_by(body_owner)
}

/// Collects every use of the local `binding_hir_id` inside `body` (including
/// closures and async blocks) paired with the source text that renames it to
/// `replacement`. Returns `None` when a use comes from a macro expansion.
pub(crate) fn collect_local_use_sites<'tcx>(
    cx: &LateContext<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    binding_hir_id: hir::HirId,
    replacement: &str,
) -> Option<Vec<(Span, String)>> {
    let mut collector = UseSiteCollector {
        cx,
        target: binding_hir_id,
        replacement,
        use_sites: Vec::new(),
        has_expansion: false,
    };

    collector.visit_expr(body.value);

    if collector.has_expansion {
        return None;
    }

    Some(collector.use_sites)
}
