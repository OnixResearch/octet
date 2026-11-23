# Self-check finding delta (PR #38 warning budget refresh)

Comparison between the prior checked-in baseline (`standards/self-check-warning-budget/octet-baseline.json` on PR #38 `113861db`) and the refreshed baseline produced by the packaged `cargo-octet check --output-format json --write-baseline`:

- Prior baseline: **3,030** unique stable identities, **4,239** warning occurrences, `baseline_hash` `b3:97c195b6f296e9eb6b7ef38b237c8a45fb9738f2e657740d02b5d283b146c4cf`
- Refreshed baseline: **3,021** unique stable identities, **4,230** warning occurrences, `baseline_hash` `b3:f9865bc1d48bb2a198ad76d518e38fcb7846d46bd0f6416ed36d683d5c67116a`, file BLAKE3 `b3:09074b80d19a2bb63fca88b66d16d505ae1df5a5639d0ddadcf556bce0fc43fd`
- Net delta: **-9** unique stable identities (+6 added / -15 removed), **-9** warning occurrences
- Invariants verified: `schema_version == 1`, `finding_schema_version == 2`, 0 duplicate `stable_id`s, 0 absolute file locators, 0 `negated_predicate` findings.

## Added stable identities (6)

| Stable ID | Lint | Location | Message | Reason |
| --- | --- | --- | --- | --- |
| `b3:a16d8402b69e2861` | `too_many_parameters` | `cargo-octet/src/requirements/evaluation/decision.rs:46:1` | function has 6 parameters (limit: 5) — consider an options struct | New accepted finding from PR #38 (`--artifact-base` added the 6th parameter to `evaluate_contract`). Kept unbundled so callers cannot pair parsed contract data with mismatched raw bytes, avoiding 17 call-site rewrites for no behavioral gain. |
| `b3:107ec9296c742de6` | `function_length` | `cargo-octet/src/prove.rs:601:20` | function `execute_proof` is 159 lines (limit: 70) | Pre-existing finding replaced after PR #38 grew `execute_proof` from 142 lines (`b3:4f87af6ee60a7f4c`) to 159 lines. |
| `b3:87dced700697edf9` | `function_length` | `cargo-octet/src/cli.rs:2656:84` | function `parse_prove` is 284 lines (limit: 70) | Pre-existing finding replaced after PR #38 grew `parse_prove` from 268 lines (`b3:22488b9ffc1efc72`) to 284 lines. |
| `b3:9f4cf09a1602cc45` | `excessive_file_length` | `cargo-octet/src/cli.rs:1:1` | source file is 5641 lines (limit: 300) | Pre-existing finding replaced after PR #38 changed `cli.rs` from 5575 lines (`b3:99e64f286745895a`) to 5641 lines. |
| `b3:bd28b969a0d12871` | `excessive_file_length` | `cargo-octet/src/prove.rs:1:1` | source file is 5346 lines (limit: 300) | Pre-existing finding replaced after PR #38 changed `prove.rs` from 4976 lines (`b3:337fc14544dcc4df`) to 5346 lines. |
| `b3:e85f050f9ffb3a86` | `excessive_file_length` | `cargo-octet/src/artifact.rs:1:1` | source file is 3486 lines (limit: 300) | Pre-existing finding replaced after PR #38 changed `artifact.rs` from 3399 lines (`b3:e803f143c65ccc60`) to 3486 lines. |

## Removed stable identities (15)

| Stable ID | Lint | Location | Message | Reason |
| --- | --- | --- | --- | --- |
| `b3:4f87af6ee60a7f4c` | `function_length` | `cargo-octet/src/prove.rs:591:20` | function `execute_proof` is 142 lines (limit: 70) | Replaced by `b3:107ec9296c742de6` (159 lines). |
| `b3:22488b9ffc1efc72` | `function_length` | `cargo-octet/src/cli.rs:2613:84` | function `parse_prove` is 268 lines (limit: 70) | Replaced by `b3:87dced700697edf9` (284 lines). |
| `b3:99e64f286745895a` | `excessive_file_length` | `cargo-octet/src/cli.rs:1:1` | source file is 5575 lines (limit: 300) | Replaced by `b3:9f4cf09a1602cc45` (5641 lines). |
| `b3:337fc14544dcc4df` | `excessive_file_length` | `cargo-octet/src/prove.rs:1:1` | source file is 4976 lines (limit: 300) | Replaced by `b3:bd28b969a0d12871` (5346 lines). |
| `b3:e803f143c65ccc60` | `excessive_file_length` | `cargo-octet/src/artifact.rs:1:1` | source file is 3399 lines (limit: 300) | Replaced by `b3:e85f050f9ffb3a86` (3486 lines). |
| `b3:4b47c964538d840d` | `no_unwrap` | `cargo-octet/src/prove.rs:1921:5` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:a9d4756f1a528d4d` | `no_unwrap` | `cargo-octet/src/prove.rs:1922:5` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:9cbd62c243e10edb` | `no_unwrap` | `cargo-octet/src/prove.rs:1925:9` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:d74113062ce66ae4` | `no_unwrap` | `cargo-octet/src/prove.rs:1939:9` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:051b036fa1798c04` | `no_unwrap` | `cargo-octet/src/prove.rs:1952:13` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:b6403696baee2c20` | `no_unwrap` | `cargo-octet/src/prove.rs:1956:13` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:cc3dd6c7ca1de7e4` | `no_unwrap` | `cargo-octet/src/prove.rs:1960:13` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:703af9864f72c044` | `no_unwrap` | `cargo-octet/src/prove.rs:1965:9` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:52d6e10d4a3eac69` | `no_unwrap` | `cargo-octet/src/prove.rs:1977:9` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
| `b3:2c9e5c3d6ded33c5` | `no_unwrap` | `cargo-octet/src/prove.rs:1981:9` | `.expect()` in production code will panic on failure | Resolved: replaced `.expect()` on `writeln!` with propagated formatting `Result`. |
