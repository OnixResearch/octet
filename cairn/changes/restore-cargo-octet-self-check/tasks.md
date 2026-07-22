## Implementation

- [ ] [serial] I1 Add exact canonical root runner metadata and a deterministic checked-manifest regression test. r[octet.runner_self_check.metadata] r[octet.runner_self_check.metadata.scoped]
- [ ] [depends:I1] I2 Add a hermetic Nix self-check gate with packaged positive execution and malformed-metadata pre-Cargo refusal. r[octet.runner_self_check.execution] r[octet.runner_self_check.execution.pass] r[octet.runner_self_check.failure] r[octet.runner_self_check.failure.malformed]
- [ ] [depends:I2] I3 Register accepted traceability and document the bounded checked-path claim without weakening consumer policy. r[octet.runner_self_check.boundary] r[octet.runner_self_check.boundary.non_claims]

## Validation

- [ ] [depends:I3] V1 Run focused root-manifest, metadata-precedence, missing/malformed metadata, formatting, and Clippy checks. r[octet.runner_self_check.metadata.scoped] r[octet.runner_self_check.failure.malformed]
- [ ] [depends:V1] V2 Run the exact canonical command and dedicated Nix gate, retaining positive summary and malformed-negative evidence. r[octet.runner_self_check.execution.pass] r[octet.runner_self_check.failure.malformed]
- [ ] [depends:V2] V3 Run Nextest, the broad flake gate, native Cairn gates, accepted-spec sync, policy freshness, and traceability while reporting inherited baselines separately. r[octet.runner_self_check.boundary] r[octet.runner_self_check.boundary.non_claims]
- [ ] [depends:V3] V4 Prepare deterministic archive execution and record post-archive validation criteria. r[octet.runner_self_check.boundary]
