use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "concrete_construction_in_core";
const NEW_CALL_PATTERN: &str = "::new(";
const DEFAULT_CALL_PATTERN: &str = "::default(";
const STD_PREFIX: &str = "std::";
const CORE_PREFIX: &str = "core::";

declare_tool_lint! {
    /// Detects concrete infrastructure construction inside declared core scopes.
    /// Core code should accept injected ports/capabilities instead of constructing
    /// external concrete dependencies directly.
    pub tigerstyle::CONCRETE_CONSTRUCTION_IN_CORE,
    Warn,
    "concrete dependency constructed in core scope"
}

pub struct ConcreteConstructionInCore {
    core_source_scopes: Vec<String>,
}

impl ConcreteConstructionInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl_lint_pass!(ConcreteConstructionInCore => [CONCRETE_CONSTRUCTION_IN_CORE]);

fn path_is_in_core_scope(cx: &LateContext<'_>, span: rustc_span::Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let filename = cx.sess().source_map().span_to_filename(span);
    let path = filename.prefer_local_unconditionally().to_string();
    scopes.iter().any(|scope| path.contains(scope))
}

fn construction_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(expr.span).ok()?;
    if snippet.contains(NEW_CALL_PATTERN) || snippet.contains(DEFAULT_CALL_PATTERN) {
        Some(snippet)
    } else {
        None
    }
}

fn external_construction_name(snippet: &str) -> Option<String> {
    if snippet.starts_with(STD_PREFIX) || snippet.starts_with(CORE_PREFIX) {
        return None;
    }
    let separator = if snippet.contains(NEW_CALL_PATTERN) {
        NEW_CALL_PATTERN
    } else if snippet.contains(DEFAULT_CALL_PATTERN) {
        DEFAULT_CALL_PATTERN
    } else {
        return None;
    };
    snippet
        .split(separator)
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

impl<'tcx> LateLintPass<'tcx> for ConcreteConstructionInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if expr.span.from_expansion()
            || !path_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
        {
            return;
        }

        let Some(snippet) = construction_snippet(cx, expr) else {
            return;
        };
        let Some(type_name) = external_construction_name(&snippet) else {
            return;
        };

        emit_tiger_style_lint(
            cx,
            CONCRETE_CONSTRUCTION_IN_CORE,
            expr.span,
            format!(
                "core scope constructs concrete type `{type_name}`; inject the dependency through a port or capability"
            ),
            LINT_NAME,
        );
    }
}
