# Non-function body-owner evidence

This is a provider repair, not Noble M1 acceptance.
Persistent operator evidence: `/home/brittonr/git/noble/.pi/m1-inline-owner/`.

## Mechanism and retained failures

The published collector at `f8b12bea3ba5d72de74df9f569afcdf396871325` treated an unseen non-closure body owner as a function.
The two new real-compiler controls reproduced `fn_sig(): ConstBlock` with that published library (task 2468).
The collector baseline passed 150 tests (2450).

The first repair used the actual `DefKind` and disabled signatures for constants.
Its 151 collector tests and scoped Clippy passed, but actual compiler controls exposed an unsupported `tcx.visibility` query (2481).
The first source snapshot and failed candidate remain retained. The 28 existing compiler-target tests still passed.

The second repair gives expression owners private visibility without that query.
Named items still use compiler visibility. No body, call, source, target, or production obligation is excluded.
The 152 collector tests passed. Actual compilation then succeeded, but the test oracle had incorrect thread-local expectations (2504).

A bounded diagnostic run retained the compiler item and call records (2518).
For this fixture's `std::cell::Cell` import, the compiler reports `std::cell::Cell::<T>::new` and `::set`.
The thread-local macro consumes the initializer block into `LOCAL::__RUST_STD_INTERNAL_INIT`, a named constant with restricted visibility.
The fixture now checks those exact fields. It does not accept interchangeable prefixes or omit the initializer.
Separate inline, anonymous, and expanded constant cases require private expression visibility.
Pinned Rust sources are retained as `rustc-def.rs`, `rustc-thread-local.rs`, and `rustc-native-thread-local.rs`.
The native macro source independently confirms the named initializer and a separate inline constant around its `needs_drop` call.
Task 2531 adds an exact assertion for that inline caller, including private visibility and both production memberships. All eleven controls passed again.

## Compiler controls

Task 2519 ran all eleven explicit compiler controls with the rebuilt second library: eleven passed.
Ten controls invoke the actual compiler with ten units each. One rejects a source symlink before collection.
The two new controls cover default and all-feature selections.
Each checks both production library configurations for:

1. Named, associated, inline, anonymous, and expanded constant callers; static initialization.
2. Function signatures, separate closure identities, and retained same-unit caller references.
3. Thread-local construction and mutation calls, without executing their effects.
4. A forbidden filesystem effect in a function nested inside a constant.
5. Complete target coverage and rejection after removing a required shard.

New body-owner controls use architecture gate mode and require blocked production effects.
Their lint phase is explicitly warning-only. These compiler-fixture results are not deny-all evidence.
Fixture policy DTOs are constructed by the existing Rust test helper; they are not Nickel evaluation evidence.

## Noble candidate probe

The existing Noble staged tree remains `4dea0a1db3c1ae0207cfac8f67eaab9783c27190`.
The required thread-local source was copied unchanged from the retained failed fixture.
Selection preflight preceded each controlled candidate probe.

Task 2520 used the unchanged full deny-all hook with the candidate provider.
It returned exit 2 / Cargo 101 with two intended `interior_mutability_in_core` errors at kernel source.
This was not an internal compiler error.
Task 2523 ran the separate default-lint observation and full artifact replay.
The unchanged Noble comparator returned `boundary control interior-state: passed (not M1 acceptance)`.
Both kernel production units retain the required `core::cell::Cell::<T>::set` call.
Raw compiler observations were archived before replay.

These are candidate observations, not published-provider adoption or a successful current Noble flake.
Noble remains 5/21 complete. Task 2.2 and full M1 acceptance remain open.

## Secondary reporter issue

The historical JSON failure was inspected separately.
`cargo-octet/src/engine.rs::build_finding` copies the compiler diagnostic level into the finding severity.
`cargo-octet/src/output.rs` rejects values other than the defined warning/error domain.
An ICE level therefore prevents complete artifact production.
This repair does not relax that domain, accept missing artifacts, or claim generic ICE-reporting support.
The corrected fixture no longer emits that malformed diagnostic; its complete observation bundle replays successfully.

## Scoped verification and handoff

Task 2527 passed 33 IR, 185 standards unit, 24 standards integration, 474 runner, and 152 collector tests.
With the eleven explicit controls, the scoped cohort is 879 tests. Repeated runs are not additional tests.
The runner default ignores eighteen tests; eleven ran explicitly and seven unrelated tests remain outside this scope.
Strict four-package Clippy and formatting passed. Task 2531 repeated Clippy after the stronger expanded-owner assertion.

Task 2528 built the provider and passed Nix `fmt`, `project-architecture-policy`, `standards-fixtures`, `ui`, and `compiler-architecture-targets`.
The UI check ran four harness tests. The compiler check ran thirty overlapping tests, not thirty additional controls.
Five policy artifact exports and their manifest passed without a schema change.
Task 2535 rebuilt the provider, formatting check, and thirty compiler tests with the stronger inline-owner assertion.

Task 2529 checked the unchanged Noble workspace with the candidate full deny-all hook: clean, zero findings, valid full replay.
Its staged tree remained unchanged before and after the probe. Existing ignored compiler observations were archived first.
Tasks 2534 and 2537 verified byte equality across tested and final collector libraries and replayed both Noble bundles with the rebuilt runners.
Collector BLAKE3: `95762941b0902d497fcce5173c626ff5f26c7094a5cdc538552fa6810f7c2b0b`.

Full Pueue JSON records and source snapshots are retained outside the worktree.
Lifecycle synchronization, archival, and publication remain separate finalization operations.
Noble adoption must follow publication and update both immutable pins and reviewed selection identities.
No provider result completes Noble task 2.2 or establishes M1 acceptance.
