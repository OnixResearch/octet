use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context::late_hir_id_is_in_test_context;

const LINT_NAME: &str = "public_unsafe_api";

declare_tool_lint! {
    /// Detects public Rust APIs that expose `unsafe fn` directly.
    /// Tiger Style prefers containing unsafe operations behind safe public wrappers
    /// and small private helpers so caller obligations stay localized and auditable.
    pub tigerstyle::PUBLIC_UNSAFE_API,
    Warn,
    "public unsafe Rust API exposes caller obligations"
}

declare_lint_pass!(PublicUnsafeApi => [PUBLIC_UNSAFE_API]);

fn signature_prefix(cx: &LateContext<'_>, span: rustc_span::Span) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(span).ok()?;
    Some(snippet.split('{').next().unwrap_or(&snippet).to_string())
}

fn snippet_declares_unsafe_fn(cx: &LateContext<'_>, span: rustc_span::Span) -> bool {
    signature_prefix(cx, span).is_some_and(|signature| signature.contains("unsafe fn"))
}

fn snippet_declares_extern(cx: &LateContext<'_>, span: rustc_span::Span) -> bool {
    signature_prefix(cx, span).is_some_and(|signature| signature.contains("extern"))
}

fn lint_public_unsafe(cx: &LateContext<'_>, span: rustc_span::Span) {
    emit_tiger_style_lint(
        cx,
        PUBLIC_UNSAFE_API,
        span,
        "public unsafe API leaks caller obligations; prefer a safe public wrapper around a small private unsafe helper",
        LINT_NAME,
    );
}

impl<'tcx> LateLintPass<'tcx> for PublicUnsafeApi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { .. } = &item.kind else {
            return;
        };
        if item.span.from_expansion()
            || late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
            || !cx.tcx.visibility(item.owner_id.def_id).is_public()
            || !snippet_declares_unsafe_fn(cx, item.span)
            || snippet_declares_extern(cx, item.span)
        {
            return;
        }

        lint_public_unsafe(cx, item.span);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(_, _) = &impl_item.kind else {
            return;
        };
        let def_id = impl_item.owner_id.def_id;
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some()
            || impl_item.span.from_expansion()
            || late_hir_id_is_in_test_context(cx, impl_item.hir_id(), impl_item.span)
            || !cx.tcx.visibility(def_id).is_public()
            || !snippet_declares_unsafe_fn(cx, impl_item.span)
            || snippet_declares_extern(cx, impl_item.span)
        {
            return;
        }

        lint_public_unsafe(cx, impl_item.span);
    }
}
