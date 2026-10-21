# Immutable producer handoff design

## Context

`standards/proof-rails/aeneas-lean/consumer-profile.ncl` and its pinned provider own the existing external-fixture theorem rail. Mantle's ADR 0034 requires producer-authored execution and opaque preservation of external proof authority. The Nix graph and source objects are acquisition inputs; neither graph export nor process success admits a theorem.

## Decision

Reuse the existing profile, resource policy, execution commands and artifact verifier in `mantle-job.ncl`; register its Nickel ownership in the policy registry. `nix/mantle-aeneas-job.nix` executes `cargo-octet prove` followed by `cargo-octet artifact verify`. It requires the owner's supported proof envelope and valid artifact report, retains original bytes and artifacts, and records its immutable source revision. It does not manufacture receipts from exit status.

`tools/mantle-producer/export.py` requires a committed revision, resolves the locked flake source before exporting Nickel, and captures the complete recursive graph and declared source objects. Version-specific graph admission preserves original Nix JSON bytes and identities. It rejects missing edges, mixed/unknown formats and dynamic outputs, and applies explicit graph, member and source-byte bounds. Acquisition never executes the proof or silently substitutes a mutable checkout.

The ordinary derivation depends on the existing provider, Rust toolchain and policy artifacts. It has no alternative verifier implementation and no new solver. The complete tool graph includes declared binary/bootstrap inputs, not a seed-free or source-only claim.

## Verification

- Export the typed job and registry; run positive and negative graph/export fixtures through the existing Nix check.
- Export a real graph from a committed immutable producer revision and retain original metadata, graph, source-object and BLAKE3 identities.
- Exercise the ordinary owner job, preserving its actual proof and artifact reports or the exact failed prerequisite. Only a valid supported owner result satisfies execution acceptance.
- Run native proposal/design/tasks gates with Octet's pinned Cairn executable and policy; separately retain sync/archive and post-archive validation.

## Authority and Maintenance

Octet maintains these producer-owned files and decides external proof acceptance. Mantle owns its execution, store signatures, export/cache/replay and evidence associations. Valence owns canonical linkage; Cairn owns lifecycle decisions; Trellis retains its theorem campaign authority. None of these roles is transferred by a build, graph, receipt attachment or successful lifecycle gate.
