## Why

Octet checks Rust source shape, policy, standards, and evidence rails, and it gates changes with deterministic receipts. It has no first-class rail that rechecks an implementation against a pinned behavioral model by replaying model-generated traces. A protocol crate today can pass every static check while drifting from the model that design-time checks assumed, with no checked, receipted admission surface.

The stack model layer will emit bounded traces from a pinned external engine. Octet is the gate owner and must turn trace replay into a checked rail with fixed seeds and identity-bound freshness.

## What Changes

- Add the spec-trace-parity check to Octet: replay pinned, seed-bound traces against a crate through an injected adapter.
- Bind each run to model hash, checker identity and version, seed, trace-set hash, and adapter identity.
- Refuse to pass when the model hash, checker version, seed, or trace bytes are stale or mismatched.
- Emit deterministic verdict receipts that include parity, divergence, and unsupported-verifier classes.
- Add positive and negative fixtures for matching implementations, divergence, stale traces, seed drift, and missing adapters.

## Impact

- **Files**: Octet spec-trace-parity check, trace-replay adapter contract, receipts, fixtures, and docs.
- **Testing**: Positive parity cases plus negative stale-spec, stale-checker, seed-drift, tamper, divergence, and adapter-missing cases.
- **Boundary**: A passing parity rail proves that the replayed traces match the implementation under the pinned model and checker identity. It does not prove model validity, engine correctness, or behavior outside the traced set.
