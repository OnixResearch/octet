use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_lint! {
    /// Detects `let` bindings of type `bool` whose name does not start with
    /// a predicate prefix (`is_`, `has_`, `should_`, `can_`, `was_`, `will_`,
    /// `needs_`). Boolean variables without such prefixes read ambiguously —
    /// `active` could be a flag, a count, an enum variant.
    pub BOOL_NAMING,
    Warn,
    "boolean binding should have a predicate prefix"
}

declare_lint_pass!(BoolNaming => [BOOL_NAMING]);

const BOOL_PREFIXES: &[&str] = &["is_", "has_", "should_", "can_", "was_", "will_", "needs_"];

impl<'tcx> LateLintPass<'tcx> for BoolNaming {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        // Only check simple ident bindings, not destructuring
        let PatKind::Binding(_, _, ident, _) = &local.pat.kind else {
            return;
        };

        let name = ident.name.as_str();

        // Skip underscore-prefixed bindings
        if name.starts_with('_') {
            return;
        }

        // Skip if already has a predicate prefix
        if BOOL_PREFIXES.iter().any(|p| name.starts_with(p)) {
            return;
        }

        // Check if the binding type is bool
        let ty = cx.typeck_results().pat_ty(local.pat);
        if !ty.is_bool() {
            return;
        }

        emit_tiger_style_lint(
            cx,
            BOOL_NAMING,
            ident.span,
            format!("boolean binding `{name}` should have a predicate prefix"),
            "bool_naming",
        );
    }
}
