use std::collections::BTreeMap;
use std::collections::BTreeSet;

const LOCKED_PACKAGE_DOMAIN: &str = "octet-effect-closure-locked-package/v1";
const CLOSURE_DOMAIN: &str = "octet-effect-closure-implementation-closure/v1";

#[derive(Clone)]
struct ScannedPackage {
    input: super::ResolvedPackageInput,
    kind: super::SourceKind,
    snapshot: super::scan::PackageSnapshot,
}

#[derive(Clone)]
struct ComputedNode {
    completeness: super::CompletenessStatus,
    diagnostics: Vec<String>,
    closure_blake3: String,
    transitive_ids: BTreeSet<String>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Done,
}

struct ResolveCtx<'a> {
    input: &'a super::ClosureMetadataInput,
    policy: &'a super::policy::Settings,
    scanned: &'a BTreeMap<String, ScannedPackage>,
    states: BTreeMap<String, VisitState>,
    computed: BTreeMap<String, ComputedNode>,
}

pub(crate) fn compute_provider_envelopes(
    input: &super::ClosureMetadataInput,
    policy: &super::policy::Settings,
) -> Vec<super::ProviderRecord> {
    let scanned = input
        .packages
        .iter()
        .map(|pkg| {
            let (kind, snapshot) = inspect_package(pkg, &input.cargo_lock_blake3, policy);
            (pkg.id.clone(), ScannedPackage {
                input: pkg.clone(),
                kind,
                snapshot,
            })
        })
        .collect::<BTreeMap<_, _>>();
    let ids = scanned.keys().cloned().collect::<Vec<_>>();
    let mut ctx = ResolveCtx {
        input,
        policy,
        scanned: &scanned,
        states: BTreeMap::new(),
        computed: BTreeMap::new(),
    };
    for id in &ids {
        visit_package(id, 0, &mut ctx);
    }
    let identities = build_identities(input, &scanned, &ctx.computed);
    let mut records = build_package_records(input, &scanned, &ctx.computed, &identities);
    super::append_toolchain_envelopes(
        &input.target,
        &input.toolchain,
        &input.build_context,
        &input.toolchain_provider_names,
        &mut records,
    );
    records.sort();
    records.dedup();
    records
}

fn build_package_records(
    input: &super::ClosureMetadataInput,
    scanned: &BTreeMap<String, ScannedPackage>,
    computed: &BTreeMap<String, ComputedNode>,
    identities: &BTreeMap<String, crate::effect_closure::ProviderIdentity>,
) -> Vec<super::ProviderRecord> {
    scanned
        .iter()
        .filter_map(|(id, node)| {
            let done = computed.get(id)?;
            let provider = identities.get(id)?.clone();
            let transitive_providers = done
                .transitive_ids
                .iter()
                .filter_map(|dep_id| identities.get(dep_id).cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            Some(super::ProviderRecord {
                provider,
                source_envelope_kind: node.kind,
                completeness: done.completeness,
                completeness_diagnostics: done.diagnostics.clone(),
                package_envelope_blake3: node.snapshot.package_envelope_blake3.clone(),
                build_context_blake3: input.build_context.blake3.clone(),
                implementation_closure_blake3: done.closure_blake3.clone(),
                generated_inputs: node.snapshot.generated_inputs.clone(),
                transitive_providers,
            })
        })
        .collect()
}

fn inspect_package(
    pkg: &super::ResolvedPackageInput,
    cargo_lock_blake3: &str,
    policy: &super::policy::Settings,
) -> (super::SourceKind, super::scan::PackageSnapshot) {
    if let Some(source) = &pkg.source {
        let payload = serde_json::json!({
            "id": pkg.id,
            "name": pkg.name,
            "version": pkg.version,
            "source": source,
            "features": pkg.features,
            "cargo_lock_blake3": super::canonical_blake3_identity(cargo_lock_blake3),
        });
        let digest = super::framed_blake3(LOCKED_PACKAGE_DOMAIN, &[payload.to_string().as_bytes()]);
        let snapshot = super::scan::PackageSnapshot {
            completeness: super::CompletenessStatus::Complete,
            diagnostics: Vec::new(),
            package_envelope_blake3: digest,
            generated_inputs: Vec::new(),
        };
        (super::SourceKind::LockedRegistryOrGit, snapshot)
    } else {
        (
            super::SourceKind::LocalSourceTree,
            super::scan::inspect_local_package(&pkg.manifest_path, &pkg.targets, policy),
        )
    }
}

fn visit_package(id: &str, depth: usize, ctx: &mut ResolveCtx<'_>) {
    if matches!(ctx.states.get(id), Some(VisitState::Done | VisitState::Visiting)) {
        return;
    }
    let Some(pkg) = ctx.scanned.get(id) else {
        return;
    };
    ctx.states.insert(id.to_owned(), VisitState::Visiting);
    let mut completeness = pkg.snapshot.completeness;
    let mut diagnostics = pkg.snapshot.diagnostics.clone();
    if let Some(reason) = &ctx.input.build_context.incomplete_reason {
        completeness = super::merge_completeness(completeness, super::CompletenessStatus::Incomplete);
        diagnostics.push(reason.clone());
    }
    if !ctx.input.has_resolve_graph {
        completeness = super::merge_completeness(completeness, super::CompletenessStatus::Incomplete);
        diagnostics.push("cargo metadata resolve graph is missing".to_owned());
    }
    if depth > ctx.policy.max_resolve_depth {
        completeness = super::merge_completeness(completeness, super::CompletenessStatus::Incomplete);
        diagnostics.push(format!(
            "dependency resolve depth exceeded {} at `{}`",
            ctx.policy.max_resolve_depth, pkg.input.name
        ));
    }
    let (transitive_ids, dep_digests) = visit_dep_edges(pkg, depth, ctx, &mut completeness, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let payload = serde_json::json!({
        "kind": if pkg.input.is_workspace_member { "workspace" } else { "cargo-package" },
        "name": pkg.input.name,
        "version": pkg.input.version,
        "target": ctx.input.target,
        "features": pkg.input.features,
        "package_envelope_blake3": pkg.snapshot.package_envelope_blake3,
        "build_context_blake3": ctx.input.build_context.blake3,
        "completeness": completeness,
        "diagnostics": diagnostics,
        "deps": dep_digests,
    });
    let closure_blake3 = super::framed_blake3(CLOSURE_DOMAIN, &[payload.to_string().as_bytes()]);
    ctx.states.insert(id.to_owned(), VisitState::Done);
    ctx.computed.insert(id.to_owned(), ComputedNode {
        completeness,
        diagnostics,
        closure_blake3,
        transitive_ids,
    });
}

fn visit_dep_edges(
    pkg: &ScannedPackage,
    depth: usize,
    ctx: &mut ResolveCtx<'_>,
    completeness: &mut super::CompletenessStatus,
    diagnostics: &mut Vec<String>,
) -> (BTreeSet<String>, Vec<serde_json::Value>) {
    let mut transitive_ids = BTreeSet::new();
    let mut edges = pkg.input.deps.iter().filter(|edge| is_active_dep_edge(edge)).cloned().collect::<Vec<_>>();
    edges.sort_by(|left, right| left.pkg_id.cmp(&right.pkg_id).then(left.dep_kinds.cmp(&right.dep_kinds)));
    let mut dep_digests = Vec::with_capacity(edges.len());
    for edge in edges {
        if ctx.states.get(&edge.pkg_id).copied() == Some(VisitState::Visiting) {
            *completeness = super::merge_completeness(*completeness, super::CompletenessStatus::Incomplete);
            diagnostics.push(format!("dependency resolve cycle involving `{}`", edge.pkg_id));
            continue;
        }
        if !ctx.scanned.contains_key(&edge.pkg_id) {
            *completeness = super::merge_completeness(*completeness, super::CompletenessStatus::Incomplete);
            diagnostics.push(format!("resolved dependency `{}` missing from metadata packages", edge.pkg_id));
            continue;
        }
        visit_package(&edge.pkg_id, depth + 1, ctx);
        if let Some(dep_done) = ctx.computed.get(&edge.pkg_id) {
            *completeness = super::merge_completeness(*completeness, dep_done.completeness);
            diagnostics.extend(
                dep_done
                    .diagnostics
                    .iter()
                    .map(|diag| format!("transitive `{}`: {diag}", ctx.scanned[&edge.pkg_id].input.name)),
            );
            transitive_ids.insert(edge.pkg_id.clone());
            transitive_ids.extend(dep_done.transitive_ids.iter().cloned());
            dep_digests.push(serde_json::json!({
                "name": ctx.scanned[&edge.pkg_id].input.name,
                "version": ctx.scanned[&edge.pkg_id].input.version,
                "dep_kinds": edge.dep_kinds,
                "implementation_closure_blake3": dep_done.closure_blake3,
            }));
        }
    }
    (transitive_ids, dep_digests)
}

fn is_active_dep_edge(edge: &super::ResolvedDepEdge) -> bool {
    edge.dep_kinds.is_empty() || edge.dep_kinds.iter().any(|kind| kind == "normal" || kind == "build")
}

fn build_identities(
    input: &super::ClosureMetadataInput,
    scanned: &BTreeMap<String, ScannedPackage>,
    computed: &BTreeMap<String, ComputedNode>,
) -> BTreeMap<String, crate::effect_closure::ProviderIdentity> {
    scanned
        .iter()
        .filter_map(|(id, pkg)| {
            let done = computed.get(id)?;
            let source = super::format_package_source(
                pkg.input.is_workspace_member,
                pkg.input.source.as_deref(),
                done.completeness,
                &done.closure_blake3,
            );
            let kind = if pkg.input.is_workspace_member {
                crate::effect_closure::ProviderKind::Workspace
            } else {
                crate::effect_closure::ProviderKind::CargoPackage
            };
            Some((id.clone(), crate::effect_closure::ProviderIdentity {
                kind,
                name: pkg.input.name.clone(),
                version: pkg.input.version.clone(),
                source,
                target: input.target.clone(),
                features: pkg.input.features.clone(),
            }))
        })
        .collect()
}
