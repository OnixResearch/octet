# Proposal: Add proof workspace state

## Why

Octet proof summaries explain completed obligations and blockers. They do not expose a bounded live state for queued, running, stale, failed, or cancelled proof work.

Agents and terminal frontends can misread an old passing receipt as current while new source is waiting for verification. A source-bound generation model must make that state visible.

Dafny release `v4.11.0` at commit `fcb2042d6d043a2634f0854338c08feeaaaf4ae2` provides the workspace-state design reference. Octet will adapt the state model without adopting the Dafny language server protocol.

## What Changes

- Add a versioned `octet-proof-workspace-state/v1` DTO and reducer.
- Model proof groups, rails, targets, and obligations with explicit generation and sequence identity.
- Support `stale`, `queued`, `running`, `blocked`, `passed`, `failed`, `cancelled`, and `unsupported` states.
- Make source or proof identity changes invalidate older terminal results.
- Add an opt-in `cargo octet prove --workspace-state` artifact stream and current snapshot.
- Keep live states as operator observations. Only current matching terminal receipts can support existing evidence paths.
- Add positive and negative fixtures for valid transitions, stale generations, sequence gaps, duplicate terminal events, malformed trees, and role promotion.

## Impact

- **Specs**: new `proof-workspace-state` capability.
- **Files**: pure state reducer, DTOs, proof shell event emission, artifact schema, fixtures, and operator documentation.
- **Consumers**: Tile and other frontends can render supplied state without owning proof semantics.
- **Licensing**: the design uses clean-room adaptation from MIT-licensed Dafny. No Dafny source enters Octet.

## Dependencies

The state model consumes existing proof rail IDs, proof goal-state feedback, source bindings, receipt identities, and artifact validation outcomes.

It does not depend on the proof-robustness change. A robustness run can appear as one proof group after both changes are complete.

## Non-goals

- Do not add an LSP server, editor protocol, daemon, file watcher, or remote transport.
- Do not make frontends evidence authorities.
- Do not let stale, queued, running, blocked, cancelled, or unsupported state satisfy a proof obligation.
- Do not place wall-clock time in canonical state identity.
- Do not change ordinary proof output unless the operator selects workspace-state output.

## Completion Evidence

The change is complete when deterministic event replay produces the expected snapshot and every stale or malformed transition fails closed.
