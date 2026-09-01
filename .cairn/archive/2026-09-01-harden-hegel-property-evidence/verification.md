# Verification

Date: 2026-09-01

## Implemented boundary

- The checked Hegel producer profile has BLAKE3 identity `b3:8739e90ca0707af37eae8553c3b792ec747b0668cad8c4e7e6b0b7c7a5e00e17`.
- The checked law catalog has BLAKE3 identity `b3:33710ac60efb4a7edca25de746d9740d75233b372d0016f54a2e138ec1a038eb`.
- The profile pins Hegel `0.14.27`, the native backend, one Cargo package, one library-test target, and fixed Cargo arguments.
- Run mode rejects `--command`, external reports, parent artifacts, unsafe paths, and symbolic-link substitutions.
- The shell clears the child environment and uses bounded process-group execution.
- Nix supplies the reviewed Cargo executable, vendor configuration, executable path, and library path.
- The producer writes one `octet-hegel-producer-summary/v2` artifact through a fixed output path.
- Stdout and stderr remain bounded diagnostic logs. Octet does not parse them as evidence.
- The receipt remeasures the profile, Cargo lock, package manifest, selected sources, generator catalog, summary, logs, and counterexample.
- Legacy `hegel-property-summary/v1` import remains available without invented hardened measurements.
- The law catalog records positive and negative generated properties, generated-input benefit, replay, fixture coverage, and an effect-free boundary.
- The selected generated laws exercise issue-code normalization with duplicate, whitespace, order, and Unicode variation.
- All added Rust source files contain fewer than 300 lines.

## Passing evidence

- `nix develop --builders '' -c cargo test -p cargo-octet --lib hegel -- --nocapture` passed 10 tests.
- The Hegel adapter ran 64 derandomized cases for each selected positive and negative property.
- Negative tests cover arbitrary commands, unknown or stale identities, malformed summaries, missing dedicated output, unsafe paths, symbolic links, limits, timeout policy, role promotion, and counterexample substitution.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib evidence_contracts::tests:: -- --nocapture` passed six tests. One regeneration test remained ignored after regeneration.
- `nix develop --builders '' -c cargo clippy -p cargo-octet --all-targets -- -D warnings` passed.
- `nix run --builders '' .#octet-policy-validate` passed.
- `nix build --builders '' .#checks.x86_64-linux.hegel-profiles -L --no-link` passed.
- `nix build --builders '' .#checks.x86_64-linux.cairn-traceability -L --no-link` passed.
- The real bounded producer run passed with the fixed profile and no command input.
- `cargo octet artifact verify` reported a valid bundle with seven artifacts and no diagnostics.

## Inherited blockers

The broad cargo-octet library suite passed 445 tests. It retained two inherited failures and six ignored tests:

- `config::tests::active_architecture_pointer_loads_fresh_policy`
- `config::tests::architecture_pointer_rejects_escape_and_stale_manifest`

Both failures reject the available Nickel evaluator because it is not `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.

The canonical self-check remains an integration failure. The serialized rerun reported an existing compiler internal error through an invalid finding severity:

`domain field severity must be warning or error, got error: internal compiler error`

The full serialized Nix check reached the inherited `warning-clean` failure. Its sandbox omits the generated root `dynamic-evidence-profiles.json` required by an existing cargo-fuzz test include.

The broad native traceability report also remains globally incomplete. It reported 36 referenced requirements from 1,372 accepted requirements. The focused `cairn-traceability` Nix check passed for this change.

This change does not replace a baseline, suppress a finding, weaken a gate, or claim successful inherited checks.

## Non-claims

Hegel checks bounded generated inputs only. It does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety.

The selected law catalog does not represent all Octet cores, Rust inputs, defects, generators, or environments.
