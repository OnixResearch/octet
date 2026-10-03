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

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Unvisited,
    Visiting,
    Resolved,
}

// r[impl octet.architecture.features.equivalence]
// r[impl octet.architecture.features.rejection]
pub(super) fn identity(bytes: &[u8]) -> Result<String, String> {
    let graph: Graph = serde_json::from_slice(bytes).map_err(|error| format!("parse complete Cargo plan: {error}"))?;
    if graph.units.is_empty() || graph.roots.is_empty() {
        return Err("Cargo feature plan has an empty selection".to_owned());
    }
    if graph.roots.iter().any(|&root| root >= graph.units.len()) {
        return Err("Cargo feature plan has an invalid unit reference".to_owned());
    }
    let mut identities = vec![None; graph.units.len()];
    let mut states = vec![VisitState::Unvisited; graph.units.len()];
    let mut stack = Vec::new();
    for start in 0..graph.units.len() {
        if states[start] != VisitState::Unvisited {
            continue;
        }
        states[start] = VisitState::Visiting;
        stack.push((start, 0));
        while let Some(&(index, next_edge)) = stack.last() {
            let unit = &graph.units[index];
            if let Some(edge) = unit.dependencies.get(next_edge) {
                stack.last_mut().expect("active unit").1 += 1;
                let Some(state) = states.get_mut(edge.index) else {
                    return Err("Cargo feature plan has an invalid unit reference".to_owned());
                };
                match state {
                    VisitState::Unvisited => {
                        *state = VisitState::Visiting;
                        stack.push((edge.index, 0));
                    }
                    VisitState::Visiting => return Err("Cargo feature plan has a cyclic unit dependency".to_owned()),
                    VisitState::Resolved => {}
                }
                continue;
            }
            let mut edges = Vec::with_capacity(unit.dependencies.len());
            for edge in &unit.dependencies {
                let destination = referenced_identity(&identities, edge.index)?;
                edges.push(octet_architecture_ir::canonical_identity(
                    "octet.cargo-feature-plan.edge/v1",
                    &(destination, &edge.description),
                )?);
            }
            edges.sort();
            identities[index] = Some(unit_identity(unit, &edges)?);
            states[index] = VisitState::Resolved;
            stack.pop();
        }
    }
    if identities.iter().collect::<BTreeSet<_>>().len() != identities.len() {
        return Err("Cargo feature plan has ambiguous duplicate unit identities".to_owned());
    }
    let mut units = identities.iter().map(|identity| identity.as_deref().expect("resolved unit")).collect::<Vec<_>>();
    units.sort();
    let mut roots = graph
        .roots
        .iter()
        .map(|index| referenced_identity(&identities, *index))
        .collect::<Result<Vec<_>, _>>()?;
    roots.sort();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan/v2", &(graph.properties, units, roots))
}

fn unit_identity(unit: &Unit, edges: &[String]) -> Result<String, String> {
    let features = unit
        .description
        .get("features")
        .ok_or_else(|| "Cargo feature plan omits resolved features".to_owned())?;
    let mut features = features
        .as_array()
        .ok_or_else(|| "invalid Cargo resolved features".to_owned())?
        .iter()
        .map(|feature| feature.as_str().ok_or_else(|| "invalid Cargo resolved features".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    features.sort();
    features.dedup();
    let description = unit.description.iter().filter(|(key, _)| key.as_str() != "features").collect::<BTreeMap<_, _>>();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan.unit/v2", &(description, features, edges))
}

fn referenced_identity(identities: &[Option<String>], index: usize) -> Result<&str, String> {
    identities
        .get(index)
        .and_then(Option::as_deref)
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
    fn equal_descriptions_with_distinct_resolved_children_are_unambiguous() {
        let original = json!({"version": 1, "units": [
            {"pkg_id": "app", "features": [], "dependencies": [
                {"index": 1, "extern_crate_name": "normal"},
                {"index": 2, "extern_crate_name": "build"}
            ]},
            {"pkg_id": "toml", "features": ["std"], "dependencies": [
                {"index": 3, "extern_crate_name": "serde_core"}
            ]},
            {"pkg_id": "toml", "features": ["std"], "dependencies": [
                {"index": 4, "extern_crate_name": "serde_core"}
            ]},
            {"pkg_id": "serde_core", "features": ["alloc", "default", "result", "std"], "dependencies": []},
            {"pkg_id": "serde_core", "features": ["alloc", "std"], "dependencies": []}
        ], "roots": [0]});
        let expected = identity(&original).expect("distinct dependency feature selections");
        let mut reordered = original.clone();
        reordered["units"].as_array_mut().expect("units").swap(1, 2);
        reordered["units"][0]["dependencies"][0]["index"] = json!(2);
        reordered["units"][0]["dependencies"][1]["index"] = json!(1);
        assert_eq!(identity(&reordered).expect("reindexed plan"), expected);
        let mut changed = original.clone();
        changed["units"][2]["dependencies"][0]["index"] = json!(3);
        assert!(identity(&changed).expect_err("indistinguishable resolved units").contains("duplicate"));
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn changed_features_edges_roots_and_descriptions_change_identity() {
        let original = graph();
        let expected = identity(&original).expect("valid plan");
        for (pointer, replacement) in [
            ("/units/0/features", json!(["base"])),
            ("/units/0/dependencies", json!([])),
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
            ("/units/0/dependencies/0/index", json!(0)),
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
        let helper = duplicate["units"][1].clone();
        duplicate["units"].as_array_mut().expect("units").push(helper);
        assert!(identity(&duplicate).expect_err("ambiguous units").contains("duplicate"));
        for field in ["dependencies", "features"] {
            let mut missing = graph();
            missing["units"][0].as_object_mut().expect("unit").remove(field);
            assert!(identity(&missing).is_err(), "{field}");
        }
    }

    // r[verify octet.architecture.features.rejection]
    #[test]
    fn deep_plans_are_iterative_and_disconnected_cycles_fail_closed() {
        let units = (0..4_096)
            .map(|index| {
                json!({"pkg_id": format!("unit-{index}"), "features": [],
                       "dependencies": if index == 4_095 { vec![] } else { vec![json!({"index": index + 1})] }})
            })
            .collect::<Vec<_>>();
        let mut plan = json!({"version": 1, "units": units, "roots": [0]});
        identity(&plan).expect("deep acyclic plan");
        plan["units"].as_array_mut().expect("units").push(json!({
            "pkg_id": "disconnected-cycle", "features": [],
            "dependencies": [{"index": 4_096}]
        }));
        assert!(identity(&plan).expect_err("cyclic plan").contains("cyclic"));
    }
}
