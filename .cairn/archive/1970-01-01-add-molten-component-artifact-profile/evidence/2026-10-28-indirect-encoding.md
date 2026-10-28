# Padded indirect-call admission: 2026-10-28

## Decision and consumer

Mantle's actual Molten supported-world output failed the first published profile
with `wasmparser validation failed: zero byte expected (at offset 0x491)`.
The bytes contain the LLVM linker's overlong LEB zero `call_indirect` table
immediate. Enable only wasmparser 0.253.0's `CALL_INDIRECT_OVERLONG` feature via
an explicit `call-indirect-overlong` token. Reference types, SIMD and all-Wasm-2
admission remain out of scope. The import-free portable profile is unchanged.

Immediate outcome: Octet admits the exact reviewed guest under the explicit
Molten profile. Durable contribution: a granular reviewed encoding token with
an executed positive/negative byte-validation regression. Maintenance owner:
Octet owns proposal mapping/profile policy; Mantle owns cohort pinning and
producer selection. Mantle must refresh its immutable Octet source pin,
registry/profile identities and producer evidence together.

## Executed checks

All commands ran from the isolated Octet repository root. Cargo ran in the
repository Nix shell with a private target directory. Pueue task 31 built the
Nix package; task 33 ran the library tests and quality/freshness checks.

- `nix develop -c cargo test -p cargo-octet --lib wasm_artifact::tests`:
  **15 passed, 0 failed, 507 filtered out**. Includes exact bulk-memory and
  padded-call validation; denial without the encoding feature; reference-type,
  SIMD, portable-policy and unknown-token denials.
- `nix develop -c cargo clippy -p cargo-octet --all-targets -- -D warnings`:
  exit 0.
- `nix develop -c rustfmt --check --edition 2024 cargo-octet/src/wasm_artifact.rs`:
  exit 0.
- `nix build .#checks.x86_64-linux.wasm-artifact-rail --no-link`: exit 0,
  including Nickel projection freshness and existing SpaceWasm corpus checks.
- `nix build .#cargo-octet --no-link --print-out-paths`: exit 0; packaged CLI
  `/nix/store/hsfbi59b2dznmfh9slg9gngql20xn1wq-cargo-octet-0.1.0/bin/cargo-octet`.

The earlier `--bin cargo-octet wasm_artifact::tests` invocation executed **zero
tests**. It is not test evidence; the explicit `--lib` run above supersedes it.

Package-wide `cargo fmt --check -p cargo-octet` exited 1 only for unrelated
existing formatting in `cargo-octet/tests/fingerprint_consumer.rs`. That file
was not modified. This supplementary check is accepted as non-blocking for
this narrow change; the changed source passes its exact-file formatter check.

## Actual artifact smoke

Used the packaged CLI above against the exact retained supported-world output:

```sh
cargo-octet evidence collect --rail wasm-artifact \
  --input "$NORMALIZED_COMPONENT" \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile molten-component-baseline --artifact-dir "$POSITIVE" \
  --output-format json
cargo-octet artifact verify --artifact-dir "$POSITIVE" --output-format json
cargo-octet evidence collect --rail wasm-artifact \
  --input "$NORMALIZED_COMPONENT" \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile portable-component-baseline --artifact-dir "$NEGATIVE" \
  --output-format json
```

Here `NORMALIZED_COMPONENT` was the actual Mantle retained
`supported-selected/producer-output/artifacts/normalized.wasm` under the
2026-10-28 profile-selection run directory. Observed identities and outcomes:

- Exact Wasm: `e3e1c38cb8bbc4c9866f1791f1f7f10cb48f8de2db10eb8a66de4eada9f49199`.
- Registry: `48aabf8dc2ad189ebe051ddce48746a8f878fcbcebf506007c505085e9a8ed83`.
- Molten profile: `24dc3346b817ee2645b000922a612a1988f63914b49688a8b72ebb758316c846`.
- Portable profile: `e8fbf118b283d63a3662af4853a2ce5f4a1777c18fe37e0be0dc1e235fd707bd`.
- Molten: exit 0, `status=passed`, no findings.
- Positive bundle verification: exit 0, `status=valid`, no diagnostics.
- Portable: expected exit 1, `status=failed`, two disallowed-import findings
  and one disallowed-proposal finding (`bulk memory support is not enabled`).

## Lifecycle boundary

The existing change predates explicit workflow metadata. The pinned Cairn
provider refuses late workflow selection with
`cannot add or change a workflow profile after substantive authoring`; no
metadata was fabricated and no guard was bypassed. Its existing native
compatibility proposal gate passes under `octet-default`.

## Non-claims

Static artifact validation is not execution correctness, capability authority,
provenance truth, release eligibility or whole-system safety. This direct smoke
is not the refreshed Mantle end-to-end producer/consumer run. Earlier failed
attempts and the earlier acquisition blocker remain historical observations,
not successful evidence.
