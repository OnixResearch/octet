//! Pure comparison of complete Cargo plans. Array indices are not semantic identities.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Graph {
    units: Vec<Unit>,
    roots: Vec<usize>,
    #[serde(flatten)]
    properties: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct Unit {
    dependencies: Vec<Dependency>,
    #[serde(flatten)]
    description: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct Dependency {
    index: usize,
    #[serde(flatten)]
    description: BTreeMap<String, Value>,
}

// r[impl octet.architecture.features.equivalence]
// r[impl octet.architecture.features.rejection]
pub(super) fn identity(bytes: &[u8]) -> Result<String, String> {
    let graph: Graph = serde_json::from_slice(bytes).map_err(|error| format!("parse complete Cargo plan: {error}"))?;
    let identities = graph.units.iter().map(unit_identity).collect::<Result<Vec<_>, _>>()?;
    if identities.is_empty() || graph.roots.is_empty() {
        return Err("Cargo feature plan has an empty selection".to_owned());
    }
    if identities.iter().collect::<BTreeSet<_>>().len() != identities.len() {
        return Err("Cargo feature plan has ambiguous duplicate unit identities".to_owned());
    }
    let mut units = Vec::with_capacity(graph.units.len());
    for (unit, identity) in graph.units.iter().zip(&identities) {
        let mut edges = Vec::with_capacity(unit.dependencies.len());
        for edge in &unit.dependencies {
            let destination = referenced_identity(&identities, edge.index)?;
            edges.push(octet_architecture_ir::canonical_identity(
                "octet.cargo-feature-plan.edge/v1",
                &(destination, &edge.description),
            )?);
        }
        edges.sort();
        units.push((identity, edges));
    }
    units.sort();
    let mut roots = graph
        .roots
        .iter()
        .map(|index| referenced_identity(&identities, *index))
        .collect::<Result<Vec<_>, _>>()?;
    roots.sort();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan/v1", &(graph.properties, units, roots))
}

fn unit_identity(unit: &Unit) -> Result<String, String> {
    let mut description = unit.description.clone();
    let features = description
        .remove("features")
        .ok_or_else(|| "Cargo feature plan omits resolved features".to_owned())?;
    let mut features: Vec<String> =
        serde_json::from_value(features).map_err(|error| format!("invalid Cargo resolved features: {error}"))?;
    features.sort();
    features.dedup();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan.unit/v1", &(description, features))
}

fn referenced_identity(identities: &[String], index: usize) -> Result<&str, String> {
    identities
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| "Cargo feature plan has an invalid unit reference".to_owned())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;

    fn graph() -> Value {
        json!({"version": 1, "units": [
            {"pkg_id": "app", "features": ["extra", "base"], "mode": "test",
             "dependencies": [{"index": 1, "extern_crate_name": "helper", "public": false}]},
            {"pkg_id": "helper", "features": [], "mode": "build", "dependencies": []}
        ], "roots": [0, 1]})
    }

    fn identity(value: &Value) -> Result<String, String> {
        super::identity(&serde_json::to_vec(value).expect("fixture JSON"))
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn order_and_indices_do_not_define_equivalence() {
        let original = graph();
        let mut reordered = original.clone();
        reordered["units"].as_array_mut().expect("units").reverse();
        reordered["units"][1]["dependencies"][0]["index"] = json!(0);
        reordered["units"][1]["features"] = json!(["base", "extra"]);
        reordered["roots"] = json!([1, 0]);
        assert_eq!(identity(&original), identity(&reordered));
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn dependency_order_does_not_define_equivalence() {
        let mut original = graph();
        original["units"][0]["dependencies"]
            .as_array_mut()
            .expect("edges")
            .push(json!({"index": 1, "extern_crate_name": "alias", "public": true}));
        let mut reordered = original.clone();
        reordered["units"][0]["dependencies"].as_array_mut().expect("edges").reverse();
        assert_eq!(identity(&original), identity(&reordered));
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn changed_features_edges_roots_and_descriptions_change_identity() {
        let original = graph();
        let expected = identity(&original).expect("valid plan");
        for (pointer, replacement) in [
            ("/units/0/features", json!(["base"])),
            ("/units/0/dependencies", json!([])),
            ("/units/0/dependencies/0/index", json!(0)),
            ("/units/0/dependencies/0/extern_crate_name", json!("renamed")),
            ("/units/0/dependencies/0/public", json!(true)),
            ("/units/0/mode", json!("build")),
            ("/roots", json!([1])),
            ("/version", json!(2)),
        ] {
            let mut altered = original.clone();
            *altered.pointer_mut(pointer).expect("existing field") = replacement;
            assert_ne!(identity(&altered).expect("valid altered plan"), expected, "{pointer}");
        }
        let mut extra_field = original.clone();
        extra_field["units"][0]["future_cargo_field"] = json!("must remain bound");
        assert_ne!(identity(&extra_field).expect("extra field"), expected);
    }

    // r[verify octet.architecture.features.rejection]
    #[test]
    fn incomplete_or_ambiguous_plans_fail_closed() {
        for (pointer, replacement) in [
            ("/units/0/dependencies/0/index", json!(2)),
            ("/units/0/dependencies/0/index", json!(-1)),
            ("/roots", json!([2])),
            ("/roots", json!([])),
            ("/units", json!([])),
            ("/units/0/dependencies", Value::Null),
            ("/units/0/features", Value::Null),
        ] {
            let mut altered = graph();
            *altered.pointer_mut(pointer).expect("existing field") = replacement;
            assert!(identity(&altered).is_err(), "{pointer}");
        }
        let mut duplicate = graph();
        duplicate["units"][1] = duplicate["units"][0].clone();
        assert!(identity(&duplicate).expect_err("ambiguous units").contains("duplicate"));
        for field in ["dependencies", "features"] {
            let mut missing = graph();
            missing["units"][0].as_object_mut().expect("unit").remove(field);
            assert!(identity(&missing).is_err(), "{field}");
        }
    }
}
