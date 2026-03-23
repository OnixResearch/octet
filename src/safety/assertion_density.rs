use rustc_ast::{Item, ItemKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Warns on functions exceeding a configurable line count (default: 20)
    /// that contain fewer than a required number of assertions (default: 2).
    /// Functions without assertions lack documentation of their invariants.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [tigerstyle]
    /// assertion_density_min_lines = 30
    /// assertion_density_min_assertions = 3
    /// ```
    pub ASSERTION_DENSITY,
    Allow,
    "function has low assertion density"
}

pub struct AssertionDensity {
    min_lines: usize,
    min_assertions: usize,
}

impl AssertionDensity {
    pub fn new(min_lines: usize, min_assertions: usize) -> Self {
        Self {
            min_lines,
            min_assertions,
        }
    }
}

impl_lint_pass!(AssertionDensity => [ASSERTION_DENSITY]);

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
        if lines < self.min_lines {
            return;
        }

        let assertions = count_assertions(&snippet);
        if assertions >= self.min_assertions {
            return;
        }

        let min_a = self.min_assertions;
        cx.opt_span_lint(
            ASSERTION_DENSITY,
            Some(body.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "function `{}` has {assertions} assertion(s) in {lines} lines (need {min_a}+)",
                    func.ident.name,
                ));
                diag.help(
                    "add assertions documenting the function's invariants",
                );
            }),
        );
    }
}
