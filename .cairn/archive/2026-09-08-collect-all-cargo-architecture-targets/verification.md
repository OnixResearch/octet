# Verification

The change starts from published Octet `890f424cd7b1b2015f39c0eb5747f1bd1e576028`. All evidence below is local compiler or fixture evidence. No tenant requests or runtime workloads ran. The review is same-session and correlated.

Private logs live under `/home/brittonr/.local/state/octet-architecture-targets/`, outside this worktree. Cargo commands use the immutable base revision's Nix shell and compile the current worktree source. This avoids rebuilding the development shell during intermediate source edits. The final Nix checks build the changed source directly.

## Baseline

`baseline-ir.log` records 15 passing architecture-IR library tests. `baseline-runner-lib.log` records eight passing runner architecture tests. An earlier binary-target command ran zero tests and does not count as runner evidence.

The pinned Cargo check unit graph emits indistinguishable normal/test check entries. `unit-graph2.json` records that observation. `build-unit-graph.json` retains separate build/test modes. The planner does not infer configuration from array order. Cargo check rejects `--build-plan`.

The broader runner suite exposed two unchanged fixture failures: both used the unsupported `nickel-test-identity`. The test fixture now uses the shared reviewed evaluator identity. No runtime validation was weakened.

## Implemented boundaries

- `crates/octet-architecture-ir/src/selection.rs` owns exact, pure unit matching. It rejects duplicate keys and unexpected configurations inside selected packages.
- Collector context v2 requires workspace root, source root, and test configuration. Missing fields do not default to an ordinary library.
- The runner obtains a bounded, locked Cargo build graph with exact selectors. Profiles and source roots enter unit identity. Missing package units, source roles, ambiguous keys, and unsupported modes fail closed.
- The compiler adapter supplies actual source, test, feature, and target observations. External packages remain outside declared collection scope.
- Source envelopes include package Rust sources beyond `src`, including tests and build scripts. Selected roots in excluded build/evidence directories are rejected.
- Generated Nickel contracts now include the v2 collector context. The registry records that Rust-owned family.

## Positive and negative controls

`scoped-r4.log` records 465 passing runner tests (eight ignored), 145 passing lint-library tests, 23 passing IR tests, and 167 passing standards tests. It also records successful all-target strict Clippy for the first three packages with `-D warnings`, followed by workspace formatting verification.

The parallel r3 sweep had one failure in the unchanged coverage-process fixture. Its exact focused rerun and the complete serial r4 sweep passed. No assertion was weakened, and no root cause is claimed.

`compiler-fixtures-r3.log` records both ignored compiler fixture tests passing. They compile pure, feature-conditional, test-only macro, binary, and procedural-macro sources. The pure configuration has no effect facts. The effectful configurations retain effect facts. Removing a required compiled shard makes normalization incomplete.

`cargo-compiler-r5.log` records the end-to-end Cargo adapter control: `actual Cargo target coverage: 10 units`. The fixture covers library, binary, integration-test, example, benchmark, procedural-macro, and build-script units. The test compiles with the real collector, verifies complete coverage, then removes one observed shard and verifies rejection. Its policy is validated in-memory fixture data, not a claim about Nickel execution or policy authenticity.

The first concurrent Cargo control exposed shared temporary shard paths. Equivalent procedural-macro compilations can publish the same content hash concurrently. Per-process reserved staging now prevents that race. Direct tests cover concurrent identical publication and refusal to overwrite an occupied stage. The publication contract is atomic visibility, not crash durability.

## Nix checks

The first `compiler-architecture-targets` Nix check failed before execution because the source filter omitted an existing compiled fixture input, `dynamic-evidence-profiles.json`. The filter now includes that named file. The corrected combined r2 Nix run passed, including the real ten-unit Cargo control. The final r3 rerun also passed after explicit feature-selector validation changes. It built the current source at product commit `1d8a6f88` and repeated the ten-unit compiler control in the Nix sandbox.

The new dedicated Nix check runs the ignored Cargo compiler control with the current packaged collector and pinned driver. Existing UI checks run the expanded direct compiler fixtures.

## Acceptance limits and remaining steps

No whole-program purity, runtime determinism, or clean full-catalog consumer claim follows from these results. H7 still requires its own policy, source-role matrix, strict catalog, and entry-point checks after publication.

The policy v1 row requires explicit feature sets. Ambiguous shorthand, all-features expansion, and repeated target or feature selectors now fail before collection. Tests cover those rejections. Supporting build units remain host-bound, while non-host planned units must match the declared target.

Product checks are complete at `1d8a6f88`. The review remains correlated and makes no independent-assurance claim. Final proposal, design, and tasks gates report valid true and PASS. Root validation reports valid true. The sync dry-run was unblocked, and sync execution reported blocked false and mutated true. The accepted specification is `.cairn/specs/compiler-architecture-targets/spec.md`. Logs are `final-gate-*.json`, `final-validate.json`, `sync-dry.json`, and `sync.json` in the private evidence directory. Archive and publication follow those checks.

The owner approved inclusion of the existing spec-trace plan `37ba20b5` on 2026-09-08. A normal merge preserved its exact commit. The merge changed only that plan's five files. Staged benchmark-plan edits and the untracked durability plan remain outside this change.

The merged source passed both dedicated Nix checks. `approved-merge-nix.log` records the ten-unit compiler control and missing-shard rejection. `approved-merge-validate.json` reports valid true with no issues or findings. Product source is unchanged from the published compiler correction. These focused checks cover the documentation-only merge without a new full-suite claim.

The feature revision `38b3d281` remains a valid immutable input for H7. Main integration requires a fresh remote fetch, normal fast-forward publication, and ancestry verification. The worktree stays until those checks and the evidence guard succeed. Private integration and teardown logs record those final observations outside the disposable worktree.
