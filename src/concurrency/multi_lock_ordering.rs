use rustc_ast::{Item, ItemKind};
use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::{declare_lint, declare_lint_pass};

declare_lint! {
    /// Detects struct definitions containing 2 or more `Mutex` or `RwLock`
    /// fields without a comment documenting lock acquisition order.
    /// Undocumented lock ordering leads to deadlocks that only appear
    /// under production concurrency.
    pub MULTI_LOCK_ORDERING,
    Allow,
    "struct has multiple lock fields without documented ordering"
}

declare_lint_pass!(MultiLockOrdering => [MULTI_LOCK_ORDERING]);

const LOCK_TYPES: &[&str] = &["Mutex", "RwLock"];

impl EarlyLintPass for MultiLockOrdering {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Struct(ident, _variant_data, _) = &item.kind else {
            return;
        };

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(item.span) else {
            return;
        };

        // Count fields that look like lock types (by name in source)
        let lock_count = LOCK_TYPES
            .iter()
            .map(|lt| {
                snippet
                    .matches(&format!("{lt}<"))
                    .count()
            })
            .sum::<usize>();

        if lock_count < 2 {
            return;
        }

        // Check if the struct body has an ordering comment
        let lower = snippet.to_ascii_lowercase();
        if lower.contains("lock order") || lower.contains("ordering") || lower.contains("lock acquisition") {
            return;
        }

        cx.opt_span_lint(
            MULTI_LOCK_ORDERING,
            Some(item.span),
            DiagDecorator(|diag| {
                diag.primary_message(format!(
                    "struct `{}` has {lock_count} lock fields without documented ordering",
                    ident.name,
                ));
                diag.help(
                    "add a comment documenting the lock acquisition order, e.g.: \
                     `// Lock ordering: acquire `a` before `b``",
                );
            }),
        );
    }
}
