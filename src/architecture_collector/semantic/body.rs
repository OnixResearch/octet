//! Per-body MIR semantic traversal: implicit drops, resolved trait/operator calls,
//! place-sensitive memory writes, and unsupported construct detection.

pub(super) fn inspect_mir(
    collector: &mut super::super::ArchitectureCollector,
    cx: &rustc_lint::LateContext<'_>,
    def_id: rustc_span::def_id::LocalDefId,
    qualified_path: &str,
    remaining_steps: &mut usize,
) -> Result<octet_architecture_ir::semantic::BodyObservation, ()> {
    let item_id = collector
        .item_ids
        .get(&def_id)
        .cloned()
        .unwrap_or_else(|| collector.local_item_identity(cx, def_id));
    let span_id = collector.collect_span(cx, cx.tcx.def_span(def_id));
    let mir_body = cx.tcx.optimized_mir(def_id.to_def_id());
    let typing_env = rustc_middle::ty::TypingEnv::post_analysis(cx.tcx, def_id.to_def_id());
    let flags = UnsupportedFlags {
        is_async: cx.tcx.def_kind(def_id) == rustc_hir::def::DefKind::SyntheticCoroutineBody
            || cx.tcx.coroutine_kind(def_id.to_def_id()).is_some(),
        is_unsafe: is_fn_unsafe(cx, def_id),
    };

    let mut drops = std::collections::BTreeSet::new();
    let mut calls = std::collections::BTreeSet::new();
    let mut memory_writes = std::collections::BTreeSet::new();
    let mut unsupported = std::collections::BTreeSet::new();
    let mut sinks = super::MirSinks {
        drops: &mut drops,
        calls: &mut calls,
        memory_writes: &mut memory_writes,
        unsupported: &mut unsupported,
    };
    seed_unsupported_flags(flags, qualified_path, &span_id, &mut sinks);

    let mut state = super::places::BodyState::from_body(cx, mir_body, typing_env);
    let mut acc = WalkAccumulator {
        state: &mut state,
        sinks: &mut sinks,
        has_unwind_edges: false,
        implicit_panic_assert_count: 0,
        step_count: 0,
    };
    let mir_ref = super::places::MirBodyRef {
        cx,
        body: mir_body,
        typing_env,
    };

    for (_bb, block_data) in mir_body.basic_blocks.iter_enumerated() {
        if block_data.is_cleanup {
            acc.has_unwind_edges = true;
        }
        for stmt in &block_data.statements {
            consume_step(remaining_steps, &mut acc.step_count)?;
            super::places::record_statement(collector, &mir_ref, stmt, acc.state, acc.sinks);
        }
        consume_step(remaining_steps, &mut acc.step_count)?;
        record_terminator(collector, &mir_ref, block_data.is_cleanup, block_data.terminator(), &mut acc);
    }

    let has_unwind_edges = acc.has_unwind_edges;
    let implicit_panic_assert_count = acc.implicit_panic_assert_count;
    let step_count = acc.step_count;
    Ok(octet_architecture_ir::semantic::BodyObservation {
        item_id,
        qualified_path: qualified_path.to_owned(),
        mir_stage: octet_architecture_ir::semantic::MirStage::OptimizedMir,
        is_unsafe: flags.is_unsafe,
        is_async: flags.is_async,
        has_unwind_edges,
        implicit_panic_assert_count,
        basic_block_count: mir_body.basic_blocks.len(),
        step_count,
        drops: drops.into_iter().collect(),
        calls: calls.into_iter().collect(),
        memory_writes: memory_writes.into_iter().collect(),
        unsupported: unsupported.into_iter().collect(),
        partial_moves_observed: state.partial_moves_observed,
        ended_borrow_reassignments: state.ended_borrow_reassignments,
        span_id,
    })
}

#[derive(Clone, Copy)]
struct UnsupportedFlags {
    is_async: bool,
    is_unsafe: bool,
}

struct WalkAccumulator<'a, 's> {
    state: &'a mut super::places::BodyState,
    sinks: &'a mut super::MirSinks<'s>,
    has_unwind_edges: bool,
    implicit_panic_assert_count: u32,
    step_count: usize,
}

fn seed_unsupported_flags(
    flags: UnsupportedFlags,
    qualified_path: &str,
    span_id: &str,
    sinks: &mut super::MirSinks<'_>,
) {
    if flags.is_async {
        sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
            kind: octet_architecture_ir::semantic::UnsupportedKind::AsyncCoroutine,
            detail: format!("async/coroutine body `{qualified_path}` is outside bounded sequential MIR"),
            span_id: Some(span_id.to_owned()),
        });
    }
    if flags.is_unsafe {
        sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
            kind: octet_architecture_ir::semantic::UnsupportedKind::UnsafeBlock,
            detail: format!("unsafe function `{qualified_path}` is outside safe sequential subset"),
            span_id: Some(span_id.to_owned()),
        });
    }
}

fn consume_step(remaining_steps: &mut usize, step_count: &mut usize) -> Result<(), ()> {
    if *remaining_steps == 0 {
        return Err(());
    }
    *remaining_steps -= 1;
    *step_count += 1;
    Ok(())
}

fn is_fn_unsafe(cx: &rustc_lint::LateContext<'_>, def_id: rustc_span::def_id::LocalDefId) -> bool {
    if !matches!(cx.tcx.def_kind(def_id), rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn) {
        return false;
    }
    cx.tcx.fn_sig(def_id.to_def_id()).skip_binder().safety() == rustc_hir::Safety::Unsafe
}

fn record_terminator<'tcx>(
    collector: &mut super::super::ArchitectureCollector,
    mir_ref: &super::places::MirBodyRef<'_, 'tcx>,
    is_cleanup_block: bool,
    term: &rustc_middle::mir::Terminator<'tcx>,
    acc: &mut WalkAccumulator<'_, '_>,
) {
    let span_id = collector.collect_span(mir_ref.cx, term.source_info.span);
    match &term.kind {
        rustc_middle::mir::TerminatorKind::Drop { place, unwind, .. } => {
            if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) || is_cleanup_block {
                acc.has_unwind_edges = true;
            }
            let path_kind = if is_cleanup_block {
                octet_architecture_ir::semantic::DropPathKind::UnwindCleanup
            } else if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) {
                octet_architecture_ir::semantic::DropPathKind::NormalAndUnwind
            } else {
                octet_architecture_ir::semantic::DropPathKind::Normal
            };
            let dropped_ty = place.ty(&mir_ref.body.local_decls, mir_ref.cx.tcx).ty;
            if dropped_ty.needs_drop(mir_ref.cx.tcx, mir_ref.typing_env) {
                let drop_ctx = super::drops::DropContext {
                    cx: mir_ref.cx,
                    typing_env: mir_ref.typing_env,
                    path_kind,
                    span_id: &span_id,
                };
                super::drops::collect_drop_edges(&drop_ctx, dropped_ty, acc.sinks);
            }
        }
        rustc_middle::mir::TerminatorKind::Call {
            func,
            args,
            destination,
            unwind,
            ..
        } => {
            if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) {
                acc.has_unwind_edges = true;
            }
            record_call_operands(args, destination, acc.state);
            super::calls::collect_call_edge(mir_ref.cx, mir_ref.typing_env, func, &span_id, acc.sinks);
        }
        rustc_middle::mir::TerminatorKind::TailCall { func, .. } => {
            super::calls::collect_call_edge(mir_ref.cx, mir_ref.typing_env, func, &span_id, acc.sinks);
        }
        rustc_middle::mir::TerminatorKind::Assert { unwind, .. } => {
            acc.implicit_panic_assert_count = acc.implicit_panic_assert_count.saturating_add(1);
            if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) {
                acc.has_unwind_edges = true;
            }
        }
        rustc_middle::mir::TerminatorKind::UnwindResume | rustc_middle::mir::TerminatorKind::UnwindTerminate(_) => {
            acc.has_unwind_edges = true;
        }
        rustc_middle::mir::TerminatorKind::InlineAsm { .. } => {
            acc.sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                kind: octet_architecture_ir::semantic::UnsupportedKind::InlineAsm,
                detail: "inline assembly terminator observed in MIR".to_owned(),
                span_id: Some(span_id),
            });
        }
        _ => {}
    }
}

fn record_call_operands<'tcx>(
    args: &[rustc_span::Spanned<rustc_middle::mir::Operand<'tcx>>],
    destination: &rustc_middle::mir::Place<'tcx>,
    state: &mut super::places::BodyState,
) {
    let dest_local = destination.local.as_u32();
    if destination.projection.is_empty() && state.active_borrows.iter().any(|(l, _)| *l == dest_local) {
        state.active_borrows.retain(|(l, _)| *l != dest_local);
        state.ended_borrow_reassignments += 1;
    }
    for arg in args {
        if let rustc_middle::mir::Operand::Move(moved) = &arg.node
            && !moved.projection.is_empty()
        {
            state.moved_fields.insert((moved.local.as_u32(), super::places::projection_label(moved)));
            state.partial_moves_observed += 1;
        }
    }
}
