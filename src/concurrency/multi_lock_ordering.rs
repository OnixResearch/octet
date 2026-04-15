use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

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
        let lock_count = LOCK_TYPES.iter().map(|lt| snippet.matches(&format!("{lt}<")).count()).sum::<usize>();

        if lock_count < 2 {
            return;
        }

        // Check if the struct body has an ordering comment
        let lower = snippet.to_ascii_lowercase();
        if lower.contains("lock order") || lower.contains("ordering") || lower.contains("lock acquisition") {
            return;
        }

        emit_tiger_style_lint(
            cx,
            MULTI_LOCK_ORDERING,
            item.span,
            format!("struct `{}` has {lock_count} lock fields without documented ordering", ident.name,),
            "multi_lock_ordering",
        );
    }
}
