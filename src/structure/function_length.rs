use rustc_ast::{Attribute, Block, Item, ItemKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::sym;

declare_lint! {
    /// Detects functions whose body exceeds 70 non-blank, non-comment lines.
    /// Long functions are hard to reason about and typically indicate missing
    /// decomposition.
    pub FUNCTION_LENGTH,
    Warn,
    "function body exceeds 70 lines"
}

declare_lint_pass!(FunctionLength => [FUNCTION_LENGTH]);

const MAX_LINES: usize = 70;

fn is_test_function(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.has_name(sym::test))
}

fn count_code_lines(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            if let Some(pos) = trimmed.find("*/") {
                in_block_comment = false;
                let after = trimmed[pos + 2..].trim();
                if !after.is_empty() && !after.starts_with("//") {
                    count += 1;
                }
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        count += 1;
    }

    count
}

fn check_fn_body(cx: &EarlyContext<'_>, name: rustc_span::Symbol, body: &Block, attrs: &[Attribute]) {
    if is_test_function(attrs) {
        return;
    }

    let source_map = cx.sess().source_map();
    let Ok(snippet) = source_map.span_to_snippet(body.span) else {
        return;
    };

    let code_lines = count_code_lines(&snippet);

    if code_lines > MAX_LINES {
        cx.opt_span_lint(FUNCTION_LENGTH, Some(body.span), DiagDecorator(|diag| {
            diag.primary_message(format!(
                "function `{name}` is {code_lines} lines (limit: {MAX_LINES})",
            ));
            diag.help("decompose into smaller functions with clear responsibilities");
        }));
    }
}

impl EarlyLintPass for FunctionLength {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Fn(func) = &item.kind {
            if let Some(body) = &func.body {
                check_fn_body(cx, func.ident.name, body, &item.attrs);
            }
        }
    }
}
