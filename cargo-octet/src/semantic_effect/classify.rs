//! Effect classification helpers for MIR body observations and calls.

use octet_architecture_ir::semantic::BodyObservation;
use octet_architecture_ir::semantic::CallResolutionKind;
use octet_architecture_ir::semantic::DropPathKind;
use octet_architecture_ir::semantic::UnsupportedKind;

use super::model::SemanticDiagnostic;
use super::model::SemanticEffect;
use super::model::SemanticEffectProfile;
use super::model::TrustedContractSummary;
use super::model::UnwindMode;

#[allow(clippy::too_many_arguments)]
pub(super) fn classify_body_effects(
    body: &BodyObservation,
    profile: &SemanticEffectProfile,
    summary_index: &std::collections::BTreeMap<String, &TrustedContractSummary>,
    effects: &mut std::collections::BTreeSet<SemanticEffect>,
    summary_ids: &mut std::collections::BTreeSet<String>,
    unsupported_reasons: &mut Vec<String>,
    callees: &mut Vec<String>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    if body.has_implicit_panic_assert {
        effects.insert(SemanticEffect::Panic);
    }
    if profile.unwind_mode == UnwindMode::RejectUnwind && body.has_unwind_edges {
        effects.insert(SemanticEffect::UnwindDrop);
        unsupported_reasons.push(format!("unwind edge rejected in `{}`", body.qualified_path));
    }
    for write in &body.memory_writes {
        if write.is_interior_mutability {
            effects.insert(SemanticEffect::InteriorMutation);
        } else if write.is_observable_external {
            effects.insert(SemanticEffect::ExternalMutation);
        }
    }
    for drop in &body.drops {
        if !drop.is_resolved {
            effects.insert(SemanticEffect::Unknown);
            unsupported_reasons.push(format!("unresolved drop `{}`", drop.dropped_type));
        } else if profile.unwind_mode == UnwindMode::RejectUnwind
            && matches!(drop.path_kind, DropPathKind::UnwindCleanup | DropPathKind::NormalAndUnwind)
        {
            effects.insert(SemanticEffect::UnwindDrop);
            unsupported_reasons.push(format!("unwind drop `{}` rejected", drop.dropped_type));
        }
    }
    for item in &body.unsupported {
        unsupported_reasons.push(item.detail.clone());
        effects.insert(effect_for_unsupported(item.kind));
    }
    for call in &body.calls {
        classify_call_effect(
            body,
            call,
            summary_index,
            effects,
            summary_ids,
            unsupported_reasons,
            callees,
            diagnostics,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_call_effect(
    body: &BodyObservation,
    call: &octet_architecture_ir::semantic::MirCallEdge,
    summary_index: &std::collections::BTreeMap<String, &TrustedContractSummary>,
    effects: &mut std::collections::BTreeSet<SemanticEffect>,
    summary_ids: &mut std::collections::BTreeSet<String>,
    unsupported_reasons: &mut Vec<String>,
    callees: &mut Vec<String>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    match call.resolution {
        CallResolutionKind::ConcreteLocal => {
            if let Some(target) = call.resolved_instance.as_ref().or(call.callee_definition.as_ref()) {
                callees.push(target.clone());
            }
        }
        CallResolutionKind::ConcreteTraitImpl => {
            if let Some(target) = call.resolved_instance.as_ref() {
                callees.push(target.clone());
                if let Some(known) = classify_external_path(target) {
                    effects.extend(known);
                } else if let Some(summary) = summary_index.get(target) {
                    summary_ids.insert(summary.summary_id.clone());
                    effects.extend(summary.effects.iter().copied());
                }
            }
        }
        CallResolutionKind::ConcreteExternal | CallResolutionKind::Intrinsic => {
            let target = call.resolved_instance.as_deref().or(call.callee_definition.as_deref()).unwrap_or("");
            if let Some(known) = classify_external_path(target) {
                effects.extend(known);
            } else if let Some(summary) = summary_index.get(target) {
                summary_ids.insert(summary.summary_id.clone());
                effects.extend(summary.effects.iter().copied());
            } else {
                effects.insert(SemanticEffect::Unknown);
                diagnostics.push(SemanticDiagnostic {
                    code: "external_call_unreviewed".to_owned(),
                    message: format!("external call `{target}` has no reviewed summary"),
                    item_path: Some(body.qualified_path.clone()),
                });
            }
        }
        CallResolutionKind::ParametricTraitCall => {
            let key = call.callee_definition.as_deref().unwrap_or("");
            if let Some(summary) = summary_index.get(key) {
                summary_ids.insert(summary.summary_id.clone());
                effects.extend(summary.effects.iter().copied());
            } else {
                effects.insert(SemanticEffect::Unknown);
                unsupported_reasons.push(format!("parametric trait call `{key}` lacks sound contract"));
            }
        }
        CallResolutionKind::DynamicDispatch | CallResolutionKind::UnresolvedFnPtr => {
            effects.insert(SemanticEffect::Unknown);
        }
        CallResolutionKind::ForeignFfi => {
            effects.insert(SemanticEffect::Ffi);
        }
    }
}

fn effect_for_unsupported(kind: UnsupportedKind) -> SemanticEffect {
    match kind {
        UnsupportedKind::UnsafeBlock | UnsupportedKind::RawPointerDeref => SemanticEffect::Unsafe,
        UnsupportedKind::ForeignFfi => SemanticEffect::Ffi,
        UnsupportedKind::UnwindCleanup => SemanticEffect::UnwindDrop,
        _ => SemanticEffect::Unknown,
    }
}

fn classify_external_path(path: &str) -> Option<Vec<SemanticEffect>> {
    if path.starts_with("alloc::") || path.starts_with("std::vec::") || path.starts_with("std::string::") {
        Some(vec![SemanticEffect::Alloc])
    } else if path.starts_with("std::fs::") {
        Some(vec![SemanticEffect::Fs])
    } else if path.starts_with("std::net::") {
        Some(vec![SemanticEffect::Net])
    } else if path.contains("Instant::now") || path.contains("SystemTime::now") {
        Some(vec![SemanticEffect::Clock])
    } else if path.starts_with("std::sync::Mutex") || path.starts_with("std::sync::RwLock") {
        Some(vec![SemanticEffect::Lock])
    } else if path.starts_with("std::thread::spawn") {
        Some(vec![SemanticEffect::Spawn])
    } else if path.starts_with("core::panicking::") || path.starts_with("std::panicking::") {
        Some(vec![SemanticEffect::Panic])
    } else if path.starts_with("core::intrinsics::")
        || path.starts_with("core::ops::")
        || path.starts_with("core::cmp::")
    {
        Some(Vec::new())
    } else {
        None
    }
}
