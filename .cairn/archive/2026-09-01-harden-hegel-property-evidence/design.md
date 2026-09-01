## Context

The archived `add-hegel-property-evidence-rail` change established receipt fields, non-claims, Valence export, and one Hegel-backed pure-core law.

The rail is useful, but its run mode accepts any explicit command. It then parses combined stdout and stderr as `hegel-property-summary/v1`.

Hegel is a test library rather than a general workspace scanner. Octet therefore needs a narrow producer-adapter contract instead of a generic command runner.

## Decisions

### Decision: Harden instead of readopting Hegel

**Choice:** Preserve the accepted rail and add a follow-up hardening delta.

**Rationale:** A second adoption change would duplicate completed lifecycle work and obscure the existing evidence boundary.

### Decision: Use typed producer profiles

**Choice:** Nickel profiles bind a producer package, binary target, property catalog, Hegel version, backend, case budget, phases, seed posture, source scope, limits, and non-claims.

The runner constructs one fixed Cargo argv shape from validated identifiers. It rejects `--command` for Hegel runs.

**Rationale:** A profile provides reviewable authority without accepting shell text.

### Decision: Require a dedicated summary artifact

**Choice:** A consumer-owned producer adapter writes one versioned summary to a capability-rooted temporary output path.

Octet does not parse mixed test-harness stdout or stderr as evidence. Logs remain bounded diagnostic artifacts.

**Rationale:** Test output is not a stable machine-readable contract.

### Decision: Remeasure provider and source identity

**Choice:** The shell obtains locked package metadata and hashes the selected profile, lockfile, source scope, generator inputs, summary, and bounded logs.

The pure core verifies that summary fields match the measured facts. A self-declared version or hash cannot establish identity alone.

**Rationale:** The receipt must bind the executed property producer, not only its claims.

### Decision: Preserve import compatibility

**Choice:** Existing import-only `hegel-property-summary/v1` inputs remain accepted under their current bounded claims.

Profiled runs emit the hardened summary version. Artifact verification distinguishes both versions and does not invent missing v1 measurements.

**Rationale:** Hardening run authority does not require breaking existing external evidence intake.

### Decision: Expand laws by value, not count

**Choice:** A checked law catalog names selected pure cores, law IDs, generators, positive properties, negative properties, budgets, and fixture complements.

A core enters the catalog only when generated inputs improve coverage beyond concrete tests.

**Rationale:** Property-test volume is not evidence quality.

## Functional Core and Imperative Shell

The pure core validates profiles, producer summaries, measured identities, law-catalog entries, results, counterexamples, limits, and deterministic diagnostics.

The shell resolves locked metadata, creates a capability-rooted output location, runs fixed argv, reads bounded artifacts, and writes receipts atomically.

## Risks / Trade-offs

- Consumer adapters add integration work. Import mode remains available when a profiled producer is not practical.
- Property tests can duplicate examples. The law catalog requires a stated generated-input benefit.
- Replaying failures can expose sensitive values. Profiles define bounded redaction and counterexample retention policy.

## Validation

Positive fixtures cover a profiled pass, a replayable counterexample, compatible v1 import, and matching measured identities.

Negative fixtures cover arbitrary commands, unknown profiles, malformed summaries, mixed-output parsing, unsafe paths, symlink substitution, stale identities, limits, timeouts, and overclaims.
