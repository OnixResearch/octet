use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects direct calls to `Instant::now()`, `SystemTime::now()`,
    /// and `Utc::now()` (chrono) in production code. Reading the system
    /// clock is a side effect — core/pure crates should receive time as
    /// a parameter so the logic stays deterministic and testable.
    ///
    /// Boundary crates (e.g. a web shell or CLI entry point) that
    /// legitimately own the clock can suppress this with:
    ///
    /// ```rust
    /// #[allow(tigerstyle::ambient_clock, reason = "entry-point owns time capture")]
    /// ```
    pub tigerstyle::AMBIENT_CLOCK,
    Warn,
    "direct system clock read — inject time as a parameter instead"
}

declare_lint_pass!(AmbientClock => [AMBIENT_CLOCK]);

/// Known clock types; a call to their associated `now` reads the clock.
const CLOCK_TYPES: &[&str] = &[
    "Instant",
    "SystemTime",
    "Utc",   // chrono::Utc::now()
    "Local", // chrono::Local::now()
];

/// Whether `callee` is a path resolving to `<ClockType>::now`. The match uses
/// the definition path rather than the spelled path, so renamed imports
/// (`use Instant as Clock`) and type aliases (`type Clock = Instant`) are
/// recognized.
fn is_clock_now_path(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> bool {
    let ExprKind::Path(qpath) = &callee.kind else {
        return false;
    };
    let hir::def::Res::Def(_, def_id) = cx.typeck_results().qpath_res(qpath, callee.hir_id) else {
        return false;
    };
    matches!(
        cx.get_def_path(def_id).as_slice(),
        [.., owner, now] if now.as_str() == "now" && CLOCK_TYPES.contains(&owner.as_str())
    )
}

impl<'tcx> LateLintPass<'tcx> for AmbientClock {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        // Match function-style calls: `Instant::now()`, `Utc::now()`
        if let ExprKind::Call(func, _) = &expr.kind
            && is_clock_now_path(cx, func)
            && !test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
        {
            emit(cx, expr);
            return;
        }

        // Match method calls where we can resolve the impl type:
        // `chrono::Utc::now()` sometimes desugars to a method call
        if let ExprKind::MethodCall(segment, receiver, _, _) = &expr.kind
            && segment.ident.name.as_str() == "now"
            && let ty::Adt(adt, _) = cx.typeck_results().expr_ty(receiver).kind()
        {
            let name = cx.tcx.item_name(adt.did());
            let type_name = name.as_str();
            if CLOCK_TYPES.contains(&type_name)
                && !test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
            {
                emit(cx, expr);
            }
        }
    }
}

fn emit(cx: &LateContext<'_>, expr: &hir::Expr<'_>) {
    let suggestion = LintSuggestion::Single {
        span: expr.span,
        message: "inject time as a parameter instead of reading the system clock".to_owned(),
        replacement: "/* inject time parameter */".to_owned(),
        applicability: Applicability::HasPlaceholders,
    };
    emit_tiger_style_lint_with_suggestion(
        cx,
        AMBIENT_CLOCK,
        expr.span,
        "direct system clock read in production code",
        "ambient_clock",
        suggestion,
    );
}
