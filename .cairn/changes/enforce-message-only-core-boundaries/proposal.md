# Proposal: Enforce message-only core boundaries

## Why

Octet can classify core effects, resource handles, external types, ports, adapters, and dependency direction. It does not yet model independent state owners or require their communication contracts to use explicit messages.

A project can avoid direct network calls while a core still stores a vendor connection, receives a borrowed transport buffer, returns a channel sender, invokes a callback into another state owner, or reads transport-local state.

Source-token scans also miss aliases, generic wrappers, re-exports, and nonstandard runtime types. The project architecture gate needs a compiler-backed message-boundary mechanism.

## What Changes

- Extend typed project architecture policy with state owners, message boundaries, message types, transition functions, effect-plan types, and allowed runtime-handle scopes.
- Extend compiler Architecture IR facts for relevant fields, signatures, aliases, generic arguments, associated types, async returns, trait objects, and unresolved type paths.
- Add a pure evaluator that requires declared owner boundaries to cross through admitted message types.
- Reject live socket, connection, client, endpoint, channel, executor, task, borrowed-buffer, and configured vendor-handle families in core contracts and state.
- Require declared core transitions to consume explicit state and message values and return explicit decisions, state, outbound messages, or effect plans.
- Treat unresolved required type or flow facts as blocking in `gate` and `admission` modes.
- Emit a distinct message-boundary result inside the existing architecture receipt.
- Add positive and negative compiler fixtures, consumer templates, and documentation.

## Impact

- **Specs**: add `message-boundary-architecture` and extend project architecture policy behavior.
- **Files**: Nickel policy contract, generated policy projection, Architecture IR, pure evaluator, diagnostics, receipts, templates, docs, and fixtures.
- **Testing**: direct and aliased handle leakage, nested generics, associated types, borrowed buffers, async returns, channel escape, shared-state bypass, valid logical session values, valid adapters, missing facts, and tampered evidence.

## Dependencies

This change builds on the accepted project architecture policy, integrated architecture gate, compiler Architecture IR, capability classification, and effect-closure contracts.

Molten is the first planned consumer. Octet does not depend on an ambient Molten checkout.

## Non-goals

- Do not create a universal message type, actor framework, broker, scheduler, or runtime adapter.
- Do not ban resource handles from declared shells, adapters, composition roots, tests, or generated code unless project policy selects a narrower rule.
- Do not infer message semantics from a Rust type name alone.
- Do not prove canonical serialization, runtime scheduling, deterministic replay, delivery, correctness, authority, or release readiness.
- Do not make project architecture policy mandatory for consumers that have not selected it.

## Completion Evidence

The change is complete when compiler-backed positive fixtures pass, every handle and hidden-boundary negative fixture blocks, unknown required facts fail closed, and the architecture receipt preserves the exact policy, type, boundary, finding, and non-claim identities.
