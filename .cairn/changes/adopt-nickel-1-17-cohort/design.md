# Design: Adopt Nickel 1.17 for checked-path configuration

## Context

Octet uses Nickel for lint policy, standards profiles, proof configuration, and evidence contracts. The current package set provides Nickel `1.16.0`.

## Decisions

### Decision: Pin the exact upstream evaluator

**Choice:** Select Nickel `1.17.0` from commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.

Use a narrow input when the current Nix package set lacks the target.

**Rationale:** A broad package-set update would mix unrelated checker changes into this migration.

### Decision: Preserve checked-path ownership

**Choice:** Nickel validates configuration shape. Octet retains lint meaning, proof-rail policy, evidence roles, and findings.

**Rationale:** Evaluator alignment does not strengthen checker claims.

### Decision: Re-run positive and negative configuration rails

**Choice:** Run valid profiles and rejected disabled, duplicate, empty, warning-mode, malformed, and unknown-value fixtures.

Regenerate `flake.lock` only with Nix lock commands.

**Rationale:** Negative fixtures protect fail-closed policy behavior.

### Decision: Bind evidence only where relevant

**Choice:** Evidence records the evaluator identity when Nickel affects the produced artifact.

It must not promote syntax conformance into semantic proof.

## Risks and trade-offs

- Policy export or diagnostics can change without changing lint meaning.
- New validation can reveal stale fixtures.
- A passing profile does not prove checked Rust behavior.
