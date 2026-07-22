## Why

Octet documents `[workspace.metadata.octet]` as mandatory runner policy, but its own root `Cargo.toml` omits that table. Consequently, the canonical local command `nix run .#cargo-octet -- check` exits `2` before invoking Cargo, so contributors must substitute other gates instead of exercising the published no-argument path.

## What Changes

- Add canonical root workspace metadata selecting all workspace libraries without tests, examples, or binaries.
- Retain fail-closed rejection for missing or malformed consumer metadata; do not add implicit defaults or legacy top-level metadata.
- Add a deterministic root-manifest regression test and a hermetic Nix check covering successful self-check execution plus pre-Cargo malformed-metadata rejection.
- Record bounded evidence that the canonical path executes successfully without weakening lint policy.

## Impact

- **Files**: root `Cargo.toml`, `tests/consumer_integration.rs`, `flake.nix`, accepted Cairn policy/spec sources, and this native change package.
- **Testing**: pre-change failure capture, focused positive and negative runner tests, canonical Nix-app execution, dedicated Nix check, formatting, Clippy, Nextest, strict Cairn validation, traceability, and the broad flake gate.
- **Compatibility**: consumer metadata remains mandatory and canonical at `[workspace.metadata.octet]`; legacy aliases retain their existing lower-precedence behavior.

## Non-Goals

This change does not add implicit runner defaults, weaken lints, allow style violations, certify tests/examples/binaries through the no-argument scope, prove whole-program correctness, or establish release eligibility. Cargo metadata remains TOML because Cargo owns that native interoperability surface; this does not introduce TOML as a general Octet configuration default.
