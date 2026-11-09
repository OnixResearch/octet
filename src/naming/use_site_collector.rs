use rustc_hir as hir;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_span::Span;

/// How each read of the renamed local is rewritten.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadRewrite {
    /// The new name holds the same value: every read becomes the new name.
    Rename,
    /// The new name holds the negated value (`is_not_ready` becomes
    /// `is_ready`): every read becomes `!new_name`, and an existing `!old_name`
    /// collapses to `new_name`.
    Negate,
}

struct UseSiteCollector<'cx, 'tcx> {
    typeck: &'tcx rustc_middle::ty::TypeckResults<'tcx>,
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    target: hir::HirId,
    replacement: &'cx str,
    rewrite: ReadRewrite,
    use_sites: Vec<(Span, String)>,
    /// Method receivers already rewritten as `(!name)`.
    rewritten_receivers: Vec<hir::HirId>,
    is_unsupported: bool,
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
            self.is_unsupported = true;
        }
        self.use_sites.push((span, replacement));
    }

    /// Records a plain read of the target. `needs_parens` marks a postfix
    /// position such as a method receiver, where `!name` would bind wrongly.
    fn record_read(&mut self, expr: &hir::Expr<'_>, needs_parens: bool) {
        let replacement = match self.rewrite {
            ReadRewrite::Rename => self.replacement.to_owned(),
            ReadRewrite::Negate => {
                // `!name` is a temporary: a borrow of the binding (`&name`,
                // method autoref) cannot be rewritten to keep its meaning.
                if self
                    .typeck
                    .expr_adjustments(expr)
                    .iter()
                    .any(|adjustment| matches!(adjustment.kind, rustc_middle::ty::adjustment::Adjust::Borrow(_)))
                {
                    self.is_unsupported = true;
                }
                if needs_parens {
                    format!("(!{})", self.replacement)
                } else {
                    format!("!{}", self.replacement)
                }
            }
        };
        self.record(expr.span, replacement);
    }
}

impl<'tcx> Visitor<'tcx> for UseSiteCollector<'_, 'tcx> {
    // Closures and async blocks are nested bodies; uses inside them must be
    // renamed too or the rewritten code no longer resolves.
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.rewrite == ReadRewrite::Negate {
            match expr.kind {
                hir::ExprKind::Unary(hir::UnOp::Not, operand) if self.is_target_path(operand) => {
                    self.record(expr.span, self.replacement.to_owned());
                    return;
                }
                hir::ExprKind::MethodCall(_, receiver, _, _) if self.is_target_path(receiver) => {
                    self.record_read(receiver, true);
                    self.rewritten_receivers.push(receiver.hir_id);
                }
                hir::ExprKind::AddrOf(_, _, operand) if self.is_target_path(operand) => {
                    self.is_unsupported = true;
                    return;
                }
                _ => {}
            }
        }
        if self.is_target_path(expr) && !self.rewritten_receivers.contains(&expr.hir_id) {
            self.record_read(expr, false);
        }
        intravisit::walk_expr(self, expr);
    }

    fn visit_expr_field(&mut self, field: &'tcx hir::ExprField<'tcx>) {
        // `S { name }` must become `S { name: new_name }`, not `S { new_name }`.
        if field.is_shorthand && self.is_target_path(field.expr) {
            let value = match self.rewrite {
                ReadRewrite::Rename => self.replacement.to_owned(),
                ReadRewrite::Negate => format!("!{}", self.replacement),
            };
            self.record(field.expr.span, format!("{}: {value}", field.ident));
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
/// closures and async blocks) paired with the source text that rewrites it for
/// `replacement` under `rewrite`. Returns `None` when a use comes from a macro
/// expansion or cannot be rewritten without changing meaning.
pub(crate) fn collect_local_use_sites<'tcx>(
    cx: &LateContext<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    binding_hir_id: hir::HirId,
    replacement: &str,
    rewrite: ReadRewrite,
) -> Option<Vec<(Span, String)>> {
    let mut collector = UseSiteCollector {
        typeck: cx.tcx.typeck_body(body.id()),
        tcx: cx.tcx,
        target: binding_hir_id,
        replacement,
        rewrite,
        use_sites: Vec::new(),
        rewritten_receivers: Vec::new(),
        is_unsupported: false,
    };

    collector.visit_expr(body.value);

    if collector.is_unsupported {
        return None;
    }

    Some(collector.use_sites)
}

/// Finds bindings and single-segment paths spelled `name`.
struct NameFinder<'tcx> {
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    name: rustc_span::Symbol,
    is_found: bool,
}

impl<'tcx> Visitor<'tcx> for NameFinder<'tcx> {
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    fn visit_pat(&mut self, pat: &'tcx hir::Pat<'tcx>) {
        if let hir::PatKind::Binding(_, _, ident, _) = pat.kind
            && ident.name == self.name
        {
            self.is_found = true;
        }
        intravisit::walk_pat(self, pat);
    }

    fn visit_path(&mut self, path: &hir::Path<'tcx>, _id: hir::HirId) {
        if let [segment] = path.segments
            && segment.ident.name == self.name
        {
            self.is_found = true;
        }
        intravisit::walk_path(self, path);
    }
}

/// Whether renaming the local `binding_hir_id` to `name` could capture or be
/// captured by something already named `name`: a parameter or local anywhere
/// in the enclosing function (closures included), a single-segment path that
/// spells `name` there, or a crate item whose name a `let` pattern would
/// resolve to instead of introducing a binding.
pub(crate) fn is_name_taken(cx: &LateContext<'_>, binding_hir_id: hir::HirId, name: &str) -> bool {
    let name = rustc_span::Symbol::intern(name);
    let body_owner = cx.tcx.hir_enclosing_body_owner(binding_hir_id);
    let root_owner = cx.tcx.typeck_root_def_id(body_owner.to_def_id()).expect_local();
    let root_body = cx.tcx.hir_body_owned_by(root_owner);

    let mut finder = NameFinder {
        tcx: cx.tcx,
        name,
        is_found: false,
    };
    finder.visit_body(root_body);
    if finder.is_found {
        return true;
    }

    cx.tcx.hir_crate_items(()).definitions().any(|def_id| {
        let is_pattern_resolvable = matches!(
            cx.tcx.def_kind(def_id),
            rustc_hir::def::DefKind::Const { .. }
                | rustc_hir::def::DefKind::Static { .. }
                | rustc_hir::def::DefKind::ConstParam
                | rustc_hir::def::DefKind::Use
                | rustc_hir::def::DefKind::Ctor(_, rustc_hir::def::CtorKind::Const)
        );
        is_pattern_resolvable && cx.tcx.opt_item_name(def_id.to_def_id()) == Some(name)
    })
}
