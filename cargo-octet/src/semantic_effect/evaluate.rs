//! Evaluation, correspondence checking, and fixed-point closure for semantic MIR effect receipts.

const EFFECT_COUNT: usize = 14;

pub struct EvaluationInput<'a> {
    pub profile: &'a super::model::SemanticEffectProfile,
    pub profile_blake3: &'a str,
    pub unit_summary: &'a octet_architecture_ir::semantic::UnitSummary,
    pub function_addresses: &'a [crate::effect_closure::FunctionAddressRecord],
    pub trusted_summaries: &'a [super::model::TrustedContractSummary],
    pub cargo_lock_blake3: &'a str,
}

pub fn evaluate(input: &EvaluationInput<'_>) -> Result<super::model::SemanticEffectReceipt, String> {
    let mut diagnostics = Vec::with_capacity(16);
    super::correspondence::validate_profile(input.profile, &mut diagnostics);
    super::correspondence::validate_unit_summary(input.unit_summary, input.profile, &mut diagnostics);

    let (coverage, address_by_name) = super::correspondence::check_correspondence(
        input.unit_summary,
        input.function_addresses,
        input.profile,
        &mut diagnostics,
    );
    let summary_index = index_trusted_summaries(input.trusted_summaries, input.cargo_lock_blake3, &mut diagnostics);

    let mut used_summaries = std::collections::BTreeSet::new();
    let mut unsupported_ops = std::collections::BTreeSet::new();
    let mut call_adj = Vec::with_capacity(input.unit_summary.bodies.len());
    let mut nodes = build_initial_nodes(&mut NodeBuildCtx {
        input,
        address_by_name: &address_by_name,
        summary_index: &summary_index,
        used_summaries: &mut used_summaries,
        unsupported_ops: &mut unsupported_ops,
        call_adj: &mut call_adj,
        diagnostics: &mut diagnostics,
    });

    propagate_closure(&mut nodes, &call_adj, input.profile.budgets.max_rounds, &mut diagnostics);
    check_profile_policy(&nodes, input.profile, &mut diagnostics);

    let observed_effects = collect_observed_effects(&nodes);
    let status = if diagnostics.is_empty() && coverage.is_complete {
        crate::effect_closure::AdmissionStatus::Admitted
    } else {
        crate::effect_closure::AdmissionStatus::Denied
    };

    let mut receipt = super::model::SemanticEffectReceipt {
        schema_version: super::model::RECEIPT_SCHEMA_VERSION.to_owned(),
        status,
        claim_tier: super::model::CLAIM_TIER.to_owned(),
        profile_id: input.profile.profile_id.clone(),
        unwind_mode: input.profile.unwind_mode,
        subjects: super::model::SubjectIdentities {
            unit_id: input.unit_summary.unit_id.clone(),
            package_name: input.unit_summary.package_name.clone(),
            target_name: input.unit_summary.target_name.clone(),
            toolchain_blake3: input.unit_summary.toolchain_blake3.clone(),
            source_blake3: input.unit_summary.source_blake3.clone(),
            profile_blake3: input.profile_blake3.to_owned(),
            cargo_lock_blake3: input.cargo_lock_blake3.to_owned(),
            compiler_stage: input.unit_summary.compiler_stage.clone(),
        },
        coverage,
        observed_effects,
        trusted_contracts: input
            .trusted_summaries
            .iter()
            .filter(|summary| used_summaries.contains(&summary.summary_id))
            .cloned()
            .collect(),
        unsupported_operations: unsupported_ops.into_iter().collect(),
        nodes,
        diagnostics,
        non_claims: input.profile.non_claims.clone(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = super::model::compute_receipt_identity(&receipt)?;
    Ok(receipt)
}

struct NodeBuildCtx<'a> {
    input: &'a EvaluationInput<'a>,
    address_by_name: &'a std::collections::BTreeMap<String, String>,
    summary_index: &'a std::collections::BTreeMap<String, &'a super::model::TrustedContractSummary>,
    used_summaries: &'a mut std::collections::BTreeSet<String>,
    unsupported_ops: &'a mut std::collections::BTreeSet<String>,
    call_adj: &'a mut Vec<Vec<String>>,
    diagnostics: &'a mut Vec<super::model::SemanticDiagnostic>,
}

fn index_trusted_summaries<'a>(
    summaries: &'a [super::model::TrustedContractSummary],
    cargo_lock_blake3: &str,
    diagnostics: &mut Vec<super::model::SemanticDiagnostic>,
) -> std::collections::BTreeMap<String, &'a super::model::TrustedContractSummary> {
    let mut map = std::collections::BTreeMap::new();
    for summary in summaries {
        if summary.cargo_lock_blake3 != cargo_lock_blake3 || summary.implementation_blake3.trim().is_empty() {
            push_diag(
                diagnostics,
                "trusted_summary_stale",
                &format!("trusted summary `{}` failed lock/implementation binding", summary.summary_id),
                Some(&summary.target_definition),
            );
            continue;
        }
        map.insert(summary.target_definition.clone(), summary);
    }
    map
}

fn build_initial_nodes(ctx: &mut NodeBuildCtx<'_>) -> Vec<super::model::SemanticNodeResult> {
    let mut nodes = Vec::with_capacity(ctx.input.unit_summary.bodies.len());
    for body in &ctx.input.unit_summary.bodies {
        let mut effects = std::collections::BTreeSet::new();
        let mut summary_ids = std::collections::BTreeSet::new();
        let mut unsupported_reasons = Vec::with_capacity(body.unsupported.len());
        let mut callees = Vec::with_capacity(body.calls.len());

        let mut out = super::classify::BodyClassifyOut {
            effects: &mut effects,
            summary_ids: &mut summary_ids,
            unsupported_reasons: &mut unsupported_reasons,
            callees: &mut callees,
            diagnostics: ctx.diagnostics,
        };
        super::classify::classify_body_effects(body, ctx.input.profile, ctx.summary_index, &mut out);
        ctx.used_summaries.extend(summary_ids.iter().cloned());
        ctx.unsupported_ops.extend(unsupported_reasons.iter().cloned());
        ctx.call_adj.push(callees);

        let short_name = body.qualified_path.rsplit("::").next().unwrap_or(&body.qualified_path);
        let function_address = ctx
            .address_by_name
            .get(short_name)
            .or_else(|| ctx.address_by_name.get(&body.qualified_path))
            .cloned()
            .unwrap_or_else(|| body.item_id.clone());
        let direct = effects.into_iter().collect::<Vec<_>>();
        nodes.push(super::model::SemanticNodeResult {
            item_id: body.item_id.clone(),
            qualified_path: body.qualified_path.clone(),
            function_address,
            closure_effects: direct.clone(),
            direct_effects: direct,
            trusted_summary_ids: summary_ids.into_iter().collect(),
            unsupported_reasons,
            has_private_local_mutation: body.memory_writes.iter().any(|w| !w.is_observable_external),
            has_external_mutation: body.memory_writes.iter().any(|w| w.is_observable_external),
            has_interior_mutation: body.memory_writes.iter().any(|w| w.is_interior_mutability),
            partial_moves_observed: body.partial_moves_observed,
            ended_borrow_reassignments: body.ended_borrow_reassignments,
        });
    }
    nodes
}

fn propagate_closure(
    nodes: &mut [super::model::SemanticNodeResult],
    call_adj: &[Vec<String>],
    max_rounds: usize,
    diagnostics: &mut Vec<super::model::SemanticDiagnostic>,
) {
    let index_by_path = nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| (node.qualified_path.clone(), idx))
        .collect::<std::collections::BTreeMap<_, _>>();
    let bound = max_rounds.max(nodes.len().saturating_mul(EFFECT_COUNT).saturating_add(1));
    let mut has_converged = false;
    for _ in 0..bound {
        let prev = nodes.iter().map(|node| node.closure_effects.clone()).collect::<Vec<_>>();
        let mut is_changed = false;
        for (idx, node) in nodes.iter_mut().enumerate() {
            let mut merged = node.closure_effects.iter().copied().collect::<std::collections::BTreeSet<_>>();
            for callee in &call_adj[idx] {
                if let Some(callee_idx) = index_by_path.get(callee).copied() {
                    merged.extend(prev[callee_idx].iter().copied());
                }
            }
            let next = merged.into_iter().collect::<Vec<_>>();
            if next != node.closure_effects {
                node.closure_effects = next;
                is_changed = true;
            }
        }
        if !is_changed {
            has_converged = true;
            break;
        }
    }
    if !has_converged && !nodes.is_empty() {
        push_diag(diagnostics, "fixed_point_non_convergent", "semantic closure did not converge", None);
    }
}

fn check_profile_policy(
    nodes: &[super::model::SemanticNodeResult],
    profile: &super::model::SemanticEffectProfile,
    diagnostics: &mut Vec<super::model::SemanticDiagnostic>,
) {
    let allowed = profile.allowed_effects.iter().copied().collect::<std::collections::BTreeSet<_>>();
    for node in nodes {
        for effect in node.closure_effects.iter().copied().filter(|effect| !allowed.contains(effect)) {
            push_diag(
                diagnostics,
                "semantic_effect_denied",
                &format!(
                    "function `{}` semantic closure contains denied effect `{}`",
                    node.qualified_path,
                    effect.as_str()
                ),
                Some(&node.qualified_path),
            );
        }
    }
}

fn collect_observed_effects(nodes: &[super::model::SemanticNodeResult]) -> Vec<super::model::SemanticEffect> {
    nodes
        .iter()
        .flat_map(|node| node.closure_effects.iter().copied())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn push_diag(
    diagnostics: &mut Vec<super::model::SemanticDiagnostic>,
    code: &str,
    message: &str,
    item_path: Option<&str>,
) {
    diagnostics.push(super::model::SemanticDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        item_path: item_path.map(str::to_owned),
    });
}
