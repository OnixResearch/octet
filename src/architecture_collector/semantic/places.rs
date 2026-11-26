//! Place-sensitive memory-region and borrow/partial-move tracking for MIR semantic analysis.

pub(super) struct BodyState {
    pub(super) local_regions: std::collections::BTreeMap<u32, octet_architecture_ir::semantic::MemoryRegion>,
    pub(super) active_borrows: std::collections::BTreeSet<(u32, String)>,
    pub(super) escaped_places: std::collections::BTreeSet<(u32, String)>,
    pub(super) moved_fields: std::collections::BTreeSet<(u32, String)>,
    pub(super) partial_moves_observed: usize,
    pub(super) ended_borrow_reassignments: usize,
}

impl BodyState {
    pub(super) fn from_body<'tcx>(
        cx: &rustc_lint::LateContext<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        typing_env: rustc_middle::ty::TypingEnv<'tcx>,
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

pub(super) struct MirBodyRef<'a, 'tcx> {
    pub(super) cx: &'a rustc_lint::LateContext<'tcx>,
    pub(super) body: &'a rustc_middle::mir::Body<'tcx>,
    pub(super) typing_env: rustc_middle::ty::TypingEnv<'tcx>,
}

pub(super) fn classify_initial_local_region<'tcx>(
    cx: &rustc_lint::LateContext<'tcx>,
    ty: rustc_middle::ty::Ty<'tcx>,
    is_arg: bool,
    typing_env: rustc_middle::ty::TypingEnv<'tcx>,
) -> octet_architecture_ir::semantic::MemoryRegion {
    match ty.kind() {
        rustc_middle::ty::Ref(_, referent, rustc_middle::mir::Mutability::Mut) => {
            if is_arg {
                octet_architecture_ir::semantic::MemoryRegion::ParamMutRef
            } else if !referent.is_freeze(cx.tcx, typing_env) {
                octet_architecture_ir::semantic::MemoryRegion::InteriorMutable
            } else {
                octet_architecture_ir::semantic::MemoryRegion::OwnedLocal
            }
        }
        rustc_middle::ty::Ref(_, referent, rustc_middle::mir::Mutability::Not)
            if !referent.is_freeze(cx.tcx, typing_env) =>
        {
            octet_architecture_ir::semantic::MemoryRegion::InteriorMutable
        }
        rustc_middle::ty::RawPtr(_, _) => octet_architecture_ir::semantic::MemoryRegion::RawPointer,
        _ if !ty.is_freeze(cx.tcx, typing_env) => octet_architecture_ir::semantic::MemoryRegion::InteriorMutable,
        _ => octet_architecture_ir::semantic::MemoryRegion::OwnedLocal,
    }
}

pub(super) fn record_statement<'tcx>(
    collector: &mut super::super::ArchitectureCollector,
    mir_ref: &MirBodyRef<'_, 'tcx>,
    stmt: &rustc_middle::mir::Statement<'tcx>,
    state: &mut BodyState,
    sinks: &mut super::SemanticSinks<'_>,
) {
    let span_id = collector.collect_span(mir_ref.cx, stmt.source_info.span);
    match &stmt.kind {
        rustc_middle::mir::StatementKind::Assign(boxed) => {
            let (dest, rvalue) = &**boxed;
            inspect_rvalue(mir_ref, dest, rvalue, &span_id, state, sinks);
            record_place_write(mir_ref, *dest, &span_id, state, sinks);
        }
        rustc_middle::mir::StatementKind::StorageDead(local) => {
            let local_u32 = local.as_u32();
            state.active_borrows.retain(|(borrowed_local, _)| *borrowed_local != local_u32);
        }
        rustc_middle::mir::StatementKind::Intrinsic(_) => {
            sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                kind: octet_architecture_ir::semantic::UnsupportedKind::Other,
                detail: "MIR intrinsic statement observed".to_owned(),
                span_id: Some(span_id),
            });
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn inspect_rvalue<'tcx>(
    mir_ref: &MirBodyRef<'_, 'tcx>,
    dest: &rustc_middle::mir::Place<'tcx>,
    rvalue: &rustc_middle::mir::Rvalue<'tcx>,
    span_id: &str,
    state: &mut BodyState,
    sinks: &mut super::SemanticSinks<'_>,
) {
    match rvalue {
        rustc_middle::mir::Rvalue::Ref(_, rustc_middle::mir::BorrowKind::Mut { .. }, borrowed) => {
            let key = (borrowed.local.as_u32(), projection_label(borrowed));
            state.active_borrows.insert(key.clone());
            let dest_region = state.local_regions.get(&dest.local.as_u32()).copied();
            if dest.local == rustc_middle::mir::RETURN_PLACE
                || dest_region == Some(octet_architecture_ir::semantic::MemoryRegion::ParamMutRef)
            {
                state.escaped_places.insert(key);
            }
        }
        rustc_middle::mir::Rvalue::RawPtr(_, place) => {
            let key = (place.local.as_u32(), projection_label(place));
            state.escaped_places.insert(key);
            sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                kind: octet_architecture_ir::semantic::UnsupportedKind::RawPointerDeref,
                detail: "raw pointer creation observed in MIR".to_owned(),
                span_id: Some(span_id.to_owned()),
            });
        }
        rustc_middle::mir::Rvalue::Use(rustc_middle::mir::Operand::Move(moved))
        | rustc_middle::mir::Rvalue::Cast(_, rustc_middle::mir::Operand::Move(moved), _)
            if !moved.projection.is_empty() =>
        {
            state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
            state.partial_moves_observed += 1;
        }
        rustc_middle::mir::Rvalue::Aggregate(_, operands) => {
            for operand in operands {
                if let rustc_middle::mir::Operand::Move(moved) = operand
                    && !moved.projection.is_empty()
                {
                    state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
                    state.partial_moves_observed += 1;
                }
            }
        }
        _ => {}
    }
    let dest_ty = dest.ty(&mir_ref.body.local_decls, mir_ref.cx.tcx).ty;
    if !dest_ty.is_freeze(mir_ref.cx.tcx, mir_ref.typing_env) {
        state
            .local_regions
            .insert(dest.local.as_u32(), octet_architecture_ir::semantic::MemoryRegion::InteriorMutable);
    }
}

fn record_place_write<'tcx>(
    mir_ref: &MirBodyRef<'_, 'tcx>,
    place: rustc_middle::mir::Place<'tcx>,
    span_id: &str,
    state: &mut BodyState,
    sinks: &mut super::SemanticSinks<'_>,
) {
    let local_u32 = place.local.as_u32();
    let proj = projection_label(&place);
    let place_ty = place.ty(&mir_ref.body.local_decls, mir_ref.cx.tcx).ty;
    let is_interior_mut = !place_ty.is_freeze(mir_ref.cx.tcx, mir_ref.typing_env);
    let has_deref = place.projection.iter().any(|elem| matches!(elem, rustc_middle::mir::ProjectionElem::Deref));
    let has_field = place.projection.iter().any(|elem| matches!(elem, rustc_middle::mir::ProjectionElem::Field(..)));
    let base_ty = mir_ref.body.local_decls[place.local].ty;

    if place.projection.is_empty() && state.active_borrows.iter().any(|(l, _)| *l == local_u32) {
        state.active_borrows.retain(|(l, _)| *l != local_u32);
        state.ended_borrow_reassignments += 1;
    }

    let region = resolve_write_region(
        mir_ref,
        local_u32,
        &proj,
        place_ty,
        base_ty,
        has_deref,
        has_field,
        is_interior_mut,
        span_id,
        state,
        sinks,
    );
    let is_observable_external = matches!(
        region,
        octet_architecture_ir::semantic::MemoryRegion::ParamMutRef
            | octet_architecture_ir::semantic::MemoryRegion::EscapingAlias
            | octet_architecture_ir::semantic::MemoryRegion::InteriorMutable
            | octet_architecture_ir::semantic::MemoryRegion::RawPointer
            | octet_architecture_ir::semantic::MemoryRegion::StaticGlobal
            | octet_architecture_ir::semantic::MemoryRegion::UnknownRegion
    );
    if place.local != rustc_middle::mir::RETURN_PLACE || is_observable_external || has_deref {
        sinks.memory_writes.insert(octet_architecture_ir::semantic::MemoryWrite {
            local_index: local_u32,
            projection: proj,
            resolved_type: super::super::qualified_type(mir_ref.cx, place_ty),
            region,
            is_observable_external,
            is_interior_mutability: is_interior_mut,
            span_id: Some(span_id.to_owned()),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_write_region<'tcx>(
    mir_ref: &MirBodyRef<'_, 'tcx>,
    local_u32: u32,
    proj: &str,
    place_ty: rustc_middle::ty::Ty<'tcx>,
    base_ty: rustc_middle::ty::Ty<'tcx>,
    has_deref: bool,
    has_field: bool,
    is_interior_mut: bool,
    span_id: &str,
    state: &mut BodyState,
    sinks: &mut super::SemanticSinks<'_>,
) -> octet_architecture_ir::semantic::MemoryRegion {
    if has_deref {
        return match base_ty.kind() {
            rustc_middle::ty::RawPtr(_, _) => {
                sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                    kind: octet_architecture_ir::semantic::UnsupportedKind::RawPointerDeref,
                    detail: format!("write through raw pointer `{place_ty}`"),
                    span_id: Some(span_id.to_owned()),
                });
                octet_architecture_ir::semantic::MemoryRegion::RawPointer
            }
            rustc_middle::ty::Ref(_, referent, _) if !referent.is_freeze(mir_ref.cx.tcx, mir_ref.typing_env) => {
                octet_architecture_ir::semantic::MemoryRegion::InteriorMutable
            }
            _ => {
                let base_region = state
                    .local_regions
                    .get(&local_u32)
                    .copied()
                    .unwrap_or(octet_architecture_ir::semantic::MemoryRegion::UnknownRegion);
                if state.escaped_places.contains(&(local_u32, String::new())) {
                    octet_architecture_ir::semantic::MemoryRegion::EscapingAlias
                } else {
                    base_region
                }
            }
        };
    }
    if state.escaped_places.contains(&(local_u32, proj.to_owned()))
        || state.escaped_places.contains(&(local_u32, String::new()))
    {
        octet_architecture_ir::semantic::MemoryRegion::EscapingAlias
    } else if is_interior_mut {
        octet_architecture_ir::semantic::MemoryRegion::InteriorMutable
    } else if has_field {
        state.moved_fields.remove(&(local_u32, proj.to_owned()));
        octet_architecture_ir::semantic::MemoryRegion::LocalField
    } else {
        state
            .local_regions
            .get(&local_u32)
            .copied()
            .unwrap_or(octet_architecture_ir::semantic::MemoryRegion::OwnedLocal)
    }
}

pub(super) fn projection_label(place: &rustc_middle::mir::Place<'_>) -> String {
    if place.projection.is_empty() {
        return String::new();
    }
    let mut parts = Vec::with_capacity(place.projection.len());
    for elem in place.projection {
        if parts.len() >= place.projection.len() {
            break;
        }
        let part = match elem {
            rustc_middle::mir::ProjectionElem::Deref => "*".to_owned(),
            rustc_middle::mir::ProjectionElem::Field(field, _) => format!(".{}", field.as_usize()),
            rustc_middle::mir::ProjectionElem::Index(_) => "[_]".to_owned(),
            rustc_middle::mir::ProjectionElem::ConstantIndex { offset, .. } => format!("[{offset}]"),
            rustc_middle::mir::ProjectionElem::Subslice { from, to, .. } => format!("[{from}..{to}]"),
            rustc_middle::mir::ProjectionElem::Downcast(_, variant) => format!("@{}", variant.as_usize()),
            rustc_middle::mir::ProjectionElem::OpaqueCast(_) => "opaque".to_owned(),
            rustc_middle::mir::ProjectionElem::UnwrapUnsafeBinder(_) => "binder".to_owned(),
        };
        parts.push(part);
    }
    parts.join("")
}
