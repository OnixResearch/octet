## Why

Cairn now offers a signed Git COB collaboration provider, but Octet has no downstream operator evidence showing that a real native change can be imported, updated preview-first, moved only through explicit Git ref transport, and reloaded into a second checkout. Cairn's own interoperability fixtures establish provider behavior; they do not establish Octet adoption or Octet's local evidence boundary.

## What Changes

- Add an Octet operator guide for the local `radicle-cob.v1` backend, explicit Git repository paths, mutation-only keystores, preview-first writes, and external Git ref transport.
- Exercise this native change through a signed initial import and a signed follow-up update in a separate bare Git COB store seeded from the Octet base revision.
- Move the exact branch and namespaced COB refs to a second bare store with an explicit local Git fetch, then materialize the change into a second Octet checkout.
- Retain typed Nickel configuration, bounded receipts, identity observations, and a positive/negative Rust evidence verifier without adding Cairn or Radicle runtime dependencies to Octet.

## Impact

- **Files**: `docs/cairn-cob-collaboration.md`, `fixtures/cairn-cob-pilot/`, `tests/cairn_cob_pilot.rs`, and this native Cairn change package.
- **Testing**: focused evidence tests, Nickel export freshness, strict Cairn gates, Octet policy validation, `cargo octet check`, and the smallest applicable Nix checks.
- **Runtime**: no Octet runtime, dependency, network, profile, node, identity-document, or synchronization behavior changes.
