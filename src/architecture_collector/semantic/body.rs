//! Per-body MIR semantic traversal: implicit drops, resolved trait/operator calls,
//! place-sensitive memory writes, and unsupported construct detection.

use super::places::BodyState;
use super::places::MirBodyRef;
use super::places::projection_label;
use super::places::record_statement;

pub(super) fn analyze_body(
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
    let is_async = cx.tcx.def_kind(def_id) == rustc_hir::def::DefKind::SyntheticCoroutineBody
        || cx.tcx.coroutine_kind(def_id.to_def_id()).is_some();
    let is_unsafe = is_fn_unsafe(cx, def_id);

    let mut drops = std::collections::BTreeSet::new();
    let mut calls = std::collections::BTreeSet::new();
    let mut memory_writes = std::collections::BTreeSet::new();
    let mut unsupported = std::collections::BTreeSet::new();
    let mut sinks = super::SemanticSinks {
        drops: &mut drops,
        calls: &mut calls,
        memory_writes: &mut memory_writes,
        unsupported: &mut unsupported,
    };
    seed_unsupported_body_flags(is_async, is_unsafe, qualified_path, &span_id, &mut sinks);

    let mut state = BodyState::from_body(cx, mir_body, typing_env);
    let mut flags = (false, 0u32, 0usize);
    let mir_ref = MirBodyRef {
        cx,
        body: mir_body,
        typing_env,
    };

    for (_bb, block_data) in mir_body.basic_blocks.iter_enumerated() {
        if block_data.is_cleanup {
            flags.0 = true;
        }
        for stmt in &block_data.statements {
            consume_step(remaining_steps, &mut flags.2)?;
            record_statement(collector, &mir_ref, stmt, &mut state, &mut sinks);
        }
        consume_step(remaining_steps, &mut flags.2)?;
        record_terminator(
            collector,
            &mir_ref,
            block_data.is_cleanup,
            block_data.terminator(),
            &mut state,
            &mut sinks,
            &mut flags,
        );
    }

    Ok(octet_architecture_ir::semantic::BodyObservation {
        item_id,
        qualified_path: qualified_path.to_owned(),
        mir_stage: octet_architecture_ir::semantic::MirStage::OptimizedMir,
        is_unsafe,
        is_async,
        has_unwind_edges: flags.0,
        implicit_panic_assert_count: flags.1,
        basic_block_count: mir_body.basic_blocks.len(),
        step_count: flags.2,
        drops: drops.into_iter().collect(),
        calls: calls.into_iter().collect(),
        memory_writes: memory_writes.into_iter().collect(),
        unsupported: unsupported.into_iter().collect(),
        partial_moves_observed: state.partial_moves_observed,
        ended_borrow_reassignments: state.ended_borrow_reassignments,
        span_id,
    })
}

fn seed_unsupported_body_flags(
    is_async: bool,
    is_unsafe: bool,
    qualified_path: &str,
    span_id: &str,
    sinks: &mut super::SemanticSinks<'_>,
) {
    if is_async {
        sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
            kind: octet_architecture_ir::semantic::UnsupportedKind::AsyncCoroutine,
            detail: format!("async/coroutine body `{qualified_path}` is outside bounded sequential MIR"),
            span_id: Some(span_id.to_owned()),
        });
    }
    if is_unsafe {
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

#[allow(clippy::too_many_arguments)]
fn record_terminator<'tcx>(
    collector: &mut super::super::ArchitectureCollector,
    mir_ref: &MirBodyRef<'_, 'tcx>,
    is_cleanup_block: bool,
    term: &rustc_middle::mir::Terminator<'tcx>,
    state: &mut BodyState,
    sinks: &mut super::SemanticSinks<'_>,
    flags: &mut (bool, u32, usize),
) {
    let span_id = collector.collect_span(mir_ref.cx, term.source_info.span);
    match &term.kind {
        rustc_middle::mir::TerminatorKind::Drop { place, unwind, .. } => {
            if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) || is_cleanup_block {
                flags.0 = true;
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
                super::drops::collect_drop_edges(
                    mir_ref.cx,
                    mir_ref.typing_env,
                    dropped_ty,
                    path_kind,
                    &span_id,
                    sinks,
                );
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
                flags.0 = true;
            }
            let dest_key = (destination.local.as_u32(), projection_label(destination));
            if destination.projection.is_empty() && state.active_borrows.iter().any(|(l, _)| *l == dest_key.0) {
                state.active_borrows.retain(|(l, _)| *l != dest_key.0);
                state.ended_borrow_reassignments += 1;
            }
            for arg in args {
                if let rustc_middle::mir::Operand::Move(moved) = &arg.node
                    && !moved.projection.is_empty()
                {
                    state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
                    state.partial_moves_observed += 1;
                }
            }
            super::calls::collect_call_edge(mir_ref.cx, mir_ref.typing_env, func, &span_id, sinks);
        }
        rustc_middle::mir::TerminatorKind::TailCall { func, .. } => {
            super::calls::collect_call_edge(mir_ref.cx, mir_ref.typing_env, func, &span_id, sinks);
        }
        rustc_middle::mir::TerminatorKind::Assert { unwind, .. } => {
            flags.1 = flags.1.saturating_add(1);
            if matches!(unwind, rustc_middle::mir::UnwindAction::Cleanup(_)) {
                flags.0 = true;
            }
        }
        rustc_middle::mir::TerminatorKind::UnwindResume | rustc_middle::mir::TerminatorKind::UnwindTerminate(_) => {
            flags.0 = true;
        }
        rustc_middle::mir::TerminatorKind::InlineAsm { .. } => {
            sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                kind: octet_architecture_ir::semantic::UnsupportedKind::InlineAsm,
                detail: "inline assembly terminator observed in MIR".to_owned(),
                span_id: Some(span_id),
            });
        }
        _ => {}
    }
}
