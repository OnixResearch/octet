use std::collections::BTreeMap;

use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "type_accumulation";

declare_tool_lint! {
    /// Detects types/modules accumulating too many responsibilities.
    /// High impl-block or public-item counts are SRP review signals.
    pub tigerstyle::TYPE_ACCUMULATION,
    Allow,
    "type or module accumulates too many responsibilities"
}

pub struct TypeAccumulation {
    impl_blocks_max: usize,
    module_pub_items_max: usize,
    impl_counts: BTreeMap<String, usize>,
}

impl TypeAccumulation {
    pub fn new(impl_blocks_max: usize, module_pub_items_max: usize) -> Self {
        Self {
            impl_blocks_max,
            module_pub_items_max,
            impl_counts: BTreeMap::new(),
        }
    }
}

impl_lint_pass!(TypeAccumulation => [TYPE_ACCUMULATION]);

fn span_snippet(cx: &LateContext<'_>, span: Span) -> String {
    cx.sess().source_map().span_to_snippet(span).unwrap_or_default()
}

/// Counts the module's own public free items, for inline and file-backed modules alike.
/// Items inside nested modules, impl members, and struct fields are not counted.
fn public_item_count(cx: &LateContext<'_>, module: &hir::Mod<'_>) -> usize {
    module
        .item_ids
        .iter()
        .filter(|item_id| {
            // `use a::{B, C}` lowers to `use a::B; use a::C;` plus a `use a::{}` stem carrying the same
            // visibility.
            !matches!(cx.tcx.hir_item(**item_id).kind, hir::ItemKind::Use(_, hir::UseKind::ListStem))
                && cx.tcx.visibility(item_id.owner_id.def_id).is_public()
        })
        .count()
}

impl<'tcx> LateLintPass<'tcx> for TypeAccumulation {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if item.span.from_expansion() || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span) {
            return;
        }

        match &item.kind {
            hir::ItemKind::Impl(impl_block) if impl_block.of_trait.is_none() => {
                let type_name = span_snippet(cx, impl_block.self_ty.span).trim().to_owned();
                if type_name.is_empty() {
                    return;
                }
                let count = self.impl_counts.entry(type_name.clone()).or_default();
                *count = count.saturating_add(1);
                if *count == self.impl_blocks_max.saturating_add(1) {
                    emit_tiger_style_lint(
                        cx,
                        TYPE_ACCUMULATION,
                        item.span,
                        format!(
                            "type `{type_name}` has more than {} inherent impl blocks; split responsibilities or consolidate cohesive behavior",
                            self.impl_blocks_max
                        ),
                        LINT_NAME,
                    );
                }
            }
            hir::ItemKind::Mod(_, module) => {
                let count = public_item_count(cx, module);
                if count > self.module_pub_items_max {
                    emit_tiger_style_lint(
                        cx,
                        TYPE_ACCUMULATION,
                        item.span,
                        format!(
                            "module exposes {count} public items (limit: {}) — split unrelated public surface into focused modules",
                            self.module_pub_items_max
                        ),
                        LINT_NAME,
                    );
                }
            }
            _ => {}
        }
    }
}
