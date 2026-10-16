# Type-driven domain modeling

Octet uses small Rust domain types at selected core boundaries to prevent raw
string, path, numeric, and boolean combinations from reaching evidence logic
unchecked. The goal is boring algebraic modeling: newtypes for domain identities,
enums for mutually exclusive states, and refined constructors for non-empty,
canonical, sorted/deduplicated, path-normalized, and threshold/count values.

The first migrated runner seam validates finding DTOs before JSONL and SARIF
rendering while preserving existing wire fields. Lint names, group names, stable
IDs, semantic IDs, artifact locators, severity, and one-based source positions
are checked through adapters; output remains the same for valid findings.
Malformed identities, non-canonical paths, unsupported severities, and zero line
or column positions fail before rendering.

This is not Haskell cleverness. Octet should avoid lens-heavy APIs, monad
terminology, laziness assumptions, macro-heavy encodings, and type-level
machinery unless a later change ties them to a concrete bug-prevention payoff.
Runtime receipts and emitted evidence DTOs stay Rust-owned; domain types protect
internal cores and adapters without creating a second wire-format source of
truth.

## Nominal-domain policy

`standards/nominal-domains/catalog.ncl` is the reviewed source of truth. Its
JSON export gives the lint driver deterministic runtime data.

Each policy row declares these items:

- the domain symbols and raw representation
- the core source scopes
- explicit compatibility scopes
- the wrapper and constructor contract

`primitive_domain_alias` finds raw type aliases. `raw_domain_value` checks
registered fields and API values. `newtype_invariant_bypass` checks refined
wrapper escape paths.

Use this boundary flow:

```text
wire DTO -> checked adapter -> typed core -> wire projection
```

Wire, FFI, protocol, generated, and vendored scopes can stay structural. Each
exception must be explicit. A typed adapter must reject malformed values before
they enter the core.

## Rollout modes

Omit `nominal_domain_mode` to keep the three lints at their default levels.
Set it to `inventory` for non-blocking exact and suffix findings. Set it to
`warn` for exact warnings only. Set it to `deny` for exact errors only.

A passing check proves configured source-shape conformance. It does not prove
authority, secrecy, runtime correctness, wire safety, or release eligibility.

The catalog admits `marker-generic` wrappers with either `marker-only` or
`private-fallible` construction; `private-infallible` remains rejected for this
wrapper kind. The latter pair represents checked generic storage such as Molten's
reference wrappers. This declaration does not make the refined-newtype bypass
checker verify generic constructor semantics.

## Molten reference-domain adoption

The existing nominal checker is reused; Molten adds policy rows under
`molten.nominal-reference.*`, not a consumer-supplied runtime catalog.
All rows use `raw_representation = "string"`, `wrapper_kind = "marker-generic"`,
and `constructor_policy = "private-fallible"`. The core source is exactly
`crates/molten-core/src/nominal.rs`; `src/authority/nominal.rs` is explicit
compatibility, with `compatibility_required = true`. UI fixture paths in the
same rows exercise this contract without changing the consumer scope.

| Domain suffix | Alias | Registered core fields |
| --- | --- | --- |
| `principal` | `PrincipalRef` | `AuthorityReferenceSet::holder` |
| `node` | `NodeRef` | `ExecutionReferenceSet::node` |
| `actor` | `ActorRef` | `ExecutionReferenceSet::actor` |
| `service` | `ServiceRef` | `ExecutionReferenceSet::service` |
| `session` | `SessionRef` | `AuthorityReferenceSet::session`, `ExecutionReferenceSet::session` |
| `authority-context` | `AuthorityContextRef` | `AuthorityReferenceSet::context` |
| `delegation` | `DelegationRef` | `AuthorityReferenceSet::delegation` |
| `revocation` | `RevocationRef` | `AuthorityReferenceSet::revocation` |
| `key` | `KeyRef` | `AuthorityReferenceSet::key` |
| `policy` | `PolicyRef` | `AuthorityReferenceSet::policy` |
| `resource` | `ResourceRef` | `AuthorityReferenceSet::resource`, `ExecutionReferenceSet::resource` |
| `evidence` | `EvidenceRef` | `AuthorityReferenceSet::evidence`, `ArtifactReferenceSet::evidence`, `HistoricalReferenceSet::evidence` |
| `artifact` | `ArtifactRef` | `ArtifactReferenceSet::artifact`, `HistoricalReferenceSet::artifact` |
| `operation` | `OperationRef` | `ExecutionReferenceSet::operation`, `ArtifactReferenceSet::operation` |
| `receipt` | `ReceiptRef` | `ArtifactReferenceSet::receipt`, `HistoricalReferenceSet::receipt` |

Scopes are normalized exact file paths, including matching absolute-path suffixes;
they are not directory prefixes, globs, or logical migration labels. Symbols are
exact alias names or qualified Rust field-path suffixes. Compatibility takes
precedence. Wire DTOs, projections, display text, private generic wrapper storage,
and `ReferenceError::role` are not claimed as migrated primitive domain fields.

Local macro-generated aliases use the user invocation callsite for policy scope
and diagnostics, and their compiler-resolved type for representation checks.
Only exact core rows are enforceable for expanded aliases; suffix-only macro
names do not generate inventory noise. External macro expansions remain excluded.
Nominal generic aliases are admitted because their resolved type is a nominal
wrapper, not primitive storage. `molten_nominal_macro_raw_alias` exercises the
local macro-to-String regression for all 15 domains; `molten_nominal_raw_alias`
and `molten_nominal_raw_core` cover direct aliases and registered fields.
`molten_nominal_typed` and `molten_nominal_compat` cover admitted shapes.

The raw-value checker covers direct `String` and its supported `Vec`, `Option`,
`Result` success, and `BTreeMap` containers. It does not inspect borrowed
`&str`/`&String` APIs, arbitrary generic internals, macro-generated fields,
or unregistered enum tuple payloads. In particular, Molten
`require_session(&SessionRef) -> &SessionRef` has Rust nominal separation but
is not evidence that borrowed raw representations are enforced by this policy.
Broader cache, retention, node, effect, or authority migration claims are not
implied by the one declared core source file.

The published provider base inspected for this adoption is
`620833ca621efd3908ae4f3d76c3975effa8bd1a` from the recorded
`git@github.com:OnixResearch/octet.git` upstream. It contains the pre-existing
checker but no Molten catalog rows or local-macro alias enforcement. A consumer
immutable pin requires an approved, verified provider commit containing these
changes and upstream availability; a local working tree or alias declaration
is not a published cohort or enforcement proof. Publication and coordinated
consumer pinning require separate owner authorization and passing provider
acceptance; neither is implied by this guide.

The reviewed implementation was first published at
`82a6f01a15cfbfda7ed9b37e03605d0c052391c4`. The
[native lifecycle linkage](../.cairn/archive/1970-01-01-adopt-molten-nominal-reference-domains/evidence/native-lifecycle-linkage.json)
retains successful specification sync/archive, unchanged native plan inputs,
and exact mutation hashes under the provider-pinned Cairn and unchanged policy.
The archive also retains the named unblessed UI replay, complete new snapshots,
catalog freshness, 154 library and 13 metadata tests, formatting, and review.
This provider evidence is not proof of immutable Molten adoption or runtime correctness.

