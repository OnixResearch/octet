# Aspen lint-precision comparison

- replay helper: `bash openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/replay-aspen-comparison.sh`
- Aspen commit: `909e5e9fa`
- baseline tigerstyle commit: `f04333d6640228d2305904b981ce70f7d803424d`
- candidate tigerstyle commit: `51b6c6cabee12748fb971e36151464927cb54d7e`
- Aspen comparison scope: `--workspace -- --all-targets`
- enabled lint families: `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, `unbounded_loop`
- phase-1 archive reference: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- phase-2 archive reference: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/`
- baseline transcript: `/home/brittonr/git/tigerstyle/openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-workspace-before.txt`
- candidate transcript: `/home/brittonr/git/tigerstyle/openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-workspace-after.txt`

| lint family | before | after | delta | note |
|---|---:|---:|---:|---|
| `ignored_result` | 29 | 29 | +0 | no count change |
| `no_unwrap` | 144 | 144 | +0 | no count change |
| `no_panic` | 18 | 18 | +0 | no count change |
| `unchecked_narrowing` | 3 | 3 | +0 | no count change |
| `unbounded_loop` | 139 | 147 | +8 | added warnings are limited to vendored `openraft/*` workspace members |

## Added workspace-only `unbounded_loop` paths

- `openraft/openraft/src/core/heartbeat/worker.rs:73`
- `openraft/openraft/src/core/raft_core.rs:424`
- `openraft/openraft/src/core/raft_core.rs:1062`
- `openraft/openraft/src/raft/stream_append.rs:53`
- `openraft/openraft/src/replication/mod.rs:530`
- `openraft/openraft/src/replication/snapshot_transmitter.rs:96`
- `openraft/openraft/src/testing/runtime/mod.rs:336`
- `openraft/openraft/src/testing/runtime/mod.rs:454`

## Acceptance

- **Blocking completion gate:** owned Aspen rollout scope (`-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets`), recorded in `aspen-comparison.md`.
- **Non-blocking context:** workspace-wide `unbounded_loop +8` in vendored `openraft/*` members.
- This archive treats those vendored workspace hits as documented out-of-scope context rather than as a blocker for completing `lint-precision-hardening`, because the change contract for downstream validation is consumer-owned Aspen code, not vendored workspace members.
