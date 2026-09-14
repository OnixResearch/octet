# Compiler randomness effect tasks

- [x] [serial] V1 Run the collector baseline and reproduce the missing effect with real registry calls. r[octet.compiler_architecture_ir.randomness_effects]
- [x] [serial] I1 Add exact randomness classification and preserve existing call, identity, role, and membership behavior. r[octet.compiler_architecture_ir.randomness_effects]
- [x] [serial] V2 Run positive and negative classifier tests, explicit compiler controls, strict Clippy, formatting, and relevant Nix checks. r[octet.compiler_architecture_ir.randomness_effects]
- [x] [serial] V3 Retain the unchanged Noble candidate probes and compare tested and final-built collector bytes. r[octet.compiler_architecture_ir.randomness_effects]
- [x] [serial] H1 Record scope and outcomes before inspected sync/archive execution and publication. command=selected-Cairn-validation-and-gates input=dedicated-provider-worktree asserts=complete-required-controls-and-explicit-non-claims evidence=evidence.md r[octet.compiler_architecture_ir.randomness_effects]

## Verification Coverage

| Scenario | Required evidence |
|---|---|
| Real registry randomness calls | Four exact calls and matching effects in both library configurations, both feature invocation forms, production findings, complete coverage and omitted-shard rejection |
| Similar names are not randomness primitives | Exact-match unit negatives and a compiled local-name control without promoted effect facts |

Compiler fixture policies and warning-only runs are not published deny-all acceptance.
Noble remains active at 5/21 tasks until its separate task criteria pass.
After provider publication, the operator must compare published bytes before Noble adoption. This rollout check is not a prerequisite for its own publication.
