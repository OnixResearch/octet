# Verification

Date: 2026-09-01

## Implemented boundary

- The checked consumer profile has identity `b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879`.
- The profile binds provider identity `b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625`.
- It also binds compatibility-matrix identity `b3:403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5`.
- `load_profile` measures exact external profile bytes before validation.
- The checked fixture selects `external_consumer::increment` from one exact source file.
- Its source identity is `b3:691e8de8b8f92665dd00b938b48b398b451c71a06c99dae10b38b38184d4dddd`.
- Its maintained theorem identity is `b3:776e9f85261080fbcfd75cb481c4f563cff50ffefaf8bdaae3d5fad010d8c77f`.
- Durable selected-item identity excludes transient Charon `def_id` values.
- The pure core validates source bytes, spans, LLBC, translation rows, generated Lean, theorem mappings, resources, trust inputs, and non-claims.
- The outcome core keeps `supported`, `blocked`, and `failed` distinct.
- `cargo octet prove` activates the binding only for the exact consumer policy and theorem.
- The formal envelope requires the consumer binding for that policy.
- Consumer stage identities must match exact provider-stage roles and bytes.
- Consumer resources must match the bounded provider execution policy.
- Existing Aeneas and Verus envelopes omit the optional consumer field and retain their prior serialization.
- All added Rust source files contain fewer than 300 lines.

## Passing evidence

- `nix develop --builders '' -c cargo test -p octet-standards --lib aeneas_consumer:: -- --test-threads 1` passed 12 tests.
- `nix develop --builders '' -c cargo test -p octet-standards --lib formal_proof:: -- --test-threads 1` passed 22 tests.
- `nix develop --builders '' -c cargo test -p cargo-octet --lib prove::tests:: -- --test-threads 1` passed 17 tests; one packaged-provider test remained ignored in this command.
- Tests cover valid measured downstream profiles, exact provider artifacts, transient `def_id` changes, profile drift, source drift, ambiguity, blocked and failed outcomes, missing blockers, receipt tampering, resource drift, stage-role drift, policy substitution, missing consumer bindings, and unknown fields.
- `nix develop --builders '' -c cargo clippy -p octet-standards -p cargo-octet --lib -- -D warnings` passed.
- `nixfmt --check flake.nix` passed.
- `git diff --cached --check` passed.
- `nix run --builders '' .#octet-policy-validate` passed.
- `nix build --builders '' .#checks.x86_64-linux.aeneas-consumer-contract --no-link -L` passed.
- `nix build --builders '' .#checks.x86_64-linux.cairn-traceability --no-link -L` passed.
- The proof-rail manifest check accepted the positive profile and rejected all six invalid Nickel profiles.
- Cairn proposal, design, and tasks gates passed with the canonical workspace policy.
- Cairn review completeness passed with no findings.
- Strict Cairn validation passed.
- The generated guard resolver returned metadata for `scripts/check-cairn-task-evidence.rs`.
- That legacy script is absent from the current Cairn checkout, so no standalone script result is claimed.
- The current native tasks gate and review-completeness command supplied the executable task evidence.

## Live provider and artifact evidence

The exact external consumer rail ran through the packaged Charon, Aeneas, and Lean provider closure. The proof completed with one passing rail and no diagnostics.

`cargo octet artifact verify` accepted the complete bundle. The verified identities included:

- LLBC: `a4d7eb6892d949ad0b5c50bb1a5a0d54deb7a81732a6d7e760326db6b49cd7b2`
- translation manifest: `ac9089c17b063507da0515308a099eafe255e22605c0d9e633d6bb89b23b0abb`
- generated Lean: `417f6e10736a1267c42deac018402ee17384d0a433d35c524e11372c3be5beb3`
- consumer formal-proof receipt: `38a31cc536604a75d572c007743ed8103c9bd9b6b688556fb9f7de6b151e08d7`

The receipt records outcome `supported`, requirement `octet.aeneas_consumer_contract.compatibility`, and a typed consumer binding. Valence provenance export succeeded after artifact verification and retained the consumer field.

## Inherited blockers

The broad cargo-octet library suite retained two existing Nickel evaluator failures. It passed 459 tests and kept seven ignored tests:

- `config::tests::active_architecture_pointer_loads_fresh_policy`
- `config::tests::architecture_pointer_rejects_escape_and_stale_manifest`

Both failures reject the available Nickel evaluator because it is not `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.

The broad Aeneas compatibility matrix reached its existing live-check compilation blocker. The pure compatibility derivation passed 25 tests. The live derivation could not compile because its sandbox omits the generated root `dynamic-evidence-profiles.json` required by an existing cargo-fuzz test include.

The live external consumer proof was therefore run directly with the same packaged provider closure. It passed and its artifact and Valence exports validated. This change does not hide the inherited Nix blocker, replace a baseline, suppress a finding, or weaken a gate.

## Non-claims

The consumer theorem proves only its named statement over exact bound artifacts. This work does not prove Aeneas or Charon translation soundness, Lean or verifier soundness, semantic equivalence, whole-program correctness, release eligibility, or support for arbitrary consumers.

Eurydice and Scylla remain deferred references. Trellis proof evidence requires an Octet rerun before it can gain this consumer rail's formal-proof role.
