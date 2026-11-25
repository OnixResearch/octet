//! Evaluation, correspondence checking, and fixed-point closure for semantic MIR effect receipts.

use octet_architecture_ir::semantic::UnitSummary;

use super::correspondence::check_correspondence;
use super::correspondence::validate_profile;
use super::correspondence::validate_unit_summary;
use super::model::SemanticDiagnostic;
use super::model::SemanticEffect;
use super::model::SemanticEffectProfile;
use super::model::SemanticEffectReceipt;
use super::model::SemanticNodeResult;
use super::model::SubjectIdentities;
use super::model::TrustedContractSummary;

const EFFECT_COUNT: usize = 14;

pub struct EvaluationInput<'a> {
    pub profile: &'a SemanticEffectProfile,
    pub profile_blake3: &'a str,
    pub unit_summary: &'a UnitSummary,
    pub function_addresses: &'a [crate::effect_closure::FunctionAddressRecord],
    pub trusted_summaries: &'a [TrustedContractSummary],
    pub cargo_lock_blake3: &'a str,
}

pub fn evaluate(input: &EvaluationInput<'_>) -> Result<SemanticEffectReceipt, String> {
    let mut diagnostics = Vec::new();
    validate_profile(input.profile, &mut diagnostics);
    validate_unit_summary(input.unit_summary, input.profile, &mut diagnostics);

    let (coverage, address_by_name) =
        check_correspondence(input.unit_summary, input.function_addresses, input.profile, &mut diagnostics);
    let summary_index = index_trusted_summaries(input.trusted_summaries, input.cargo_lock_blake3, &mut diagnostics);

    let mut used_summaries = std::collections::BTreeSet::new();
    let mut unsupported_ops = std::collections::BTreeSet::new();
    let mut call_adj = Vec::with_capacity(input.unit_summary.bodies.len());
    let mut nodes = build_initial_nodes(
        input,
        &address_by_name,
        &summary_index,
        &mut used_summaries,
        &mut unsupported_ops,
        &mut call_adj,
        &mut diagnostics,
    );

    propagate_closure(&mut nodes, &call_adj, input.profile.budgets.max_rounds, &mut diagnostics);
    check_profile_policy(&nodes, input.profile, &mut diagnostics);

    let observed_effects = collect_observed_effects(&nodes);
    let trusted_contracts = input
        .trusted_summaries
        .iter()
        .filter(|summary| used_summaries.contains(&summary.summary_id))
        .cloned()
        .collect::<Vec<_>>();
    let status = if diagnostics.is_empty() && coverage.is_complete {
        crate::effect_closure::AdmissionStatus::Admitted
    } else {
        crate::effect_closure::AdmissionStatus::Denied
    };

    let mut receipt = SemanticEffectReceipt {
        schema_version: super::model::RECEIPT_SCHEMA_VERSION.to_owned(),
        status,
        claim_tier: input.profile.claim_tier.clone(),
        profile_id: input.profile.profile_id.clone(),
        unwind_mode: input.profile.unwind_mode,
        subjects: SubjectIdentities {
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
        trusted_contracts,
        unsupported_operations: unsupported_ops.into_iter().collect(),
        nodes,
        diagnostics,
        non_claims: input.profile.non_claims.clone(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = super::model::compute_receipt_identity(&receipt)?;
    Ok(receipt)
}

fn index_trusted_summaries<'a>(
    summaries: &'a [TrustedContractSummary],
    cargo_lock_blake3: &str,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> std::collections::BTreeMap<String, &'a TrustedContractSummary> {
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

#[allow(clippy::too_many_arguments)]
fn build_initial_nodes(
    input: &EvaluationInput<'_>,
    address_by_name: &std::collections::BTreeMap<String, String>,
    summary_index: &std::collections::BTreeMap<String, &TrustedContractSummary>,
    used_summaries: &mut std::collections::BTreeSet<String>,
    unsupported_ops: &mut std::collections::BTreeSet<String>,
    call_adj: &mut Vec<Vec<String>>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Vec<SemanticNodeResult> {
    let mut nodes = Vec::with_capacity(input.unit_summary.bodies.len());
    for body in &input.unit_summary.bodies {
        let mut effects = std::collections::BTreeSet::new();
        let mut summary_ids = std::collections::BTreeSet::new();
        let mut unsupported_reasons = Vec::new();
        let mut callees = Vec::new();

        super::classify::classify_body_effects(
            body,
            input.profile,
            summary_index,
            &mut effects,
            &mut summary_ids,
            &mut unsupported_reasons,
            &mut callees,
            diagnostics,
        );
        used_summaries.extend(summary_ids.iter().cloned());
        unsupported_ops.extend(unsupported_reasons.iter().cloned());
        call_adj.push(callees);

        let short_name = body.qualified_path.rsplit("::").next().unwrap_or(&body.qualified_path);
        let function_address = address_by_name
            .get(short_name)
            .or_else(|| address_by_name.get(&body.qualified_path))
            .cloned()
            .unwrap_or_else(|| body.item_id.clone());
        let direct = effects.into_iter().collect::<Vec<_>>();
        nodes.push(SemanticNodeResult {
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
    nodes: &mut [SemanticNodeResult],
    call_adj: &[Vec<String>],
    max_rounds: usize,
    diagnostics: &mut Vec<SemanticDiagnostic>,
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
    nodes: &[SemanticNodeResult],
    profile: &SemanticEffectProfile,
    diagnostics: &mut Vec<SemanticDiagnostic>,
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

fn collect_observed_effects(nodes: &[SemanticNodeResult]) -> Vec<SemanticEffect> {
    nodes
        .iter()
        .flat_map(|node| node.closure_effects.iter().copied())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn push_diag(diagnostics: &mut Vec<SemanticDiagnostic>, code: &str, message: &str, item_path: Option<&str>) {
    diagnostics.push(SemanticDiagnostic {
        code: code.to_owned(),
        message: message.to_owned(),
        item_path: item_path.map(str::to_owned),
    });
}
