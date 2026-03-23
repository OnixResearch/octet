use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::Span;

declare_lint! {
    /// Detects functions that both accept a time parameter (name matching
    /// `*_ms`, `*_secs`, `now`, `timestamp`, etc.) AND call
    /// `Instant::now()` or `SystemTime::now()` in their body. Two time
    /// sources in the same function is always a design error — verified-logic
    /// functions take time as an explicit parameter for purity and testability.
    pub CONTRADICTORY_TIME,
    Warn,
    "function takes time parameter but also reads system time"
}

declare_lint_pass!(ContradictoryTime => [CONTRADICTORY_TIME]);

/// Time parameter name patterns.
const TIME_SUFFIXES: &[&str] = &["_ms", "_secs", "_us", "_ns"];
const TIME_NAMES: &[&str] = &[
    "now",
    "timestamp",
    "current_time",
    "elapsed",
    "now_ms",
    "now_secs",
];

fn is_time_param_name(name: &str) -> bool {
    TIME_NAMES.contains(&name) || TIME_SUFFIXES.iter().any(|s| name.ends_with(s))
}

/// Visitor that checks if a function body contains `Instant::now()` or
/// `SystemTime::now()` calls.
struct TimeCallFinder {
    found: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for TimeCallFinder {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let ExprKind::Call(func, _) | ExprKind::MethodCall(_, func, _, _) = &expr.kind {
            // Check for method calls like `Instant::now()`, `SystemTime::now()`
            if let ExprKind::Path(qpath) = &func.kind {
                let path_str = path_to_string(qpath);
                if path_str.ends_with("::now") {
                    self.found = Some(expr.span);
                    return;
                }
            }
        }

        // Also check method calls directly
        if let ExprKind::MethodCall(segment, _, _, _) = &expr.kind {
            if segment.ident.name.as_str() == "now" {
                self.found = Some(expr.span);
                return;
            }
        }

        intravisit::walk_expr(self, expr);
    }
}

fn path_to_string(qpath: &hir::QPath<'_>) -> String {
    match qpath {
        hir::QPath::Resolved(_, path) => path
            .segments
            .iter()
            .map(|s| s.ident.name.as_str())
            .collect::<Vec<_>>()
            .join("::"),
        hir::QPath::TypeRelative(_, segment) => segment.ident.name.to_string(),
    }
}

impl<'tcx> LateLintPass<'tcx> for ContradictoryTime {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        _span: Span,
        _def_id: rustc_hir::def_id::LocalDefId,
    ) {
        // Check if any parameter has a time-related name
        let has_time_param = body.params.iter().any(|param| {
            if let hir::PatKind::Binding(_, _, ident, _) = &param.pat.kind {
                is_time_param_name(ident.name.as_str())
            } else {
                false
            }
        });

        if !has_time_param {
            return;
        }

        // Walk the body looking for .now() calls
        let mut finder = TimeCallFinder { found: None };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
            let time_params: Vec<&str> = body
                .params
                .iter()
                .filter_map(|p| {
                    if let hir::PatKind::Binding(_, _, ident, _) = &p.pat.kind {
                        let name = ident.name.as_str();
                        if is_time_param_name(name) {
                            return Some(name);
                        }
                    }
                    None
                })
                .collect();

            let param_list = time_params.join("`, `");

            cx.opt_span_lint(
                CONTRADICTORY_TIME,
                Some(call_span),
                DiagDecorator(|diag| {
                    diag.primary_message(format!(
                        "function takes time parameter `{param_list}` but also reads system time",
                    ));
                    diag.help(
                        "use the time parameter instead of reading the system clock, \
                         or remove the parameter if this function should own its time source",
                    );
                }),
            );
        }
    }
}
