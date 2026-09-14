# Implementation tasks

No task completion implies Noble M1 acceptance. Evidence belongs outside this worktree.
The [membership record](evidence.md) covers phase one. The [role-binding record](role-binding-evidence.md) covers phase two.
The [production-view record](production-view-evidence.md) covers phase three and its source-origin repair.
The [parsed no-std record](no-std-evidence.md) completes the bounded provider checks and records the clean unchanged Noble candidate.
Provider publication and downstream adoption remain separate from Noble M1 acceptance.

- [x] [serial] V1 Run the Architecture IR and project gate baselines. r[octet.architecture.unit_membership] r[octet.project_architecture_gate.test_ownership]
- [x] [serial] I1 Add versioned normalized unit membership, identity binding, validation, and its regression fixtures. r[octet.architecture.unit_membership]
- [x] [serial] V2 Run deterministic membership, malformed-input, and missing-shard controls; regenerate the Rust-owned contract. r[octet.architecture.unit_membership]
- [x] [serial] I2 Bind independent integration-test roles to selected Cargo roots without changing production roles. r[octet.project_architecture_gate.test_ownership]
- [x] [serial] I3 Add a typed production evaluation view and explicit test-only receipt dispositions. r[octet.project_architecture_gate.test_ownership]
- [x] [serial] V3 Run real compiler target, production-import, fake production evidence, and artifact replay controls. r[octet.project_architecture_gate.test_ownership]
- [x] [serial] I4 Read the compiler's parsed crate-level no-std attribute without a source-text fallback. r[octet.project_architecture_gate.test_ownership]
- [x] [serial] V5 Run active/inactive cfg-attribute, library-test, and integration-only no-std compiler controls. r[octet.project_architecture_gate.test_ownership]
- [x] [serial] V4 Run formatting, strict package Clippy, affected package tests, generated-artifact freshness, relevant Nix checks, and the provider build. r[octet.architecture.unit_membership] r[octet.project_architecture_gate.test_ownership]
- [x] [serial] H1 Retain exact evidence: command=`nix develop -c cargo test -p octet-architecture-ir --all-targets --locked --offline`; input=reviewed membership fixtures and the pinned devshell/vendor environment in no-std/commands.log; asserts=complete canonical membership with malformed relations rejected; evidence=/home/brittonr/git/noble/.pi/m1-test-ownership/no-std/.  r[octet.architecture.unit_membership]
- [x] [serial] H2 Retain exact compiler and consumer evidence: command=`nix develop -c cargo test -p cargo-octet --lib real_cargo_all_targets --locked --offline -- --ignored --nocapture --test-threads=1`; input=checked compiler fixtures and the pinned driver/library environment in no-std/commands.log; asserts=complete target coverage with test ownership separated and bypass controls rejected; evidence=/home/brittonr/git/noble/.pi/m1-test-ownership/no-std/.  r[octet.project_architecture_gate.test_ownership]

## Verification Coverage

| Scenario | Verification |
|---|---|
| Shared facts retain both observing units | V2: membership normalization and permutation fixtures |
| Malformed membership is resealed | V2: resealed relation mutation fixtures |
| Legacy or incomplete evidence is supplied | V2: v1 decoding and missing-shard controls |
| Independent tests coexist with production units | V3: real Cargo production/test fixture and consumer gate |
| A production unit imports test-labelled source | V3: normal and library-test source import controls |
| Test evidence attempts to satisfy production obligations | V3: authority, port, no-std, unknown, shared-fact, and replay controls |
| A resealed receipt changes its test-only disposition | V3: full bundle replay with refreshed outer hashes |
| A production import uses a parent-relative path or source alias | V3: compiler import rejection and precollection symlink rejection |
| Crate configuration differs between production and test observations | V5: active/inactive cfg attributes and per-unit no-std membership |
