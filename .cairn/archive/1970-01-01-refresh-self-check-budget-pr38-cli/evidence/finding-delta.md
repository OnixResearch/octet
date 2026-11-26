# Self-check finding delta (PR #38 reviewed CLI restoration)

Comparison between the prior checked-in baseline (`standards/self-check-warning-budget/octet-baseline.json` at `88c0264c`) and the refreshed baseline after restoring `cargo-octet/src/cli.rs` to the owner-approved `113861db` bytes (`d181d2d9a38551c4aef2ad5ec7a7074ebc436846a5b7171f9cc1f2b0746ecd1d`):

- Prior baseline (`88c0264c`): **3,021** unique stable identities, **4,230** warning occurrences, `baseline_hash` `b3:f9865bc1d48bb2a198ad76d518e38fcb7846d46bd0f6416ed36d683d5c67116a`, file BLAKE3 `b3:09074b80d19a2bb63fca88b66d16d505ae1df5a5639d0ddadcf556bce0fc43fd`
- Refreshed baseline: **3,022** unique stable identities, **4,231** warning occurrences, `baseline_hash` `b3:88bbada65e65334e9f82bfaaeac4b6df400a11d25427a690104dab73cbcfda53`, file BLAKE3 `b3:650d5081dd91b7a536a37d8413e1b22e74d7d6400aa263ce25c22cb53b5e5b70`
- Net delta vs `88c0264c`: **+1** unique stable identity (+2 added / -1 removed), **+1** warning occurrence
- Net delta vs `113861db`: **-8** unique stable identities (+7 added / -15 removed), **-8** warning occurrences (`4239 -> 4231`)
- Invariants verified: `schema_version == 1`, `finding_schema_version == 2`, 0 duplicate `stable_id`s, 0 absolute file locators, 0 `negated_predicate` findings.

## Added stable identities vs `88c0264c` (2)

| Stable ID | Lint | Location | Message | Reason |
| --- | --- | --- | --- | --- |
| `b3:3600e3c1b7e8cf1b` | `function_length` | `cargo-octet/src/cli.rs:793:76` | function `parse_requirements` is 72 lines (limit: 70) | Restoring `cargo-octet/src/cli.rs` to owner-approved `113861db` bytes (`d181d2d9...`) preserves byte parity with `reviewed-source-manifest.json` and `pr38-object-refresh-owner-approval.txt` and keeps the linear subcommand flag match inline per the existing comment. |
| `b3:df34a77a382f3e0f` | `excessive_file_length` | `cargo-octet/src/cli.rs:1:1` | source file is 5639 lines (limit: 300) | Pre-existing finding replaced after restoring `cargo-octet/src/cli.rs` from 5641 lines (`b3:9f4cf09a1602cc45`) to 5639 lines (`113861db`). |

## Removed stable identities vs `88c0264c` (1)

| Stable ID | Lint | Location | Message | Reason |
| --- | --- | --- | --- | --- |
| `b3:9f4cf09a1602cc45` | `excessive_file_length` | `cargo-octet/src/cli.rs:1:1` | source file is 5641 lines (limit: 300) | Replaced by `b3:df34a77a382f3e0f` (5639 lines) after restoring `cargo-octet/src/cli.rs` to `113861db`. |
