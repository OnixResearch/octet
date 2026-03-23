use rustc_ast::visit::{self, Visitor};
use rustc_ast::{Block, Expr, ExprKind, Item, ItemKind, StmtKind};
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

const ASSERT_MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
];

/// Walk the AST counting assertion macro invocations (not text matching).
struct AssertionCounter {
    count: usize,
}

impl<'ast> Visitor<'ast> for AssertionCounter {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let ExprKind::MacCall(mac) = &expr.kind {
            if let Some(last) = mac.path.segments.last() {
                if ASSERT_MACROS.contains(&last.ident.name.as_str()) {
                    self.count += 1;
                }
            }
        }
        visit::walk_expr(self, expr);
    }
}

/// Returns true if the block is a single expression or a trivial delegation
/// like `self.inner.method(args)`.
fn is_single_expression_body(body: &Block) -> bool {
    // A block with just one expression (no statements, just the tail expr)
    if body.stmts.is_empty() {
        return true;
    }
    // A block with one statement that's an expression
    if body.stmts.len() == 1 {
        if let StmtKind::Expr(_) | StmtKind::Semi(_) = &body.stmts[0].kind {
            return true;
        }
    }
    false
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

        // Skip #[cfg(test)] items
        if item.attrs.iter().any(|attr| {
            if attr.has_name(sym::cfg) {
                let source = format!("{attr:?}");
                source.contains("test")
            } else {
                false
            }
        }) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        // Skip single-expression bodies
        if is_single_expression_body(body) {
            return;
        }

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let lines = count_code_lines(&snippet);
        if lines < self.min_lines {
            return;
        }

        // Count assertions via AST walk, not text search
        let mut counter = AssertionCounter { count: 0 };
        visit::walk_block(&mut counter, body);

        if counter.count >= self.min_assertions {
            return;
        }

        let min_a = self.min_assertions;
        let assertions = counter.count;
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
