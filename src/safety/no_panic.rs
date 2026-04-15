use std::collections::HashSet;

use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::CtorOf;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::sym;

use crate::test_context;

declare_lint! {
    /// Detects `panic!()`, `todo!()`, `unimplemented!()`, and
    /// `unreachable!()` in non-test code. These abort the process
    /// instead of returning an error. Production code should return
    /// `Result` and let the caller decide how to handle failure.
    pub NO_PANIC,
    Warn,
    "panic macro in production code"
}

#[derive(Default)]
pub struct NoPanic;

impl_lint_pass!(NoPanic => [NO_PANIC]);

const PANIC_MACROS: &[&str] = &["panic!", "todo!", "unimplemented!", "unreachable!"];

fn panic_macro_name_from_callsite(snippet: &str) -> Option<&'static str> {
    let trimmed = snippet.trim();
    PANIC_MACROS.iter().copied().find(|name| trimmed.starts_with(name))
}

fn suggestion_for(name: &str) -> &'static str {
    match name {
        "todo!" => "return an error or use a tracking issue instead of `todo!()`",
        "unimplemented!" => "return `Err(...)` explaining what's missing",
        "unreachable!" => "if truly unreachable, use `debug_assert!` or prove it with types",
        _ => "return `Err(...)` instead of panicking",
    }
}

fn variant_def_id_from_qpath(
    cx: &LateContext<'_>,
    qpath: &hir::QPath<'_>,
    hir_id: hir::HirId,
) -> Option<rustc_hir::def_id::DefId> {
    match cx.qpath_res(qpath, hir_id) {
        Res::Def(DefKind::Variant, did) => Some(did),
        Res::Def(DefKind::Ctor(CtorOf::Variant, _), did) => cx.tcx.opt_parent(did),
        _ => None,
    }
}

fn collect_variant_ids_from_pat(
    cx: &LateContext<'_>,
    pat: &hir::Pat<'_>,
    covered: &mut HashSet<rustc_hir::def_id::DefId>,
) -> bool {
    match pat.kind {
        hir::PatKind::Struct(qpath, ..) | hir::PatKind::TupleStruct(qpath, ..) => {
            variant_def_id_from_qpath(cx, &qpath, pat.hir_id).is_some_and(|did| covered.insert(did))
        }
        hir::PatKind::Expr(pat_expr) => match pat_expr.kind {
            hir::PatExprKind::Path(qpath) => {
                variant_def_id_from_qpath(cx, &qpath, pat_expr.hir_id).is_some_and(|did| covered.insert(did))
            }
            hir::PatExprKind::Lit { .. } => false,
        },
        hir::PatKind::Or(pats) => pats.iter().all(|pat| collect_variant_ids_from_pat(cx, pat, covered)),
        hir::PatKind::Ref(pat, ..) | hir::PatKind::Box(pat) | hir::PatKind::Deref(pat) => {
            collect_variant_ids_from_pat(cx, pat, covered)
        }
        _ => false,
    }
}

fn item_span_indicates_non_exhaustive(cx: &LateContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let Ok(lines) = source_map.span_to_lines(span) else {
        return false;
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return false;
    };

    for idx in (0..first_line_idx).rev() {
        let Some(line) = lines.file.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            if trimmed.contains("non_exhaustive") {
                return true;
            }
            continue;
        }

        break;
    }

    false
}

fn is_line_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!")
}

fn line_has_no_panic_allow(line: &str) -> bool {
    let normalized = line.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    normalized.contains("#[allow(tigerstyle::no_panic)]")
}

fn span_has_justified_no_panic_allow(cx: &LateContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let Ok(lines) = source_map.span_to_lines(span) else {
        return false;
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return false;
    };

    for idx in (0..first_line_idx).rev() {
        let Some(line) = lines.file.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();

        if trimmed.is_empty() {
            break;
        }

        if line_has_no_panic_allow(trimmed) {
            if trimmed.contains("//") {
                return true;
            }

            if idx > 0
                && let Some(prev_line) = lines.file.get_line(idx - 1)
                && is_line_comment(&prev_line)
            {
                return true;
            }

            return false;
        }

        if is_line_comment(trimmed) || trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            continue;
        }

        break;
    }

    false
}

fn has_justified_todo_allow(cx: &LateContext<'_>, expr: &hir::Expr<'_>, name: &str) -> bool {
    if name != "todo!" {
        return false;
    }

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Expr(parent_expr) => {
                if let ExprKind::Block(block, _) = &parent_expr.kind
                    && span_has_justified_no_panic_allow(cx, block.span.source_callsite())
                {
                    return true;
                }
            }
            hir::Node::Item(item) => {
                if span_has_justified_no_panic_allow(cx, item.span.source_callsite()) {
                    return true;
                }
            }
            hir::Node::ImplItem(impl_item) => {
                if span_has_justified_no_panic_allow(cx, impl_item.span.source_callsite()) {
                    return true;
                }
            }
            hir::Node::TraitItem(trait_item) => {
                if span_has_justified_no_panic_allow(cx, trait_item.span.source_callsite()) {
                    return true;
                }
            }
            _ => {}
        }
    }

    false
}

fn is_exhaustive_unreachable_wildcard(cx: &LateContext<'_>, expr: &hir::Expr<'_>, name: &str) -> bool {
    if name != "unreachable!" {
        return false;
    }

    let mut enclosing_arm = None;

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Arm(arm) => enclosing_arm = Some(arm),
            hir::Node::Expr(parent_expr) => {
                let Some(arm) = enclosing_arm else {
                    continue;
                };

                let ExprKind::Match(scrutinee, arms, _) = parent_expr.kind else {
                    continue;
                };

                let Some(last_arm) = arms.last() else {
                    return false;
                };

                if last_arm.hir_id != arm.hir_id || !matches!(arm.pat.kind, hir::PatKind::Wild) || arm.guard.is_some() {
                    return false;
                }

                if arm.body.span.source_callsite() != expr.span.source_callsite() {
                    return false;
                }

                let ty::Adt(adt_def, _) = cx.typeck_results().expr_ty(scrutinee).kind() else {
                    return false;
                };

                if !adt_def.is_enum() {
                    return false;
                }

                let Some(local_enum_def_id) = adt_def.did().as_local() else {
                    return false;
                };
                let enum_hir_id = cx.tcx.local_def_id_to_hir_id(local_enum_def_id);
                let enum_attrs = cx.tcx.hir_attrs(enum_hir_id);
                if enum_attrs.iter().any(|attr| attr.has_name(sym::non_exhaustive))
                    || item_span_indicates_non_exhaustive(cx, cx.tcx.hir_span(enum_hir_id))
                {
                    return false;
                }

                let all_variants = adt_def.variants().iter().map(|variant| variant.def_id).collect::<HashSet<_>>();
                let mut covered = HashSet::new();

                for prior_arm in arms.iter().take(arms.len().saturating_sub(1)) {
                    if prior_arm.guard.is_some() || !collect_variant_ids_from_pat(cx, prior_arm.pat, &mut covered) {
                        return false;
                    }
                }

                return covered == all_variants;
            }
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    false
}

impl<'tcx> LateLintPass<'tcx> for NoPanic {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        // Panic macros are expanded before linting runs. We recover the
        // original macro invocation from the expansion callsite snippet.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded block expressions to avoid duplicate diagnostics on
        // nested nodes that share the same callsite.
        if !matches!(expr.kind, ExprKind::Block(..)) {
            return;
        }

        if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };

        let Some(name) = panic_macro_name_from_callsite(&snippet) else {
            return;
        };

        if is_exhaustive_unreachable_wildcard(cx, expr, name) || has_justified_todo_allow(cx, expr, name) {
            return;
        }

        let suggestion = suggestion_for(name);

        cx.opt_span_lint(
            NO_PANIC,
            Some(callsite),
            DiagDecorator(|diag| {
                diag.primary_message(format!("`{name}` will abort the process on execution"));
                diag.help(suggestion);
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::panic_macro_name_from_callsite;

    #[test]
    fn finds_panic_macro_name() {
        assert_eq!(panic_macro_name_from_callsite("panic!(\"oops\")"), Some("panic!"));
        assert_eq!(panic_macro_name_from_callsite("todo!(\"later\")"), Some("todo!"));
        assert_eq!(panic_macro_name_from_callsite("unimplemented!(\"x\")"), Some("unimplemented!"));
        assert_eq!(panic_macro_name_from_callsite("unreachable!(\"x\")"), Some("unreachable!"));
        assert_eq!(panic_macro_name_from_callsite("assert!(x > 0)"), None);
    }
}
