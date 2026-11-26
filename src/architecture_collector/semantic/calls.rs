//! Resolved and parametric call-edge classification for MIR semantic analysis.

use rustc_middle::ty::TypeVisitableExt;

struct ResolvedTarget<'a, 'tcx> {
    cx: &'a rustc_lint::LateContext<'tcx>,
    callee_def_id: rustc_span::def_id::DefId,
    callee_path: &'a str,
    trait_def: Option<String>,
    instance: rustc_middle::ty::Instance<'tcx>,
}

pub(super) fn collect_call_edge<'tcx>(
    cx: &rustc_lint::LateContext<'tcx>,
    typing_env: rustc_middle::ty::TypingEnv<'tcx>,
    func: &rustc_middle::mir::Operand<'tcx>,
    span_id: &str,
    sinks: &mut super::MirSinks<'_>,
) {
    let Some((callee_def_id, generic_args)) = func.const_fn_def() else {
        record_fn_ptr(span_id, sinks);
        return;
    };

    let callee_path = super::super::qualified_definition(cx, callee_def_id);
    if super::super::definition_is_foreign(cx, callee_def_id) {
        record_foreign_call(&callee_path, span_id, sinks);
        return;
    }

    let trait_def = cx
        .tcx
        .trait_of_assoc(callee_def_id)
        .map(|trait_id| super::super::qualified_definition(cx, trait_id));

    if generic_args.has_param() {
        sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
            callee_definition: Some(callee_path),
            resolved_instance: None,
            trait_definition: trait_def,
            resolution: octet_architecture_ir::semantic::CallResolutionKind::ParametricTraitCall,
            has_generic_params: true,
            span_id: Some(span_id.to_owned()),
        });
        return;
    }

    let drop_ctx = super::drops::DropContext {
        cx,
        typing_env,
        path_kind: octet_architecture_ir::semantic::DropPathKind::NormalAndUnwind,
        span_id,
    };
    for arg_ty in generic_args.types() {
        if arg_ty.needs_drop(cx.tcx, typing_env) {
            super::drops::collect_drop_edges(&drop_ctx, arg_ty, sinks);
        }
    }

    match rustc_middle::ty::Instance::try_resolve(cx.tcx, typing_env, callee_def_id, generic_args) {
        Ok(Some(instance)) => {
            let target = ResolvedTarget {
                cx,
                callee_def_id,
                callee_path: &callee_path,
                trait_def,
                instance,
            };
            classify_resolved_instance(target, span_id, sinks);
        }
        Ok(None) | Err(_) => {
            sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
                callee_definition: Some(callee_path),
                resolved_instance: None,
                trait_definition: trait_def,
                resolution: octet_architecture_ir::semantic::CallResolutionKind::ParametricTraitCall,
                has_generic_params: true,
                span_id: Some(span_id.to_owned()),
            });
        }
    }
}

fn record_fn_ptr(span_id: &str, sinks: &mut super::MirSinks<'_>) {
    sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
        callee_definition: None,
        resolved_instance: None,
        trait_definition: None,
        resolution: octet_architecture_ir::semantic::CallResolutionKind::UnresolvedFnPtr,
        has_generic_params: false,
        span_id: Some(span_id.to_owned()),
    });
    sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
        kind: octet_architecture_ir::semantic::UnsupportedKind::UnresolvedFnPtr,
        detail: "unresolved function pointer or closure call in MIR".to_owned(),
        span_id: Some(span_id.to_owned()),
    });
}

fn record_foreign_call(callee_path: &str, span_id: &str, sinks: &mut super::MirSinks<'_>) {
    sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
        callee_definition: Some(callee_path.to_owned()),
        resolved_instance: None,
        trait_definition: None,
        resolution: octet_architecture_ir::semantic::CallResolutionKind::ForeignFfi,
        has_generic_params: false,
        span_id: Some(span_id.to_owned()),
    });
    sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
        kind: octet_architecture_ir::semantic::UnsupportedKind::ForeignFfi,
        detail: format!("foreign FFI call `{callee_path}`"),
        span_id: Some(span_id.to_owned()),
    });
}

fn classify_resolved_instance(target: ResolvedTarget<'_, '_>, span_id: &str, sinks: &mut super::MirSinks<'_>) {
    match target.instance.def {
        rustc_middle::ty::InstanceKind::Virtual(virt_def_id, _) => {
            let virt_path = super::super::qualified_definition(target.cx, virt_def_id);
            sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
                callee_definition: Some(target.callee_path.to_owned()),
                resolved_instance: Some(virt_path.clone()),
                trait_definition: target.trait_def,
                resolution: octet_architecture_ir::semantic::CallResolutionKind::DynamicDispatch,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
            sinks.unsupported.insert(octet_architecture_ir::semantic::UnsupportedConstruct {
                kind: octet_architecture_ir::semantic::UnsupportedKind::DynamicDispatch,
                detail: format!("dynamic dispatch call `{virt_path}`"),
                span_id: Some(span_id.to_owned()),
            });
        }
        rustc_middle::ty::InstanceKind::Intrinsic(_) => {
            sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
                callee_definition: Some(target.callee_path.to_owned()),
                resolved_instance: Some(target.callee_path.to_owned()),
                trait_definition: target.trait_def,
                resolution: octet_architecture_ir::semantic::CallResolutionKind::Intrinsic,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
        }
        _ => {
            let resolved_def_id = target.instance.def_id();
            let resolved_path = super::super::qualified_definition(target.cx, resolved_def_id);
            let resolution = if resolved_def_id.is_local() && target.trait_def.is_none() {
                octet_architecture_ir::semantic::CallResolutionKind::ConcreteLocal
            } else if resolved_def_id.is_local()
                || (target.trait_def.is_some() && resolved_def_id != target.callee_def_id)
            {
                octet_architecture_ir::semantic::CallResolutionKind::ConcreteTraitImpl
            } else {
                octet_architecture_ir::semantic::CallResolutionKind::ConcreteExternal
            };
            sinks.calls.insert(octet_architecture_ir::semantic::MirCallEdge {
                callee_definition: Some(target.callee_path.to_owned()),
                resolved_instance: Some(resolved_path),
                trait_definition: target.trait_def,
                resolution,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
        }
    }
}
