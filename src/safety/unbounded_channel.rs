use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::emit_tiger_style_lint;

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

        emit_tiger_style_lint(
            cx,
            UNBOUNDED_CHANNEL,
            expr.span,
            "unbounded channel — use a bounded channel with explicit capacity",
            "unbounded_channel",
        );
    }
}
