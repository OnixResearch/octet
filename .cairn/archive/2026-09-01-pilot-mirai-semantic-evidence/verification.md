# Verification

Date: 2026-09-01

## Implemented boundary

- The policy identity is `b3:cd4120f1df6d9715a615b87bbce4cbf41cdb3e0abd0a40dfd3ae175dc085b1ca`.
- The corpus identity is `b3:1dfb3e7ddeb1bc470d0e17e0a0b1c6b2895895643fa7c98a7f529a20ee99b015`.
- The decision receipt identity is `b3:59fa56f8a28ff9c8831b45f524c172e798bb1c48292d05acfbee9e76564e8fc4`.
- The provider source is `endorlabs/MIRAI` revision `aae0a56f91d9a0a62073bb86e7b2c9793cefc24b` under the MIT license.
- The provider record pins MIRAI `1.1.12`, Rust `nightly-2025-01-10`, target `x86_64-unknown-linux-gnu`, and feature `bundled`.
- Nix source acquisition uses `sha256-+H0JIbrwxMi3uMuxBqgVBzNkDYom8X616rH9y7YqIXg=` for Nix interoperability.
- The built provider path was `/nix/store/9kplzjdxxmmkfkhs3k8h72fa6p2v6anf-octet-mirai-source-provider-1.1.12`.
- Its derivation was `/nix/store/r6ikfpkrvzl1q4nj9b7yaf40flchkydp-octet-mirai-source-provider-1.1.12.drv`.
- Its only runtime reference was `/nix/store/cnmqcmwpz02m3xiaai883kw7fz2iibrs-endorlabs-mirai-source-aae0a56f91d9a0a62073bb86e7b2c9793cefc24b`.
- The selected corpus binds eight source files by relative path, byte count, and BLAKE3 identity.
- Five semantic classes have supported review oracles. One macro fixture remains explicitly unsupported.
- The fixed command, environment, timeout, log limit, source closure, and no-network posture are typed policy facts.
- The provider shell has no arbitrary command input. It blocks before spawn because a stable machine output schema is unavailable.
- The pure comparison core preserves Charon and source-token parent identities across six declared outcomes.
- The deterministic receipt reports `blocked`, retains all unavailable criteria, and claims no MIRAI semantic evidence.
- No starter, strict-profile, Charon, Aeneas, Kani, Verus, or release path selects MIRAI.
- Every added Rust source file contains fewer than 300 lines.

## Passing evidence

- `nix develop --builders '' -c cargo test -p cargo-octet --lib mirai_pilot:: -- --test-threads 1` passed 14 tests.
- Positive tests cover policy, source identity, corpus oracles, fixed planning, all comparison outcomes, replay, and blocked decision identity.
- Negative tests cover malformed receipts, unsupported syntax, provider drift, source drift, limit drift, timeout substitution, missing parents, and overclaims.
- The checked decision artifact replayed byte-for-byte and passed its generated Nickel contract.
- The invalid `adopt` fixture failed the generated blocked-decision contract.
- `evidence_contracts::tests::generated_contracts_are_fresh` passed after contract regeneration.
- `evidence_contracts::tests::evidence_contract_fixtures_accept_valid_and_reject_invalid_records` passed.
- `nix develop --builders '' -c cargo clippy -p cargo-octet --lib --tests -- -D warnings` passed.
- `nix build --builders '' .#mirai-source-provider -L` passed.
- `nix build --builders '' .#checks.x86_64-linux.mirai-semantic-pilot -L --no-link` passed.
- `nix run --builders '' .#octet-policy-validate` passed.
- `nix build --builders '' .#checks.x86_64-linux.cairn-traceability -L --no-link` passed.
- Cairn strict validation and the proposal, design, and tasks gates passed with the canonical workspace policy.

## Inherited blockers

The broad cargo-octet library suite passed 458 tests. It retained two inherited failures and six ignored tests:

- `config::tests::active_architecture_pointer_loads_fresh_policy`
- `config::tests::architecture_pointer_rejects_escape_and_stale_manifest`

Both failures reject the available Nickel evaluator because it is not `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.

The inherited canonical self-check and broad `warning-clean` blockers remain unchanged from the preceding accepted Octet changes.
This change does not replace a baseline, suppress a finding, weaken a gate, or claim those checks passed.

## Non-claims

The source package does not prove a successful MIRAI build or execution.
The blocked receipt does not contain MIRAI semantic evidence.
The pilot does not prove compiler correctness, verifier soundness, whole-program correctness, semantic equivalence, or release eligibility.
