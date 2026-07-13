## Why

Octet and standalone Valence both currently expose materially different crates named `valence-core` at the same package version. Aspen resolves the older Octet-hosted copy, so package name and version do not identify one semantic implementation. Valence already owns the canonical manifest and Octet's accepted extraction spec defines the standalone cutover, but the consumer and dependency migration is incomplete.

Octet must complete that cutover: consume one exact standalone Valence revision, retain only explicit compatibility shims, and eliminate ambiguous duplicate package identity.

## What Changes

- Add a pinned standalone Valence dependency manifest and validate its canonical source identity.
- Route Valence-owned pure semantics through the standalone dependency while keeping Octet-owned shells in Octet.
- Rename or retire the Octet-hosted `valence-core` package so two different implementations cannot share one package name and version.
- Run positive compatibility fixtures and negative stale-copy, wrong-source, missing-row, and package-identity collision fixtures.
- Publish Aspen migration guidance bound to the exact accepted standalone source.
- Add a checked-in CI workflow whose verification scope is `nix flake check`.

## Impact

- **Files**: workspace manifests, lockfile, compatibility manifest, Octet adapters, legacy crate disposition, fixtures, docs, Nix checks, and CI workflow.
- **Dependencies**: implementation waits for an archived Valence `harden-preserves-integrity-boundaries` receipt and consumes the existing Valence canonical-core manifest.
- **Compatibility**: Octet command behavior remains shell-owned; compatibility is proven only for the declared fixture corpus.
- **Claims**: cutover proves source ownership, package identity, and selected fixture compatibility only, not semantic equivalence of arbitrary programs, correctness, or release eligibility.
