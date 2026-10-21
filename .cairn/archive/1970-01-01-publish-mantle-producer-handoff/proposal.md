# Publish the immutable Mantle producer handoff

## Why

Mantle issue #14 needs an Octet-owned executable producer contract before Mantle can build verification artifacts without taking over theorem admission. The existing Aeneas consumer profile and external-fixture rail already own source selection, tool identities, resource bounds, verification and non-claims. An exported graph must come from the exact committed producer source, not a mutable checkout or reconstructed receipt.

## What Changes

- Export the existing selected consumer rail as the bounded Nickel job `octet-mantle-producer-job/v1` and an ordinary sandboxable derivation.
- Preserve actual proof, artifact-verification, provenance and original receipt bytes with their selected profile and immutable producer revision.
- Add a bounded acquisition-only exporter for the complete recursive derivation graph and source objects. Admit legacy and explicit Nix v4 graph shapes independently; reject unsupported, mixed, incomplete or dynamic-output graphs without rewriting original bytes.
- Document producer/consumer ownership and verify the real handoff under the owner-pinned toolchain and lifecycle policy.

## Scope and Non-goals

The immediate consumer is Mantle's external build bridge; Octet maintains the producer contract, exporter and existing proof admission. This change does not add theorem policy, grant trust, prove verifier/compiler soundness, promise source-only bootstrap or establish Mantle execution or release acceptance. Cached owner output records its original verification; cache reuse is not fresh proof work. Mantle must record and verify its own separate build/replay observations.
