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
| `unbounded_loop` | 139 | 147 | +8 | regressed by 8 |
