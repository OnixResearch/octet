## Tasks

- [ ] [serial] Define the spec-trace-parity check surface: pinned model, checker identity and version, seed, trace-set identity, and replay-adapter contract. r[octet.spec_trace_parity.rail]
- [ ] [serial] Implement pure freshness admission that denies stale or mismatched model, checker, seed, or trace identities before any replay step. r[octet.spec_trace_parity.freshness]
- [ ] [serial] Implement deterministic, seed-bound trace replay against the injected adapter with explicit step and time bounds. r[octet.spec_trace_parity.replay]
- [ ] [parallel] Define the injected replay-adapter contract with declared state projection and loss bound, and reject adapters that omit either. r[octet.spec_trace_parity.adapter]
- [ ] [parallel] Implement verdict classification and deterministic receipt construction and validation for parity, divergence, and unsupported-verifier classes. r[octet.spec_trace_parity.receipt]
- [ ] [parallel] Add positive fixtures for fresh parity, and negative fixtures for stale model, stale checker, seed drift, tampered traces, divergence, adapter loss, and unsupported verifier. r[octet.spec_trace_parity.fixtures]
- [ ] [serial] Run focused core/shell tests, clippy with warnings denied, the full Octet check, lifecycle validation, Cairn gates, and relevant flake checks. r[octet.spec_trace_parity.validation]
