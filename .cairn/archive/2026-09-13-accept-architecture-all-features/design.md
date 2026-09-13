## Context

The collector already obtains a bounded locked Cargo unit graph for the actual invocation.
Policy rows name explicit features and default-feature behavior.
The current selector check rejects `--all-features` unconditionally.
Enumerating manifest names cannot reproduce Cargo dependency, host, test, and target resolution.

## Decisions

### Decision: Compare actual and declared Cargo plans

**Choice:** For `--all-features`, obtain a second plan with that flag replaced by the explicit policy features.
Preserve package, target, default-feature, profile, and target-kind selectors.
Both calls use the existing bounded non-executing Cargo adapter.

**Rationale:** Equivalent complete plans establish the required binding without implementing another feature resolver.
A feature-gated target, optional dependency, or host feature difference must reject the invocation.
Metadata continues to use the checked explicit row. Compiler units continue to record Cargo-resolved features from the actual invocation.

### Decision: Normalize complete graph content in a pure function

**Choice:** Separate graph comparison from process execution.
Canonical unit descriptions retain Cargo fields, except dependency indices that need content-based references.
Normalize feature order, unit order, dependency order, and root order without discarding graph content.
Bind dependency edges to their unit identities. Reject missing, invalid, or ambiguous graph references.

**Rationale:** Indices and output order are not semantic identities.
Two plans can contain the same units but different dependency edges.
The existing BLAKE3 identity helper supplies content identity, not authenticity.

### Decision: Preserve policy rejection and receipt scope

**Choice:** Reject mixed `--all-features` and explicit feature selectors, repeated selectors, and unsupported shorthand.
Keep existing explicit-feature matching and default, target, package, and shard checks.
A mismatch returns an integration error before collector context publication or clean architecture evidence.

**Rationale:** Accepting a flag must not grant permission to broaden the reviewed row.
Existing invocation identities retain the actual flag. No public receipt schema changes.

## Verification

Baseline: `nix develop -c cargo test --locked -p cargo-octet --lib architecture_ir` with the Nix-owned vendor configuration in an isolated Cargo home.

Pure controls cover reordered equivalent graphs, feature changes, root changes, dependency-only changes, duplicate identities, invalid indices, and missing graph fields.
Cargo controls cover complete and incomplete policy rows, optional dependencies, workspace features, defaults, and feature-gated targets.
The existing ignored compiler control runs with explicit features and all features. Both runs assert complete coverage and reject an omitted shard.
A full deny-all consumer probe must pass with the complete row and reject an under-declared row before compiler acceptance.

Publication requires format checks, focused Rust tests, strict Clippy, `checks.x86_64-linux.compiler-architecture-targets`, and lifecycle gates.
Generated contracts must remain unchanged because this change adds no emitted DTO.
Operator logs remain in the primary repository under `.pi/architecture-all-features-1629/`.

## Risks / Trade-offs

All-feature collection needs two bounded Cargo plans instead of one.
Each plan retains the existing time and byte limits. Neither plan compiles code or runs a build script.
A Cargo format change, duplicate semantic unit identity, or unresolved comparison fails closed rather than assigning a clean result.
The comparison is conservative. It does not excuse unreviewed defaults or features.
