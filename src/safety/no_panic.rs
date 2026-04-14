use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_errors::DiagDecorator;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint;
use rustc_session::impl_lint_pass;

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
pub struct NoPanic {
    test_context_depth: usize,
}

impl NoPanic {
    fn in_test_context(&self) -> bool {
        self.test_context_depth > 0
    }

    fn enter_test_context(&mut self) {
        self.test_context_depth += 1;
    }

    fn leave_test_context(&mut self) {
        debug_assert!(self.test_context_depth > 0);
        self.test_context_depth -= 1;
    }
}

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

impl EarlyLintPass for NoPanic {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if test_context::ast_attrs_indicate_test(cx.sess(), &item.attrs)
            || test_context::item_span_indicates_test(cx.sess(), item.span)
        {
            self.enter_test_context();
        }
    }

    fn check_item_post(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if test_context::ast_attrs_indicate_test(cx.sess(), &item.attrs)
            || test_context::item_span_indicates_test(cx.sess(), item.span)
        {
            self.leave_test_context();
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Panic macros are expanded before EarlyLintPass runs. We recover the
        // original macro invocation from the expansion callsite snippet.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded block expressions to avoid duplicate diagnostics on
        // nested nodes that share the same callsite.
        if !matches!(expr.kind, ExprKind::Block(..)) {
            return;
        }

        if self.in_test_context() {
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
