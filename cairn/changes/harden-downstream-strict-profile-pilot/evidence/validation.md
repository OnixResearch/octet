# Downstream strict-profile hardening validation

Validation date: 2026-07-11

## Reproduced downstream blockers

The Valence pilot first ran the packaged command outside Valence's development shell:

```bash
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- check \
  --function-addresses observe \
  --function-address-path crates/valence-core/src/shared_identity.rs \
  --artifact-dir target/octet-pilot-observe
```

Before the runtime-closure fix, `serde_derive` compilation exited 101 because the package did not expose a linker. Octet recorded `integration-failure`, one unclassified compiler finding, and no function-address artifact. Running the same package from `nix develop` supplied the missing ambient linker and reached lint execution, confirming packaging rather than Valence source was the boundary.

The first focused `deterministic-core` run then reported 63 deny findings in the shared-identity source; 58 were `concrete_construction_in_core`. They repeated one `SharedIdentityIssue::new()` across parent expression spans and classified local DTO construction as external despite the accepted lint contract.

## Focused implementation checks

| Command | Outcome |
| --- | --- |
| `nix develop -c cargo test --lib structure::concrete_construction_in_core::tests` | PASS; three pure classifier tests cover external emission plus local/standard, indirect-parent, and out-of-scope rejection. |
| `nix develop -c env BLESS=1 cargo test --test ui ui -- --ignored --nocapture` | PASS; the complete UI corpus passed. The focused snapshot now contains exactly one external `serde_json::Map::new()` diagnostic and no local `DomainValue::new()`, standard `Vec::new()`, or parent-expression duplicate. |
| `nix develop -c cargo fmt --check` | PASS. |
| `nix develop -c cargo clippy -p octet --lib -- -D warnings` | PASS. |
| `nix build .#checks.x86_64-linux.downstream-proc-macro --no-link -L` | PASS; standalone packaged Cargo compiled a Serde derive consumer, ran the robust profile/function-address rail, and verified the bundle without a consumer dev shell. |
| `nix build .#checks.x86_64-linux.strict-profile-catalog --no-link -L` | PASS. |
| `nix run .#octet-policy-validate` | PASS. |
| `nix build .#cargo-octet --no-link -L` | PASS. |

## Real Valence replay

Using the fixed standalone package, Valence ran:

```bash
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- check \
  --profile deterministic-core \
  --function-address-baseline evidence/octet/deterministic-core/baseline/function-addresses.json \
  --artifact-dir evidence/octet/deterministic-core/admission
nix run path:/home/brittonr/git/OnixResearch/octet#cargo-octet -- artifact verify \
  --artifact-dir evidence/octet/deterministic-core/admission \
  --output-format json
```

Observed result:

- Cargo and proc-macro dependencies linked without Valence's dev shell.
- Status was `warning-only`, exit code zero, with eight non-required advisory findings and zero errors.
- The selected profile was `deterministic-core` with configuration identity `b3:c26d00917e4feb3e9a7f8307222758fc925b98cda6bc7e088781815bcad4c646` and profile identity `b3:5ef5d578922af5664cdfaa792b0a4db164dbdbdf573d2b40bb647f5088f87657`.
- Deny-mode function-address evidence contained 16 supported records across two production source envelopes, zero diagnostics, and record-root identity `b3:68ef940eef5a36448ec6292468e50807e7dd7ada1b6769d4ca21d8dbbd10a869`.
- Artifact verification returned `status: valid` with no diagnostics.
- A second identical command produced byte-identical `status.json` and `function-addresses.json`.
- A temporary source change to `blake3_hex` against the unchanged baseline exited one with `lint-failure` and `function_address_stale` diagnostics.

## Existing broad-test blocker

`nix develop -c cargo test --lib` currently reports 117 passing and eight failing snapshot/catalog tests. Every failure is pre-existing drift for the independently added `state_machine_exhaustiveness` lint: missing inventory/default/group/alias/guardrail/emitting-fixture rows plus stale README/reference counts. This change does not modify that lint or its snapshot artifacts. The focused classifier tests, complete UI suite, Clippy target, Nix package smoke, and real downstream replay pass.

## Claim boundary

These checks establish packaged linker availability, direct HIR constructor-classification precision, profile execution, source-token function identity, deterministic local replay for selected artifacts, and artifact hash integrity only. They do not establish transitive effect absence, dependency closure, behavioral correctness, semantic equivalence, compiler correctness, verifier soundness, whole-program correctness, cross-machine reproducibility, or release eligibility.
