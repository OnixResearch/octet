# Compiler randomness effect evidence

## Scope and source

Base: `e3e705a7c04a30b814000b251d245cf083894e4a`.
Candidate build tree: `b2b28e65db08724fab9848cc55e36f3dd7596a7b`.
Final verification tree: `86b69ae760b42ceddb50b8011ee22e69f7538942`.
Persistent evidence: `/home/brittonr/git/noble/.pi/m1-random-effects/`.
Full Pueue commands, actual statuses, source archives, logs, raw observations, and bundles belong there, outside the disposable worktree.

The compiler shell supplies defining-crate, item-name, and root-function facts to the pure randomness classifier.
The four selected APIs are `getrandom::{fill,fill_uninit,u32,u64}`. Similar names and non-root functions do not qualify.
Recorded diagnostic paths remain unchanged. Effects preserve their corresponding calls' callers, spans, stable identities, and production observers.
Existing standard-library classification remains unchanged. No schema, role, target, lint level, exemption, or production dependency changes.

The fixture manifest and Cargo-generated lock select real registry `getrandom 0.4.3`.
Its dependencies match the existing provider vendor closure: `cfg-if 1.0.4`, `libc 0.2.186`, and `r-efi 6.0.0`.
No local same-named implementation replaces that registry crate. The separate local helper is a negative resolution control only.

## Baseline and causal failures

- 2497: all 152 pre-change collector tests passed.
- 2505: unit red returned no randomness classification. One classifier test failed; the name-negative test passed.
- 2505: the initial compiler oracle expected `getrandom::fill`, but the recorded alias path differed. This was an oracle failure, not accepted enforcement.
- 2513: bounded diagnostics recorded `randomness::entropy::{fill,fill_uninit,u32,u64}` and the separate local helper.
- 2516: the corrected oracle retained those paths. Both feature invocation forms compiled, then failed on the missing randomness effect.

These failures remain in `unit-red.log`, `compiler-red.log`, `compiler-diagnostic.log`, and `compiler-effect-red.log`.
The fix uses compiler ownership rather than rewriting diagnostic paths or assuming an alias spelling identifies the defining crate.

## Passing checks

- 2520: all 154 collector tests passed; the candidate package built.
- 2529: thirteen explicit compiler controls passed; strict four-package all-target/all-feature Clippy passed.
- 2540: 33 IR tests, 185 standards unit tests, 24 integration tests, 474 runner tests, and 154 collector tests passed. Strict Clippy passed again.
- 2541: package build and five Nix checks passed: formatting, project-architecture policy, standards fixtures, UI, and compiler architecture targets.
- 2541: four UI harness tests and 32 overlapping compiler-target tests passed. Five policy exports and their artifact manifest completed.
- 2558: tested and final-built collector bytes matched. Both retained Noble bundles passed full replay with the final runner.

The scoped total is 883 tests, including thirteen explicit controls. Repeated runs and the 32 overlapping Nix tests do not increase it.
The runner's normal suite ignores twenty controls. Thirteen ran explicitly; seven unrelated controls remain outside this scope.
The thirteen controls contain twelve compiler runs with ten selected units each and one precollection symlink rejection.

Both new compiler controls require all four real calls and matching effects in `lib` and `lib+test`.
Each effect must retain its call's owner and span, the owner's same-unit membership, and the production gate finding.
The local helper remains visible without a randomness effect. Complete coverage and omitted-shard rejection remain enforced.
Compiler controls use typed Rust test policies and a warning-only lint lane. They are not Nickel-policy or published deny-all acceptance.

## Unchanged Noble candidate probes

Noble stayed at tree `66c8444496533fa482a2e94ddbfc94811f025843` throughout the candidate probes.
Its pins remain at the published base revision. Selection preflight ran before explicit candidate-tool selection.

2537 reused the failed random-call fixture without source, dependency, policy, or comparator changes.
The unchanged published full deny-all hook exited 2 with the intended `ambient_random` diagnostic.
The separate auxiliary run exited 1 after successful compilation and a blocked architecture phase.
Both kernel production observers retain the required randomness effect. Full replay is valid.
The unchanged comparator reports `boundary control randomness: passed (not M1 acceptance)`.
Raw `.octet` observations were archived before replay.

2543 checked the actual Noble workspace with the same candidate. Full deny-all was clean with zero findings and valid full replay.
The prior and new raw observations were retained. Before/after staged trees match for both probes.

Tested and final collector BLAKE3:
`f6d3a9001ed7d835581b33a3b6713a69ac4519cbb91f6e75ceaadf3d9a88c866`.
The final runner replay results are `final-baseline-replay.json` and `final-random-replay.json`.

## Lifecycle and limits

2499 and 2561 completed their shell commands, but their tasks receipts failed on five missing concurrency markers.
The initial command completion was incorrectly treated as a passing tasks gate. Receipt inspection during sync review exposed the mistake.
The blocked sync plan was not executed. Explicit serial markers fixed the task metadata, and 2563 reran validation and all planning gates successfully.
The original failed receipts remain retained. No policy was relaxed.
2568 synced the spec, then failed traceability under the generic `cairn-default` policy selected by Noble's wrapper: 49/1403 references.
2576 confirmed that Noble's Cairn binary cannot parse Octet's existing task-marker policy. Neither failure is accepted traceability evidence.
Octet instead selects Cairn `695124d459574ba7aeba6097310d237f393c243c` through its unchanged flake lock.
Provider lifecycle commands must use that tool and Octet's unchanged `cairn-policy/generated/cairn-policy.json`, with an explicit worktree root.
Noble's wrapper remains appropriate for Noble only. No policy file or source selection was edited to conceal these failures.

Octet's existing traceability profile selects five maintained spec families. Its 181-requirement scope excludes the compiler-IR specification.
The new compiler requirement has direct implementation and verification markers in the collector and real compiler tests; those tests ran as recorded above.
A passing project traceability report must not be presented as coverage of the new compiler requirement.
Inspect each mutation plan before execution. Publication must precede Noble pin changes.
Build the published immutable revision and compare collector bytes before adoption.

This repair covers four reviewed getrandom entry points, not every randomness provider or dependency implementation.
Fixture effects compile but do not execute. Replay does not authenticate the compiler, store, host, cache, or external execution.
No refinement proof or independent downstream acceptance is established. Noble remains active at 5/21 tasks; task 2.2 remains open.
