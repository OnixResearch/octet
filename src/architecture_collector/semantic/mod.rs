//! Compiler MIR semantic effect and memory-region collector.

mod body;
mod calls;
mod drops;
mod places;

pub(super) struct MirSinks<'a> {
    pub(super) drops: &'a mut std::collections::BTreeSet<octet_architecture_ir::semantic::DropEdge>,
    pub(super) calls: &'a mut std::collections::BTreeSet<octet_architecture_ir::semantic::MirCallEdge>,
    pub(super) memory_writes: &'a mut std::collections::BTreeSet<octet_architecture_ir::semantic::MemoryWrite>,
    pub(super) unsupported: &'a mut std::collections::BTreeSet<octet_architecture_ir::semantic::UnsupportedConstruct>,
}

/// Collects semantic MIR observations for the active compilation unit and publishes
/// `<unit_id>.semantic.json` inside the collector output directory.
pub(super) fn collect_and_write_unit_summary(
    collector: &mut super::ArchitectureCollector,
    cx: &rustc_lint::LateContext<'_>,
) -> Result<usize, String> {
    let mut body_ids = collect_mir_body_ids(collector, cx);
    body_ids.sort_by_key(|def_id| super::qualified_definition(cx, def_id.to_def_id()));
    body_ids.dedup();

    let max_bodies = collector.context.limits.max_value_flow_bodies.max(1);
    let mut remaining_steps = collector.context.limits.max_value_flow_steps.max(1);
    let mut is_budget_exhausted = false;
    let mut skipped_bodies = Vec::with_capacity(body_ids.len());
    let mut bodies = Vec::with_capacity(body_ids.len().min(max_bodies));

    for def_id in body_ids {
        let qualified_path = super::qualified_definition(cx, def_id.to_def_id());
        if bodies.len() >= max_bodies || remaining_steps == 0 {
            is_budget_exhausted = true;
            skipped_bodies.push(qualified_path);
            continue;
        }
        if let Ok(observation) = body::inspect_mir(collector, cx, def_id, &qualified_path, &mut remaining_steps) {
            bodies.push(observation);
        } else {
            is_budget_exhausted = true;
            skipped_bodies.push(qualified_path);
        }
    }

    let mut summary = octet_architecture_ir::semantic::UnitSummary {
        schema_version: octet_architecture_ir::semantic::SEMANTIC_MIR_UNIT_SCHEMA_VERSION.to_owned(),
        unit_id: collector.selected.unit.unit_id.clone(),
        package_name: collector.selected.unit.package_name.clone(),
        target_name: collector.selected.unit.target_name.clone(),
        compiler_stage: "rustc-mir-optimized".to_owned(),
        toolchain_blake3: collector.selected.unit.toolchain_blake3.clone(),
        source_blake3: collector.selected.unit.source_blake3.clone(),
        bodies,
        is_budget_exhausted,
        skipped_bodies,
        summary_blake3: String::new(),
    };
    summary.summary_blake3 = octet_architecture_ir::semantic::compute_unit_summary_blake3(&summary)?;

    let observed_bodies = summary.bodies.len();
    let mut bytes =
        serde_json::to_vec_pretty(&summary).map_err(|error| format!("serialize semantic MIR unit summary: {error}"))?;
    bytes.push(b'\n');

    let output_directory = std::path::Path::new(&collector.context.output_directory);
    let output_path = output_directory.join(format!("{}.semantic.json", summary.unit_id));
    let temporary_path =
        output_path.with_extension(format!("semantic.{}.{}", std::process::id(), super::TEMP_FILE_SUFFIX));
    super::publish_shard(&temporary_path, &output_path, &bytes)
        .map_err(|error| format!("publish semantic MIR unit summary {}: {error}", output_path.display()))?;
    Ok(observed_bodies)
}

fn collect_mir_body_ids(
    collector: &super::ArchitectureCollector,
    cx: &rustc_lint::LateContext<'_>,
) -> Vec<rustc_span::def_id::LocalDefId> {
    let max_ids = collector.item_ids.len();
    let mut ids = Vec::with_capacity(max_ids);
    for def_id in collector.item_ids.keys().copied() {
        let kind = cx.tcx.def_kind(def_id);
        let is_body_kind = matches!(
            kind,
            rustc_hir::def::DefKind::Fn
                | rustc_hir::def::DefKind::AssocFn
                | rustc_hir::def::DefKind::Closure
                | rustc_hir::def::DefKind::SyntheticCoroutineBody
        );
        if is_body_kind && cx.tcx.is_mir_available(def_id.to_def_id()) && ids.len() < max_ids {
            ids.push(def_id);
        }
    }
    ids
}
