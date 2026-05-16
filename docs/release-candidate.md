# Slotcar release candidate checklist

This document is the pre-tag release candidate receipt for Slotcar. It is intentionally a local verification checklist; it does not tag, publish, sign, or deploy artifacts.

## Candidate identity

- Candidate version: `0.1.0`
- Canonical Cargo package: `cargo-slotcar`
- Compatibility Cargo binary: `cargo-tigerstyle`
- Lint library package outputs: `slotcar` and compatibility alias `tigerstyle`

## Required pre-tag gates

Run these from the repository root before tagging:

```bash
nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link
nix flake check -L
```

## Install smoke commands

The release candidate smoke check verifies these installed binary surfaces from the current flake build:

```bash
nix run .#cargo-slotcar -- --version
nix run .#cargo-tigerstyle -- --version
nix build .#cargo-slotcar .#cargo-tigerstyle .#slotcar .#tigerstyle --no-link
```

Expected version output:

```text
cargo-slotcar 0.1.0
```

## Release notes checklist

Before tagging, update release notes or the GitHub release body with:

- the candidate version (`0.1.0` for this candidate),
- the package outputs that shipped (`cargo-slotcar`, `cargo-tigerstyle`, `slotcar`, `tigerstyle`),
- the successful `release-candidate-smoke` and `nix flake check -L` receipts,
- any known compatibility aliases or non-goals.

## Acceptance signal

The candidate is ready to tag only after `checks.x86_64-linux.release-candidate-smoke` and `nix flake check -L` pass from a clean checkout and the OpenSpec evidence bundle records the exact commands and outputs.
