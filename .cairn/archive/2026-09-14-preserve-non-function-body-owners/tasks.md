# Implementation tasks

Noble remains a separate consumer. No task establishes Noble M1 acceptance.

- [x] [serial] V1 Run the collector baseline and reproduce the new compiler regression with the published library. r[octet.compiler_architecture_ir.body_owner_kinds]
- [x] [serial] I1 Preserve compiler body kinds and signature/visibility query eligibility in lazy item collection, with regression fixtures. r[octet.compiler_architecture_ir.body_owner_kinds]
- [x] [serial] V2 Run actual compiler controls for all selected body kinds, caller closure, production memberships, forbidden effects, and omitted shards. r[octet.compiler_architecture_ir.body_owner_kinds]
- [x] [serial] V3 Run affected package tests, strict Clippy, formatting, relevant Nix checks, a fresh provider build, and the retained Noble fixture probe. r[octet.compiler_architecture_ir.body_owner_kinds]
- [x] [serial] H1 Retain checked evidence: command=`nix develop -c cargo test -p cargo-octet --lib real_cargo_all_targets --locked --offline -- --ignored --nocapture --test-threads=1`; input=source fixtures and exact pinned driver/library environment in the operator commands; asserts=body kinds and observations retained with no invalid signature or visibility query; evidence=/home/brittonr/git/noble/.pi/m1-inline-owner/. r[octet.compiler_architecture_ir.body_owner_kinds]

## Verification Coverage

| Scenario | Verification |
|---|---|
| Constant and static bodies contain calls | V1 red and V2 compiler item/call assertions |
| Functions and closures coexist with constants | V2 signature, reference, binding, and membership assertions |
| Expanded state contains a forbidden production effect | V2 thread-local and nested effect controls, V3 Noble probe |
