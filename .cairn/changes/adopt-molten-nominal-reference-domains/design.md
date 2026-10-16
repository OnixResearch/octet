## Context

The existing embedded `octet-nominal-domain-policy/v1` catalog is the runtime
source of truth. Published provider base `620833ca621efd3908ae4f3d76c3975effa8bd1a`
already contains nominal enforcement; canonical checkout user changes are not
copied. Existing committed exploratory Molten rows at
`73d5f8e9172d40af935c8d4f9bbac24a26647870` supply the established domain ID and
field mapping convention, not an accepted or published cohort.

## Decisions

### Decision: Adopt exact central catalog rows

**Choice:** Declare all 15 alias and qualified struct-field symbols listed in
`docs/type-driven-domain-modeling.md`, with raw `string`, `marker-generic`, and
`private-fallible`. Core is exactly `crates/molten-core/src/nominal.rs`;
compatibility is exactly `src/authority/nominal.rs`, required for every row.
Fixture file selectors are included in the same rows to exercise the real policy.

**Rationale:** Existing source-owned Nickel/export conventions avoid a second
consumer-config catalog abstraction. File suffix matching is exact, not prefix
or logical-label matching; explicit compatibility takes precedence.

### Decision: Represent checked generic construction honestly

**Choice:** Permit the additional `marker-generic` / `private-fallible` pair in
both Nickel and Rust admission. Preserve `marker-only` and continue rejecting
`marker-generic` / `private-infallible`. Add matching valid/invalid Nickel inputs
and a public JSON-admission regression, without diagnostic wording assertions.

**Rationale:** Molten's actual private generic storage has fallible constructors;
mislabeling it as marker-only would discard that reviewed contract. The existing
refined-newtype bypass checker remains unchanged and does not prove generic
constructor semantics.

### Decision: Check local macro aliases at their user callsite

**Choice:** Keep compiler-resolved alias representation checking. Replace blanket
expansion suppression with external-macro exclusion; select policy source and
expanded diagnostics through `source_callsite()`. Expanded aliases report only
`ExactCore`; unexpanded aliases retain existing inventory semantics.

**Rationale:** Molten's domain macro is its actual API, so removing the macro
would repair the fixture rather than the checker gap. Exact matching avoids
heuristic-only macro noise and preserves compatibility. Arbitrary external macro
expansions remain outside this bounded change.

### Decision: Keep evidence and publication separate from implementation

**Choice:** Register deny-level compiler UI fixtures for raw direct aliases,
local macro-to-String aliases, and direct/optional raw core fields across every
domain. Nominal macro aliases and explicit raw wire compatibility must compile
without nominal diagnostics. The parent generates actual new diagnostic snapshots
and runs clean plus mutated Molten source with the integrated provider; no empty
or fabricated negative snapshots are supplied.

**Rationale:** Rust nominal compile-fail tests alone do not establish Octet
policy enforcement. Generation is allowed here, verification is reserved for the
integration owner. An unpublished working tree is not an immutable consumer pin.

## Risks / Trade-offs

- Borrowed primitives and arbitrary generic internals are not traversed. The
  borrowed `require_session` API is Rust type separation, not raw-value lint proof.
- Marker-generic rows describe the wrapper contract; the refined-newtype bypass
  checker does not establish generic constructor semantic validity.
- The installed Cairn is incompatible with this published provider's policy.
  Use the already-realized provider-pinned Cairn binary resolved through its flake;
  do not weaken or regenerate policy to admit a different lifecycle graph.
- Parent verification and separately authorized immutable publication remain
  mandatory before consumer lifecycle sync/archive or accepted enforcement claims.
