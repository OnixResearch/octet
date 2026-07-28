## Why

Octet has a performance review guide and methodology rules for public claims. Its earlier review-kit change explicitly excluded benchmark automation and performance gates.

Octet still lacks reproducible runtime, cache, memory, diagnostic-signal, suppression, and false-positive measurements across declared workloads.

## What Changes

- Add an opt-in `benchmark-evidence` rail with typed Nickel benchmark profiles.
- Add an Octet-owned Rust harness for fixed commands, bounded samples, environment capture, workload identity, and deterministic receipts.
- Add an adjudicated signal corpus for true-positive, false-positive, false-negative, suppression, and inconclusive outcomes.
- Compare only compatible baseline and candidate runs with declared practical thresholds and statistical policy.
- Link every publishable benchmark claim to verified raw artifacts and an interpretation boundary.

## Impact

- **Specs**: new `benchmark-evidence` capability
- **Files**: benchmark profile Nickel, Rust harness core and shell, workload registry, signal oracle, receipt DTOs, fixtures, artifact validation, and benchmark documentation
- **Testing**: stable and regressed runs, warm and cold modes, signal classification, incompatible environments, malformed samples, stale artifacts, limits, and default-path independence

## Non-goals

- Do not add benchmarking to starter `cargo octet check`.
- Do not make wall-clock noise or one sample a release gate.
- Do not claim that a selected corpus represents all Rust projects or all defects.
- Do not require Criterion, hyperfine, iai-callgrind, CodSpeed, or a hosted benchmark service.
- Do not grant arbitrary executable, argument, workspace, environment, or network authority.

## Completion Evidence

The change is complete when performance and signal suites produce replayable receipts, incompatible comparisons fail closed, and published metrics remain traceable to raw evidence.
