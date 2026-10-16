## Why

Molten's 15 reference domains already use nominal generic wrappers, but its old
Octet cohort predates nominal enforcement. The published Octet provider contains
the nominal checker yet has no Molten catalog rows, and its blanket expansion
skip misses Molten's local `domains!` alias-to-String regression. Catalog rows
alone therefore cannot satisfy the consumer's declared acceptance.

## What Changes

- Reuse the existing three nominal-domain lints and central Nickel catalog.
- Declare the 15 `molten.nominal-reference.*` domains at the exact source and
  compatibility boundaries documented in `docs/type-driven-domain-modeling.md`.
- Admit `marker-generic` with `private-fallible` construction in both catalog
  validators while retaining `marker-only` support and rejecting unchecked generic
  construction; the existing refined-wrapper checker gains no generic proof claim.
- Resolve local macro-generated primitive aliases at the real user callsite;
  enforce exact configured rows only, preserving compatibility and external
  macro exclusions without heuristic-only macro noise.
- Add compiler UI rejection cases for direct and local-macro raw aliases and raw
  registered core fields, plus admitted nominal generic and wire-compatible cases.
- Extend existing config-family fixture registration and regenerate catalog JSON.

## Impact

- **Files**: central nominal catalog/export and Nickel/Rust admission contracts,
  `primitive_domain_alias` source,
  compiler UI fixture inventory/guardrails, existing config-family registry,
  domain-modeling guide, changelog, and this native Cairn package.
- **Testing**: parent-owned compiler UI diagnostic generation/replay, catalog
  freshness, focused library/metadata checks, and real Molten clean/String mutation
  smoke. No mid-flight builds, checks, tests, formatters, commits, or publication.
- **Compatibility**: preserve consumer `domains!` API and wire DTO bytes. Borrowed
  raw APIs, enum tuple payloads, generic wrapper internals, macro-generated fields,
  broader migrated seams, authority, and release eligibility are not new claims.
- **Publication**: base `620833ca621efd3908ae4f3d76c3975effa8bd1a` was available
  from the recorded GitHub upstream; the required changed policy/checker is not
  an immutable published cohort. Publication requires separate authorization.
