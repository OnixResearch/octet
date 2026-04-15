# Aspen lint-precision comparison

- replay helper: `bash openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/replay-aspen-comparison.sh`
- Aspen commit: `909e5e9fa`
- baseline tigerstyle commit: `f04333d6640228d2305904b981ce70f7d803424d`
- candidate tigerstyle commit: `51b6c6cabee12748fb971e36151464927cb54d7e`
- primary comparison scope: `-p aspen-time -p aspen-hlc -p aspen-core -p aspen-coordination -- --all-targets`
- enabled lint families: `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, `unbounded_loop`
- phase-1 archive reference: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`
- phase-2 archive reference: `openspec/changes/archive/2026-04-14-aspen-phase2-widening/`
- primary baseline transcript: `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-before.txt`
- primary candidate transcript: `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-after.txt`
- supplementary workspace baseline transcript: `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-workspace-before.txt`
- supplementary workspace candidate transcript: `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-workspace-after.txt`
- supplementary workspace summary: `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-workspace-comparison.md`

## Primary owned-package rerun

| lint family | before | after | delta | note |
|---|---:|---:|---:|---|
| `ignored_result` | 2 | 2 | +0 | no count change |
| `no_unwrap` | 122 | 122 | +0 | no count change |
| `no_panic` | 3 | 3 | +0 | no count change |
| `unchecked_narrowing` | 0 | 0 | +0 | no count change |
| `unbounded_loop` | 0 | 0 | +0 | no count change |

## Supplementary workspace rerun

The broader `--workspace -- --all-targets` rerun changed only one family:

| lint family | before | after | delta | note |
|---|---:|---:|---:|---|
| `ignored_result` | 29 | 29 | +0 | no count change |
| `no_unwrap` | 144 | 144 | +0 | no count change |
| `no_panic` | 18 | 18 | +0 | no count change |
| `unchecked_narrowing` | 3 | 3 | +0 | no count change |
| `unbounded_loop` | 139 | 147 | +8 | new warnings only outside the owned rollout scope |

Added workspace-only `unbounded_loop` paths:

- `openraft/openraft/src/core/heartbeat/worker.rs:73`
- `openraft/openraft/src/core/raft_core.rs:424`
- `openraft/openraft/src/core/raft_core.rs:1062`
- `openraft/openraft/src/raft/stream_append.rs:53`
- `openraft/openraft/src/replication/mod.rs:530`
- `openraft/openraft/src/replication/snapshot_transmitter.rs:96`
- `openraft/openraft/src/testing/runtime/mod.rs:336`
- `openraft/openraft/src/testing/runtime/mod.rs:454`

## Conclusion

- The owned Aspen phase-1/phase-2 rollout scope shows **no count delta** for the
  five precision-hardened lint families between baseline `f04333d` and current
  `51b6c6c`.
- That means this rerun found **no evidence that real owned-package findings were
  lost** by the precision changes.
- This rerun also found **no owned-package count reduction**, so it does **not**
  justify promoting any additional inventory-first family in
  `docs/downstream-rollout.md` yet.
- The only observed regression came from vendored `openraft` workspace members in
  the supplementary workspace-wide rerun, so it is tracked as out-of-scope for
  current Aspen rollout promotion decisions rather than as a consumer-owned
  regression.
