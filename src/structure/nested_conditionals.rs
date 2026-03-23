use rustc_ast::visit::{self, Visitor};
use rustc_ast::{Expr, ExprKind, Item, ItemKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};


declare_lint! {
    /// Detects `if`/`match` nesting deeper than 3 levels inside a function
    /// body. Deep nesting buries the important logic and makes control flow
    /// hard to follow. Flatten with early returns, guard clauses, or
    /// extracted helper functions.
    pub NESTED_CONDITIONALS,
    Warn,
    "conditional nesting exceeds 3 levels"
}

declare_lint_pass!(NestedConditionals => [NESTED_CONDITIONALS]);

const MAX_NESTING: usize = 3;

struct NestingVisitor<'a, 'b> {
    cx: &'a EarlyContext<'b>,
    depth: usize,
    reported: bool,
}

impl<'ast> Visitor<'ast> for NestingVisitor<'_, '_> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.reported {
            return;
        }

        let is_branching = matches!(
            expr.kind,
            ExprKind::If(..) | ExprKind::Match(..)
        );

        if is_branching {
            self.depth += 1;

            if self.depth > MAX_NESTING {
                self.cx.opt_span_lint(
                    NESTED_CONDITIONALS,
                    Some(expr.span),
                    DiagDecorator(|diag| {
                        diag.primary_message(format!(
                            "conditional nesting is {} levels deep (limit: {MAX_NESTING})",
                            self.depth,
                        ));
                        diag.help(
                            "flatten with early returns, guard clauses, \
                             or extract into a helper function",
                        );
                    }),
                );
                self.reported = true;
                self.depth -= 1;
                return;
            }

            visit::walk_expr(self, expr);
            self.depth -= 1;
        } else {
            visit::walk_expr(self, expr);
        }
    }
}

impl EarlyLintPass for NestedConditionals {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Fn(func) = &item.kind else {
            return;
        };

        let Some(body) = &func.body else {
            return;
        };

        let mut visitor = NestingVisitor {
            cx,
            depth: 0,
            reported: false,
        };
        visit::walk_block(&mut visitor, body);
    }
}
