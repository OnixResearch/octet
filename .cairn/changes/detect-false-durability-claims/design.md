# Design: Detect false durability claims

## Context

Octet owns static source checks and bounded evidence profiles. Consumers own their durability meaning and backend behavior.

The design needs explicit consumer declarations. Function-name heuristics alone create unstable findings and false authority.

## Decisions

### Decision: Consumers declare durability boundaries

Nickel policy identifies maintained source scopes and exact boundary functions or traits. Each declaration names its required observation class and uncertain-outcome type.

The generated profile feeds Rust checks. Runtime Nickel evaluation is not added.

### Decision: Discarded results are direct findings

The source check reports direct discard, `let _ =`, empty error arms, unconditional error-to-success mapping, and configured commit-result suppression for synchronization calls.

### Decision: `flush()` is never persistence proof

On a declared durability boundary, `Write::flush()` cannot satisfy the required synchronization observation. The check accepts an explicit configured adapter observation or supported synchronization call whose result is handled.

The finding proves source shape only.

### Decision: Retry detection is bounded

A check reports a failed synchronization followed by another synchronization on the same visible receiver when no intervening write, reopen, replacement handle, or explicit reconciliation call appears in the analyzed block.

Unknown aliasing or control flow produces an incomplete-analysis fact, not a correctness claim.

### Decision: External evidence stays separate

The assurance profile links saved producer receipts by BLAKE3 and checks selected case coverage and non-claims. Octet does not execute ChaosControl during normal lint runs.

## Test Design

Positive fixtures cover handled synchronization, explicit outcome unknown, quarantine, reopen, rewrite-before-retry, and reviewed adapter observations.

Negative fixtures cover direct discard, `let _`, empty error handling, `flush()`-only durability, same-handle retry, missing uncertainty type, stale receipt, missing negative case, and overclaim.

## Claim Boundary

Passing source checks proves only absence of selected source shapes in declared scopes. Passing assurance checks proves only linkage to selected saved evidence.
