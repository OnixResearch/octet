# Validation evidence

Date: 2026-07-26

## Baseline

Before implementation:

- `nix run .#octet-policy-validate` passed.
- `nix develop -c cargo test -p octet-standards artifacts` passed the existing focused artifact tests.
- Cairn repository validation and the proposal, design, and tasks gates passed.

## Profile identity

- Profile ID: `onixos-lifecycle-proof-v1`.
- Profile JSON BLAKE3: `1c7a5bf42d998da49e2ae732f03198cdb318d793199de0b7c0a3247f96dad9fa`.
- Authored profile BLAKE3: `c7f49352f6f35476502dbd69839123bb1499e73f9c73abba3222600f2d612403`.
- Contract dependency BLAKE3: `59941ad61aeb41e56d01e2086e8271dc3f86bcd10b123c6d6193e42593d3db29`.
- Manifest schema: `octet-nickel-export-manifest/v1`.
- Evaluator: `nickel-lang-cli nickel 1.16.0 (rev cargore)`.

The focused Nix check exported the profile and manifest twice. Both runs produced identical bytes. The check then changed the exported output bytes and confirmed that the manifest and output BLAKE3 changed.

## Positive and negative validation

- `nix build .#checks.x86_64-linux.assurance-profile-onixos-lifecycle --no-link -L` passed.
- The full profile and minimal profile fixtures exported successfully.
- Invalid fixtures for empty scope, empty non-claims, duplicate IDs, unresolved references, unused declarations, unsupported assurance planes, and unknown fields failed closed.
- The focused check confirmed all five assurance planes and all seven required OnixOS evidence IDs in generated JSON.
- Every mechanism, evidence requirement, and the full profile retain non-empty non-claims.

## Distribution

- `nix build .#octet-assurance-profile-onixos-lifecycle --no-link --print-out-paths` passed.
- Output: `/nix/store/mi6mm5da6knjkygvq1hdsygzcibz41ga-octet-assurance-profile-onixos-lifecycle-proof-v1`.
- The focused check confirmed the package includes authored Nickel, the contract, generated JSON, the manifest, and `profile.blake3`.
- The packaged BLAKE3 matches the generated manifest output identity.

The Nix store path is local build evidence, not a durable consumer pin. OnixOS must select a published immutable Octet revision before it adopts the package as durable input.

## Regression and lifecycle checks

- `nix run .#octet-policy-validate` passed after implementation.
- `nix build .#checks.x86_64-linux.standards-fixtures --no-link -L` passed.
- `nix develop -c cargo test -p octet-standards artifacts` passed four focused artifact tests with no failures.
- Cairn repository validation passed.
- Cairn proposal, design, and tasks gates passed.
- Cairn synced `cairn/specs/assurance-profiles/spec.md` and archived the completed change.
- Post-archive Cairn repository validation passed.
- `nix flake check -L` passed all checks for `x86_64-linux`. Nix reported that other systems were omitted as incompatible.
- `git diff --check HEAD` passed.

Nix reported an unavailable configured remote builder and a missing Git HTTPS helper while it used cached pinned inputs. It completed the requested checks locally.

## Pilot boundary

This profile maps requirements for the active OnixOS `onixos-lifecycle-proof-receipts` change. It does not report that the required OnixOS artifacts exist or pass. The active OnixOS change remains blocked until its canonical FSM, Rust core, Kani, Octet, Valence, and Cairn chain work passes.

A local secondary-model review request returned `fetch failed`. Repository contracts, positive and negative fixtures, deterministic identity checks, package checks, policy checks, lifecycle gates, and the full flake check provide the recorded validation evidence.
