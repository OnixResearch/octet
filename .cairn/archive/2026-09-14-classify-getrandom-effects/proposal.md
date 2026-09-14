# Classify compiler-resolved getrandom effects

## Why

Published Octet records `getrandom::fill` calls but emits no randomness effect for them.
Noble's required randomness fixture reports `ambient_random`, yet its architecture comparison correctly rejects the missing effect.
The existing collector effect table covers selected standard-library paths only.

## What Changes

- Classify the exact resolved `getrandom::fill`, `fill_uninit`, `u32`, and `u64` definitions as randomness effects.
- Preserve calls, constructors, callers, spans, and unit memberships.
- Add real-crate compiler controls for both production library configurations and both selected feature invocations.
- Reject name-prefix near matches and local same-named helpers as randomness primitives.

## Scope

The compiler Architecture IR remains v2. No schema, role, gate mode, lint level, or exception changes.
The four entry points use the published `getrandom` 0.4.3 crate and checksum-bound dependency lock.
This does not claim coverage of every randomness API, dependency implementation, or runtime execution.
The existing semantic enum and provider gate own randomness classification and enforcement.
Noble must consume a published revision after provider verification and lifecycle completion.
