# Tasks: Prove The Rollout Kit On A Second Consumer

## Phase 1: Docs and spec

- [ ] Extend the documentation spec so the rollout kit requires one saved non-Aspen consumer proof
- [ ] Update `docs/downstream-rollout.md` with a pointer to the second-consumer replay helper and transcript

## Phase 2: Proof

- [ ] Pick a non-Aspen Rust consumer and create a disposable sandbox that copies `templates/tigerstyle-check.sh` and `templates/dylint.toml`
- [ ] Save a replay helper and transcript showing the copied templates run end to end on that second consumer
