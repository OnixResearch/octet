use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects struct definitions containing 2 or more `Mutex` or `RwLock`
    /// fields without a comment documenting lock acquisition order.
    /// Undocumented lock ordering leads to deadlocks that only appear
    /// under production concurrency.
    pub tigerstyle::MULTI_LOCK_ORDERING,
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

        let message = format!("struct `{}` has {lock_count} lock fields without documented ordering", ident.name);
        if let Some(suggestion) = lock_ordering_suggestion(&snippet, item.span, lock_count) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                MULTI_LOCK_ORDERING,
                item.span,
                message,
                "multi_lock_ordering",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, MULTI_LOCK_ORDERING, item.span, message, "multi_lock_ordering");
        }
    }
}

fn lock_ordering_suggestion(snippet: &str, item_span: rustc_span::Span, lock_count: usize) -> Option<LintSuggestion> {
    const OPENING_BRACE_WIDTH_BYTES: usize = 1;

    let brace_pos = snippet.find('{')?;

    // Build insertion point: just after the opening brace
    let insert_offset_bytes = u32::try_from(brace_pos + OPENING_BRACE_WIDTH_BYTES).ok()?;
    let insert_offset = rustc_span::BytePos(insert_offset_bytes);
    let insert_span = item_span.with_lo(item_span.lo() + insert_offset).with_hi(item_span.lo() + insert_offset);

    let replacement = format!("\n    // Lock order: <document acquisition order for {lock_count} locks>");

    Some(LintSuggestion::Single {
        span: insert_span,
        message: "add lock ordering documentation".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}
