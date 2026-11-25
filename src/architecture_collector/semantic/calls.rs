//! Resolved and parametric call-edge classification for MIR semantic analysis.

use octet_architecture_ir::semantic::CallResolutionKind;
use octet_architecture_ir::semantic::DropEdge;
use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::MirCallEdge;
use octet_architecture_ir::semantic::UnsupportedConstruct;
use octet_architecture_ir::semantic::UnsupportedKind;
use rustc_lint::LateContext;
use rustc_middle::mir;
use rustc_middle::ty;
use rustc_middle::ty::TypeVisitableExt;
use rustc_span::def_id::DefId;

pub(super) fn collect_call_edge<'tcx>(
    cx: &LateContext<'tcx>,
    typing_env: ty::TypingEnv<'tcx>,
    func: &mir::Operand<'tcx>,
    span_id: &str,
    drops: &mut std::collections::BTreeSet<DropEdge>,
    calls: &mut std::collections::BTreeSet<MirCallEdge>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    let Some((callee_def_id, generic_args)) = func.const_fn_def() else {
        calls.insert(MirCallEdge {
            callee_definition: None,
            resolved_instance: None,
            trait_definition: None,
            resolution: CallResolutionKind::UnresolvedFnPtr,
            has_generic_params: false,
            span_id: Some(span_id.to_owned()),
        });
        unsupported.insert(UnsupportedConstruct {
            kind: UnsupportedKind::UnresolvedFnPtr,
            detail: "unresolved function pointer or closure call in MIR".to_owned(),
            span_id: Some(span_id.to_owned()),
        });
        return;
    };

    let callee_path = super::super::qualified_definition(cx, callee_def_id);
    if super::super::definition_is_foreign(cx, callee_def_id) {
        calls.insert(MirCallEdge {
            callee_definition: Some(callee_path.clone()),
            resolved_instance: None,
            trait_definition: None,
            resolution: CallResolutionKind::ForeignFfi,
            has_generic_params: false,
            span_id: Some(span_id.to_owned()),
        });
        unsupported.insert(UnsupportedConstruct {
            kind: UnsupportedKind::ForeignFfi,
            detail: format!("foreign FFI call `{callee_path}`"),
            span_id: Some(span_id.to_owned()),
        });
        return;
    }

    let trait_def = cx
        .tcx
        .trait_of_assoc(callee_def_id)
        .map(|trait_id| super::super::qualified_definition(cx, trait_id));

    if generic_args.has_param() {
        calls.insert(MirCallEdge {
            callee_definition: Some(callee_path),
            resolved_instance: None,
            trait_definition: trait_def,
            resolution: CallResolutionKind::ParametricTraitCall,
            has_generic_params: true,
            span_id: Some(span_id.to_owned()),
        });
        return;
    }

    for arg_ty in generic_args.types() {
        if arg_ty.needs_drop(cx.tcx, typing_env) {
            super::drops::collect_drop_edges(
                cx,
                typing_env,
                arg_ty,
                DropPathKind::NormalAndUnwind,
                span_id,
                drops,
                calls,
                unsupported,
            );
        }
    }

    match ty::Instance::try_resolve(cx.tcx, typing_env, callee_def_id, generic_args) {
        Ok(Some(instance)) => {
            classify_resolved_instance(cx, callee_def_id, callee_path, trait_def, instance, span_id, calls, unsupported)
        }
        Ok(None) | Err(_) => {
            calls.insert(MirCallEdge {
                callee_definition: Some(callee_path),
                resolved_instance: None,
                trait_definition: trait_def,
                resolution: CallResolutionKind::ParametricTraitCall,
                has_generic_params: true,
                span_id: Some(span_id.to_owned()),
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_resolved_instance<'tcx>(
    cx: &LateContext<'tcx>,
    callee_def_id: DefId,
    callee_path: String,
    trait_def: Option<String>,
    instance: ty::Instance<'tcx>,
    span_id: &str,
    calls: &mut std::collections::BTreeSet<MirCallEdge>,
    unsupported: &mut std::collections::BTreeSet<UnsupportedConstruct>,
) {
    match instance.def {
        ty::InstanceKind::Virtual(virt_def_id, _) => {
            let virt_path = super::super::qualified_definition(cx, virt_def_id);
            calls.insert(MirCallEdge {
                callee_definition: Some(callee_path),
                resolved_instance: Some(virt_path.clone()),
                trait_definition: trait_def,
                resolution: CallResolutionKind::DynamicDispatch,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
            unsupported.insert(UnsupportedConstruct {
                kind: UnsupportedKind::DynamicDispatch,
                detail: format!("dynamic dispatch call `{virt_path}`"),
                span_id: Some(span_id.to_owned()),
            });
        }
        ty::InstanceKind::Intrinsic(_) => {
            calls.insert(MirCallEdge {
                callee_definition: Some(callee_path.clone()),
                resolved_instance: Some(callee_path),
                trait_definition: trait_def,
                resolution: CallResolutionKind::Intrinsic,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
        }
        _ => {
            let resolved_def_id = instance.def_id();
            let resolved_path = super::super::qualified_definition(cx, resolved_def_id);
            let resolution = if resolved_def_id.is_local() && trait_def.is_none() {
                CallResolutionKind::ConcreteLocal
            } else if trait_def.is_some() && resolved_def_id != callee_def_id {
                CallResolutionKind::ConcreteTraitImpl
            } else if resolved_def_id.is_local() {
                CallResolutionKind::ConcreteTraitImpl
            } else {
                CallResolutionKind::ConcreteExternal
            };
            calls.insert(MirCallEdge {
                callee_definition: Some(callee_path),
                resolved_instance: Some(resolved_path),
                trait_definition: trait_def,
                resolution,
                has_generic_params: false,
                span_id: Some(span_id.to_owned()),
            });
        }
    }
}
