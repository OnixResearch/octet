# Validation

## Outcome

Octet's canonical `--workspace --lib` self-check now has a reviewed stable-ID warning budget. The packaged runner emits portable repository-relative finding locators, rejects invalid locator authority, normalizes exact workspace-root text before stable-ID construction, and deduplicates repeated stable identities when writing a baseline.

The final canonical inventory contains 2,670 warning occurrences represented by 1,820 unique stable identities. The bounded cleanup removed the sole `negated_predicate` finding by renaming `has_no_todo` to the positive `has_supported_lint` without suppression or behavior change. The checked-in baseline BLAKE3 identity is `b3:7bce67812e0d57029644efc644ed44035250b2891391b846fc9a2a92caf45de9`.

The unchanged self-check exits `0` with `warning-only` status, zero errors, zero new IDs, and zero stale IDs. A hermetic behavior-preserving source mutation adds exactly one `negated_predicate` identity and exits `1`. Malformed metadata and Cargo-child failures remain integration failures with exit `2`.

Current Cairn traceability policy hash is `db83e2b311a6782f30b97ce502fcec424005196bc94cd671bb4403230c255705`. Traceability reports `missing: []` and `dangling: []`.

## Functional core and shell

- `cargo-octet/src/engine.rs` owns pure locator and diagnostic-message normalization over an explicit workspace root. The runner shell supplies the discovered root and propagates invalid locators before artifact or baseline mutation.
- `cargo-octet/src/baseline.rs` deterministically projects repeated findings into one sorted record per stable identity. Existing set-based evaluation remains the admission decision core.
- `flake.nix` supplies immutable source, isolated Cargo homes and targets, a writable negative copy, and exact status assertions. It does not implement a second comparison algorithm or rewrite the checked-in baseline.

## Checks

- Pre-change baseline positive, rename, and incomplete-run regression tests — passed.
- Focused locator normalization tests — 2 passed; outside-root, empty, and traversal locators were rejected.
- Focused diagnostic-message normalization test — 1 passed.
- Focused baseline tests — 2 passed, including repeated-stable-ID deduplication and positive/new-ID behavior.
- Focused `no_todo` behavior tests — 2 passed.
- Canonical structured self-check — exit `0`, `warning-only`, 2,670 warnings, 1,820 accepted/current identities, zero errors, zero new IDs, zero stale IDs, no absolute finding locator, and no `negated_predicate` finding.
- Hermetic `cargo-octet-self-check` Nix gate — positive baseline and exactly-one-new-ID mutation cases passed; malformed metadata and Cargo-child exit contracts remained covered.
- Cairn policy export freshness — passed.
- Strict Cairn validation — passed with no issues.
- Cairn traceability — `missing: []`, `dangling: []`.
- Permanent positive/negative `cairn-traceability` Nix gate — passed.
- `cargo fmt --all -- --check` — passed.
- Workspace/all-target Clippy with warnings denied — passed.
- Nextest — 232 passed, 2 configured tests skipped.
- Repo-owned object-corpus fixture refresh — 1 explicit ignored refresh test passed; the six new selected helper functions changed source identity without changing Function Object v1 semantics.
- Focused object-corpus tests — 12 passed; fixture freshness and independent fresh-build determinism Nix gates passed.
- Full compatible-system `nix flake check -L --option secret-key-files ''` — passed.

## Adversarial findings

The permanent traceability negative initially passed unexpectedly because `flake.nix` had been added as an evidence root and contained the fixture's own effect-closure marker text. Both typed Nickel profiles were narrowed to maintained implementation, test, and documentation roots; generated policy was refreshed. The negative gate then failed on removed real markers as intended, while positive traceability remained complete.

The first full flake run found stale internal and expanded function-object corpus receipts because the selected `baseline.rs` and `engine.rs` source set gained six helper functions. The repository-owned ignored refresh test regenerated the internal receipt, expanded receipt, and caveat taxonomy. Focused freshness and independent fresh-build checks passed before the full flake rerun.

## Non-claims

The baseline proves only stable-ID membership for the pinned local static-lint run and configured workspace-library scope. Accepted warnings are not thereby correct, low-noise, harmless, complete, or permanently authorized. This evidence does not cover tests, binaries, examples, benches, dependencies, runtime behavior, whole-program correctness, lint completeness, cross-machine determinism, or release eligibility.

Traceability proves discoverable linkage under the selected policy, not runtime reachability, behavioral correctness, verifier soundness, deployment, or remote trust. Function-object fixture refresh proves bounded source-identity replay only, not semantic equivalence or correctness.

Nix completed locally after the known unavailable Git HTTPS helper and configured remote-builder warnings.
