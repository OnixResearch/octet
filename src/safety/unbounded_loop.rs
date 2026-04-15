use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::intravisit;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;
use rustc_span::Span;
use rustc_span::Symbol;

declare_lint! {
    /// Detects `loop { }` blocks that appear to lack an iteration bound.
    /// Unbounded loops are a class of liveness bug — a retry loop without
    /// a max-attempts counter can stall a system indefinitely.
    pub UNBOUNDED_LOOP,
    Warn,
    "loop without visible iteration bound"
}

declare_lint_pass!(UnboundedLoop => [UNBOUNDED_LOOP]);

const COUNTER_MARKERS: &[&str] = &["MAX_RETRIES", "max_retries", "max_attempts", "attempt", "remaining"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokioReceiverKind {
    Bounded,
    Unbounded,
}

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn normalized_without_whitespace(snippet: &str) -> String {
    snippet.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn loop_has_visible_counter(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let Ok(snippet) = cx.sess().source_map().span_to_snippet(expr.span.source_callsite()) else {
        return false;
    };

    COUNTER_MARKERS.iter().any(|marker| snippet.contains(marker))
}

fn span_comes_from_tokio_select(cx: &LateContext<'_>, span: Span) -> bool {
    if !span.from_expansion() {
        return false;
    }

    let Ok(snippet) = cx.sess().source_map().span_to_snippet(span.source_callsite()) else {
        return false;
    };
    let normalized = normalized_without_whitespace(&snippet);
    if !normalized.starts_with("tokio::select!") && !normalized.starts_with("select!") {
        return false;
    }

    span.macro_backtrace().any(|expn| {
        let Some(def_id) = expn.macro_def_id else {
            return false;
        };
        let Some(crate_name) = cx.get_def_path(def_id).first().copied() else {
            return false;
        };
        matches!(crate_name.as_str(), "tokio" | "tokio_macros")
    })
}

struct TokioSelectVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for TokioSelectVisitor<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }

        if span_comes_from_tokio_select(self.cx, expr.span) {
            self.found = true;
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

fn block_contains_tokio_select<'tcx>(cx: &LateContext<'tcx>, block: &'tcx hir::Block<'tcx>) -> bool {
    let mut visitor = TokioSelectVisitor { cx, found: false };
    visitor.visit_block(block);
    visitor.found
}

fn while_let_header_snippet(cx: &LateContext<'_>, header_span: Span) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(header_span.source_callsite()).ok()?;
    Some(normalized_without_whitespace(&snippet))
}

fn header_is_recv_while_let(cx: &LateContext<'_>, header_span: Span) -> bool {
    let Some(normalized) = while_let_header_snippet(cx, header_span) else {
        return false;
    };
    normalized.starts_with("whileletSome(") && normalized.contains(".recv().await")
}

fn header_is_iter_next_while_let(cx: &LateContext<'_>, header_span: Span) -> bool {
    let Some(normalized) = while_let_header_snippet(cx, header_span) else {
        return false;
    };
    normalized.starts_with("whileletSome(") && normalized.contains(".next()")
}

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            hir::ExprKind::DropTemps(inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    break expr;
                };
                expr = inner;
            }
            _ => break expr,
        }
    }
}

fn recv_receiver_from_expr<'tcx>(expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let expr = peel_transparent_expr(expr);

    let hir::ExprKind::Match(scrutinee, _, hir::MatchSource::AwaitDesugar) = &expr.kind else {
        return None;
    };
    let hir::ExprKind::Call(_, [future_expr]) = &scrutinee.kind else {
        return None;
    };
    let hir::ExprKind::MethodCall(segment, receiver, args, _) = &future_expr.kind else {
        return None;
    };

    (segment.ident.name.as_str() == "recv" && args.is_empty()).then_some(*receiver)
}

fn while_let_recv_receiver<'tcx>(expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let hir::ExprKind::Loop(block, _, hir::LoopSource::While, _) = &expr.kind else {
        return None;
    };
    let condition = block.expr?;
    let hir::ExprKind::If(cond, _, Some(_)) = &condition.kind else {
        return None;
    };
    let hir::ExprKind::Let(let_expr) = &cond.kind else {
        return None;
    };

    recv_receiver_from_expr(let_expr.init)
}

fn tokio_receiver_kind(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<TokioReceiverKind> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).peel_refs().kind() else {
        return None;
    };

    if match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "Receiver"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "bounded", "Receiver"])
    {
        Some(TokioReceiverKind::Bounded)
    } else if match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "UnboundedReceiver"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "unbounded", "UnboundedReceiver"])
    {
        Some(TokioReceiverKind::Unbounded)
    } else {
        None
    }
}

fn lint_unbounded_loop(cx: &LateContext<'_>, span: Span) {
    cx.opt_span_lint(
        UNBOUNDED_LOOP,
        Some(span),
        DiagDecorator(|diag| {
            diag.primary_message("loop without visible iteration bound");
            diag.help("add a counter or max-iterations guard, or use `for _ in 0..MAX_RETRIES` instead");
        }),
    );
}

impl<'tcx> LateLintPass<'tcx> for UnboundedLoop {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if expr.span.from_expansion() {
            return;
        }

        let hir::ExprKind::Loop(body, _, loop_source, header_span) = &expr.kind else {
            return;
        };

        match loop_source {
            hir::LoopSource::Loop => {
                if block_contains_tokio_select(cx, body) || loop_has_visible_counter(cx, expr) {
                    return;
                }
            }
            hir::LoopSource::While => {
                if header_is_recv_while_let(cx, *header_span) {
                    if while_let_recv_receiver(expr).and_then(|receiver| tokio_receiver_kind(cx, receiver))
                        == Some(TokioReceiverKind::Bounded)
                    {
                        return;
                    }
                } else if header_is_iter_next_while_let(cx, *header_span) {
                    if loop_has_visible_counter(cx, expr) {
                        return;
                    }
                } else {
                    return;
                }
            }
            hir::LoopSource::ForLoop => return,
        }

        lint_unbounded_loop(cx, expr.span.source_callsite());
    }
}
