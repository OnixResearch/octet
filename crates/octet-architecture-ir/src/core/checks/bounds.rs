#[derive(Default)]
struct FactCounts {
    modules: usize,
    items: usize,
    types: usize,
    traits: usize,
    implementations: usize,
    signatures: usize,
    calls: usize,
    constructors: usize,
    effects: usize,
    expansions: usize,
    configurations: usize,
    type_shapes: usize,
    function_safety_facts: usize,
    value_provenance_facts: usize,
}

fn count_facts(facts: &[crate::model::CompilerArchitectureFact]) -> FactCounts {
    let mut counts = FactCounts::default();
    for fact in facts {
        match fact {
            crate::model::CompilerArchitectureFact::Module { .. } => counts.modules += 1,
            crate::model::CompilerArchitectureFact::Item { .. } => counts.items += 1,
            crate::model::CompilerArchitectureFact::Type { .. } => counts.types += 1,
            crate::model::CompilerArchitectureFact::Trait { .. } => counts.traits += 1,
            crate::model::CompilerArchitectureFact::Implementation { .. } => counts.implementations += 1,
            crate::model::CompilerArchitectureFact::Signature { .. } => counts.signatures += 1,
            crate::model::CompilerArchitectureFact::Call { .. } => counts.calls += 1,
            crate::model::CompilerArchitectureFact::Constructor { .. } => counts.constructors += 1,
            crate::model::CompilerArchitectureFact::Effect { .. } => counts.effects += 1,
            crate::model::CompilerArchitectureFact::Expansion { .. } => counts.expansions += 1,
            crate::model::CompilerArchitectureFact::Configuration { .. } => counts.configurations += 1,
            crate::model::CompilerArchitectureFact::TypeShape { .. } => counts.type_shapes += 1,
            crate::model::CompilerArchitectureFact::FunctionSafety { .. } => counts.function_safety_facts += 1,
            crate::model::CompilerArchitectureFact::CallArgumentProvenance { .. } => counts.value_provenance_facts += 1,
        }
    }
    counts
}

pub(super) fn validate_fact_limits(
    facts: &[crate::model::CompilerArchitectureFact],
    limits: &crate::model::ArchitectureCollectionLimits,
    unit_id: Option<&str>,
    issues: &mut super::super::diagnostics::Issues,
) {
    let counts = count_facts(facts);
    for (name, observed, limit) in [
        ("modules", counts.modules, limits.max_modules),
        ("items", counts.items, limits.max_items),
        ("types", counts.types, limits.max_types),
        ("traits", counts.traits, limits.max_traits),
        ("implementations", counts.implementations, limits.max_implementations),
        ("signatures", counts.signatures, limits.max_signatures),
        ("calls", counts.calls, limits.max_calls),
        ("constructors", counts.constructors, limits.max_constructors),
        ("effects", counts.effects, limits.max_effects),
        ("expansions", counts.expansions, limits.max_expansions),
        ("configurations", counts.configurations, limits.max_configurations),
        ("type-shapes", counts.type_shapes, limits.max_type_shapes),
        ("function-safety-facts", counts.function_safety_facts, limits.max_function_safety_facts),
        ("value-provenance-facts", counts.value_provenance_facts, limits.max_value_provenance_facts),
    ] {
        super::super::normalization::validate_count_limit(name, observed, limit, unit_id, issues);
    }
}
