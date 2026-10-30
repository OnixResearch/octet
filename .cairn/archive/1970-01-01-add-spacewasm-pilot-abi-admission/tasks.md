# Tasks: Exact SpaceWasm pilot ABI admission

- [x] [serial] Resolve core function signatures, bind ABI facts into interface identity, and enforce hostless run signature. r[octet.spacewasm_pilot.resolved_abi]
- [x] [depends:resolved-abi] Add dedicated actual Molten actor ABI profile and deterministic Nickel export. r[octet.spacewasm_pilot.molten_abi]
- [x] [depends:resolved-abi] Add separate observe/pause diagnostic profile and mutable total check. r[octet.spacewasm_pilot.diagnostic_abi]
- [x] [parallel] Bound shell reads and pure parsing; enforce reviewed proposal intersection. r[octet.spacewasm_pilot.bounded_intersection]
- [x] [parallel] Document persisted real-producer receipt intake, immutable NASA source/cohort, and nonclaims; coordinate exact contracts with Mantle, ChaosControl, and Molten consumer owners. r[octet.spacewasm_pilot.receipt_boundary]
- [x] [depends:molten-abi] Add real producer CLI smoke and positive/negative ABI/resource fixtures without asserting unexecuted outcomes. r[octet.spacewasm_pilot.operator_smoke]
- [x] [serial] Run focused package Wasm/SpaceWasm tests, profile freshness, actual pinned producer smoke, and fixture-candidate receipt verification; retain genuine output. r[octet.spacewasm_pilot.operator_smoke]
- [x] [serial] Integrate on published Octet main `abb6082062251944a40093a274df3d35331fa168`, keep every SpaceWasm-touched source file pedantic-Clippy clean, rerun focused tests, strict package Clippy, actual pinned producer smoke and replay, Nix Wasm/standards/policy/traceability checks, and repo-pinned native Cairn validation, policy freshness, workflow status and proposal/design/tasks gates; record the pre-existing workspace pedantic Clippy blocker. Exact commands and results: `evidence/verification.txt` section "Final integration 2026-10-30". r[octet.spacewasm_pilot.operator_smoke] r[octet.spacewasm_pilot.receipt_boundary]

Downstream Mantle final-workload bytes, Molten intake of the generated actor-profile
receipt and Molten's immutable producer pin are consumer-owned scope (see proposal
Impact) and are not prerequisites to this producer archive; they are not claimed here.

## Verification Coverage

| Scenario | Required proof |
| --- | --- |
| Same export name, wrong signature | focused hostless signature denial test |
| Import shifts function index | focused resolved-signature test |
| Real actor shape admitted | actual CLI smoke admitted case plus artifact verify |
| Memory or host contract differs | focused actor matrix and CLI unbounded/wrong-host/wrong-alloc cases |
| Diagnostic import/global differs | focused diagnostic signature/global denial matrix |
| Artifact exceeds ceiling | focused pre-parse rejection test and bounded CLI intake |
| Profile exceeds support intersection | registry validation against reviewed support |
| Candidate bytes change | actual producer smoke artifact tamper verification denial |
| External evidence mismatch/overclaim | existing external-runtime mismatch and role-promotion denial tests |
| Operator actual producer replay | `python3 standards/wasm-artifact/spacewasm/smoke.py /absolute/cargo-octet /absolute/wasm-tools` |

## Execution Status

Provider source was frozen before post-implementation verification. Actual
SpaceWasm tests passed (12), actual Wasm artifact tests passed (17; overlapping
selection), and both Nix Wasm freshness and generated standards-manifest checks
passed. The built producer admitted both fixture profiles and verified their
genuine receipts; valid-Wasm signature mutations, stale cohort and forged
external evidence were denied. The operator smoke observed exact-byte tamper
denial and every required artifact denial. Exact commands, identities, receipt
paths and limits are in `evidence/verification.txt`; `evidence/replay.py` is the
retained actual-producer replay helper. Formatting and a new analyzer Clippy
finding required scoped repairs. Pedantic Clippy also reports unrelated existing
provider errors; see execution evidence rather than interpreting tests as lint
closure.

Final integration on published main resolved the earlier open items. The
`outcome-machine` parse failure came from a newer ambient `cairn` binary; the
repository-pinned Cairn (`nix run .#cairn`, flake lock rev
`695124d459574ba7aeba6097310d237f393c243c`) reports the generated policy fresh and
passes validation, workflow status and all three gates, so no policy change was
needed. The four SpaceWasm-touched source files now have zero pedantic Clippy
findings, and `cargo clippy -p cargo-octet --all-targets -- -D warnings` passes.
The workspace pedantic rail (`checks.clippy` flags) still fails on unchanged
published main with 193 pre-existing findings outside this change's scope; this
change does not claim to pass it. All 517 non-ignored cargo-octet library tests pass (21 ignored). Final
Mantle workload bytes and Molten consumer execution remain consumer-owned. No
runtime, production, release or independent-review authority is claimed.
