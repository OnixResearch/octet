## Decisions

### Decision: Cargo owns target selection

The shell obtains a locked unit graph from the pinned Cargo resolver. Cargo check unit-graph v1 erases the test distinction in duplicate check units. Therefore the planner uses the same explicit selectors with the build unit graph, which retains test mode. The graph requests no execution. Compilation still runs through the existing checked path. Unsupported modes or ambiguous unit keys reject admission rather than infer coverage.

### Decision: Bind compiler observations

Collector context v2 adds workspace-relative source root and test-configuration fields. Each unit identity binds these values, the resolved features, target triple, and Cargo profile. Rustc session selection checks those observations before collecting facts. Missing or unexpected compilation units remain errors. The shared core owns deterministic observation matching, while shells own Cargo, files, and rustc session translation.

### Decision: Keep evidence boundaries

The change does not claim runtime behavior or external dependency bodies. Build-script execution is not a Rust compilation unit. Its compiled source can be selected, but successful compilation does not authorize its runtime effects. Generated and test roles remain explicit. No finding budget or disabled catalog establishes acceptance.

## Verification

Baseline: architecture IR library tests and runner architecture tests. Positive controls cover distinct library, binary, test, example, benchmark, proc-macro, and build-script units. Negative controls cover duplicate observation keys, missing source roots, malformed versions, absent units, wrong features, and unknown selected compiler sessions. Compiling effect fixtures must produce actual compiler facts before any rejection claim. Publication requires focused tests, strict Clippy, generated-contract freshness, and a real all-target compiler run.
