## Context

`QUANTITY_WORDS` describes words, but `has_quantity_word` uses substring matching. `port` therefore matches imports, exports, transport, and support. HWR's existing count fields expose the defect through the actual compiler-backed deny-all path.

## Decisions

### Decision: Match complete identifier components

**Choice:** Preserve ASCII case normalization; split the snake-case identifier on underscores and compare complete components with the existing fixed quantity vocabulary. Keep raw-numeric type admission, nominal-type acceptance, skip names, unit suffixes, diagnostics, and severity unchanged.

**Rationale:** `network_port`, `request_timeout`, and `buffer_size` remain quantities; import/export counters do not acquire units through an unrelated substring. This is a generic lexical correction, not a consumer or spelling exception. Identifier splitting is borrowed and adds no collection allocation.

### Decision: Preserve consumer contracts and actual compiler proof

**Choice:** Extend the existing numeric fixture with positive quantity and negative substring cases across fields, parameters, and local bindings. Retain the original diagnostics, build the actual full owning library, exercise the full maintained corpus, and rerun HWR with the unchanged 69-lint policy. Do not retain the attempted runtime count-field renames.

**Rationale:** Pure classification tests alone do not establish registration or compiler behavior. Existing HWR runtime, serialized field, and diagnostic contracts need no migration.

## Risks / Trade-offs

- The documented input grammar is underscore-delimited identifiers, not arbitrary semantic word segmentation or camel-case inference.
- Passing lexical/compiler checks is not a proof of units, semantics, runtime correctness, source availability, or release eligibility. Octet's recorded publication/rights prerequisites remain unchanged.
