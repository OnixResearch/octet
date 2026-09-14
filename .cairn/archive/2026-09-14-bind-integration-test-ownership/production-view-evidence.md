# Production-view evidence

## Status and limits

I3 and the bounded V3 controls are complete. V4, H1, and H2 remain open for final whole-change review and evidence checks.
No sync, archive, commit, push, or Noble repin occurred. Noble M1 remains 2/21.
These results are local observations, not independent acceptance or release eligibility.

Evidence root: `/home/brittonr/git/noble/.pi/m1-test-ownership/production-view/`.
Earlier membership and role-binding records remain historical evidence.

## Implementation

- `ProductionView` admits only validated production observations. It has no normalized IR identity or schema field.
- Membership validation precedes interpretation. Shared production facts and unknowns remain visible.
- `verify_observation_set` checks fact, span, and unknown identities and references before and after partitioning.
- Missing test-only reference targets cannot turn production checks into silent skips.
- Effect and construction checks visit every observing production unit. No-std evidence is required per production unit.
- Project receipt schema and identity domain are v2. The Rust generator owns the updated Nickel contract.
- The required `test_only` field retains test unit IDs and exclusively test-observed fact and unknown IDs.
- Full-input identities remain unchanged. Input protocol IDs do not imply production ownership.
- Full bundle replay rejects altered dispositions after receipt, phase-status, and provenance hashes are refreshed.
- Registry fixtures cover positive partitioning, fake production evidence, source imports, and replay rejection.

## Round one: typed production evaluation

Task 2109 passed the scoped baseline. Task 2114 retained the expected red integration-role control.
Task 2135 failed compilation because the new call used `architecture_ir_identity` instead of the public `ir_identity` function.
Task 2136 passed 36 behavior tests but rejected the stale generated contract.
Task 2138 passed all 37 gate tests after regeneration, then rejected a wildcard import under strict Clippy.
Task 2145 retained a fixture compilation error: the normalization result required explicit error handling.
These errors were corrected without lint allowances or weaker checks.

Task 2148 passed the first broad package cohort and strict Clippy.
Task 2150 repeated that cohort after observation integrity checks were extended to spans and unknowns.
Task 2153 built the candidate runner and passed Nix formatting, project architecture policy, and standards fixture checks.

## Round two: compiler source origin

Task 2160 passed the two existing real compiler runs, but failed the new production-import assertion.
A parent-relative `#[path]` import reached inventory findings instead of a source-role integrity failure.
The collector's lexical path filter discarded paths containing `..` and classified them as generated source.

The collector now resolves physical paths against the selected context's workspace root.
Parent-relative imports retain their physical test source role. The resolver does not lexically collapse symlink escapes.
Task 2170 rebuilt the collector and runner and passed the same three Nix checks.

Task 2175 passed three compiler runs. Its symlink fixture failed an assertion intended for the later root-binding phase.
The source envelope already rejects source symlinks before collector context publication.
The fixture now checks that earlier rejection explicitly; it does not claim compiler execution for the symlink case.

Task 2178 passed four explicit controls and strict Clippy for all four affected packages:

| Control | Result |
|---|---|
| Explicit features, all targets | 10 compiler units; production roles retained |
| All features, all targets | 10 compiler units; production roles retained |
| Production import of test-labelled source | Rejected in lib, lib+test, bin, and bin+test; all 10 units covered |
| Source symlink alias | Rejected by the source envelope before compiler invocation |

Each compiler run also rejects a missing shard. Library-test effects remain production findings.
These compiler fixtures do not run the consumer deny-all hook; they do not establish a clean full-catalog lint result.

## Package verification

Task 2171 passed 33 Architecture IR tests, 185 standards unit tests, 24 standards integration tests, and 474 runner library tests.
Strict all-target/all-feature Clippy passed for Architecture IR, standards, runner, and the collector package with `-D warnings`.
Task 2176 passed 150 collector-library unit tests.
With the four explicit controls, the bounded cohort contains 870 passed tests.
The default runner suite ignored 11 tests; four were run explicitly. UI targets compiled under Clippy, but UI execution is not claimed.
Final whole-change checks remain open.

## Unchanged Noble probes

Tasks 2161 and 2179 ran the candidate deny-all hook with `--workspace --all-targets --all-features`.
Both retained all six units and had clean lint results.
The two independent test units have explicit dispositions. Production units keep their roles, including library and binary test configurations.
All five retained unknowns are test-only; production unknowns were not waived or hidden.

Both architecture evaluations are blocked by one `no-std-obligation` finding for `noble-kernel`.
The compiler IR contains no `crate:no-std` configuration fact. The cause remains unconfirmed.
Neither role-conflict findings nor the phase-two pending-evaluation rejection remain in these candidates.

Task 2181 verified both full artifact bundles as structurally valid failure bundles.
The latest receipt is `7ac537ca6e6b2f8afbd00997d1b922b1b70de00e750b9fc6498bee1594f76bfc`.
A valid bundle is not architecture acceptance. Noble still consumes published revision `74a5d6b057272b149ae86049107cba5eac986b28`.

## Next checks

1. Finish whole-change UI, policy export, and lifecycle evidence review before publication.
2. Investigate the missing compiler-derived no-std fact without weakening Noble's obligation or inventing evidence.
3. Publish verified provider work before repinning either Noble consumer.
4. Repeat Noble's full scope, extraction inventory, Miri, and independent acceptance controls.
