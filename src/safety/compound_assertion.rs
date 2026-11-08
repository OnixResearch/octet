use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `assert!(a && b)` and `assert!(a || b)` where the macro
    /// argument contains a top-level `&&` or `||`. Compound assertions
    /// produce a single "assertion failed" message on failure, hiding
    /// which condition actually broke.
    pub tigerstyle::COMPOUND_ASSERTION,
    Warn,
    "compound assertion should be split into separate assertions"
}

#[derive(Default)]
pub struct CompoundAssertion {
    /// Expression of the statement entered most recently. The early lint visitor
    /// runs `check_stmt` immediately before `check_expr` on that expression, so a
    /// matching id means the expanded `assert!` sits in statement position.
    statement_expr: Option<rustc_ast::NodeId>,
}

impl_lint_pass!(CompoundAssertion => [COMPOUND_ASSERTION]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TopLevelBooleanOp {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TopLevelAssertionSplit {
    op: TopLevelBooleanOp,
    clauses: Vec<String>,
    has_trailing_semicolon: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TopLevelToken {
    Op(TopLevelBooleanOp),
    Comma,
}

/// Whether the text after an opening `'` starts a char literal rather than a lifetime.
fn is_char_literal_start(rest: &str) -> bool {
    let mut chars = rest.chars();
    match chars.next() {
        Some('\\') => true,
        Some(_) => chars.next() == Some('\''),
        None => false,
    }
}

/// Scan for top-level `&&`, `||`, and `,` tokens outside nested delimiters and
/// string/char literals. Scanning stops at the first unmatched closing delimiter,
/// which ends a macro's arguments when `text` starts right after its opening `(`.
/// Returns the tokens with their byte offsets plus the offset where scanning
/// stopped, or `None` when a literal or delimiter is left open.
fn scan_top_level_tokens(text: &str) -> Option<(Vec<(usize, TopLevelToken)>, usize)> {
    let mut depth = 0usize;
    let mut tokens = Vec::new();
    let mut chars = text.char_indices().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while let Some((idx, ch)) = chars.next() {
        if in_string || in_char {
            if escaped {
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' if in_string => in_string = false,
                    '\'' if in_char => in_char = false,
                    _ => {}
                }
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '\'' => in_char = is_char_literal_start(&text[idx + 1..]),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                let Some(outer) = depth.checked_sub(1) else {
                    return Some((tokens, idx));
                };
                depth = outer;
            }
            ',' if depth == 0 => tokens.push((idx, TopLevelToken::Comma)),
            '&' | '|' if depth == 0 && chars.peek().is_some_and(|&(_, next)| next == ch) => {
                chars.next();
                let op = if ch == '&' {
                    TopLevelBooleanOp::And
                } else {
                    TopLevelBooleanOp::Or
                };
                tokens.push((idx, TopLevelToken::Op(op)));
            }
            _ => {}
        }
    }

    (!in_string && !in_char && depth == 0).then_some((tokens, text.len()))
}

/// Scan a macro-call snippet for top-level `&&` or `||` in the condition of `assert!(...)`.
///
/// We work from the callsite source text because Dylint does not reliably run
/// pre-expansion passes. The expanded AST still carries the original macro
/// callsite span, which lets us re-read the user's source.
fn has_top_level_compound_assertion(snippet: &str) -> bool {
    let Some(rest) = snippet.strip_prefix("assert!(") else {
        return false;
    };
    let Some((tokens, _)) = scan_top_level_tokens(rest) else {
        return false;
    };

    // The condition ends at the first top-level comma; the rest is the panic message.
    tokens
        .iter()
        .take_while(|(_, token)| *token != TopLevelToken::Comma)
        .any(|(_, token)| matches!(token, TopLevelToken::Op(_)))
}

fn top_level_assertion_split(snippet: &str) -> Option<TopLevelAssertionSplit> {
    let trimmed = snippet.trim();
    let rest = trimmed.strip_prefix("assert!(")?;
    let rest = rest.trim_end();
    let has_trailing_semicolon = rest.ends_with(';');
    let rest = if has_trailing_semicolon {
        rest[..rest.len() - 1].trim_end()
    } else {
        rest
    };
    let body = rest.strip_suffix(')')?;
    let (op, clauses) = parse_top_level_boolean_clauses(body)?;

    Some(TopLevelAssertionSplit {
        op,
        clauses,
        has_trailing_semicolon,
    })
}

fn parse_top_level_boolean_clauses(body: &str) -> Option<(TopLevelBooleanOp, Vec<String>)> {
    let (tokens, end) = scan_top_level_tokens(body)?;
    if end != body.len() {
        return None;
    }

    let mut start = 0usize;
    let mut op = None;
    let mut clauses = Vec::new();
    for (idx, token) in tokens {
        // A top-level comma means a custom panic message follows the condition.
        let TopLevelToken::Op(token_op) = token else {
            return None;
        };
        if op.is_some_and(|current| current != token_op) {
            return None;
        }
        op = Some(token_op);
        clauses.push(body[start..idx].trim().to_owned());
        start = idx + 2;
    }

    let op = op?;
    clauses.push(body[start..].trim().to_owned());
    if clauses.iter().any(String::is_empty) {
        return None;
    }

    Some((op, clauses))
}

fn compound_assertion_suggestion(
    cx: &EarlyContext<'_>,
    callsite: Span,
    snippet: &str,
    is_statement: bool,
) -> Option<LintSuggestion> {
    let split = top_level_assertion_split(snippet)?;
    if split.op != TopLevelBooleanOp::And || split.clauses.len() < 2 {
        return None;
    }

    let (replacement, tail) = if is_statement {
        let indent = " ".repeat(cx.sess().source_map().lookup_char_pos(callsite.lo()).col.0);
        let replacement = format!("assert!({})", split.clauses[0]);
        let mut tail = String::new();
        for (index, clause) in split.clauses.iter().enumerate().skip(1) {
            tail.push_str(";\n");
            tail.push_str(&indent);
            tail.push_str("assert!(");
            tail.push_str(clause);
            tail.push(')');
            if index + 1 == split.clauses.len() && split.has_trailing_semicolon {
                tail.push(';');
            }
        }
        (replacement, tail)
    } else {
        // Outside statement position (match arm, closure body, ...) no `;` may follow
        // the call, so keep a single unit-typed block expression.
        let replacement = format!("{{ assert!({})", split.clauses[0]);
        let mut tail = String::new();
        for clause in split.clauses.iter().skip(1) {
            tail.push_str("; assert!(");
            tail.push_str(clause);
            tail.push(')');
        }
        tail.push_str(" }");
        (replacement, tail)
    };

    Some(LintSuggestion::Multipart {
        message: "split into separate assertions".to_owned(),
        parts: vec![(callsite, replacement), (callsite.shrink_to_hi(), tail)],
        applicability: Applicability::MachineApplicable,
    })
}

impl EarlyLintPass for CompoundAssertion {
    fn check_stmt(&mut self, _cx: &EarlyContext<'_>, stmt: &rustc_ast::Stmt) {
        self.statement_expr = match &stmt.kind {
            rustc_ast::StmtKind::Expr(expr) | rustc_ast::StmtKind::Semi(expr) => Some(expr.id),
            _ => None,
        };
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Only inspect expressions originating from macro expansion.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded `if` nodes from `assert!` to avoid duplicate
        // diagnostics on children that share the same callsite.
        if !matches!(expr.kind, ExprKind::If(..)) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };
        let trimmed = snippet.trim();

        if !trimmed.starts_with("assert!(") {
            return;
        }

        if !has_top_level_compound_assertion(trimmed) {
            return;
        }

        let is_statement = self.statement_expr == Some(expr.id);
        if let Some(suggestion) = compound_assertion_suggestion(cx, callsite, trimmed, is_statement) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                COMPOUND_ASSERTION,
                callsite,
                "compound assertion should be split into separate assertions",
                "compound_assertion",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(
                cx,
                COMPOUND_ASSERTION,
                callsite,
                "compound assertion should be split into separate assertions",
                "compound_assertion",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TopLevelAssertionSplit;
    use super::TopLevelBooleanOp;
    use super::has_top_level_compound_assertion;
    use super::top_level_assertion_split;

    #[test]
    fn detects_top_level_compound_assertion() {
        assert!(has_top_level_compound_assertion("assert!(a && b)"));
        assert!(has_top_level_compound_assertion("assert!(a || b)"));
        assert!(!has_top_level_compound_assertion("assert!(a)"));
        assert!(!has_top_level_compound_assertion("assert!(items.iter().all(|x| x > 0 && x < 10))"));
        assert!(!has_top_level_compound_assertion("debug_assert!(a && b)"));
    }

    #[test]
    fn scans_only_the_condition_outside_literals() {
        assert!(!has_top_level_compound_assertion("assert!(ready, \"left && right must hold\")"));
        assert!(!has_top_level_compound_assertion("assert!(ready, \"state {}\", a || b)"));
        assert!(!has_top_level_compound_assertion("assert!(c == '|' , \"pipe\")"));
        assert!(has_top_level_compound_assertion("assert!(a && b, \"bounds broke\")"));
        assert!(has_top_level_compound_assertion("assert!(c == '\"' && d)"));
        assert!(has_top_level_compound_assertion("assert!(f::<'static>() && b)"));
    }

    #[test]
    fn parses_machine_applicable_top_level_conjunction() {
        assert_eq!(
            top_level_assertion_split("assert!(a > 0 && b > 0);"),
            Some(TopLevelAssertionSplit {
                op: TopLevelBooleanOp::And,
                clauses: vec!["a > 0".into(), "b > 0".into()],
                has_trailing_semicolon: true,
            })
        );
    }

    #[test]
    fn parses_top_level_disjunction_without_machine_applicable_fix() {
        assert_eq!(
            top_level_assertion_split("assert!(a > 0 || b > 0)"),
            Some(TopLevelAssertionSplit {
                op: TopLevelBooleanOp::Or,
                clauses: vec!["a > 0".into(), "b > 0".into()],
                has_trailing_semicolon: false,
            })
        );
    }

    #[test]
    fn rejects_message_bearing_assertions_for_machine_applicable_fix() {
        assert_eq!(top_level_assertion_split("assert!(a > 0 && b > 0, \"boom\")"), None);
    }
}
