use rustc_ast::token::TokenKind;
use rustc_ast::tokenstream::TokenTree;
use rustc_ast::{Expr, ExprKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects `assert!(a && b)` and `assert!(a || b)` where the macro
    /// argument contains a top-level `&&` or `||`. Compound assertions
    /// produce a single "assertion failed" message on failure, hiding
    /// which condition actually broke.
    pub COMPOUND_ASSERTION,
    Warn,
    "compound assertion should be split into separate assertions"
}

declare_lint_pass!(CompoundAssertion => [COMPOUND_ASSERTION]);

impl EarlyLintPass for CompoundAssertion {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::MacCall(mac) = &expr.kind else {
            return;
        };

        let Some(last) = mac.path.segments.last() else {
            return;
        };

        // Only fire on `assert!`, not `assert_eq!`, `assert_ne!`, `debug_assert!`
        if last.ident.name.as_str() != "assert" {
            return;
        }

        // Scan top-level tokens for && or ||.
        // Delimited groups (parens, brackets, braces) are opaque — tokens
        // inside closures or sub-expressions are not top-level.
        for tt in mac.args.tokens.iter() {
            match tt {
                TokenTree::Token(tok, _) => match tok.kind {
                    TokenKind::Comma => break, // stop before optional message arg
                    TokenKind::AndAnd | TokenKind::OrOr => {
                        cx.opt_span_lint(COMPOUND_ASSERTION, Some(expr.span), DiagDecorator(|diag| {
                            diag.primary_message(
                                "compound assertion should be split into separate assertions",
                            );
                            diag.help(
                                "split into separate `assert!()` calls so each condition \
                                 reports independently on failure",
                            );
                        }));
                        return;
                    }
                    _ => {}
                },
                TokenTree::Delimited(..) => {}
            }
        }
    }
}
