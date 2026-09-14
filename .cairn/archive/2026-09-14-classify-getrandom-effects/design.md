# Compiler randomness effect design

## Context

The collector already records the exact `getrandom::fill` call in both Noble kernel production units.
Its `effect_family` function lacks a randomness entry, so the corresponding effect and production findings are absent.
The consumer retains this valid but insufficient bundle. It does not replace the required effect with a call-only expectation.

## Decision

Use the compiler's defining crate, root-function shape, and exact item name for four reviewed getrandom entry points.
Rustc's diagnostic paths can retain consumer aliases, such as `randomness::entropy::fill`.
The classifier must not infer the defining crate from that display string. Recorded call and effect paths remain unchanged.
Do not use a blanket `getrandom::` prefix, because error helpers and similarly named functions are not randomness requests.
Keep existing standard-library prefix behavior unchanged.

The pure classifier returns the existing `CapabilityFamily::Randomness` variant.
The existing collector shell creates the effect fact and retains its caller, span, identity, and memberships.
The existing pure architecture gate evaluates production obligations. No consumer-side classifier or fabricated effect enters this repair.

## Compiler controls

A separate fixture module uses the published registry crate through an import alias.
It calls `fill`, `fill_uninit`, `u32`, and `u64` without executing these operations.
A local same-named module supplies a negative name-resolution control, not a randomness witness.
Both `lib` and `lib+test` must retain all four calls and effects with matching callers and spans.
Both production observers must retain the required blocked randomness findings.
The existing target roster, role checks, reference closure, coverage checks, and omitted-shard rejection remain required.

Fixture dependencies stay out of production manifests. A Cargo-generated fixture lock uses versions already present in the provider vendor closure.
Rust owns typed compiler fixture policies. These controls do not establish Nickel evaluation or full deny-all acceptance.
Actual Noble probes separately exercise the complete published hook and full bundle replay.

## Verification and rollout

Record a pre-change collector baseline and a red real-compiler regression.
Run focused tests, explicit compiler controls, strict Clippy, formatting, provider builds, policy exports, and relevant Nix checks.
Retain source snapshots and actual Noble observations outside the disposable provider worktree.
Publish only after inspected sync and archive plans, execution, and final validation.
Verify the published collector bytes before Noble changes either pin.
