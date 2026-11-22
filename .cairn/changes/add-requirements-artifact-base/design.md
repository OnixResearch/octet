# Design: Location-independent proof-deck locators

## Context

`requirements check` evaluates an evidence bundle against a reviewed contract.
For a `formal_proof` clause it reads each allowed `proof_decks` entry, resolves
`deck` against `--source-root`, and resolves `artifact_root` and each proof
card's retained artifact locator from that root. The pilot's deck was produced
by a realized Mantle job, so the retained evidence had to name the deck and its
artifact root by absolute store paths. The decision records `evidence_blake3`,
`source_snapshot_blake3`, and the evaluated clause outcomes, so those absolute
locators made the decision identity depend on where the output happened to be
stored.

## Decisions

### Decision: Resolve with an optional artifact base

**Choice:** Carry `artifact_base: Option<&Path>` from the CLI through
`evaluation::decision::evaluate_contract` and `evaluation::clause::Context` to
`formal::evaluate`. A new `resolve_artifact_locator` helper in
`cargo-octet/src/requirements.rs` resolves `artifact_root` and `deck`:

- no base: `resolve_path(source_root, locator)`, exactly today's behavior;
- absolute locator: use it directly, exactly today's behavior, except when it
  names a path inside the base, where the stripped relative remainder is still
  subject to the containment rule;
- relative locator: hold it to the containment rule and resolve it under the
  base.

**Rationale:** One helper covers both locator fields, and the no-flag path is a
literal call to the existing resolver, so existing evidence and decisions cannot
change. Threading the option through the existing `Context` reuses the
evaluation seam instead of adding a parallel mechanism.

### Decision: Mirror Cairn's containment rule and diagnostic shape

**Choice:** Reject an empty locator, a root or prefix component, a `..`
component, and any locator that is still absolute after the base prefix is
stripped. Allow `.` and `Normal` components, which the pilot evidence needs for
`artifact_root: "."`. Report a diagnostic named by the locator's role
(`proof deck` or `artifact root`), the offending locator, and "escapes the
artifact base", then fail the clause.

**Rationale:** Cairn's `deck_relative_path` refuses a deck that is not a pure
relative path under the producer output. Octet's artifact root legitimately is
the base itself, so `CurDir` must stay admissible there; the shared rule still
refuses every escape.

### Decision: Keep the base out of the decision

**Choice:** The base only participates in filesystem resolution. It is never
copied into the decision, the diagnostics of a passing decision, or the digest
input.

**Rationale:** The acceptance property is byte-identical decisions for
identical producer-output content in different directories. Recording the base
would reintroduce the defect the flag removes.

### Decision: Convert the pilot evidence and accept one decision-identity change

**Choice:** Rewrite the pilot `evidence.json` to relative locators, refresh the
committed `decision.json` from a real `requirements check --artifact-base <store>`
run, and update every command that names the family.

**Rationale:** The evidence bytes change, so `evidence_blake3` and
`decision_blake3` change once, as intended; from then on the decision is stable
across store locations. The proof cards and every reviewed source byte stay
identical.

## Risks / Trade-offs

- A consumer that relies on the pilot's historical `906cc950…` decision must
  re-record the new identity once; the CHANGELOG states this explicitly.
- An absolute locator outside the base remains authoritative as before, so the
  flag does not sandbox deliberately absolute evidence. The containment rule
  applies only where a base is declared and the locator is relative or inside
  that base.
