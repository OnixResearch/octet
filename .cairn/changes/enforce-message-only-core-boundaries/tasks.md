# Tasks: Enforce message-only core boundaries

## Policy and compiler facts

- [ ] [serial] Extend the typed Nickel architecture policy with state owners, message boundaries, state types, message types, transitions, effect plans, permitted runtime-handle scopes, provider identities, limits, and non-claims. r[octet.message_boundary.policy]
- [ ] [serial] Extend Architecture IR with the field, signature, alias, generic, associated-type, trait-object, lifetime, async-return, callback, and provider type facts required by the mechanism. r[octet.message_boundary.compiler_facts]
- [ ] [parallel] Add deterministic policy fixtures for valid owners and boundaries plus missing, unknown, overlapping, unbounded, stale, and unsupported declarations. r[octet.message_boundary.policy] r[octet.message_boundary.verification]

## Pure evaluation core

- [ ] [serial] Implement message-boundary path evaluation over declared ingress, transition, state, outbound message, and effect-plan types. r[octet.message_boundary.path]
- [ ] [serial] Implement exact built-in and configured runtime-handle family classification through aliases, generic arguments, associated types, and provider identities. r[octet.message_boundary.handle_containment]
- [ ] [serial] Compose existing global-state, interior-mutability, synchronization, capability, effect-closure, role, port, and dependency findings without merging their claims. r[octet.message_boundary.shared_state]
- [ ] [serial] Block required unknown type, dispatch, feature, target, or shard facts in `gate` and `admission` modes. r[octet.message_boundary.unknowns]

## Evidence and consumer surfaces

- [ ] [serial] Extend the architecture receipt with policy, owner, boundary, type coverage, finding, unknown, contributing mechanism, waiver, status, and non-claim identities. r[octet.message_boundary.evidence]
- [ ] [serial] Extend artifact verification to reject tampered or stale message-boundary receipt links without rerunning Cargo. r[octet.message_boundary.evidence]
- [ ] [parallel] Update the architecture policy template, consumer guide, assurance-profile guidance, CLI output, SARIF, and provenance surfaces. r[octet.message_boundary.consumer]

## Positive and negative verification

- [ ] [parallel] Add positive fixtures for logical session state, local pure calls, adapter-owned connections, application-shell mailbox execution, and owned message contracts. r[octet.message_boundary.verification]
- [ ] [parallel] Add negative fixtures for direct, aliased, nested-generic, associated-type, async, callback, borrowed-buffer, channel, shared-lock, SDK-type, bypass-ingress, inactive-feature, and configured vendor-handle leaks. r[octet.message_boundary.verification]
- [ ] [parallel] Add negative fixtures for missing compiler shards, unresolved required dispatch, stale waivers, narrowed package scope, and receipt tampering. r[octet.message_boundary.unknowns] r[octet.message_boundary.verification]
- [ ] [serial] Repeat normalized fact and receipt generation under unchanged inputs and compare byte identities. r[octet.message_boundary.evidence] r[octet.message_boundary.verification]
- [ ] [serial] Run focused runner and standards tests, policy freshness, artifact verification, formatting, Clippy, Octet self-check, Cairn validation and gates, Tracey coverage, and compatible Nix checks. r[octet.message_boundary.verification]
