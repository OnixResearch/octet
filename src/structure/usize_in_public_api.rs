use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects inherent public APIs that expose platform-sized integers.
    /// Tiger Style prefers fixed-width types at boundaries so signatures stay
    /// stable across 32-bit and 64-bit targets.
    pub tigerstyle::USIZE_IN_PUBLIC_API,
    Warn,
    "platform-sized integer in public API"
}

declare_lint_pass!(UsizeInPublicApi => [USIZE_IN_PUBLIC_API]);

fn platform_sized_int_name(ty: ty::Ty<'_>) -> Option<&'static str> {
    match ty.kind() {
        ty::Uint(ty::UintTy::Usize) => Some("usize"),
        ty::Int(ty::IntTy::Isize) => Some("isize"),
        ty::Ref(_, inner, _) => platform_sized_int_name(*inner),
        ty::Slice(inner) => platform_sized_int_name(*inner),
        ty::Array(inner, _) => platform_sized_int_name(*inner),
        ty::Tuple(items) => items.iter().find_map(platform_sized_int_name),
        ty::Adt(_, args) => args.types().find_map(platform_sized_int_name),
        _ => None,
    }
}

fn lint_message(name: &str) -> String {
    if name == "usize" {
        "`usize` in public API — use `u32` or `u64` for cross-platform consistency".to_owned()
    } else {
        "`isize` in public API — use `i32` or `i64` for cross-platform consistency".to_owned()
    }
}

impl<'tcx> LateLintPass<'tcx> for UsizeInPublicApi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let ItemKind::Fn { .. } = item.kind else {
            return;
        };

        let def_id = item.owner_id.def_id;
        if !cx.tcx.visibility(def_id).is_public() {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(def_id).instantiate_identity();
        let sig = fn_sig.skip_binder();

        if let Some(name) = sig.inputs().iter().find_map(|ty| platform_sized_int_name(*ty)) {
            emit_tiger_style_lint(cx, USIZE_IN_PUBLIC_API, item.span, lint_message(name), "usize_in_public_api");
            return;
        }

        if let Some(name) = platform_sized_int_name(sig.output()) {
            emit_tiger_style_lint(cx, USIZE_IN_PUBLIC_API, item.span, lint_message(name), "usize_in_public_api");
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(_, _) = impl_item.kind else {
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
        if impl_block.of_trait.is_some() || !cx.tcx.visibility(def_id).is_public() {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(def_id).instantiate_identity();
        let sig = fn_sig.skip_binder();

        if let Some(name) = sig.inputs().iter().find_map(|ty| platform_sized_int_name(*ty)) {
            emit_tiger_style_lint(cx, USIZE_IN_PUBLIC_API, impl_item.span, lint_message(name), "usize_in_public_api");
            return;
        }

        if let Some(name) = platform_sized_int_name(sig.output()) {
            emit_tiger_style_lint(cx, USIZE_IN_PUBLIC_API, impl_item.span, lint_message(name), "usize_in_public_api");
        }
    }
}
