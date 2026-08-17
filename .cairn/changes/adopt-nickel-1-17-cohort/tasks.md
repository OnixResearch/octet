# Tasks: Adopt Nickel 1.17 for checked-path configuration

## Pin and checked-path boundary

- [x] [serial] Add an exact Nickel `1.17.0` source input at commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`. r[octet.nickel_toolchain.cohort]
- [x] [serial] Replace policy, standards, proof, evidence, and developer-tool uses of the older evaluator. r[octet.nickel_toolchain.cohort]
- [x] [serial] Regenerate `flake.lock` only with a Nix lock command. r[octet.nickel_toolchain.cohort]
- [x] [parallel] Add a guard for older, floating, ambient, or mixed evaluators. r[octet.nickel_toolchain.cohort]
- [x] [parallel] Prove evaluator acceptance does not replace Octet checker policy. r[octet.nickel_toolchain.boundary]

## Compatibility and evidence

- [x] [parallel] Run valid lint, standards, proof-profile, import, contract, and evidence fixtures. r[octet.nickel_toolchain.compatibility]
- [x] [parallel] Add disabled, duplicate, empty, warning-mode, malformed, import, contract, and unknown-value negative fixtures. r[octet.nickel_toolchain.compatibility]
- [x] [serial] Record the evaluator identity where Nickel affects checked-path evidence. r[octet.nickel_toolchain.evidence]
- [x] [parallel] Add overclaim and evaluator-identity substitution evidence failures. r[octet.nickel_toolchain.evidence]

## Validation

- [x] [serial] Run policy fixtures, standards checks, proof checks, formatting, Clippy, Cairn gates, and relevant Nix checks. r[octet.nickel_toolchain.validation]
