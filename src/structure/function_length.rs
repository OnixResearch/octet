use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::visit::Visitor;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects functions and methods whose body exceeds a configurable number
    /// of non-blank, non-comment lines (default: 70). Long functions are
    /// hard to reason about and typically indicate missing decomposition.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// function_length_max = 100
    /// ```
    pub tigerstyle::FUNCTION_LENGTH,
    Warn,
    "function body exceeds line limit"
}

pub struct FunctionLength {
    max_lines: usize,
    harness_test_fns: std::collections::HashSet<rustc_ast::NodeId>,
}

impl FunctionLength {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            harness_test_fns: std::collections::HashSet::new(),
        }
    }
}

impl_lint_pass!(FunctionLength => [FUNCTION_LENGTH]);

/// Collects the functions registered as harness tests. Early lints see the
/// expanded AST: without `--test` a `#[test]` function is removed, and under
/// `--test` the builtin macro consumes the attribute and registers the function
/// through a same-named sibling `const` marked `#[rustc_test_marker]`.
#[derive(Default)]
struct HarnessTestFns {
    ids: std::collections::HashSet<rustc_ast::NodeId>,
}

impl HarnessTestFns {
    /// Records functions that share a name with a test marker in the same item list.
    fn record<'a>(&mut self, items: impl Iterator<Item = &'a Item> + Clone) {
        let markers = items
            .clone()
            .filter_map(|item| match &item.kind {
                ItemKind::Const(constant) if item.attrs.iter().any(|attr| attr.has_name(sym::rustc_test_marker)) => {
                    Some(constant.ident.name)
                }
                _ => None,
            })
            .collect::<std::collections::HashSet<rustc_span::Symbol>>();
        if markers.is_empty() {
            return;
        }
        self.ids.extend(items.filter_map(|item| match &item.kind {
            ItemKind::Fn(func) if markers.contains(&func.ident.name) => Some(item.id),
            _ => None,
        }));
    }
}

impl<'ast> Visitor<'ast> for HarnessTestFns {
    fn visit_item(&mut self, item: &'ast Item) {
        if let ItemKind::Mod(_, _, rustc_ast::ModKind::Loaded(items, ..)) = &item.kind {
            self.record(items.iter().map(|item| &**item));
        }
        rustc_ast::visit::walk_item(self, item);
    }

    fn visit_block(&mut self, block: &'ast rustc_ast::Block) {
        self.record(block.stmts.iter().filter_map(|stmt| match &stmt.kind {
            rustc_ast::StmtKind::Item(item) => Some(&**item),
            _ => None,
        }));
        rustc_ast::visit::walk_block(self, block);
    }
}

fn harness_test_fns(krate: &rustc_ast::Crate) -> std::collections::HashSet<rustc_ast::NodeId> {
    let mut collector = HarnessTestFns::default();
    collector.record(krate.items.iter().map(|item| &**item));
    rustc_ast::visit::walk_crate(&mut collector, krate);
    collector.ids
}

/// Counts the code lines between a body's delimiting braces; the brace lines
/// themselves are not body lines.
fn count_body_lines(body_source: &str) -> usize {
    let inner = body_source.strip_prefix('{').and_then(|source| source.strip_suffix('}')).unwrap_or(body_source);
    count_code_lines(inner)
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

impl FunctionLength {
    fn check_function(&self, cx: &EarlyContext<'_>, id: rustc_ast::NodeId, func: &rustc_ast::Fn) {
        if self.harness_test_fns.contains(&id) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let code_lines = count_body_lines(&snippet);
        let max = self.max_lines;

        if code_lines > max {
            let name = func.ident.name;
            emit_tiger_style_lint(
                cx,
                FUNCTION_LENGTH,
                body.span,
                format!("function `{name}` is {code_lines} lines (limit: {max})"),
                "function_length",
            );
        }
    }
}

impl EarlyLintPass for FunctionLength {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        self.harness_test_fns = if cx.sess().opts.test {
            harness_test_fns(krate)
        } else {
            std::collections::HashSet::new()
        };
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::count_body_lines;

    #[test]
    fn body_line_count_excludes_delimiting_braces() {
        assert_eq!(count_body_lines("{\n    let a = 1;\n    a\n}"), 2);
        assert_eq!(count_body_lines("{}"), 0);
    }

    #[test]
    fn body_line_count_keeps_code_sharing_a_brace_line() {
        assert_eq!(count_body_lines("{ 1 }"), 1);
        assert_eq!(count_body_lines("{ let a = 1;\n    a }"), 2);
    }
}
