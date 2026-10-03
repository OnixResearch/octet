use rustc_ast::Block;
use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::Local;
use rustc_ast::LocalKind;
use rustc_ast::StmtKind;
use rustc_ast::visit::{self, Visitor};
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Warns on functions exceeding a configurable line count (default: 20)
    /// that contain fewer than a required number of invariant checks (default: 2).
    /// Assertions and conditional error returns both make invariants visible.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// assertion_density_min_lines = 30
    /// assertion_density_min_assertions = 3
    /// ```
    pub tigerstyle::ASSERTION_DENSITY,
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

fn is_assertion_macro(snippet: &str) -> bool {
    let Some((path, args)) = snippet.trim_start().split_once('!') else {
        return false;
    };
    let name = path.trim_end().rsplit("::").next().unwrap_or(path);
    ASSERT_MACROS.contains(&name) && matches!(args.trim_start().chars().next(), Some('(' | '[' | '{'))
}

fn is_failure_return(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Paren(inner) => is_failure_return(inner),
        ExprKind::Call(callee, _) => {
            matches!(&callee.kind, ExprKind::Path(_, path)
                if path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == "Err"))
        }
        ExprKind::Path(_, path) => {
            path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == "None")
        }
        _ => false,
    }
}

struct CheckVisitor<'a, 'b> {
    cx: &'a EarlyContext<'b>,
    seen_macro_calls: Vec<Span>,
    checks: usize,
    in_guard: usize,
}

impl CheckVisitor<'_, '_> {
    fn count_macro_assertion(&mut self, expr: &Expr) {
        if !matches!(expr.kind, ExprKind::MacCall(..)) && !expr.span.from_expansion() {
            return;
        }
        // Expansion nodes share their authored invocation's callsite. Inspect
        // each invocation once, rather than counting its expanded child nodes.
        let callsite = expr.span.source_callsite();
        if self.seen_macro_calls.contains(&callsite) {
            return;
        }
        self.seen_macro_calls.push(callsite);
        if let Ok(snippet) = self.cx.sess().source_map().span_to_snippet(callsite)
            && is_assertion_macro(&snippet)
        {
            self.checks += 1;
        }
    }
}

impl<'ast> Visitor<'ast> for CheckVisitor<'_, '_> {
    // Local items and closures have their own bodies and invariants.
    fn visit_item(&mut self, _item: &'ast Item) {}

    fn visit_local(&mut self, local: &'ast Local) {
        if let LocalKind::InitElse(init, else_block) = &local.kind {
            self.visit_expr(init);
            self.in_guard += 1;
            visit::walk_block(self, else_block);
            self.in_guard -= 1;
        } else {
            visit::walk_local(self, local);
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        if matches!(expr.kind, ExprKind::Closure(..) | ExprKind::Gen(..) | ExprKind::ConstBlock(..)) {
            return;
        }

        self.count_macro_assertion(expr);
        match &expr.kind {
            ExprKind::Ret(Some(value))
                if self.in_guard > 0 && !expr.span.from_expansion() && is_failure_return(value) =>
            {
                self.checks += 1;
            }
            ExprKind::If(condition, then_block, else_expr) => {
                self.visit_expr(condition);
                let conditional = !matches!(condition.kind, ExprKind::Lit(..));
                self.in_guard += usize::from(conditional);
                visit::walk_block(self, then_block);
                if let Some(else_expr) = else_expr {
                    self.visit_expr(else_expr);
                }
                self.in_guard -= usize::from(conditional);
                return;
            }
            ExprKind::Match(scrutinee, arms, _) => {
                self.visit_expr(scrutinee);
                let conditional = arms.len() > 1;
                self.in_guard += usize::from(conditional);
                for arm in arms {
                    self.visit_arm(arm);
                }
                self.in_guard -= usize::from(conditional);
                return;
            }
            _ => {}
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
    if body.stmts.len() == 1
        && let StmtKind::Expr(_) | StmtKind::Semi(_) = &body.stmts[0].kind
    {
        return true;
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

        let mut visitor = CheckVisitor {
            cx,
            seen_macro_calls: Vec::new(),
            checks: 0,
            in_guard: 0,
        };
        visit::walk_block(&mut visitor, body);
        if visitor.checks >= self.min_assertions {
            return;
        }

        let min_a = self.min_assertions;
        emit_tiger_style_lint(
            cx,
            ASSERTION_DENSITY,
            body.span,
            format!("function `{}` has {} invariant check(s) in {lines} lines (need {min_a}+)", func.ident.name, visitor.checks),
            "assertion_density",
        );
    }
}
