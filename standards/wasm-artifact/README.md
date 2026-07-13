# Wasm/component artifact rail

Octet validates exact WebAssembly module or component bytes without executing them. The rail owns static fact extraction and policy decisions; Mantle and runtime consumers retain build, sandbox, capability, execution, and release authority.

## Profile and cohort

`profiles.ncl` is the typed source of truth and `generated/profiles.json` is the checked runtime input. The profile binds the reviewed wasm-tools `0.253.0` cohort with a BLAKE3 identity. Cohort changes require replay of the positive and negative corpus.

## Standalone invocation

```console
cargo octet evidence collect \
  --rail wasm-artifact \
  --input module.wasm \
  --config standards/wasm-artifact/generated/profiles.json \
  --profile portable-component-baseline \
  --artifact-dir artifacts/wasm-artifact
cargo octet artifact verify --artifact-dir artifacts/wasm-artifact
```

The bundle includes `wasm-artifact.bin`, exact-byte and normalized-interface identities, extracted facts, ordered JSON findings, a SARIF projection, profile/cohort identities, provenance, and explicit non-claims.

## Mantle handoff

Mantle supplies the exact final bytes through the same command and may add a `mantle:` parent artifact identity. Octet records a `mantle-exact-byte` handoff role but does not trust store paths, build success, runtime reports, or release decisions as static artifact facts.

## Claim boundary

A passing report establishes only bounded syntax, profile, identity, and source-shape facts. It does not prove execution correctness, semantic equivalence, sandbox effectiveness, capability authority, provenance truth, release eligibility, or whole-system safety.
