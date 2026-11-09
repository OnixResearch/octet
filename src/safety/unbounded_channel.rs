use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects known unbounded channel constructors from std, crossbeam, and
    /// Tokio. Tiger Style prefers explicit queue bounds so memory growth and
    /// backpressure remain reviewable.
    pub tigerstyle::UNBOUNDED_CHANNEL,
    Warn,
    "unbounded channel constructor"
}

declare_lint_pass!(UnboundedChannel => [UNBOUNDED_CHANNEL]);

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn resolved_callee_def_id(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> Option<rustc_hir::def_id::DefId> {
    let ExprKind::Path(qpath) = &callee.kind else {
        return None;
    };

    match cx.typeck_results().qpath_res(qpath, callee.hir_id) {
        Res::Def(_, did) => Some(did),
        _ => None,
    }
}

fn is_unbounded_channel_ctor(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> bool {
    match_def_path(cx, did, &["std", "sync", "mpsc", "channel"])
        || match_def_path(cx, did, &["crossbeam_channel", "channel", "unbounded"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded", "unbounded_channel"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded_channel"])
}

impl<'tcx> LateLintPass<'tcx> for UnboundedChannel {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Call(callee, _) = &expr.kind else {
            return;
        };

        let Some(did) = resolved_callee_def_id(cx, callee) else {
            return;
        };
        if !is_unbounded_channel_ctor(cx, did) {
            return;
        }

        let message = "unbounded channel — use a bounded channel with explicit capacity";
        if let Some(suggestion) = bounded_suggestion(cx, expr, did) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                UNBOUNDED_CHANNEL,
                expr.span,
                message,
                "unbounded_channel",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, UNBOUNDED_CHANNEL, expr.span, message, "unbounded_channel");
        }
    }
}

/// Unbounded constructor name and its bounded sibling in the same public module.
fn bounded_counterpart(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> Option<(&'static str, &'static str)> {
    if match_def_path(cx, did, &["std", "sync", "mpsc", "channel"]) {
        Some(("channel", "sync_channel"))
    } else if match_def_path(cx, did, &["crossbeam_channel", "channel", "unbounded"]) {
        Some(("unbounded", "bounded"))
    } else if match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded", "unbounded_channel"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded_channel"])
    {
        Some(("unbounded_channel", "channel"))
    } else {
        None
    }
}

/// Module path of a `use <crate>::...::<ctor> as <alias>` import of `did`, when it
/// starts at an extern crate root and so names the same module from any scope.
fn aliased_import_module_path(cx: &LateContext<'_>, did: DefId, alias: Symbol) -> Option<String> {
    cx.tcx.hir_free_items().find_map(|item_id| {
        let hir::ItemKind::Use(path, hir::UseKind::Single(ident)) = cx.tcx.hir_item(item_id).kind else {
            return None;
        };
        if ident.name != alias || path.res.value_ns != Some(Res::Def(DefKind::Fn, did)) {
            return None;
        }
        let (_, modules) = path.segments.split_last()?;
        let crate_root = modules.first()?;
        if !matches!(crate_root.res, Res::Def(DefKind::Mod, root) if root.is_crate_root() && !root.is_local()) {
            return None;
        }
        Some(modules.iter().map(|segment| segment.ident.as_str()).collect::<Vec<_>>().join("::"))
    })
}

fn bounded_suggestion(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    did: rustc_hir::def_id::DefId,
) -> Option<LintSuggestion> {
    let ExprKind::Call(callee, _) = &expr.kind else {
        return None;
    };
    let ExprKind::Path(hir::QPath::Resolved(None, path)) = &callee.kind else {
        return None;
    };
    let name = path.segments.last()?.ident;
    let name_span = name.span;
    if !expr.span.contains(callee.span) || !callee.span.contains(name_span) {
        return None;
    }
    let (unbounded_name, bounded_name) = bounded_counterpart(cx, did)?;
    let sm = cx.sess().source_map();
    // Generic args like `::<()>` follow the renamed segment and stay intact.
    let generic_args = sm.span_to_snippet(callee.span.with_lo(name_span.hi())).ok()?;

    let bounded_path = if name.name.as_str() == unbounded_name {
        // Rename only the constructor segment; any module prefix stays as written.
        let prefix = sm.span_to_snippet(callee.span.with_hi(name_span.lo())).ok()?;
        format!("{prefix}{bounded_name}")
    } else {
        // An aliased import (`use std::sync::mpsc::channel as make`) says nothing
        // about the bounded sibling's name, so spell it through the module the
        // import names; without such an import, offer no rewrite.
        let module_path = aliased_import_module_path(cx, did, name.name)?;
        format!("{module_path}::{bounded_name}")
    };

    // Unbounded constructors take no arguments, so the argument list becomes the
    // capacity placeholder.
    let replacement = format!("{bounded_path}{generic_args}(/* CAPACITY */)");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "use bounded channel with explicit capacity".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}
