# Verification

Date: 2026-08-31

## Implemented boundary

- `standards/sarif/profiles.ncl` defines typed `codeql-local` and `semgrep-local` profiles.
- The generated registry has BLAKE3 identity `b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7`.
- Each profile fixes the driver, version, severity mapping, local path policy, evidence tier, resource limits, and non-claims.
- The pure core parses a bounded SARIF 2.1.0 subset and returns ordered findings or ordered diagnostics.
- Stable finding identities frame the driver, rule, source level, message, path, line, and column with BLAKE3.
- The shell reads the profile and report through one explicit capability root.
- The shell rejects `--run`, arbitrary `--command`, stale profile bytes, unsafe paths, and oversized files.
- Receipts keep source severity separate from Octet gate severity.
- Raw reports and normalized projections retain separate identities and provenance links.
- Valence export keeps the external checker role and the `static_lint` tier.

## Positive and negative evidence

- Positive fixtures cover CodeQL and Semgrep driver profiles, direct URIs, artifact indexes, and deterministic projection identity.
- Negative fixtures cover malformed JSON, wrong versions, unknown drivers, undeclared rules, duplicate rules, and unsupported levels.
- Location tests reject traversal, remote URIs, URI schemes, drive paths, missing structures, invalid indexes, and invalid coordinates.
- Limit tests reject oversized reports and excess artifact tables.
- The local shell test rejects run authority and reads no hosted service.
- Receipt tests reject proof-role promotion, stale profile identity, stale raw identity, and stale projection identity.
- Nickel accepts the valid receipt fixture and rejects the wrong receipt version.

## Passing commands

- `nix develop --builders '' -c cargo test -p cargo-octet --lib external::sarif` passed 5 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib sarif_import` passed 3 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib evidence_rails::tests` passed 38 tests.
- The focused evidence-contract fixture test passed.
- Library and test Clippy gates passed with `-D warnings` and all features.
- `sarif-profiles`, `policy-registry`, and `cairn-traceability` Nix checks passed with local builders.
- Cairn validation and proposal, design, and tasks gates passed with the canonical workspace policy.
- The new SARIF modules report zero Octet findings.
- `git diff --cached --check` passed.

## Adversarial audit

The first audit found missing diagnostics for absent physical locations, artifact locations, and regions. The implementation now rejects each structure with a field diagnostic.

The audit also found that drive paths and scheme forms without `//` could pass. Path admission now rejects every colon.

The profile promised an artifact-table limit but did not contain one. The checked profile now binds and enforces `max_artifacts`.

The capability reader used an unchecked growth-sentinel addition. It now rejects an unrepresentable read limit before file access.

## Inherited blockers

The broad cargo-octet library suite retains two pre-existing Nickel identity failures. Both failures reproduced on untouched `origin/main` before this implementation.

The broad self-check also retains its pre-existing warning-budget drift. This change does not replace a baseline or suppress a finding.

## Non-claims

SARIF intake preserves bounded external static findings. It does not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.
