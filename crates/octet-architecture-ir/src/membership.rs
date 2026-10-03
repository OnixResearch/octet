//! Complete, canonical observation membership; this is not source authenticity.

const ADJACENT_COUNT: usize = 2;

// r[impl octet.architecture.unit_membership]
pub(crate) fn memberships_from_shards(shards: &[crate::CompilerArchitectureShard]) -> Vec<crate::UnitFactMembership> {
    let mut rows = shards
        .iter()
        .map(|shard| crate::UnitFactMembership {
            unit_id: shard.unit.unit_id.clone(),
            fact_ids: shard
                .facts
                .iter()
                .map(|fact| fact.fact_id().to_owned())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            unknown_ids: shard
                .unknowns
                .iter()
                .map(|unknown| unknown.unknown_id.clone())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    rows
}

/// Check the normalized membership relation, independently of its enclosing digest.
/// This verifies consistency, not the authenticity of a compiler invocation.
#[must_use]
// r[impl octet.architecture.unit_membership]
pub fn verify_ir_unit_memberships(ir: &crate::CompilerArchitectureIr) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if ir.schema_version != crate::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION {
        diagnostics.push("compiler Architecture IR membership schema is unsupported".to_owned());
    }
    let units = ir.units.iter().map(|unit| unit.unit_id.as_str()).collect::<Vec<_>>();
    let facts = ir.facts.iter().map(crate::CompilerArchitectureFact::fact_id).collect::<Vec<_>>();
    let unknowns = ir.unknowns.iter().map(|unknown| unknown.unknown_id.as_str()).collect::<Vec<_>>();
    if units.is_empty() || !strictly_ordered(&units) || !strictly_ordered(&facts) || !strictly_ordered(&unknowns) {
        diagnostics.push("compiler Architecture IR membership subjects are empty or noncanonical".to_owned());
    }
    let row_units = ir.unit_memberships.iter().map(|row| row.unit_id.as_str()).collect::<Vec<_>>();
    if row_units != units {
        diagnostics.push("compiler Architecture IR memberships do not exactly cover canonical units".to_owned());
    }
    let expected_facts = facts.into_iter().collect::<std::collections::BTreeSet<_>>();
    let expected_unknowns = unknowns.into_iter().collect::<std::collections::BTreeSet<_>>();
    let mut observed_facts = std::collections::BTreeSet::new();
    let mut observed_unknowns = std::collections::BTreeSet::new();
    let mut has_unexpected_fact = false;
    let mut has_unexpected_unknown = false;
    let mut has_noncanonical_membership = false;
    for row in &ir.unit_memberships {
        has_noncanonical_membership |= !strictly_ordered(&row.fact_ids) || !strictly_ordered(&row.unknown_ids);
        for id in &row.fact_ids {
            if expected_facts.contains(id.as_str()) {
                observed_facts.insert(id.as_str());
            } else {
                has_unexpected_fact = true;
            }
        }
        for id in &row.unknown_ids {
            if expected_unknowns.contains(id.as_str()) {
                observed_unknowns.insert(id.as_str());
            } else {
                has_unexpected_unknown = true;
            }
        }
    }
    if has_noncanonical_membership {
        diagnostics.push("compiler Architecture IR membership IDs are not canonical".to_owned());
    }
    if has_unexpected_fact || observed_facts != expected_facts {
        diagnostics
            .push("compiler Architecture IR fact memberships contain orphan or unreferenced identities".to_owned());
    }
    if has_unexpected_unknown || observed_unknowns != expected_unknowns {
        diagnostics
            .push("compiler Architecture IR unknown memberships contain orphan or unreferenced identities".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn strictly_ordered<T: Ord>(values: &[T]) -> bool {
    values.windows(ADJACENT_COUNT).all(|pair| pair[0] < pair[1])
}
