# Self-check warning budget

Octet ratchets its canonical `--workspace --lib` self-check with the runner's existing stable-ID baseline mechanism. The machine-readable source is [`standards/self-check-warning-budget/octet-baseline.json`](../standards/self-check-warning-budget/octet-baseline.json).

<!-- r[impl octet.runner_self_check.inventory] -->
<!-- r[impl octet.runner_self_check.warning_budget] -->
<!-- r[impl octet.runner_self_check.warning_budget.reduction] -->
<!-- r[verify octet.runner_self_check.warning_budget.baseline] -->

## Current budget

The initial inventory contained 2,671 warning occurrences. Before the first baseline capture, the local `has_no_todo` name was rewritten in positive form without changing behavior or adding a suppression, leaving 2,670 accepted occurrences and 1,820 unique stable identities.

The first monotonic reduction replaced the unique private `ExitCode` import in `cargo-octet/src/lib.rs` with explicit `std::process::ExitCode` owner paths. The old baseline reported exactly one stale identity, `b3:8515683389cc483f`, and zero additions before refresh. The current run contains 2,669 accepted warning occurrences represented by 1,819 unique stable identities, no `negated_predicate` finding, and no absolute artifact locator. Repeated stable IDs are stored once in the baseline.

<!-- r[verify octet.runner_self_check.warning_budget.reduction.old_baseline] -->
<!-- r[verify octet.runner_self_check.warning_budget.reduction.refresh] -->

Baseline BLAKE3 identity: `b3:1327af062e287cdd51eba7cbc26dd63ee4a1f47714ac4491884a644e05e22d5b`.

| Domain | Accepted warnings |
| --- | ---: |
| `tigerstyle::naming` | 1,670 |
| `tigerstyle::structure` | 476 |
| `tigerstyle::safety` | 473 |
| `tigerstyle::purity` | 50 |

| Lint | Accepted warnings |
| --- | ---: |
| `non_trait_imports` | 1,113 |
| `path_segment_repetition` | 396 |
| `borrowed_argument_types` | 253 |
| `explicit_defaults` | 169 |
| `unbounded_collection_growth` | 166 |
| `excessive_file_length` | 113 |
| `underscore_in_module_filename` | 85 |
| `bool_naming` | 74 |
| `function_length` | 58 |
| `ambient_env` | 46 |
| `no_unwrap` | 36 |
| `no_recursion` | 24 |
| `too_many_parameters` | 23 |
| `no_panic` | 22 |
| `ignored_result` | 20 |
| `sentinel_fallback` | 13 |
| `fallible_int_fallback` | 12 |
| `catch_all_on_enum` | 10 |
| `module_file_count` | 10 |
| `usize_in_public_api` | 10 |
| `nested_conditionals` | 8 |
| `ambient_clock` | 4 |
| `renamed_imports` | 2 |
| `fat_trait` | 1 |
| `unbounded_loop` | 1 |
| `negated_predicate` | 0 |

## Gate behavior

The `cargo-octet-self-check` Nix derivation runs structured output against the checked-in baseline. The unchanged source must report 2,669 warning occurrences, 1,819 accepted stable identities, zero new IDs, zero stale IDs, a matching baseline hash, and repository-relative locators. A writable negative copy injects one behavior-preserving, intentionally negated predicate binding and must exit `1` with exactly one new ID. Malformed metadata and Cargo-child failures remain integration failures with exit `2`.

<!-- r[verify octet.runner_self_check.warning_budget.regression] -->
<!-- r[verify octet.runner_self_check.warning_budget.reduction.regression] -->
<!-- r[verify octet.runner_self_check.warning_budget.cleanup] -->

Baseline refresh is an explicit review operation, not an automatic gate action:

```bash
nix run .#cargo-octet -- check \
  --output-format json \
  --artifact-dir target/self-check-warning-budget \
  --write-baseline target/octet-baseline.json
```

Review the stable-ID additions, removals, lint/domain counts, relative locators, and BLAKE3 identity before replacing the checked-in file. Resolved entries become stale and should be pruned through a reviewed refresh; they do not justify adding unrelated findings.

## Boundaries

<!-- r[impl octet.runner_self_check.warning_budget.boundary] -->
<!-- r[verify octet.runner_self_check.warning_budget.reduction.boundary] -->

The baseline proves only that current findings belong to one reviewed stable-ID set for the pinned local static-lint run. It does not prove that accepted warnings are correct, low-noise, harmless, or complete. It does not cover tests, binaries, examples, benches, dependencies, runtime behavior, whole-program correctness, lint completeness, or release eligibility.
