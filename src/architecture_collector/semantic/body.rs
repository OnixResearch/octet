//! Per-body MIR semantic traversal: implicit drops, resolved trait/operator calls,
//! place-sensitive memory writes, and unsupported construct detection.

use octet_architecture_ir::semantic::BodyObservation;
use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::MirStage;
use octet_architecture_ir::semantic::UnsupportedConstruct;
use octet_architecture_ir::semantic::UnsupportedKind;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_lint::LateContext;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_span::def_id::LocalDefId;

use super::places::BodyState;
use super::places::projection_label;
use super::places::record_statement;

pub(super) fn analyze_body(
    collector: &mut super::super::ArchitectureCollector,
    cx: &LateContext<'_>,
    def_id: LocalDefId,
    qualified_path: &str,
    remaining_steps: &mut usize,
) -> Result<BodyObservation, ()> {
    let item_id = collector
        .item_ids
        .get(&def_id)
        .cloned()
        .unwrap_or_else(|| collector.local_item_identity(cx, def_id));
    let span_id = collector.collect_span(cx, cx.tcx.def_span(def_id));
    let mir_body = cx.tcx.optimized_mir(def_id.to_def_id());
    let typing_env = ty::TypingEnv::post_analysis(cx.tcx, def_id.to_def_id());
    let is_async = cx.tcx.def_kind(def_id) == DefKind::SyntheticCoroutineBody
        || cx.tcx.coroutine_kind(def_id.to_def_id()).is_some();
    let is_unsafe = is_fn_unsafe(cx, def_id);

    let mut drops = std::collections::BTreeSet::new();
    let mut calls = std::collections::BTreeSet::new();
    let mut memory_writes = std::collections::BTreeSet::new();
    let mut unsupported = std::collections::BTreeSet::new();

    if is_async {
        unsupported.insert(UnsupportedConstruct {
            kind: UnsupportedKind::AsyncCoroutine,
            detail: format!("async/coroutine body `{qualified_path}` is outside bounded sequential MIR"),
            span_id: Some(span_id.clone()),
        });
    }
    if is_unsafe {
        unsupported.insert(UnsupportedConstruct {
            kind: UnsupportedKind::UnsafeBlock,
            detail: format!("unsafe function `{qualified_path}` is outside safe sequential subset"),
            span_id: Some(span_id.clone()),
        });
    }

    let mut state = BodyState::from_body(cx, mir_body, typing_env);
    let mut has_unwind_edges = false;
    let mut has_implicit_panic_assert = false;
    let mut step_count = 0usize;

    for (_bb, block_data) in mir_body.basic_blocks.iter_enumerated() {
        if block_data.is_cleanup {
            has_unwind_edges = true;
        }
        for stmt in &block_data.statements {
            consume_step(remaining_steps, &mut step_count)?;
            record_statement(
                collector,
                cx,
                mir_body,
                typing_env,
                stmt,
                &mut state,
                &mut memory_writes,
                &mut unsupported,
            );
        }
        consume_step(remaining_steps, &mut step_count)?;
        record_terminator(
            collector,
            cx,
            mir_body,
            typing_env,
            block_data.is_cleanup,
            block_data.terminator(),
            &mut state,
            &mut drops,
            &mut calls,
            &mut unsupported,
            &mut has_unwind_edges,
            &mut has_implicit_panic_assert,
        );
    }

    Ok(BodyObservation {
        item_id,
        qualified_path: qualified_path.to_owned(),
        mir_stage: MirStage::OptimizedMir,
        is_unsafe,
        is_async,
        has_unwind_edges,
        has_implicit_panic_assert,
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

fn consume_step(remaining_steps: &mut usize, step_count: &mut usize) -> Result<(), ()> {
    if *remaining_steps == 0 {
        return Err(());
    }
    *remaining_steps -= 1;
    *step_count += 1;
    Ok(())
}

fn is_fn_unsafe(cx: &LateContext<'_>, def_id: LocalDefId) -> bool {
    if !matches!(cx.tcx.def_kind(def_id), DefKind::Fn | DefKind::AssocFn) {
        return false;
    }
    cx.tcx.fn_sig(def_id.to_def_id()).skip_binder().safety() == hir::Safety::Unsafe
}

#[allow(clippy::too_many_arguments)]
fn record_terminator<'tcx>(
    collector: &mut super::super::ArchitectureCollector,
    cx: &LateContext<'tcx>,
    body: &mir::Body<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    is_cleanup_block: bool,
    term: &mir::Terminator<'tcx>,
    state: &mut BodyState,
    drops: &mut std::collections::BTreeSet<octet_architecture_ir::semantic::DropEdge>,
    calls: &mut std::collections::BTreeSet<octet_architecture_ir::semantic::MirCallEdge>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
    has_unwind_edges: &mut bool,
    has_implicit_panic_assert: &mut bool,
) {
    let span_id = collector.collect_span(cx, term.source_info.span);
    match &term.kind {
        mir::TerminatorKind::Drop { place, unwind, .. } => {
            if matches!(unwind, mir::UnwindAction::Cleanup(_)) || is_cleanup_block {
                *has_unwind_edges = true;
            }
            let path_kind = if is_cleanup_block {
                DropPathKind::UnwindCleanup
            } else if matches!(unwind, mir::UnwindAction::Cleanup(_)) {
                DropPathKind::NormalAndUnwind
            } else {
                DropPathKind::Normal
            };
            let dropped_ty = place.ty(&body.local_decls, cx.tcx).ty;
            if dropped_ty.needs_drop(cx.tcx, typing_env) {
                super::drops::collect_drop_edges(
                    cx,
                    typing_env,
                    dropped_ty,
                    path_kind,
                    &span_id,
                    drops,
                    calls,
                    unsupported,
                );
            }
        }
        mir::TerminatorKind::Call {
            func,
            args,
            destination,
            unwind,
            ..
        } => {
            if matches!(unwind, mir::UnwindAction::Cleanup(_)) {
                *has_unwind_edges = true;
            }
            let dest_key = (destination.local.as_u32(), projection_label(destination));
            if destination.projection.is_empty() && state.active_borrows.iter().any(|(l, _)| *l == dest_key.0) {
                state.active_borrows.retain(|(l, _)| *l != dest_key.0);
                state.ended_borrow_reassignments += 1;
            }
            for arg in args {
                if let mir::Operand::Move(moved) = &arg.node
                    && !moved.projection.is_empty()
                {
                    state.moved_fields.insert((moved.local.as_u32(), projection_label(moved)));
                    state.partial_moves_observed += 1;
                }
            }
            super::calls::collect_call_edge(cx, typing_env, func, &span_id, drops, calls, unsupported);
        }
        mir::TerminatorKind::TailCall { func, .. } => {
            super::calls::collect_call_edge(cx, typing_env, func, &span_id, drops, calls, unsupported);
        }
        mir::TerminatorKind::Assert { unwind, .. } => {
            *has_implicit_panic_assert = true;
            if matches!(unwind, mir::UnwindAction::Cleanup(_)) {
                *has_unwind_edges = true;
            }
        }
        mir::TerminatorKind::UnwindResume | mir::TerminatorKind::UnwindTerminate(_) => {
            *has_unwind_edges = true;
        }
        mir::TerminatorKind::InlineAsm { .. } => {
            unsupported.insert(UnsupportedConstruct {
                kind: UnsupportedKind::InlineAsm,
                detail: "inline assembly terminator observed in MIR".to_owned(),
                span_id: Some(span_id),
            });
        }
        _ => {}
    }
}
