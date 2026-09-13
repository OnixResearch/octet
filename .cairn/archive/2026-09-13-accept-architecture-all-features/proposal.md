## Why

The full deny-all consumer hook requires `--workspace -- --all-targets --all-features`.
An active architecture policy currently rejects that invocation before compiler collection.
Noble reproduced this failure at Octet revision `42dc32a6fe921d35182d7aef03c2a83999d55771`.
The same fixture passes with its explicit feature set.

## What Changes

- Accept `--all-features` only when Cargo resolves the invocation to the explicit policy row.
- Compare complete Cargo unit plans, including dependency edges and selected roots.
- Retain rejection for mismatched defaults, targets, packages, ambiguous selectors, and incomplete compiler shards.
- Add actual compiler and full deny-all consumer controls.

## Impact

Affected files include `cargo-octet/src/architecture_ir.rs`, its planner and fixtures, and `docs/project-architecture-policy.md`.
The accepted specification is `.cairn/specs/compiler-architecture-targets/spec.md`.
No public receipt schema, lint catalog, architecture mode, or requirement severity changes.
Cargo remains the owner of feature resolution. Noble must consume and check the published immutable fix separately.

Testing requires focused baseline and regression tests, strict Clippy, format checks, compiling all-target controls, the relevant Nix check, and lifecycle gates.
No result claims whole-program correctness, Noble M1 completion, or semantic proof.
