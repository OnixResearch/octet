# Role-binding phase evidence

## Result and boundary

The compiler now retains each exact Cargo source root in `CompilationUnit.source_path`.
The unit identity includes that root. Normalization and artifact replay reject invalid roots and stale inner identities.
The collector checks that the planned root equals the root observed by rustc.

The pure `select_compilation_unit_role` function interprets explicit source roles.
Only a Cargo `test+test` target can select `test` instead of its package role.
Library and binary test configurations retain their production roles.
Names and test-looking paths do not grant a test role.

The gate does not yet interpret test-only facts safely.
It retains the existing source-role rejection and an explicit `test-role-evaluation-pending` rejection.
Task I3 must replace that temporary rejection with the typed production view and receipt dispositions.
No production requirement, lint, or Cargo target became optional.

## Observed checks

| Check | Result |
|---|---|
| Pre-change focused baseline | Passed in task 2039 |
| Missing-root control before implementation | Failed as expected in task 2042 |
| Architecture IR suite | 32 tests passed |
| Standards package | 176 unit tests and 24 integration tests passed |
| Cargo runner library | 472 tests passed, 9 ignored |
| Strict Clippy | Three affected packages passed with all targets, all features, and `-D warnings` |
| UI harness compilation | Passed with `--no-run`, not UI execution |
| Nix runner, formatting, architecture policy | Passed in task 2068 |
| Final package tests and strict Clippy | Passed in task 2083 after the fixture correction |
| Final Nix checks and artifact export | Formatting, architecture policy, and standards fixtures passed in task 2086 |
| Real compiler controls | Two explicit ignored tests passed in task 2078 |
| Noble candidate run | Complete six-unit coverage, clean lints, rejecting architecture gate |
| Noble artifact replay | Valid failure bundle in task 2080 |

The combined result is 704 package tests and two real compiler controls. It is not a full-provider release check.
Task 2086 also exported the five policy artifacts and their BLAKE3 manifest.

The real controls select all Cargo targets with explicit features and with `--all-features`.
Both runs collect ten units, including library, binary, integration test, example, benchmark, procedural macro, and build-script targets.
The binary is deliberately named `test` and keeps its production role.
The integration-test target selects `test` from its explicit source role.
A `cfg(test)` library effect remains visible in a production-role unit.
Each run also rejects one missing compiler shard.

The first compiler-control run failed before compilation.
Its fixture incorrectly included the `tests` scope in the production source list.
The corrected fixture retains its explicit test role and removes only that incorrect production classification.
Both compiler runs still select all targets. Task 2074 and its failure log remain available.
The production-root negative control uses a structurally valid policy before Cargo binding rejects it.

## Contract changes

The draft normalized IR remains v2. This change has not published that draft.
The raw shard schema and identity domain advance to v2.
The collector context advances to v3. The unit identity domain advances to v2.
The Rust generator refreshed the checked-in Nickel contract.
The existing config/receipt families register the new positive and negative controls.

The controls cover missing roots, noncanonical roots, conflicting roles, wrong target kinds, stale inner identities, and legacy shards.
They also retain an explicit limit: a fully resealed, structurally possible root is not authenticated Cargo evidence.
The source and compiler process observations establish the local run, not signer authenticity or independent acceptance.

## Retained evidence

Evidence is in `/home/brittonr/git/noble/.pi/m1-test-ownership/role-binding/`.
The directory retains logs, source snapshots, compiler observations, artifact replay, and the local result record.
The previous phase remains unchanged in the parent directory.
The corrected fixture changes test code only. It does not change the production collector used for tasks 2075 and 2078.

The exact compiler replay command is recorded in `commands.log` and `result.json`.
It uses the candidate library and pinned driver through the published development environment.
Cargo compiles this worktree, not the published provider sources.
No dependency pin, Noble source, extraction result, or refinement claim changed.

## Next phase

I3 must replace package-only fact ownership with the complete validated membership relation.
The production view must retain shared production observations and reject production imports of test-labelled source.
Test-only witnesses, ports, no-std facts, and unknowns must not satisfy production obligations.
Receipts must retain explicit test-only dispositions and every original input identity.
Artifact replay and real compiler controls must cover those rules before publication or Noble repinning.
