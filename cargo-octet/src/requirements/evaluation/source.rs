const RANK_STALE_BINDING: u8 = 1;
const RANK_UNSUPPORTED: u8 = 2;
const RANK_INCOMPLETE: u8 = 3;

struct RecomputedGraph {
    artifact: crate::admit::DependencyGraphArtifact,
    cargo_lock_blake3: String,
    build_context: crate::envelope::policy::BuildContextOutcome,
    envelopes: Vec<crate::envelope::ProviderRecord>,
}

struct DepCheckCtx<'a> {
    source_root: &'a std::path::Path,
    settings: &'a crate::envelope::policy::Settings,
    clause_profile: Option<&'a str>,
    cache: std::collections::BTreeMap<String, Result<RecomputedGraph, (&'static str, String)>>,
}

pub(in crate::requirements) fn snapshot_hash(
    contract: &super::super::contract::Definition,
    declared: &[super::super::contract::SourceBinding],
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) -> String {
    let mut bindings = declared
        .iter()
        .map(|binding| (binding.locator.as_str(), std::borrow::Cow::Borrowed(binding.blake3.as_str())))
        .collect::<std::collections::BTreeMap<_, _>>();
    for clause in &contract.clauses {
        for binding in &clause.source_bindings {
            let path = super::super::resolve_path(source_root, &binding.locator);
            match crate::envelope::read_regular_non_symlink_file(&path) {
                Ok(bytes) => {
                    let actual = super::super::hash_bytes(&bytes);
                    if actual != binding.blake3 {
                        diagnostics.push(format!(
                            "clause `{}` source `{}` changed: expected {}, actual {actual}",
                            clause.clause_id, binding.locator, binding.blake3
                        ));
                    }
                    if let Some(declared_hash) = bindings.get(binding.locator.as_str()) {
                        if declared_hash.as_ref() != actual {
                            diagnostics.push(format!(
                                "evidence source_snapshot `{}` does not match the checked source bytes",
                                binding.locator
                            ));
                        }
                    } else {
                        bindings.entry(binding.locator.as_str()).or_insert(std::borrow::Cow::Owned(actual));
                    }
                }
                Err(error) => diagnostics.push(format!(
                    "clause `{}` source `{}` is missing, symlinked, or unreadable: {error}",
                    clause.clause_id, binding.locator
                )),
            }
        }
    }
    let canonical = serde_json::to_vec(&bindings).unwrap_or_default();
    super::super::hash_bytes(&canonical)
}

pub(in crate::requirements) fn verify_clause_live_sources(
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    clause_diagnostics: &mut Vec<String>,
) -> Option<&'static str> {
    let mut override_status = None;
    for binding in &clause.source_bindings {
        let path = super::super::resolve_path(source_root, &binding.locator);
        match crate::envelope::read_regular_non_symlink_file(&path) {
            Ok(bytes) => {
                let actual = super::super::hash_bytes(&bytes);
                if actual != binding.blake3 {
                    clause_diagnostics.push(format!(
                        "live source `{}` changed: expected {}, actual {actual}",
                        binding.locator, binding.blake3
                    ));
                    merge_live_status(&mut override_status, super::super::STATUS_STALE_BINDING);
                }
            }
            Err(error) => {
                clause_diagnostics
                    .push(format!("live source `{}` is missing, symlinked, or unreadable: {error}", binding.locator));
                merge_live_status(&mut override_status, super::super::STATUS_INCOMPLETE);
            }
        }
    }
    let settings = match crate::envelope::policy::embedded_settings() {
        Ok(settings) => settings,
        Err(error) => {
            clause_diagnostics.push(error);
            merge_live_status(&mut override_status, super::super::STATUS_INCOMPLETE);
            return override_status;
        }
    };
    verify_clause_dependency_bindings(clause, source_root, &settings, clause_diagnostics, &mut override_status);
    override_status
}

fn merge_live_status(current: &mut Option<&'static str>, next: &'static str) {
    let rank = |status: &str| {
        if status == super::super::STATUS_INCOMPLETE {
            RANK_INCOMPLETE
        } else if status == super::super::STATUS_UNSUPPORTED {
            RANK_UNSUPPORTED
        } else {
            RANK_STALE_BINDING
        }
    };
    if current.is_none_or(|cur| rank(next) > rank(cur)) {
        *current = Some(next);
    }
}

fn verify_clause_dependency_bindings(
    clause: &super::super::contract::RequirementClause,
    source_root: &std::path::Path,
    settings: &crate::envelope::policy::Settings,
    clause_diagnostics: &mut Vec<String>,
    override_status: &mut Option<&'static str>,
) {
    if let Some(profile) = &clause.required_bindings.binding_profile
        && profile != &settings.default_binding_profile
    {
        clause_diagnostics.push(format!("clause required_bindings uses unsupported binding profile `{profile}`"));
        merge_live_status(override_status, super::super::STATUS_UNSUPPORTED);
    }
    let mut ctx = DepCheckCtx {
        source_root,
        settings,
        clause_profile: clause.required_bindings.binding_profile.as_deref(),
        cache: std::collections::BTreeMap::new(),
    };
    for dep in &clause.dependency_bindings {
        if let Some(status) = verify_single_dependency(dep, &mut ctx, clause_diagnostics) {
            merge_live_status(override_status, status);
        }
    }
}

fn verify_single_dependency(
    dep: &super::super::contract::DependencyEnvelopeBinding,
    ctx: &mut DepCheckCtx<'_>,
    diagnostics: &mut Vec<String>,
) -> Option<&'static str> {
    let profile = dep
        .binding_profile
        .as_deref()
        .or(ctx.clause_profile)
        .unwrap_or(&ctx.settings.default_binding_profile);
    if profile != ctx.settings.default_binding_profile {
        diagnostics.push(format!("dependency `{}` uses unsupported binding profile `{profile}`", dep.provider_name));
        return Some(super::super::STATUS_UNSUPPORTED);
    }
    let mut dep_status = None;
    let manifest = super::super::resolve_path(ctx.source_root, &dep.manifest_path);
    let snapshot = crate::envelope::scan::inspect_local_package(&manifest, &[], ctx.settings);
    if snapshot.completeness != crate::envelope::CompletenessStatus::Complete {
        diagnostics.push(format!("dependency `{}` incomplete: {}", dep.provider_name, snapshot.diagnostics.join("; ")));
        merge_live_status(&mut dep_status, super::super::STATUS_INCOMPLETE);
    } else if snapshot.package_envelope_blake3 != dep.package_envelope_blake3 {
        diagnostics.push(format!(
            "dependency `{}` envelope changed: expected {}, actual {}",
            dep.provider_name, dep.package_envelope_blake3, snapshot.package_envelope_blake3
        ));
        merge_live_status(&mut dep_status, super::super::STATUS_STALE_BINDING);
    }
    for rel in &dep.transitive_manifest_paths {
        let t_scan = crate::envelope::scan::inspect_local_package(
            &super::super::resolve_path(ctx.source_root, rel),
            &[],
            ctx.settings,
        );
        if t_scan.completeness != crate::envelope::CompletenessStatus::Complete {
            diagnostics.push(format!(
                "transitive `{rel}` for `{}` incomplete: {}",
                dep.provider_name,
                t_scan.diagnostics.join("; ")
            ));
            merge_live_status(&mut dep_status, super::super::STATUS_INCOMPLETE);
        }
    }
    if dep.closure_blake3.is_some() || dep.dependency_graph_path.is_some() {
        verify_closure_against_graph(dep, ctx, diagnostics, &mut dep_status);
    }
    dep_status
}

fn verify_closure_against_graph(
    dep: &super::super::contract::DependencyEnvelopeBinding,
    ctx: &mut DepCheckCtx<'_>,
    diagnostics: &mut Vec<String>,
    dep_status: &mut Option<&'static str>,
) {
    let Some(graph_path) = dep.dependency_graph_path.as_deref() else {
        diagnostics
            .push(format!("dependency `{}` declares closure_blake3 without dependency_graph_path", dep.provider_name));
        merge_live_status(dep_status, super::super::STATUS_INCOMPLETE);
        return;
    };
    if !ctx.cache.contains_key(graph_path) {
        let entry = compute_graph_entry(graph_path, ctx);
        ctx.cache.insert(graph_path.to_owned(), entry);
    }
    let graph = match ctx.cache.get(graph_path) {
        Some(Ok(graph)) => graph,
        Some(Err((status, msg))) => {
            diagnostics.push(msg.clone());
            merge_live_status(dep_status, status);
            return;
        }
        None => return,
    };
    if graph.cargo_lock_blake3 != graph.artifact.cargo_lock_blake3
        || graph.build_context.blake3 != graph.artifact.build_context_blake3
    {
        diagnostics.push(format!("dependency `{}` Cargo.lock or build context changed", dep.provider_name));
        merge_live_status(dep_status, super::super::STATUS_STALE_BINDING);
    }
    let declared = graph.artifact.provider_envelopes.iter().find(|env| {
        env.provider.name == dep.provider_name
            && env.package_envelope_blake3 == dep.package_envelope_blake3
            && dep.closure_blake3.as_ref().is_none_or(|c| &env.implementation_closure_blake3 == c)
    });
    let actual = declared.and_then(|decl| {
        graph.envelopes.iter().find(|env| {
            env.provider.kind == decl.provider.kind
                && env.provider.name == decl.provider.name
                && env.provider.version == decl.provider.version
                && env.provider.target == decl.provider.target
                && env.provider.features == decl.provider.features
        })
    });
    let Some(actual) = actual else {
        diagnostics.push(format!("dependency `{}` binding missing or stale in `{graph_path}`", dep.provider_name));
        merge_live_status(dep_status, super::super::STATUS_STALE_BINDING);
        return;
    };
    if actual.completeness != crate::envelope::CompletenessStatus::Complete {
        diagnostics.push(format!(
            "dependency `{}` closure incomplete: {}",
            dep.provider_name,
            actual.completeness_diagnostics.join("; ")
        ));
        merge_live_status(dep_status, super::super::STATUS_INCOMPLETE);
    } else if actual.package_envelope_blake3 != dep.package_envelope_blake3
        || actual.build_context_blake3 != graph.artifact.build_context_blake3
        || dep
            .closure_blake3
            .as_ref()
            .is_some_and(|expected| &actual.implementation_closure_blake3 != expected)
    {
        diagnostics.push(format!(
            "dependency `{}` transitive closure drifted (`{}`)",
            dep.provider_name, actual.implementation_closure_blake3
        ));
        merge_live_status(dep_status, super::super::STATUS_STALE_BINDING);
    }
}

fn compute_graph_entry(graph_path: &str, ctx: &DepCheckCtx<'_>) -> Result<RecomputedGraph, (&'static str, String)> {
    let bytes =
        crate::envelope::read_regular_non_symlink_file(&super::super::resolve_path(ctx.source_root, graph_path))
            .map_err(|error| (super::super::STATUS_INCOMPLETE, format!("dependency graph `{graph_path}`: {error}")))?;
    let artifact = serde_json::from_slice::<crate::admit::DependencyGraphArtifact>(&bytes).map_err(|error| {
        (super::super::STATUS_INCOMPLETE, format!("parse dependency graph `{graph_path}`: {error}"))
    })?;
    if artifact.schema_version != crate::admit::DEPENDENCY_GRAPH_SCHEMA {
        return Err((
            super::super::STATUS_INCOMPLETE,
            format!("unsupported dependency graph `{graph_path}` schema `{}`", artifact.schema_version),
        ));
    }
    let (cargo_lock_blake3, build_context, envelopes) = crate::admit::recompute_dependency_envelopes(
        &crate::admit::EnvelopeRecomputeRequest {
            workspace_root: ctx.source_root,
            target: &artifact.target,
            toolchain: &artifact.toolchain,
            rustc_version: &artifact.rustc_version,
            metadata_cargo_args: &artifact.metadata_cargo_args,
            rustflags: &artifact.rustflags,
        },
        ctx.settings,
    )
    .map_err(|error| (super::super::STATUS_INCOMPLETE, error))?;
    if let Some(reason) = &build_context.incomplete_reason {
        return Err((super::super::STATUS_INCOMPLETE, reason.clone()));
    }
    Ok(RecomputedGraph {
        artifact,
        cargo_lock_blake3,
        build_context,
        envelopes,
    })
}
