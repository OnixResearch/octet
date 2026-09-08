//! Cargo owns target selection. This adapter obtains a bounded, non-executing build graph.

use std::path::Path;

use serde::Deserialize;

const UNIT_GRAPH_VERSION: u64 = 1;
const PLAN_TIMEOUT_MS: u64 = 120_000;
const PLAN_POLL_INTERVAL_MS: u64 = 10;
const PLAN_TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const ACCEPTED_EXIT_CODE: i32 = 0;
const PLAN_STDIN_BYTE_LIMIT: usize = 1;

#[derive(Debug, Deserialize)]
pub(super) struct CargoUnitGraph {
    pub version: u64,
    pub units: Vec<CargoPlannedUnit>,
    pub roots: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CargoPlannedUnit {
    pub pkg_id: String,
    pub target: CargoPlannedTarget,
    pub platform: Option<String>,
    pub mode: String,
    pub features: Vec<String>,
    pub profile: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(super) struct CargoPlannedTarget {
    pub name: String,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    pub src_path: String,
}

impl CargoUnitGraph {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let graph: Self = serde_json::from_slice(bytes).map_err(|error| format!("parse Cargo unit graph: {error}"))?;
        if graph.version != UNIT_GRAPH_VERSION || graph.roots.is_empty() || graph.units.is_empty() {
            return Err("Cargo unit graph has an unsupported version or empty selection".to_owned());
        }
        if graph.roots.iter().any(|root| *root >= graph.units.len()) {
            return Err("Cargo unit graph contains an invalid root index".to_owned());
        }
        Ok(graph)
    }
}

pub(super) fn load(
    workspace_root: &Path,
    toolchain: &str,
    scope_args: &[String],
    cargo_args: &[String],
    max_bytes: usize,
) -> Result<CargoUnitGraph, String> {
    let mut args = vec![
        "build".into(),
        "--unit-graph".into(),
        "-Zunstable-options".into(),
        "--locked".into(),
    ];
    args.extend(scope_args.iter().map(std::ffi::OsString::from));
    args.extend(cargo_args.iter().map(std::ffi::OsString::from));
    let mut environment = std::env::vars_os().collect::<std::collections::BTreeMap<_, _>>();
    environment.insert("RUSTUP_TOOLCHAIN".into(), toolchain.into());
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: cargo_program()?,
            args,
            current_dir: workspace_root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: environment.into_iter().collect(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: PLAN_TIMEOUT_MS,
            stdin_max_bytes: PLAN_STDIN_BYTE_LIMIT,
            stdout_max_bytes: max_bytes,
            stderr_max_bytes: max_bytes,
            poll_interval_ms: PLAN_POLL_INTERVAL_MS,
            teardown_timeout_ms: PLAN_TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct Cargo plan outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded Cargo plan: {error}"))?;
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return Err(format!("Cargo unit planning did not complete: {:?}", output.disposition));
    }
    CargoUnitGraph::parse(&output.stdout.bytes)
}

fn cargo_program() -> Result<std::path::PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or_else(|| "Cargo planner has no executable search path".to_owned())?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("cargo");
        if candidate.is_absolute() && candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("Cargo planner has no absolute Cargo executable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::CargoUnitGraph;

    const FIXTURE_TOOLCHAIN: &str = "nightly-2026-03-21";
    const FIXTURE_PLAN_BYTE_LIMIT: usize = 2 * 1024 * 1024;

    #[test]
    fn cargo_resolver_distinguishes_library_binary_and_test_compilations() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace");
        let graph = super::load(
            workspace,
            FIXTURE_TOOLCHAIN,
            &["-p".to_owned(), "octet-architecture-ir".to_owned()],
            &["--all-targets".to_owned()],
            FIXTURE_PLAN_BYTE_LIMIT,
        )
        .expect("bounded Cargo graph");
        let selected = graph
            .units
            .iter()
            .filter(|unit| unit.pkg_id.contains("/octet-architecture-ir#"))
            .collect::<Vec<_>>();
        for kind in ["lib", "bin"] {
            for mode in ["build", "test"] {
                assert!(
                    selected.iter().any(|unit| unit.target.kind.iter().any(|value| value == kind) && unit.mode == mode)
                );
            }
        }
    }

    #[test]
    fn empty_or_unknown_graph_is_rejected() {
        for bytes in [
            br#"{"version":1,"units":[],"roots":[]}"#.as_slice(),
            br#"{"version":0,"units":[],"roots":[]}"#.as_slice(),
            b"{}".as_slice(),
            b"not-json".as_slice(),
        ] {
            assert!(CargoUnitGraph::parse(bytes).is_err());
        }
    }

    #[test]
    fn complete_graph_preserves_test_mode_and_rejects_missing_root() {
        let graph = br#"{"version":1,"units":[{"pkg_id":"fixture","target":{"name":"fixture","kind":["lib"],"crate_types":["lib"],"src_path":"src/lib.rs"},"platform":null,"mode":"test","features":[],"profile":{}}],"roots":[0]}"#;
        let parsed = CargoUnitGraph::parse(graph).expect("complete graph");
        assert_eq!(parsed.units[0].mode, "test");
        let mut altered: serde_json::Value = serde_json::from_slice(graph).expect("JSON");
        altered["roots"] = serde_json::json!([parsed.units.len()]);
        assert!(CargoUnitGraph::parse(&serde_json::to_vec(&altered).expect("serialize")).is_err());
    }
}
