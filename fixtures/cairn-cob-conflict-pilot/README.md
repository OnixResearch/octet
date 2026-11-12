# Octet Cairn COB conflict-recovery fixture

This fixture is a **fresh 2026-11-12 observation**, not a migration or relabelling of the earlier receipts. It was generated with the original producer, Cairn source revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`, rebuilt from `github:OnixResearch/cairn/70db8fe114910feb5d162fea5db8ad4ffdb6f176#cairn`. The exact executable was `/nix/store/09nvvq1mz46vsa9kgxx126zl93ypq6dv-cairn-0.1.0/bin/cairn` (derivation `/nix/store/msshvni34279shl7aaqy423bqpgz7419-cairn-0.1.0.drv`). `provider-provenance.json` records its BLAKE3 identity and the clean base file inventory. This historical producer is explicit fixture-local replay tooling; it does not change Octet's current operator dependency.

`manifest.ncl` is the typed source of truth; `manifest.json` is its deterministic `nickel export --format json` projection. The base is a new, parentless minimal Git commit, `03c74d76cb153d88bce33a76b4c40fbf2a0fc365`, **not** an Octet repository revision or a clone of old Octet history. Its seven tracked files are only the four conflict-drill lifecycle artifacts, the fixture-local Nickel policy and contract, and the policy's generated JSON. The lifecycle and policy source bytes in `inputs/` were retained from the earlier safe fixture inputs; no old Git objects were copied. Replay exports the policy from Nickel before creating the clean root commit. The fixture uses that original producer-compatible policy, not current Octet operator policy.

The self-contained `octet-cob-conflict.bundle` retains exactly `refs/heads/main`, the creator's resolved namespace ref, and the delegate's losing namespace ref. Its resolved COB commit has exactly the creator and delegate update commits as its two direct parents; the losing revision remains reachable. `heads/` retains both exact conflicting `tasks.md` values. `evidence/` contains actual, unmodified Cairn stdout for signed creator/delegate import, same-basis divergent updates, metadata-only and exact-selector inspections, ordinary-update refusal, preview/executed choose-head resolution, fresh third-store reload, and preview/executed checkout. `transport.json` is a helper-generated observation of executed explicit Git transport, actual refs, ancestry, and empty remote lists; it is not a Cairn-issued receipt. BLAKE3 values are computed from the actual retained bytes.

## Exact capture/replay command

From the Octet checkout, with `python3`, Git, `ssh-keygen`, `b3sum`, and Nickel on `PATH`:

```sh
python3 fixtures/cairn-cob-conflict-pilot/replay.py \
  --cairn /nix/store/09nvvq1mz46vsa9kgxx126zl93ypq6dv-cairn-0.1.0/bin/cairn \
  --output fixtures/cairn-cob-conflict-pilot
```

Use a different `--output` directory to capture a separate fresh run. The helper stages the complete drill in restricted external temporary storage and replaces retained outputs only after every scenario succeeds. `replay-commands.json` records the executed Cairn argument vectors with the temporary directory represented as `<scratch>`; transport argument vectors are in `evidence/transport.json`. Fresh ephemeral Ed25519 creator/delegate keys are generated outside the tracked tree under a mode-0700 directory and removed with every temporary store on exit. Keys are never copied to the bundle, inputs, receipts, or output directory. Public actor and operation identities change on fresh runs. Every Git store has repository-local non-authoritative display metadata and no configured remote.

Head and direct-parent order follow the provider's actual output. Replay checks
that both distinct creator/delegate operations are present, without sampling
new keys to force one role to sort first. Every retained receipt is unchanged
provider output from one complete successful drill.

## Consumer checks

These commands are provided for the integrating maintainer; the fixture capture did not run tests, builds, linters, or formatters:

```sh
git bundle verify fixtures/cairn-cob-conflict-pilot/octet-cob-conflict.bundle
nix develop -c cargo test --test cairn_cob_conflict_pilot
nix develop -c nickel export --format json \
  fixtures/cairn-cob-conflict-pilot/manifest.ncl \
  > /tmp/cairn-cob-conflict-pilot-manifest.json
cmp fixtures/cairn-cob-conflict-pilot/manifest.json \
  /tmp/cairn-cob-conflict-pilot-manifest.json
nix build .#checks.x86_64-linux.cairn-cob-conflict-pilot --no-link -L
```

The resolved lifecycle observation is intentionally `stale`: selecting a content head resolves the artifact conflict but does not silently rewrite the earlier lifecycle subject root. This is local signed Git COB integration evidence only. It does not prove semantic merge correctness, Radicle runtime participation, publication, remote authorization, availability, durability, synchronization, distributed convergence, source correctness, whole-program correctness, or release eligibility.
