use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::visit::Visitor;
use rustc_ast::visit::{self};
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects `if`/`match` nesting deeper than a configurable limit
    /// (default: 3) inside a function body. Deep nesting buries the
    /// important logic and makes control flow hard to follow. An
    /// `else if` continues its chain at the same depth.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// nesting_max_depth = 4
    /// ```
    pub tigerstyle::NESTED_CONDITIONALS,
    Warn,
    "conditional nesting exceeds depth limit"
}

pub struct NestedConditionals {
    max_depth: usize,
}

impl NestedConditionals {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }
}

impl_lint_pass!(NestedConditionals => [NESTED_CONDITIONALS]);

struct NestingVisitor<'a, 'b> {
    cx: &'a EarlyContext<'b>,
    depth: usize,
    max_depth: usize,
    reported: bool,
}

impl NestingVisitor<'_, '_> {
    /// Enters one branching level; returns `false` (after reporting) when the
    /// new depth exceeds the limit.
    fn enter_branch(&mut self, expr: &Expr) -> bool {
        self.depth += 1;
        if self.depth <= self.max_depth {
            return true;
        }

        let depth = self.depth;
        let max = self.max_depth;
        emit_tiger_style_lint(
            self.cx,
            NESTED_CONDITIONALS,
            expr.span,
            format!("conditional nesting is {depth} levels deep (limit: {max})"),
            "nested_conditionals",
        );
        self.reported = true;
        self.depth -= 1;
        false
    }
}

impl<'ast> Visitor<'ast> for NestingVisitor<'_, '_> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.reported {
            return;
        }

        match &expr.kind {
            ExprKind::If(cond, then_block, else_expr) => {
                if !self.enter_branch(expr) {
                    return;
                }
                self.visit_expr(cond);
                self.visit_block(then_block);
                self.depth -= 1;

                let Some(else_expr) = else_expr else {
                    return;
                };
                if matches!(else_expr.kind, ExprKind::If(..)) {
                    // `else if` continues the same chain at the current depth.
                    self.visit_expr(else_expr);
                } else {
                    self.depth += 1;
                    self.visit_expr(else_expr);
                    self.depth -= 1;
                }
            }
            ExprKind::Match(..) => {
                if !self.enter_branch(expr) {
                    return;
                }
                visit::walk_expr(self, expr);
                self.depth -= 1;
            }
            _ => visit::walk_expr(self, expr),
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
            max_depth: self.max_depth,
            reported: false,
        };
        visit::walk_block(&mut visitor, body);
    }
}
