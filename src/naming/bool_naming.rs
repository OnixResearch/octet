use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

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

const BOOL_PREFIXES: &[&str] = &[
    "is_", "has_", "should_", "can_", "was_", "will_", "needs_",
];

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

        cx.opt_span_lint(
            BOOL_NAMING,
            Some(ident.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "boolean binding `{name}` should have a predicate prefix",
                ));
                diag.help(format!(
                    "rename to `is_{name}` or use another prefix: \
                     has_, should_, can_, was_, will_, needs_",
                ));
            }),
        );
    }
}
