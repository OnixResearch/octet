# Concurrency and distributed-state review guide

Tiger Style treats concurrency and distributed state as design contracts, not cleanup after code lands. Review shared state, lock order, peer order, retries, and transitions in plain language first so later lints and tests have something concrete to defend. Unsafe should stay behind narrow safe wrappers, dangerous escape hatches should stay grep-able and explicit, and this document is a review kit rather than the full enforcement story.

## Shared-state inventory and ownership

This section matters because deadlocks, races, and state drift usually start with unowned shared mutation. Name each mutable state cell, queue, cache, lease, timer, or replica table and say which actor owns it and who may mutate it.

- Reviewer prompt: What shared mutable state exists on this path, and who owns each piece?
- Reviewer prompt: Which state crosses threads, tasks, or peers, and where is write authority restricted?
- Checklist: Prefer one obvious owner per mutable resource and explicit handoff points.

## Lock ordering and peer ordering

This section matters because multi-lock code and peer-to-peer protocols fail when acquisition or traversal order is implicit. Document the lock order, resource order, and peer iteration order before arguing about performance or convenience.

- Reviewer prompt: If two locks, semaphores, or guards can be held together, what exact order acquires and releases them?
- Reviewer prompt: When code walks peers, shards, or resources, what deterministic ordering rule keeps retries and replay stable?
- Checklist: Keep lock-order notes close to the data structure and make peer or resource ordering derive from one explicit rule.

## State-machine transitions and invariants

This section matters because concurrency bugs often hide inside state transitions that are legal in one phase and invalid in another. Model states and transitions explicitly so invalid moves fail loudly instead of being normalized after the fact.

- Reviewer prompt: What are the named states, and which transitions are allowed from each state?
- Reviewer prompt: Which invariant must hold before and after each transition, and how does code reject an impossible state?
- Checklist: Prefer typed transitions or one obvious transition function over ad hoc flag mutation.

## Message ordering, retries, and idempotence

This section matters because distributed systems see duplication, reordering, loss, and partial completion under normal operation. Review the expected delivery order and retry behavior now so recovery paths do not invent new semantics under pressure.

- Reviewer prompt: What ordering does the protocol assume across messages, timers, or wakeups?
- Reviewer prompt: If work retries or replays after timeout, crash, or leader change, what makes the operation idempotent or otherwise safe to repeat?
- Checklist: State whether each path depends on at-most-once, at-least-once, or deduplicated delivery and name the replay boundary.

## Unsafe quarantine and escape hatches

This section matters because low-level synchronization sometimes needs `unsafe`, but the blast radius must stay narrow and auditable. Keep the safe wrapper small, state the invariant it protects, and name any dangerous bypass so reviewers can grep for it later.

- Reviewer prompt: Where does unsafe appear, and what safe wrapper or invariant fences it in?
- Reviewer prompt: Which escape hatch bypasses the normal guardrails, and is its name explicit enough to grep during audit?
- Checklist: Send work back when unsafe leaks into ordinary call sites or when the exceptional path reads like normal API usage.

## Reviewer prompts and failure handling

This section matters because concurrency review needs a clear disposition, not vague discomfort. Record what must be fixed before merge, what can ship only with a tracked follow-up, and which unresolved ordering or replay risks need escalation to a design review or maintainer sign-off.

- Reviewer prompt: If a reviewer cannot explain ownership, ordering, transition, or retry rules in one paragraph, what design note or code change is missing?
- Reviewer prompt: Which finding blocks merge now, which finding can ship only with an explicit follow-up owner, and which finding requires escalation to a Tiger Style maintainer?
- Checklist: Send work back on undocumented lock ordering, unowned shared mutation, ambiguous replay semantics, invalid-state recovery by convention, or unsafe paths without a narrow audit boundary.
- Checklist: Escalate when protocol ordering or failure handling still depends on oral tradition instead of a documented rule.
