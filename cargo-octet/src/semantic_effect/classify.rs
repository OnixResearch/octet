//! Effect classification helpers for MIR body observations and calls.

pub(super) struct ClassifiedBodyOut<'a> {
    pub(super) effects: &'a mut std::collections::BTreeSet<super::model::MirOperation>,
    pub(super) summary_ids: &'a mut std::collections::BTreeSet<String>,
    pub(super) unsupported_reasons: &'a mut Vec<String>,
    pub(super) callees: &'a mut Vec<String>,
    pub(super) diagnostics: &'a mut Vec<super::model::ProfileDiagnostic>,
}

pub(super) fn summarize_body_ops(
    body: &octet_architecture_ir::semantic::BodyObservation,
    profile: &super::model::MirClosureProfile,
    summary_index: &std::collections::BTreeMap<String, &super::model::TrustedContractSummary>,
    out: &mut ClassifiedBodyOut<'_>,
) {
    if body.implicit_panic_assert_count > 0 {
        out.effects.insert(super::model::MirOperation::Panic);
    }
    if profile.unwind_mode == super::model::UnwindMode::RejectUnwind && body.has_unwind_edges {
        out.effects.insert(super::model::MirOperation::UnwindDrop);
        out.unsupported_reasons.push(format!("unwind edge rejected in `{}`", body.qualified_path));
    }
    for write in &body.memory_writes {
        if write.is_interior_mutability {
            out.effects.insert(super::model::MirOperation::InteriorMutation);
        } else if write.is_observable_external {
            out.effects.insert(super::model::MirOperation::ExternalMutation);
        }
    }
    for drop_edge in &body.drops {
        if profile.unwind_mode == super::model::UnwindMode::RejectUnwind
            && drop_edge.path_kind != octet_architecture_ir::semantic::DropPathKind::Normal
        {
            out.effects.insert(super::model::MirOperation::UnwindDrop);
            out.unsupported_reasons
                .push(format!("unwind drop `{}` rejected by profile", drop_edge.dropped_type));
        }
        if !drop_edge.is_resolved && drop_edge.has_effectful_candidate {
            out.effects.insert(super::model::MirOperation::Unknown);
            out.unsupported_reasons.push(format!("unresolved implicit drop `{}`", drop_edge.dropped_type));
        }
    }
    for u in &body.unsupported {
        out.effects.insert(unsupported_op(u.kind));
        out.unsupported_reasons.push(u.detail.clone());
    }
    for call in &body.calls {
        resolve_call_op(body, call, summary_index, out);
    }
}

fn resolve_call_op(
    body: &octet_architecture_ir::semantic::BodyObservation,
    call: &octet_architecture_ir::semantic::MirCallEdge,
    summary_index: &std::collections::BTreeMap<String, &super::model::TrustedContractSummary>,
    out: &mut ClassifiedBodyOut<'_>,
) {
    match call.resolution {
        octet_architecture_ir::semantic::CallResolutionKind::ConcreteLocal => {
            if let Some(target) = call.resolved_instance.as_ref().or(call.callee_definition.as_ref()) {
                out.callees.push(target.clone());
            }
        }
        octet_architecture_ir::semantic::CallResolutionKind::ConcreteTraitImpl => {
            if let Some(target) = call.resolved_instance.as_ref() {
                out.callees.push(target.clone());
                if let Some(known) = resolve_external_path(target) {
                    out.effects.extend(known);
                } else if let Some(summary) = summary_index.get(target) {
                    out.summary_ids.insert(summary.summary_id.clone());
                    out.effects.extend(summary.effects.iter().copied());
                }
            }
        }
        octet_architecture_ir::semantic::CallResolutionKind::ConcreteExternal
        | octet_architecture_ir::semantic::CallResolutionKind::Intrinsic => {
            let target = call.resolved_instance.as_deref().or(call.callee_definition.as_deref()).unwrap_or("");
            if let Some(known) = resolve_external_path(target) {
                out.effects.extend(known);
            } else if let Some(summary) = summary_index.get(target) {
                out.summary_ids.insert(summary.summary_id.clone());
                out.effects.extend(summary.effects.iter().copied());
            } else {
                out.effects.insert(super::model::MirOperation::Unknown);
                out.diagnostics.push(super::model::ProfileDiagnostic {
                    code: "external_call_unreviewed".to_owned(),
                    message: format!("external call `{target}` has no reviewed summary"),
                    item_path: Some(body.qualified_path.clone()),
                });
            }
        }
        octet_architecture_ir::semantic::CallResolutionKind::ParametricTraitCall => {
            let key = call.callee_definition.as_deref().unwrap_or("");
            if let Some(summary) = summary_index.get(key) {
                out.summary_ids.insert(summary.summary_id.clone());
                out.effects.extend(summary.effects.iter().copied());
            } else {
                out.effects.insert(super::model::MirOperation::Unknown);
                out.unsupported_reasons.push(format!("parametric trait call `{key}` lacks sound contract"));
            }
        }
        octet_architecture_ir::semantic::CallResolutionKind::DynamicDispatch
        | octet_architecture_ir::semantic::CallResolutionKind::UnresolvedFnPtr => {
            out.effects.insert(super::model::MirOperation::Unknown);
        }
        octet_architecture_ir::semantic::CallResolutionKind::ForeignFfi => {
            out.effects.insert(super::model::MirOperation::Ffi);
        }
    }
}

fn unsupported_op(kind: octet_architecture_ir::semantic::UnsupportedKind) -> super::model::MirOperation {
    match kind {
        octet_architecture_ir::semantic::UnsupportedKind::UnsafeBlock
        | octet_architecture_ir::semantic::UnsupportedKind::RawPointerDeref => super::model::MirOperation::Unsafe,
        octet_architecture_ir::semantic::UnsupportedKind::ForeignFfi => super::model::MirOperation::Ffi,
        octet_architecture_ir::semantic::UnsupportedKind::UnwindCleanup => super::model::MirOperation::UnwindDrop,
        _ => super::model::MirOperation::Unknown,
    }
}

fn resolve_external_path(path: &str) -> Option<&'static [super::model::MirOperation]> {
    if path.contains("alloc::") || path.contains("std::boxed") || path.contains("std::vec") {
        return Some(&[super::model::MirOperation::Alloc]);
    }
    if path.contains("std::fs") || path.contains("::fs::") {
        return Some(&[super::model::MirOperation::Fs]);
    }
    if path.contains("std::net") || path.contains("::net::") {
        return Some(&[super::model::MirOperation::Net]);
    }
    if path.contains("std::time") || path.contains("Instant::now") || path.contains("SystemTime::now") {
        return Some(&[super::model::MirOperation::Clock]);
    }
    if path.contains("getrandom") || path.contains("rand::") {
        return Some(&[super::model::MirOperation::Random]);
    }
    if path.contains("std::thread") || path.contains("::thread::spawn") {
        return Some(&[super::model::MirOperation::Spawn]);
    }
    if path.contains("std::sync::Mutex") || path.contains("std::sync::RwLock") {
        return Some(&[super::model::MirOperation::Lock]);
    }
    if path.contains("core::cell::") {
        return Some(&[super::model::MirOperation::InteriorMutation]);
    }
    if path.contains("core::panicking") || path.contains("std::rt::begin_panic") {
        return Some(&[super::model::MirOperation::Panic]);
    }
    None
}
