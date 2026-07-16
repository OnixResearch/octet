## Why

Octet already exposes generic proof-rail selection, but its production receipt, package-lock, verifier, solver, and toolchain contracts are Verus-specific. Aeneas has a materially different proof path: Rust is compiled by Charon into LLBC, Aeneas translates that LLBC into Lean, and Lean checks a named theorem over the generated model. Treating this as one opaque command would leave source-to-LLBC, LLBC-to-Lean, external-model, and theorem-presence drift outside the receipt.

A bounded Aeneas/Lean provider would complement Verus for safe deterministic Rust cores and give Octet a second proof mechanism without making either verifier an oracle for the other. The first slice must prove the integration itself before any stack consumer relies on it.

## What Changes

- Add an opt-in `aeneas-lean` formal-proof provider under `cargo octet prove` while leaving `cargo octet check` and existing Verus rails unchanged.
- Define a typed Nickel provider profile that pins Aeneas, Aeneas's required Charon revision, OCaml, Lean, the Aeneas Lean library, command argv, limits, environment, supported Rust surface, mandatory non-claims, and the exact effective Charon extraction contract.
- Introduce a Rust-owned common formal-proof receipt envelope with provider-specific payloads, preserving the existing Verus receipt through an explicit compatibility adapter.
- Bind every Aeneas stage: checked Rust source, effective Charon options and target metadata, LLBC bytes, Aeneas `translation.json`, generated Lean bytes, hand-maintained theorem source, imported external models, Lean package/toolchain identity, theorem-adequacy decision, and final checker result.
- Treat `translation.json` declaration IDs as within-run join keys only; compute durable stack identity from BLAKE3-bound source bytes/spans, reviewed toolchain profile, generated model, and theorem source.
- Require an approved theorem-statement hash, requirement ID, explicit claim class, complete trust closure, and rejection of vacuous property promotion before `formal_proof` evidence is eligible.
- Record an exact `mechanized-llbc` revision as a semantic-foundation reference only, without transferring proof or implementation-conformance claims.
- Add an Octet-owned tiny safe-Rust positive fixture plus negative fixtures for a false theorem, a vacuous `True` theorem, stale intermediate output, extraction-policy weakening, pin drift, missing model/assumption declarations, unsupported Rust surface, and claim promotion.
- Export a frozen Aeneas receipt fixture for Valence only after Octet artifact verification succeeds.

## Impact

- **Proof core**: proof-provider models, package-lock/receipt validation, proof cards, artifact verification, and Valence export fixtures.
- **Shell**: argv-only Charon, Aeneas, and Lean execution with bounded logs, timeouts, environment allowlists, and deterministic working roots.
- **Policy and packaging**: typed Nickel provider profile, generated runtime data, and pinned Nix toolchain packages.
- **Compatibility**: existing Verus v1 artifacts remain accepted through a named adapter; no silent schema reinterpretation is allowed.
- **Testing**: positive and negative provider fixtures, extraction-contract rejection, manifest/link tampering, theorem-adequacy failures, fixture-driven pin-upgrade canaries, ordinary starter-path independence, focused proof runs, policy freshness, artifact verification, and Cairn gates.
- **Claims**: a passing receipt supports only the named Lean theorem over the exact generated model and bound source/toolchain chain. It does not prove Aeneas, Charon, Lean, rustc, external models, semantic equivalence, whole-program correctness, unsafe Rust, concurrency, or release eligibility.
