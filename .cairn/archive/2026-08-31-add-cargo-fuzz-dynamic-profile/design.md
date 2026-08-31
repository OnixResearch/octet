## Context

The dynamic-evidence runner already restricts execution to named profiles and allowed command shapes. Miri and Loom are the first supported profiles.

Coverage-guided fuzzing has additional state. Corpus bytes, dictionaries, crash inputs, minimized reproducers, and execution budgets affect the meaning of each result.

## Decisions

### Decision: Extend the existing profile runner

**Choice:** Add cargo-fuzz through `dynamic-evidence` instead of adding another top-level evidence command family.

**Rationale:** The existing runner already owns opt-in execution, bounded logs, acceptance signals, and non-proof evidence tiers.

### Decision: Keep policy in Nickel and receipts in Rust

**Choice:** Nickel owns allowed command shapes, timing, engine, sanitizer posture, and every execution or artifact limit.

Rust owns versioned receipt DTOs. Generated Nickel contracts protect receipt compatibility.

**Rationale:** Humans review policy before execution, while Rust controls emitted compatibility data.

### Decision: Treat corpus and crash artifacts as exact evidence

**Choice:** Compute a canonical BLAKE3 corpus root over safe relative paths, file lengths, and file hashes.

A crash result must bind the original crash input and any minimized reproducer. A clean result must bind the final corpus root.

**Rationale:** A pass or crash has little review value without exact input and state identity.

### Decision: Keep execution bounded and explicit

**Choice:** The shell constructs fixed argv for one selected fuzz target from reviewed profile fields.

The shell enforces policy-owned time, artifact-count, artifact-byte, log-byte, and path limits. It does not accept arbitrary command text.

**Rationale:** Fuzz engines are open-ended by default. Octet evidence must describe a finite observation.

### Decision: Harden all named dynamic profiles

**Choice:** Add consumer fixtures that prove Miri, Loom, and cargo-fuzz profiles accept declared commands and reject mismatched commands.

**Rationale:** Extending the registry must not weaken existing profile isolation.

## Functional Core and Imperative Shell

The pure core validates profiles, corpus manifests, run summaries, crash records, identities, limits, and deterministic issue ordering.

The shell resolves the pinned tool, runs fixed argv, reads bounded artifacts, writes bundles, and reports process failures.

## Risks / Trade-offs

- Fuzz outcomes depend on execution budgets and seeds. Receipts bind both and retain bounded non-claims.
- Corpora can contain sensitive data. The profile restricts paths and supports hash-only corpus rows where policy requires them.
- Toolchain drift can change findings. Exact engine, compiler, sanitizer, and Nix identities remain receipt inputs.

## Validation

Positive fixtures cover a clean target, a discovered crash, and replay of a minimized reproducer.

Negative fixtures cover unknown targets, command injection, path traversal, stale corpus roots, missing crash files, limit overflow, malformed summaries, and tool failure.
