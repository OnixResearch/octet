//! Iterative implicit drop edge collection for MIR semantic analysis.

use rustc_middle::ty::TypeVisitableExt;

const MAX_DROP_DEPTH: usize = 8;

pub(super) struct DropContext<'a, 'tcx> {
    pub(super) cx: &'a rustc_lint::LateContext<'tcx>,
    pub(super) typing_env: rustc_middle::ty::TypingEnv<'tcx>,
    pub(super) path_kind: octet_architecture_ir::semantic::DropPathKind,
    pub(super) span_id: &'a str,
}

struct DropItem<'a, 'tcx> {
    ty: rustc_middle::ty::Ty<'tcx>,
    ty_display: &'a str,
    depth: usize,
}

pub(super) fn collect_drop_edges<'tcx>(
    ctx: &DropContext<'_, 'tcx>,
    dropped_ty: rustc_middle::ty::Ty<'tcx>,
    sinks: &mut super::MirSinks<'_>,
) {
    let mut visited = std::collections::BTreeSet::new();
    let mut work = Vec::with_capacity(16);
    work.push((dropped_ty, 0usize));

    while let Some((ty, depth)) = work.pop() {
        let ty_display = super::super::qualified_type(ctx.cx, ty);
        if !visited.insert(ty_display.clone()) || depth > MAX_DROP_DEPTH {
            continue;
        }
        if ty.has_param() || matches!(ty.kind(), rustc_middle::ty::Dynamic(..) | rustc_middle::ty::Alias(..)) {
            record_unresolved_drop(&ty_display, ctx.path_kind, ctx.span_id, sinks);
            continue;
        }
        let item = DropItem {
            ty,
            ty_display: &ty_display,
            depth,
        };
        process_concrete_drop(ctx, &item, &mut work, sinks);
    }
}

fn record_unresolved_drop(
    ty_display: &str,
    path_kind: octet_architecture_ir::semantic::DropPathKind,
    span_id: &str,
    sinks: &mut super::MirSinks<'_>,
) {
    sinks.drops.insert(octet_architecture_ir::semantic::DropEdge {
        dropped_type: ty_display.to_owned(),
        destructor_definition: None,
        path_kind,
        is_resolved: false,
        has_effectful_candidate: true,
        span_id: Some(span_id.to_owned()),
    });
    sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
        kind: octet_architecture_ir::semantic::UnsupportedKind::UnresolvedDrop,
        detail: format!("implicit drop for `{ty_display}` depends on unresolved generic/opaque/dynamic type"),
        span_id: Some(span_id.to_owned()),
    });
}

fn process_concrete_drop<'tcx>(
    ctx: &DropContext<'_, 'tcx>,
    item: &DropItem<'_, 'tcx>,
    work: &mut Vec<(rustc_middle::ty::Ty<'tcx>, usize)>,
    sinks: &mut super::MirSinks<'_>,
) {
    match item.ty.kind() {
        rustc_middle::ty::Adt(adt_def, args) => {
            if let Some(dtor) = adt_def.destructor(ctx.cx.tcx) {
                record_adt_dtor(ctx, item.ty_display, dtor.did, sinks);
            }
            for field in adt_def.all_fields() {
                let field_ty = field.ty(ctx.cx.tcx, args);
                if field_ty.needs_drop(ctx.cx.tcx, ctx.typing_env) && work.len() < 128 {
                    work.push((field_ty, item.depth + 1));
                }
            }
        }
        rustc_middle::ty::Tuple(elems) => {
            for elem in *elems {
                if elem.needs_drop(ctx.cx.tcx, ctx.typing_env) && work.len() < 128 {
                    work.push((elem, item.depth + 1));
                }
            }
        }
        rustc_middle::ty::Array(elem, _) | rustc_middle::ty::Slice(elem)
            if elem.needs_drop(ctx.cx.tcx, ctx.typing_env) && work.len() < 128 =>
        {
            work.push((*elem, item.depth + 1));
        }
        _ => {}
    }
}

fn record_adt_dtor(
    ctx: &DropContext<'_, '_>,
    ty_display: &str,
    dtor_did: rustc_span::def_id::DefId,
    sinks: &mut super::MirSinks<'_>,
) {
    let dtor_path = super::super::qualified_definition(ctx.cx, dtor_did);
    sinks.drops.insert(octet_architecture_ir::semantic::DropEdge {
        dropped_type: ty_display.to_owned(),
        destructor_definition: Some(dtor_path.clone()),
        path_kind: ctx.path_kind,
        is_resolved: true,
        has_effectful_candidate: true,
        span_id: Some(ctx.span_id.to_owned()),
    });
    let resolution = if dtor_did.is_local() {
        octet_architecture_ir::semantic::CallResolutionKind::ConcreteLocal
    } else {
        octet_architecture_ir::semantic::CallResolutionKind::ConcreteExternal
    };
    sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
        callee_definition: Some(dtor_path.clone()),
        resolved_instance: Some(dtor_path),
        trait_definition: Some("core::ops::drop::Drop".to_owned()),
        resolution,
        has_generic_params: false,
        span_id: Some(ctx.span_id.to_owned()),
    });
}
