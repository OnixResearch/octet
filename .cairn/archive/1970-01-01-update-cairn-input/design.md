## Context

Octet uses its Cairn input for traceability and lifecycle validation. The admitted producer revision rejects a repository that has only the legacy `cairn/` layout. A lock-only update therefore fails the existing `cairn-traceability` check.

The clean pre-change full flake baseline also has an unrelated silent failure in `cargo-octet-self-check`. This change records that baseline and does not weaken or repair the self-check.

## Decisions

### Decision: Generate the lock update with Nix

**Choice:** Run a targeted Nix input update for `cairn`. Do not edit `flake.lock` by hand.

**Rationale:** Nix owns the transitive lock graph and records the exact producer revision and NAR identity.

### Decision: Use the hash-bound Cairn migration

**Choice:** Preview `migrate cairn`, review its source and destination entries, execute the exact plan hash, verify old/new byte parity, and then remove the legacy source tree.

**Rationale:** This preserves lifecycle bytes while preventing two writable lifecycle authorities.

### Decision: Preserve historical and external path meaning

**Choice:** Update paths that refer to Octet's maintained lifecycle root. Keep synthetic compatibility fixtures, historical evidence, and references to other repositories unchanged unless they select Octet's live tree.

**Rationale:** A mechanical global replacement would corrupt historical evidence and external source coordinates.

### Decision: Change no accepted product requirement

**Choice:** Use the `no-spec-delta` workflow profile.

**Rationale:** This change updates validation tooling and repository layout. It does not change Octet lint, runner, evidence, or policy semantics.

## Risks / Trade-offs

- The migration touches a large lifecycle corpus. Rename review and migration receipts bound the risk.
- A stale live path can silently reduce traceability coverage. Focused searches and the traceability gate detect that class.
- The inherited self-check failure limits the final whole-flake claim unless it clears independently.
