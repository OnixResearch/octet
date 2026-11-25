//! Implicit drop edge collection for MIR semantic analysis.

use octet_architecture_ir::semantic::CallResolutionKind;
use octet_architecture_ir::semantic::DropEdge;
use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::MirCallEdge;
use octet_architecture_ir::semantic::UnsupportedConstruct;
use octet_architecture_ir::semantic::UnsupportedKind;
use rustc_lint::LateContext;
use rustc_middle::ty;
use rustc_middle::ty::Ty;
use rustc_middle::ty::TypeVisitableExt;

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_drop_edges<'tcx>(
    cx: &LateContext<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    dropped_ty: Ty<'tcx>,
    path_kind: DropPathKind,
    span_id: &str,
    drops: &mut std::collections::BTreeSet<DropEdge>,
    calls: &mut std::collections::BTreeSet<MirCallEdge>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    let mut visited = std::collections::BTreeSet::new();
    collect_drop_recursive(cx, typing_env, dropped_ty, path_kind, span_id, 0, &mut visited, drops, calls, unsupported);
}

#[allow(clippy::too_many_arguments)]
fn collect_drop_recursive<'tcx>(
    cx: &LateContext<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    ty: Ty<'tcx>,
    path_kind: DropPathKind,
    span_id: &str,
    depth: usize,
    visited: &mut std::collections::BTreeSet<String>,
    drops: &mut std::collections::BTreeSet<DropEdge>,
    calls: &mut std::collections::BTreeSet<MirCallEdge>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    let ty_display = super::super::qualified_type(cx, ty);
    if !visited.insert(ty_display.clone()) || depth > 8 {
        return;
    }
    if ty.has_param() || matches!(ty.kind(), ty::Dynamic(..) | ty::Alias(..)) {
        drops.insert(DropEdge {
            dropped_type: ty_display.clone(),
            destructor_definition: None,
            path_kind,
            is_resolved: false,
            has_effectful_candidate: true,
            span_id: Some(span_id.to_owned()),
        });
        unsupported.insert(UnsupportedConstruct {
            kind: UnsupportedKind::UnresolvedDrop,
            detail: format!("implicit drop for `{ty_display}` depends on unresolved generic/opaque/dynamic type"),
            span_id: Some(span_id.to_owned()),
        });
        return;
    }
    match ty.kind() {
        ty::Adt(adt_def, args) => {
            if let Some(dtor) = adt_def.destructor(cx.tcx) {
                let dtor_path = super::super::qualified_definition(cx, dtor.did);
                drops.insert(DropEdge {
                    dropped_type: ty_display,
                    destructor_definition: Some(dtor_path.clone()),
                    path_kind,
                    is_resolved: true,
                    has_effectful_candidate: true,
                    span_id: Some(span_id.to_owned()),
                });
                let resolution = if dtor.did.is_local() {
                    CallResolutionKind::ConcreteLocal
                } else {
                    CallResolutionKind::ConcreteExternal
                };
                calls.insert(MirCallEdge {
                    callee_definition: Some(dtor_path.clone()),
                    resolved_instance: Some(dtor_path),
                    trait_definition: Some("core::ops::drop::Drop".to_owned()),
                    resolution,
                    has_generic_params: false,
                    span_id: Some(span_id.to_owned()),
                });
            }
            for field in adt_def.all_fields() {
                let field_ty = field.ty(cx.tcx, args);
                if field_ty.needs_drop(cx.tcx, typing_env) {
                    collect_drop_recursive(
                        cx,
                        typing_env,
                        field_ty,
                        path_kind,
                        span_id,
                        depth + 1,
                        visited,
                        drops,
                        calls,
                        unsupported,
                    );
                }
            }
        }
        ty::Tuple(elems) => {
            for elem in *elems {
                if elem.needs_drop(cx.tcx, typing_env) {
                    collect_drop_recursive(
                        cx,
                        typing_env,
                        elem,
                        path_kind,
                        span_id,
                        depth + 1,
                        visited,
                        drops,
                        calls,
                        unsupported,
                    );
                }
            }
        }
        ty::Array(elem, _) | ty::Slice(elem) => {
            if elem.needs_drop(cx.tcx, typing_env) {
                collect_drop_recursive(
                    cx,
                    typing_env,
                    *elem,
                    path_kind,
                    span_id,
                    depth + 1,
                    visited,
                    drops,
                    calls,
                    unsupported,
                );
            }
        }
        _ => {}
    }
}
