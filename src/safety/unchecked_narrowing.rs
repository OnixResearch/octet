use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty::Ty;
use rustc_middle::ty::{self};
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

declare_lint! {
    /// Detects `as u32`, `as u16`, `as u8`, `as i32`, `as i16`, `as i8`
    /// casts from a wider integer type without a prior bounds check.
    /// Unchecked narrowing casts silently truncate — use `TryFrom` or
    /// a checked conversion function.
    pub UNCHECKED_NARROWING,
    Warn,
    "narrowing integer cast without bounds check"
}

declare_lint_pass!(UncheckedNarrowing => [UNCHECKED_NARROWING]);

fn int_width(ty: Ty<'_>) -> Option<u64> {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width(),
        ty::Uint(uint_ty) => uint_ty.bit_width(),
        _ => None,
    }
}

fn ty_name(ty: Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

fn is_target_max_expr<'tcx>(cx: &LateContext<'tcx>, expr: &hir::Expr<'tcx>, target_ty: Ty<'tcx>) -> bool {
    let ExprKind::Path(_) = &expr.kind else {
        return false;
    };

    let Ok(snippet) = cx.sess().source_map().span_to_snippet(expr.span) else {
        return false;
    };

    let normalized = snippet.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    let target_name = ty_name(target_ty);

    normalized == format!("{target_name}::MAX") || normalized == format!("<{target_name}>::MAX")
}

fn is_min_bounded_cast<'tcx>(
    cx: &LateContext<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> bool {
    let ExprKind::MethodCall(segment, receiver, args, _) = &inner.kind else {
        return false;
    };

    if segment.ident.name.as_str() != "min" || args.len() != 1 || cx.typeck_results().expr_ty(receiver) != source_ty {
        return false;
    }

    let arg = &args[0];
    let ExprKind::Cast(max_expr, _) = &arg.kind else {
        return false;
    };

    cx.typeck_results().expr_ty(arg) == source_ty
        && cx.typeck_results().expr_ty(max_expr) == target_ty
        && is_target_max_expr(cx, max_expr, target_ty)
}

/// Check if this cast is inside an `if` guard that looks like a bounds check
/// (condition mentions MAX, max, or compares against the target type's range).
fn has_bounds_guard(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Expr(parent) => {
                if let ExprKind::If(cond, _, _) = &parent.kind {
                    let source_map = cx.sess().source_map();
                    if let Ok(snippet) = source_map.span_to_snippet(cond.span) {
                        let s = snippet.to_ascii_lowercase();
                        // Recognizes: if val <= u32::MAX, if val < MAX_VAL, etc.
                        if s.contains("max") || s.contains("try_into") || s.contains("try_from") {
                            return true;
                        }
                    }
                }
                // Check for preceding assert! in a block — look at the parent block
                if let ExprKind::Block(block, _) = &parent.kind {
                    let source_map = cx.sess().source_map();
                    if let Ok(snippet) = source_map.span_to_snippet(block.span) {
                        let s = snippet.to_ascii_lowercase();
                        // If the block contains an assert with MAX before the cast
                        if s.contains("assert") && s.contains("max") {
                            return true;
                        }
                    }
                }
            }
            // Stop at function boundaries
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }
    false
}

/// Check if the cast is inside an `#[allow(clippy::cast_possible_truncation)]` block.
fn has_truncation_allow(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (hir_id, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Item(item) => {
                let attrs = cx.tcx.hir_attrs(item.hir_id());
                for attr in attrs {
                    let s = format!("{attr:?}");
                    if s.contains("cast_possible_truncation") {
                        return true;
                    }
                }
                break;
            }
            hir::Node::Expr(_) | hir::Node::Stmt(_) | hir::Node::Block(_) => {
                let attrs = cx.tcx.hir_attrs(hir_id);
                for attr in attrs {
                    let s = format!("{attr:?}");
                    if s.contains("cast_possible_truncation") {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for UncheckedNarrowing {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        let Some(source_width) = int_width(source_ty) else {
            return;
        };
        let Some(target_width) = int_width(target_ty) else {
            return;
        };

        if source_width <= target_width {
            return;
        }

        if is_min_bounded_cast(cx, inner, source_ty, target_ty) {
            return;
        }

        // Skip if inside a bounds-checking guard
        if has_bounds_guard(cx, expr) {
            return;
        }

        // Skip if inside #[allow(clippy::cast_possible_truncation)]
        if has_truncation_allow(cx, expr) {
            return;
        }

        let source_name = ty_name(source_ty);
        let target_name = ty_name(target_ty);

        cx.opt_span_lint(
            UNCHECKED_NARROWING,
            Some(expr.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "narrowing cast from `{source_name}` to `{target_name}` without bounds check",
                ));
                diag.help(format!(
                    "use `{target_name}::try_from(value)` or guard with \
                     `if value <= {target_name}::MAX as {source_name}`",
                ));
            }),
        );
    }
}
