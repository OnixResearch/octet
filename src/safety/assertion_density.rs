use rustc_hir as hir;
use rustc_hir::def::CtorOf;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_hir::intravisit::FnKind;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Warns on functions exceeding a configurable line count (default: 20)
    /// that contain fewer than a required number of syntactic fail-closed
    /// checks (default: 2). This does not prove domain invariant coverage.
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

fn is_core_failure_variant(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    enum_name: rustc_span::Symbol,
    variant: &str,
) -> bool {
    let hir::ExprKind::Path(qpath) = &expr.kind else {
        return false;
    };
    let variant_id = match cx.qpath_res(qpath, expr.hir_id) {
        Res::Def(DefKind::Variant, did) => did,
        Res::Def(DefKind::Ctor(CtorOf::Variant, _), did) => {
            let Some(parent) = cx.tcx.opt_parent(did) else {
                return false;
            };
            parent
        }
        _ => return false,
    };
    cx.tcx.item_name(variant_id).as_str() == variant
        && cx.tcx.opt_parent(variant_id).is_some_and(|enum_id| cx.tcx.is_diagnostic_item(enum_name, enum_id))
}

fn is_failure_return(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        hir::ExprKind::DropTemps(inner) => is_failure_return(cx, inner),
        hir::ExprKind::Call(callee, _) => is_core_failure_variant(cx, callee, sym::Result, "Err"),
        hir::ExprKind::Path(..) => is_core_failure_variant(cx, expr, sym::Option, "None"),
        _ => false,
    }
}

fn is_core_result_type(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Adt(adt, _) if cx.tcx.is_diagnostic_item(sym::Result, adt.did()))
}

fn is_literal_ok(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        hir::ExprKind::DropTemps(inner) => is_literal_ok(cx, inner),
        hir::ExprKind::Call(callee, _) => is_core_failure_variant(cx, callee, sym::Result, "Ok"),
        _ => false,
    }
}

fn is_authored_result_try_exit(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    scrutinee: &hir::Expr<'_>,
    arms: &[hir::Arm<'_>],
    branch_id: hir::HirId,
) -> bool {
    // The pinned HIR lowers `operand?` to `match Try::branch(operand)`.
    // The TryDesugar id names the branch call, NOT the original operand.
    if expr.span.from_expansion() || scrutinee.hir_id != branch_id || arms.len() != 2 {
        return false;
    }
    // Pinned lowering constructs Continue first but stores [Break, Continue].
    // The Break arm returns from the function, but breaks locally in `try {}`.
    let break_arm = &arms[0];
    let hir::PatKind::Struct(qpath, ..) = break_arm.pat.kind else {
        return false;
    };
    if !matches!(cx.qpath_res(&qpath, break_arm.pat.hir_id),
        Res::Def(DefKind::Variant, did) if cx.tcx.is_lang_item(did, hir::LangItem::ControlFlowBreak))
        || !matches!(break_arm.body.kind, hir::ExprKind::Ret(Some(_)))
    {
        return false;
    }
    let hir::ExprKind::Call(_, [operand]) = scrutinee.kind else {
        return false;
    };
    is_core_result_type(cx, cx.typeck_results().expr_ty(operand)) && !is_literal_ok(cx, operand)
}

struct CheckVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    seen_macro_calls: Vec<Span>,
    checks: usize,
    in_guard: usize,
    returns_result: bool,
}

impl CheckVisitor<'_, '_> {
    fn count_macro_assertion(&mut self, expr: &hir::Expr<'_>) {
        if !expr.span.from_expansion() {
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

impl<'tcx> Visitor<'tcx> for CheckVisitor<'_, 'tcx> {
    fn visit_stmt(&mut self, stmt: &'tcx hir::Stmt<'tcx>) {
        if let hir::StmtKind::Let(local) = stmt.kind
            && let Some(else_block) = local.els
        {
            if let Some(init) = local.init {
                self.visit_expr(init);
            }
            self.in_guard += 1;
            self.visit_block(else_block);
            self.in_guard -= 1;
        } else {
            intravisit::walk_stmt(self, stmt);
        }
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if matches!(expr.kind, hir::ExprKind::Closure(..) | hir::ExprKind::ConstBlock(..)) {
            return;
        }

        self.count_macro_assertion(expr);
        match &expr.kind {
            hir::ExprKind::Ret(Some(value))
                if self.in_guard > 0 && !expr.span.from_expansion() && is_failure_return(self.cx, value) =>
            {
                self.checks += 1;
            }
            hir::ExprKind::If(condition, then_expr, else_expr) => {
                self.visit_expr(condition);
                let conditional = !matches!(condition.kind, hir::ExprKind::Lit(..));
                self.in_guard += usize::from(conditional);
                self.visit_expr(then_expr);
                if let Some(else_expr) = else_expr {
                    self.visit_expr(else_expr);
                }
                self.in_guard -= usize::from(conditional);
                return;
            }
            hir::ExprKind::Match(scrutinee, arms, source) => {
                if self.returns_result
                    && let hir::MatchSource::TryDesugar(branch_id) = source
                    && is_authored_result_try_exit(self.cx, expr, scrutinee, arms, *branch_id)
                {
                    self.checks += 1;
                }
                self.visit_expr(scrutinee);
                let conditional = arms.len() > 1;
                self.in_guard += usize::from(conditional);
                for arm in *arms {
                    self.visit_arm(arm);
                }
                self.in_guard -= usize::from(conditional);
                return;
            }
            _ => {}
        }
        intravisit::walk_expr(self, expr);
    }
}

/// Returns true if the block is a single expression or a trivial delegation
/// like `self.inner.method(args)`.
fn is_single_expression_body(body: &hir::Block<'_>) -> bool {
    // A block with just one expression (no statements, just the tail expr)
    if body.stmts.is_empty() {
        return true;
    }
    // A block with one statement that's an expression
    if body.stmts.len() == 1
        && let hir::StmtKind::Expr(_) | hir::StmtKind::Semi(_) = &body.stmts[0].kind
    {
        return true;
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for AssertionDensity {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        let FnKind::ItemFn(name, _, _) = kind else {
            return;
        };

        if test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }

        let hir::ExprKind::Block(block, _) = &body.value.kind else {
            return;
        };

        // Skip single-expression bodies
        if is_single_expression_body(block) {
            return;
        }

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.value.span) else {
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
            returns_result: is_core_result_type(
                cx,
                cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder().output(),
            ),
        };
        visitor.visit_block(block);
        if visitor.checks >= self.min_assertions {
            return;
        }

        let min_a = self.min_assertions;
        emit_tiger_style_lint(
            cx,
            ASSERTION_DENSITY,
            body.value.span,
            format!(
                "function `{name}` has {} syntactic fail-closed check(s) in {lines} lines (need {min_a}+)",
                visitor.checks
            ),
            "assertion_density",
        );
    }
}
