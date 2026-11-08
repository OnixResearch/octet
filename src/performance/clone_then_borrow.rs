use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::walk_expr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::ty;
use rustc_middle::ty::adjustment::Adjust;
use rustc_middle::ty::adjustment::AutoBorrow;
use rustc_middle::ty::adjustment::AutoBorrowMutability;
use rustc_middle::ty::adjustment::DerefAdjustKind;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

const LINT_NAME: &str = "clone_then_borrow";

declare_tool_lint! {
    /// Detects cloning a value and immediately taking a shared borrow of the clone.
    /// Tiger Style treats clone-to-satisfy-borrow-checker patterns as performance
    /// and ownership-clarity smells when no owned value crosses a boundary.
    pub tigerstyle::CLONE_THEN_BORROW,
    Warn,
    "clone immediately borrowed instead of borrowing the original value"
}

declare_lint_pass!(CloneThenBorrow => [CLONE_THEN_BORROW]);

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            hir::ExprKind::DropTemps(inner) | hir::ExprKind::AddrOf(_, _, inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    break expr;
                };
                expr = inner;
            }
            _ => break expr,
        }
    }
}

fn def_path_looks_like_clone(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) else {
        return false;
    };
    let def_path = cx.get_def_path(def_id);
    def_path.last().is_some_and(|segment| *segment == Symbol::intern("clone"))
        && def_path.iter().any(|segment| *segment == Symbol::intern("Clone"))
}

fn borrowed_clone_receiver<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    if expr.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::AddrOf(_, hir::Mutability::Not, borrowed) = &expr.kind else {
        return None;
    };
    let borrowed = peel_transparent_expr(borrowed);
    if borrowed.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::MethodCall(segment, receiver, args, _) = &borrowed.kind else {
        return None;
    };
    if segment.ident.name.as_str() != "clone" || !args.is_empty() || !def_path_looks_like_clone(cx, borrowed) {
        return None;
    }

    Some(*receiver)
}

fn borrow_original_replacement(cx: &LateContext<'_>, receiver: &hir::Expr<'_>, receiver_snippet: &str) -> String {
    if matches!(cx.typeck_results().expr_ty(receiver).kind(), ty::Ref(..)) {
        receiver_snippet.to_owned()
    } else {
        format!("&{receiver_snippet}")
    }
}

/// The `let` statement whose initializer stores `borrow`, directly or inside a
/// tuple, array, or struct literal, so the borrowed clone outlives the statement.
fn storing_let_stmt<'tcx>(cx: &LateContext<'tcx>, borrow: &hir::Expr<'_>) -> Option<&'tcx hir::Stmt<'tcx>> {
    let mut child = borrow.hir_id;
    let mut parents = cx.tcx.hir_parent_iter(borrow.hir_id);
    for (parent_id, node) in parents.by_ref() {
        match node {
            hir::Node::Expr(parent)
                if matches!(
                    parent.kind,
                    hir::ExprKind::Tup(..) | hir::ExprKind::Array(..) | hir::ExprKind::Struct(..)
                ) => {}
            hir::Node::ExprField(_) => {}
            hir::Node::LetStmt(local) if local.init.is_some_and(|init| init.hir_id == child) => break,
            _ => return None,
        }
        child = parent_id;
    }
    match parents.next() {
        Some((_, hir::Node::Stmt(stmt))) => Some(stmt),
        _ => None,
    }
}

/// The local variable that owns the place the clone was taken from.
fn receiver_root_local(mut receiver: &hir::Expr<'_>) -> Option<hir::HirId> {
    loop {
        match &receiver.kind {
            hir::ExprKind::Field(base, _)
            | hir::ExprKind::Index(base, _, _)
            | hir::ExprKind::Unary(hir::UnOp::Deref, base)
            | hir::ExprKind::MethodCall(_, base, _, _) => receiver = base,
            hir::ExprKind::Path(hir::QPath::Resolved(None, path)) => {
                return match path.res {
                    Res::Local(local) => Some(local),
                    _ => None,
                };
            }
            _ => return None,
        }
    }
}

fn borrows_mutably(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    cx.typeck_results().expr_adjustments(expr).iter().any(|adjustment| match adjustment.kind {
        Adjust::Borrow(AutoBorrow::Ref(mutability)) => matches!(mutability, AutoBorrowMutability::Mut { .. }),
        Adjust::Deref(DerefAdjustKind::Overloaded(deref)) => deref.mutbl == hir::Mutability::Mut,
        _ => false,
    })
}

/// Whether the place rooted at `path_expr` is assigned or mutably borrowed.
fn place_is_mutated(cx: &LateContext<'_>, path_expr: &hir::Expr<'_>) -> bool {
    let mut place = path_expr;
    for (_, node) in cx.tcx.hir_parent_iter(path_expr.hir_id) {
        if borrows_mutably(cx, place) {
            return true;
        }
        let hir::Node::Expr(parent) = node else {
            return false;
        };
        match &parent.kind {
            hir::ExprKind::Field(base, _)
            | hir::ExprKind::Index(base, _, _)
            | hir::ExprKind::Unary(hir::UnOp::Deref, base)
                if base.hir_id == place.hir_id =>
            {
                place = parent;
            }
            hir::ExprKind::AddrOf(_, hir::Mutability::Mut, _) => return true,
            hir::ExprKind::Assign(lhs, ..) | hir::ExprKind::AssignOp(_, lhs, _) => return lhs.hir_id == place.hir_id,
            _ => return false,
        }
    }
    false
}

/// Finds an assignment or mutable borrow of `local`, including inside closures.
struct LocalMutationFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    local: hir::HirId,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for LocalMutationFinder<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> ty::TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }
        if let hir::ExprKind::Path(hir::QPath::Resolved(None, path)) = &expr.kind
            && path.res == Res::Local(self.local)
            && place_is_mutated(self.cx, expr)
        {
            self.found = true;
            return;
        }
        walk_expr(self, expr);
    }
}

/// The spec exempts clones stored for later and independently mutated: when
/// `let before = &x.clone();` is followed in the same block by a mutation of
/// `x`, borrowing `x` instead would conflict with that mutation (E0502).
fn stored_clone_outlives_mutation(cx: &LateContext<'_>, borrow: &hir::Expr<'_>, receiver: &hir::Expr<'_>) -> bool {
    let Some(local) = receiver_root_local(receiver) else {
        return false;
    };
    let Some(let_stmt) = storing_let_stmt(cx, borrow) else {
        return false;
    };
    let hir::Node::Block(block) = cx.tcx.parent_hir_node(let_stmt.hir_id) else {
        return false;
    };
    let mut finder = LocalMutationFinder {
        cx,
        local,
        found: false,
    };
    let later_stmts = block.stmts.iter().skip_while(|stmt| stmt.hir_id != let_stmt.hir_id).skip(1);
    for stmt in later_stmts {
        finder.visit_stmt(stmt);
    }
    if let Some(tail) = block.expr {
        finder.visit_expr(tail);
    }
    finder.found
}

impl<'tcx> LateLintPass<'tcx> for CloneThenBorrow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some(receiver) = borrowed_clone_receiver(cx, expr) else {
            return;
        };
        if stored_clone_outlives_mutation(cx, expr, receiver) {
            return;
        }
        let Ok(receiver_snippet) = cx.sess().source_map().span_to_snippet(receiver.span) else {
            return;
        };
        let replacement = borrow_original_replacement(cx, receiver, &receiver_snippet);
        let message = "clone result is immediately borrowed; borrow the original value or use an explicit borrowed adapter instead";

        emit_tiger_style_lint_with_suggestion(
            cx,
            CLONE_THEN_BORROW,
            expr.span,
            message,
            LINT_NAME,
            LintSuggestion::Single {
                span: expr.span,
                message: "borrow the original value instead of cloning then borrowing".to_owned(),
                replacement,
                applicability: Applicability::MaybeIncorrect,
            },
        );
    }
}
