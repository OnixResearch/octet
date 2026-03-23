use rustc_ast::{Item, ItemKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Warns on functions exceeding 20 lines that contain fewer than 2
    /// assertion macros. Functions without assertions lack documentation
    /// of their invariants.
    pub ASSERTION_DENSITY,
    Allow,
    "function has low assertion density"
}

declare_lint_pass!(AssertionDensity => [ASSERTION_DENSITY]);

const MIN_LINES: usize = 20;
const MIN_ASSERTIONS: usize = 2;

fn count_code_lines(source: &str) -> usize {
    source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .count()
}

fn count_assertions(source: &str) -> usize {
    let assertion_macros = [
        "assert!",
        "assert_eq!",
        "assert_ne!",
        "debug_assert!",
        "debug_assert_eq!",
        "debug_assert_ne!",
    ];
    assertion_macros
        .iter()
        .map(|m| source.matches(m).count())
        .sum()
}

impl EarlyLintPass for AssertionDensity {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Fn(func) = &item.kind else {
            return;
        };

        // Skip test functions
        if item.attrs.iter().any(|a| a.has_name(sym::test)) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let lines = count_code_lines(&snippet);
        if lines < MIN_LINES {
            return;
        }

        let assertions = count_assertions(&snippet);
        if assertions >= MIN_ASSERTIONS {
            return;
        }

        cx.opt_span_lint(
            ASSERTION_DENSITY,
            Some(body.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "function `{}` has {assertions} assertion(s) in {lines} lines",
                    func.ident.name,
                ));
                diag.help(
                    "add assertions documenting the function's invariants",
                );
            }),
        );
    }
}
