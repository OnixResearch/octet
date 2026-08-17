# Nickel 1.17 checked-path evidence

## Result

Octet now supplies Nickel CLI `1.17.0` from `nickel-lang/nickel` revision `1320a983e6c3d1e2fb53dd2464b084b4903b1426` to policy, standards, proof, evidence, fixture, and developer-tool commands.

Nix generated the lockfile update. No ambient `pkgs.nickel` reference remains in `flake.nix`.

## Cohort and boundary checks

`checks.x86_64-linux.nickel-cohort-exact` passed. It checks the source revision, resolved executable, full version output, and absence of ambient package-set references.

`octet-standards` now admits checked-path export evidence only for identity:

```text
nickel-lang-cli nickel 1.17.0 (rev 1320a98)
```

A substituted or mixed identity fails before a manifest is accepted. A valid evaluator identity does not make an unsupported Octet format or policy valid.

The focused `octet-standards` suite passed eight positive and negative Nickel Export tests. Focused Clippy passed with warnings denied.

## Compatibility matrix

`checks.x86_64-linux.nickel-toolchain-fixtures` passed.

Positive fixtures cover lint policy, standards, proof profiles, imports, contracts, and evidence. Negative fixtures cover disabled, duplicate, empty, warning-only, malformed, missing-import, contract-violation, unknown-value, overclaim, and evaluator-substitution cases.

The following checked-path Nix checks passed under Nickel 1.17:

- policy registry
- proof-rail manifest
- Verus toolchain profile
- strict-profile catalog
- nominal-domain catalog
- Clippy restriction profiles
- effect-closure catalog
- ast-grep structural rail
- Wasm artifact rail
- project architecture policy
- OnixOS lifecycle assurance profile

`nix run path:$PWD#octet-policy-validate` passed.

## Evidence identity

The repository generator refreshed these manifests:

- `standards/project-architecture/octet-self-architecture.manifest.json`
- `standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.manifest.json`

Only their evaluator identity changed from Nickel 1.16 to Nickel 1.17. Source, dependency, output, command, and BLAKE3 fields remained stable.

## Lifecycle and bounded blocker

Cairn validation, proposal gate, and design gate passed.

`nix flake check --no-build -L` evaluated the main package set until the existing Nix cache/store state reported this exact blocker:

```text
path '/nix/store/4n8cwiv8swsi3hascvqwf8jrp82fwk2k-octet-standards-0.1.0.drv' is not valid
```

The focused checks that build and consume `octet-standards` passed from fresh derivations. The stale store-path blocker is not treated as Nickel acceptance evidence.

## Non-claims

Nickel acceptance proves only tested configuration shape and evaluator identity.

It does not replace Octet checker policy, prove Rust correctness, prove proof-rail claims, establish evidence truth, or decide release eligibility.
