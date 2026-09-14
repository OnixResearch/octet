# Parsed no-std evidence and final provider checks

## Scope

This record completes the bounded implementation and verification tasks for `bind-integration-test-ownership`.
The earlier membership, role-binding, and production-view records remain historical evidence.
This record supersedes their open compiler no-std blocker, but does not establish Noble M1 acceptance.

Evidence root: `/home/brittonr/git/noble/.pi/m1-test-ownership/no-std/`.
All task numbers in this record refer to `commands.log` in that directory.
Task numbers alone are not durable identities.

## Cause and correction

The selected rustc revision is `ac7f9ec7da74d37fd28667c86bf117a39ba5b02a` (nightly-2026-03-21).
Its HIR represents the built-in crate attribute as `AttributeKind::NoStd`.
The unparsed attribute-name API does not expose that parsed variant.
The old collector therefore emitted no no-std fact for an active crate attribute.

The collector now uses `find_attr!(cx.tcx, crate, AttributeKind::NoStd(_))`.
The query reads the active crate attributes from the current compiler session.
It does not read source text, infer configuration from filenames, or invent a package-wide witness.
Normalization retains the observing unit memberships before deduplication.
The production evaluator and its per-unit no-std obligation did not change.

The retained upstream sources are:

- `rustc-hir.rs`: `compiler/rustc_hir/src/hir.rs` at the selected revision.
- `rustc-attrs-mod.rs`: `compiler/rustc_hir/src/attrs/mod.rs` at that revision.
- `rustc-attrs-data-structures.rs`: `compiler/rustc_hir/src/attrs/data_structures.rs` at that revision.

Tasks 2056 and 2062 contain the exact upstream URLs. These files document the API, not independent acceptance.

## Baseline and rejection controls

Task 2059 passed the four existing explicit target controls with the previous candidate library.
Task 2061 rejected an invalid fixture policy with overlapping source scopes.
The corrected fixture declares disjoint library and binary source scopes. It retains every Cargo target.
Task 2063 then reached the intended compiler rejection:

```text
active no_std membership: mode=Always kind=lib
  left: false
 right: true
```

Task 2066 rejected an unused production `sym` import after the query change.
Task 2068 passed the three package cohorts but rejected missing `sym` in collector unit tests.
The import now belongs only to that test function. No lint suppression was added.
All failures remain in separate logs.

## Compiler controls

Task 2074 passed nine explicit controls with the rebuilt compiler library.
Eight controls run all ten selected compiler units. The source-symlink control rejects before compiler invocation.
Each compiler run also rejects an omitted shard.

| Configuration | Observed no-std memberships | Production obligation |
|---|---|---|
| Unconditional crate attribute | Library and library-test only | Satisfied |
| Active feature `cfg_attr` | Library and library-test only | Satisfied |
| Inactive feature `cfg_attr` | None | Rejected |
| `cfg_attr(not(test), no_std)` | Library only | Rejected |
| Integration-test-only attribute | Independent integration test only | Rejected |

The original target, source-import, and symlink controls also pass.
The controls assert both positive and absent memberships across every selected unit.
They retain forbidden library-test filesystem effects through an explicit `extern crate std`.
An active no-std attribute is not proof that dependencies exclude `std` or that code is functionally pure.
The compiler fixtures use warning-only lint status. They do not replace the consumer deny-all hook.

## Final verification

Task 2081 passed formatting and the following package cohorts:

| Cohort | Passed |
|---|---:|
| Architecture IR | 33 |
| Standards unit and integration | 185 + 24 |
| Runner library | 474 |
| Collector library | 150 |

Task 2081 also passed strict Clippy for all four affected packages, all targets, and all features with `-D warnings`.
Together with the nine explicit controls, this cohort has 875 passed tests.
The runner default suite ignores 16 tests. Nine run explicitly here, and seven unrelated ignored tests remain outside this cohort.

Task 2078 built the final runner and passed these Nix checks:

- `fmt`, `project-architecture-policy`, and `standards-fixtures`.
- `ui`: all four harness tests passed, including snapshots, compiler fixtures, and autofix controls.
- `compiler-architecture-targets`: all 28 selected tests passed with ignored compiler controls included.

The Nix target cohort overlaps the package and explicit controls, so its count is not added again.
Task 2078 also exported all five policy artifacts and their manifest.
Generated Rust-owned Nickel contracts passed their freshness checks.
The tested build tree is `0ce3653729aaaffd2e90c518f2fe3bfcc71167d9`.
Only lifecycle evidence and task-state documents change after that build.

Task 2084 confirmed byte-identical initial and final no-std compiler libraries.
Their BLAKE3 is `dc1edbe2188d04914de96401216f039dc73d16ab8be849079a74b3317472665a`.
It also replayed the Noble bundle with the final runner.

## Unchanged Noble candidate

Task 2075 ran the candidate deny-all hook with `--workspace --all-targets --all-features`.
Both lint and architecture phases are clean, with zero findings, warnings, or errors.
All six compiler units remain covered. The two independent test units retain explicit dispositions.

The compiler emits one deduplicated `crate:no-std=noble-kernel` fact.
Both kernel production units observe it: `lib` and `lib+test`.
Neither integration-test nor CLI observations supply that fact.
The fact ID is `7ed63a978a58580e47566c87c55b42ba160801e500cb23543b488fb1be7a23e2`.

The clean receipt is `83a02ee7cd4b1157130c0d2ae1e3dd3003a7267b1c5124846019079cd85e5c1c`.
Tasks 2077 and 2084 report a valid full artifact bundle.
Artifact replay establishes consistency with stored observations, not observation authenticity or independent acceptance.

Noble source tree `3f5f405adb7f3d4184fb6c55ccfb1cb433d97fc9` remains unchanged.
At this checkpoint, Noble still consumes published revision `74a5d6b057272b149ae86049107cba5eac986b28`.
Provider publication must precede downstream adoption.
Noble M1 remains 2/21, with extraction inventory, negative controls, Miri, and independent acceptance still open.

## Retention and replay

`commands.log` records the complete command environment, working directory, task result, and unredirected output.
The named test and build logs retain redirected output.
The Cargo commands use the pinned repair worktree only for the devshell and vendor configuration, not for candidate source.
The candidate worktree remains the working directory for every Cargo command.

`source.tar`, `source-tree.txt`, and `staged.patch` retain the final reviewed pre-archive source.
`compiler-observations.tar` retains the Noble bundle and raw collector context and shards.
`artifact-blake3.txt` binds retained artifacts and compiler libraries.
`final-artifact-verification.json` records replay with the final runner.
`policy-artifacts/octet-artifact-manifest.json` records generated policy exports.
Lifecycle plans, execution records, and publication evidence remain separate files under the same evidence root.
