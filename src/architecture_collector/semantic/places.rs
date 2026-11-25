//! Place-sensitive memory-region and borrow/partial-move tracking for MIR semantic analysis.

use octet_architecture_ir::semantic::MemoryRegion;
use octet_architecture_ir::semantic::MemoryWrite;
use octet_architecture_ir::semantic::UnsupportedConstruct;
use octet_architecture_ir::semantic::UnsupportedKind;
use rustc_lint::LateContext;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_middle::ty::Ty;

pub(super) struct BodyState {
    pub(super) local_regions: std::collections::BTreeMap<u32, MemoryRegion>,
    pub(super) active_borrows: std::collections::BTreeSet<(u32, String)>,
    pub(super) escaped_places: std::collections::BTreeSet<(u32, String)>,
    pub(super) moved_fields: std::collections::BTreeSet<(u32, String)>,
    pub(super) partial_moves_observed: usize,
    pub(super) ended_borrow_reassignments: usize,
}

impl BodyState {
    pub(super) fn from_body<'tcx>(
        cx: &LateContext<'tcx>,
        body: &mir::Body<'tcx>,
        typing_env: ty::TypingEnv<'tcx>,
    ) -> Self {
        let mut local_regions = std::collections::BTreeMap::new();
        for (idx, decl) in body.local_decls.iter().enumerate() {
            let Ok(local_u32) = u32::try_from(idx) else {
                continue;
            };
            let is_arg = idx >= 1 && idx <= body.arg_count;
            let region = classify_initial_local_region(cx, decl.ty, is_arg, typing_env);
            local_regions.insert(local_u32, region);
        }
        Self {
            local_regions,
            active_borrows: std::collections::BTreeSet::new(),
            escaped_places: std::collections::BTreeSet::new(),
            moved_fields: std::collections::BTreeSet::new(),
            partial_moves_observed: 0,
            ended_borrow_reassignments: 0,
        }
    }
}

pub(super) fn classify_initial_local_region<'tcx>(
    cx: &LateContext<'tcx>,
    ty: Ty<'tcx>,
    is_arg: bool,
    typing_env: ty::TypingEnv<'tcx>,
) -> MemoryRegion {
    match ty.kind() {
        ty::Ref(_, referent, mir::Mutability::Mut) => {
            if is_arg {
                MemoryRegion::ParamMutRef
            } else if !referent.is_freeze(cx.tcx, typing_env) {
                MemoryRegion::InteriorMutable
            } else {
                MemoryRegion::OwnedLocal
            }
        }
        ty::Ref(_, referent, mir::Mutability::Not) if !referent.is_freeze(cx.tcx, typing_env) => {
            MemoryRegion::InteriorMutable
        }
        ty::RawPtr(_, _) => MemoryRegion::RawPointer,
        _ if !ty.is_freeze(cx.tcx, typing_env) => MemoryRegion::InteriorMutable,
        _ => MemoryRegion::OwnedLocal,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn record_statement<'tcx>(
    collector: &mut super::super::ArchitectureCollector,
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    stmt: &mir::Statement<'tcx>,
    state: &mut BodyState,
    memory_writes: &mut std::collections::BTreeSet<MemoryWrite>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    let span_id = collector.collect_span(cx, stmt.source_info.span);
    match &stmt.kind {
        mir::StatementKind::Assign(boxed) => {
            let (dest, rvalue) = &**boxed;
            inspect_rvalue(cx, body, typing_env, dest, rvalue, &span_id, state, unsupported);
            record_place_write(cx, body, typing_env, *dest, &span_id, state, memory_writes, unsupported);
        }
        mir::StatementKind::StorageDead(local) => {
            let local_u32 = local.as_u32();
            state.active_borrows.retain(|(borrowed_local, _)| *borrowed_local != local_u32);
        }
        mir::StatementKind::Intrinsic(_) => {
            unsupported.insert(UnsupportedConstruct {
                kind: UnsupportedKind::Other,
                detail: "MIR intrinsic statement observed".to_owned(),
                span_id: Some(span_id),
            });
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn inspect_rvalue<'tcx>(
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    dest: &mir::Place<'tcx>,
    rvalue: &mir::Rvalue<'tcx>,
    span_id: &str,
    state: &mut BodyState,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    match rvalue {
        mir::Rvalue::Ref(_, mir::BorrowKind::Mut { .. }, borrowed) => {
            let key = (borrowed.local.as_u32(), projection_label(borrowed));
            state.active_borrows.insert(key.clone());
            let dest_region = state.local_regions.get(&dest.local.as_u32()).copied();
            if dest.local == mir::RETURN_PLACE || dest_region == Some(MemoryRegion::ParamMutRef) {
                state.escaped_places.insert(key);
            }
        }
        mir::Rvalue::RawPtr(_, place) => {
            let key = (place.local.as_u32(), projection_label(place));
            state.escaped_places.insert(key);
            unsupported.insert(UnsupportedConstruct {
                kind: UnsupportedKind::RawPointerDeref,
                detail: "raw pointer creation observed in MIR".to_owned(),
                span_id: Some(span_id.to_owned()),
            });
        }
        mir::Rvalue::Use(mir::Operand::Move(moved)) | mir::Rvalue::Cast(_, mir::Operand::Move(moved), _) => {
            if !moved.projection.is_empty() {
                state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
                state.partial_moves_observed += 1;
            }
        }
        mir::Rvalue::Aggregate(_, operands) => {
            for operand in operands {
                if let mir::Operand::Move(moved) = operand
                    && !moved.projection.is_empty()
                {
                    state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
                    state.partial_moves_observed += 1;
                }
            }
        }
        _ => {}
    }
    let dest_ty = dest.ty(&body.local_decls, cx.tcx).ty;
    if !dest_ty.is_freeze(cx.tcx, typing_env) {
        state.local_regions.insert(dest.local.as_u32(), MemoryRegion::InteriorMutable);
    }
}

#[allow(clippy::too_many_arguments)]
fn record_place_write<'tcx>(
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    place: mir::Place<'tcx>,
    span_id: &str,
    state: &mut BodyState,
    memory_writes: &mut std::collections::BTreeSet<MemoryWrite>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    let local_u32 = place.local.as_u32();
    let proj = projection_label(&place);
    let place_ty = place.ty(&body.local_decls, cx.tcx).ty;
    let is_interior_mut = !place_ty.is_freeze(cx.tcx, typing_env);

    let has_deref = place.projection.iter().any(|elem| matches!(elem, mir::ProjectionElem::Deref));
    let has_field = place.projection.iter().any(|elem| matches!(elem, mir::ProjectionElem::Field(..)));
    let base_ty = body.local_decls[place.local].ty;

    if place.projection.is_empty() && state.active_borrows.iter().any(|(l, _)| *l == local_u32) {
        state.active_borrows.retain(|(l, _)| *l != local_u32);
        state.ended_borrow_reassignments += 1;
    }

    let region = if has_deref {
        match base_ty.kind() {
            ty::RawPtr(_, _) => {
                unsupported.insert(UnsupportedConstruct {
                    kind: UnsupportedKind::RawPointerDeref,
                    detail: format!("write through raw pointer `{}`", place_ty),
                    span_id: Some(span_id.to_owned()),
                });
                MemoryRegion::RawPointer
            }
            ty::Ref(_, referent, _) if !referent.is_freeze(cx.tcx, typing_env) => MemoryRegion::InteriorMutable,
            _ => {
                let base_region = state.local_regions.get(&local_u32).copied().unwrap_or(MemoryRegion::UnknownRegion);
                if state.escaped_places.contains(&(local_u32, String::new())) {
                    MemoryRegion::EscapingAlias
                } else {
                    base_region
                }
            }
        }
    } else if state.escaped_places.contains(&(local_u32, proj.clone()))
        || state.escaped_places.contains(&(local_u32, String::new()))
    {
        MemoryRegion::EscapingAlias
    } else if is_interior_mut {
        MemoryRegion::InteriorMutable
    } else if has_field {
        state.moved_fields.remove(&(local_u32, proj.clone()));
        MemoryRegion::LocalField
    } else {
        state.local_regions.get(&local_u32).copied().unwrap_or(MemoryRegion::OwnedLocal)
    };

    let is_observable_external = matches!(
        region,
        MemoryRegion::ParamMutRef
            | MemoryRegion::EscapingAlias
            | MemoryRegion::InteriorMutable
            | MemoryRegion::RawPointer
            | MemoryRegion::StaticGlobal
            | MemoryRegion::UnknownRegion
    );

    if place.local != mir::RETURN_PLACE || is_observable_external || has_deref {
        memory_writes.insert(MemoryWrite {
            local_index: local_u32,
            projection: proj,
            resolved_type: super::super::qualified_type(cx, place_ty),
            region,
            is_observable_external,
            is_interior_mutability: is_interior_mut,
            span_id: Some(span_id.to_owned()),
        });
    }
}

pub(super) fn projection_label(place: &mir::Place<'_>) -> String {
    if place.projection.is_empty() {
        return String::new();
    }
    let mut parts = Vec::with_capacity(place.projection.len());
    for elem in place.projection.iter() {
        let part = match elem {
            mir::ProjectionElem::Deref => "*".to_owned(),
            mir::ProjectionElem::Field(field, _) => format!(".{}", field.as_usize()),
            mir::ProjectionElem::Index(_) => "[_]".to_owned(),
            mir::ProjectionElem::ConstantIndex { offset, .. } => format!("[{offset}]"),
            mir::ProjectionElem::Subslice { from, to, .. } => format!("[{from}..{to}]"),
            mir::ProjectionElem::Downcast(_, variant) => format!("@{}", variant.as_usize()),
            mir::ProjectionElem::OpaqueCast(_) => "opaque".to_owned(),
            mir::ProjectionElem::UnwrapUnsafeBinder(_) => "binder".to_owned(),
        };
        parts.push(part);
    }
    parts.join("")
}
