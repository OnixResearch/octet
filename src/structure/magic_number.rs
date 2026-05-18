use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects inline numeric literals that should be extracted into named
    /// constants, configuration fields, or domain wrappers.
    pub tigerstyle::MAGIC_NUMBER,
    Warn,
    "inline numeric literal should be named or configured"
}

declare_lint_pass!(MagicNumber => [MAGIC_NUMBER]);

const UNIT_SUFFIXES: &[&str] = &[
    "_ms", "_millis", "_secs", "_sec", "_us", "_micros", "_ns", "_nanos", "_bytes", "_kb", "_mb", "_gb",
];

const DOMAIN_WORDS: &[&str] = &[
    "timeout",
    "deadline",
    "delay",
    "retry",
    "reconnect",
    "capacity",
    "limit",
    "threshold",
    "window",
    "batch",
    "port",
];

fn has_unit_suffix(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    UNIT_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix))
}

fn has_domain_word(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    DOMAIN_WORDS.iter().any(|word| lower.contains(word))
}

fn is_conventional_literal(lit: &hir::Lit) -> bool {
    match lit.node {
        LitKind::Int(value, _) => matches!(value.get(), 0 | 1),
        LitKind::Float(symbol, _) => matches!(symbol.as_str(), "0.0" | "1.0"),
        _ => true,
    }
}

fn literal_is_documented_by_parent(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::LetStmt(local) => {
                if let hir::PatKind::Binding(_, _, ident, _) = local.pat.kind {
                    let name = ident.name.as_str();
                    if has_unit_suffix(name) || has_domain_word(name) {
                        return true;
                    }
                }
            }
            hir::Node::Expr(parent) => match parent.kind {
                ExprKind::Call(_, args) if args.iter().any(|arg| arg.hir_id == expr.hir_id) => return true,
                ExprKind::MethodCall(_, receiver, args, _) => {
                    if receiver.hir_id == expr.hir_id || args.iter().any(|arg| arg.hir_id == expr.hir_id) {
                        return true;
                    }
                }
                ExprKind::Unary(hir::UnOp::Neg, _) => {}
                _ => {}
            },
            hir::Node::Item(item) => {
                if matches!(item.kind, hir::ItemKind::Const(..) | hir::ItemKind::Static(..)) {
                    return true;
                }
                break;
            }
            hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    false
}

fn other_expr_has_domain_name(expr: &hir::Expr<'_>) -> bool {
    match expr.kind {
        ExprKind::Path(hir::QPath::Resolved(_, path)) => path.segments.last().is_some_and(|segment| {
            let name = segment.ident.name.as_str();
            has_unit_suffix(name) || has_domain_word(name)
        }),
        _ => false,
    }
}

fn literal_is_policy_threshold_comparison(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id).take(3) {
        let hir::Node::Expr(parent) = node else {
            continue;
        };
        let ExprKind::Binary(op, lhs, rhs) = parent.kind else {
            continue;
        };
        if !matches!(
            op.node,
            hir::BinOpKind::Lt
                | hir::BinOpKind::Le
                | hir::BinOpKind::Gt
                | hir::BinOpKind::Ge
                | hir::BinOpKind::Eq
                | hir::BinOpKind::Ne
        ) {
            return false;
        }
        if lhs.hir_id == expr.hir_id {
            return other_expr_has_domain_name(rhs);
        }
        if rhs.hir_id == expr.hir_id {
            return other_expr_has_domain_name(lhs);
        }
    }
    false
}

impl LateLintPass<'_> for MagicNumber {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &hir::Expr<'_>) {
        let ExprKind::Lit(lit) = expr.kind else {
            return;
        };

        if expr.span.from_expansion()
            || test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
            || is_conventional_literal(&lit)
            || literal_is_documented_by_parent(cx, expr)
            || !literal_is_policy_threshold_comparison(cx, expr)
        {
            return;
        }

        if !matches!(lit.node, LitKind::Int(..) | LitKind::Float(..)) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            MAGIC_NUMBER,
            expr.span,
            "inline numeric literal should be extracted into a named constant or configuration field",
            "magic_number",
        );
    }
}
