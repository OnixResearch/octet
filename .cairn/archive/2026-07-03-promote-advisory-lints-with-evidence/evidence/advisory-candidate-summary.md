# Advisory lint promotion candidate summary

Command: `(temp HEAD archive with [workspace.metadata.octet]) nix run path:/home/brittonr/git/octet#cargo-octet -- check --output-format json`

Input config: `cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dylint.toml`
Raw findings: `cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dogfood.jsonl`
Summary/stderr: `cairn/changes/promote-advisory-lints-with-evidence/evidence/advisory-candidate-dogfood.stderr`

The run enabled only `raw_arithmetic_overflow`, `ambiguous_params`, `multi_lock_ordering`, `defaulting_trait_method`, and `numeric_units` as Octet warnings against the `octet --lib` dogfood scope. It disabled 48 other Octet lints. One non-target rustc warning (`unused_imports`) appeared in the runner output and is excluded from the promotion table below.

| Candidate | Raw findings | Accepted debt | Fixed in this change | Residual findings | Recommendation | Rationale |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| `raw_arithmetic_overflow` | 27 | 27 | 0 | 27 | blocked | High self-dogfood count across lint implementation helpers; needs focused remediation or typed exception proof before profile/default promotion. |
| `ambiguous_params` | 1 | 1 | 0 | 1 | deferred | One self-dogfood finding is reviewable, but no representative downstream count or fix transcript is saved yet. |
| `multi_lock_ordering` | 0 | 0 | 0 | 0 | deferred | Clean on Octet self-dogfood, but needs downstream/concurrency sample evidence before promotion. |
| `defaulting_trait_method` | 0 | 0 | 0 | 0 | deferred | Clean on Octet self-dogfood, but needs downstream trait-API sample evidence before promotion. |
| `numeric_units` | 2 | 2 | 0 | 2 | deferred | Low count, but both findings need naming cleanup or explicit unit rationale before promotion. |

No default-level or starter-profile promotion is accepted by this change. The evidence gate is now documented; future promotions must add a fresh candidate row with fixed/residual counts, suppression compatibility proof, migration notes, and a final `ready` recommendation before changing defaults or copied starter profiles.
