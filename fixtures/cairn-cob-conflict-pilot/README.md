# Octet Cairn COB conflict-recovery fixture

This fixture now records a complete fresh capture with Octet's actual shipped Cairn executable `/nix/store/srgd4wpm2ad3hz72iwlbv9h1wny6gibg-cairn-0.2.0/bin/cairn`, source revision `2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b` (H3). Its executable BLAKE3 is `fc0e74108e3dd38c5dc4c14b902971a711eaa21d8a508cd8e54185bd4b356b3a`. Every scenario completed before the old outputs were replaced; no historical receipts were relabelled. This is bounded local signed fixture evidence, not upstream main integration, publication, release eligibility, or lifecycle closure.

`manifest.ncl` is the typed source of truth; `manifest.json` is its deterministic Nickel export. Current capture builds a new parentless minimal Git base containing the four native `.cairn/changes/` lifecycle artifacts retained in `inputs/`, plus current generated Octet policy JSON. It copies no quarantined Git objects and uses no historical producer-compatible policy or key-rejection sampling. Exact observed base and signer identities are recorded only after successful capture.

The self-contained `octet-cob-conflict.bundle` retains exactly `refs/heads/main`, the creator's resolved namespace ref, and the delegate's losing namespace ref. Its resolved COB commit has exactly the creator and delegate update commits as its two direct parents; the losing revision remains reachable. `heads/` retains both exact conflicting `tasks.md` values. `evidence/` contains actual, unmodified Cairn stdout for signed creator/delegate import, same-basis divergent updates, metadata-only and exact-selector inspections, ordinary-update refusal, preview/executed choose-head resolution, fresh third-store reload, and preview/executed checkout. `transport.json` is a helper-generated observation of executed explicit Git transport, actual refs, ancestry, and empty remote lists; it is not a Cairn-issued receipt. BLAKE3 values are computed from the actual retained bytes.

## Exact capture/replay command

From the Octet checkout, with `python3`, Git, `ssh-keygen`, `b3sum`, and Nickel on `PATH`:

```sh
python3 fixtures/cairn-cob-conflict-pilot/replay.py \
  --cairn /nix/store/srgd4wpm2ad3hz72iwlbv9h1wny6gibg-cairn-0.2.0/bin/cairn \
  --cairn-source-revision 2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b \
  --output fixtures/cairn-cob-conflict-pilot
```

`capture-log.txt` retains the exact isolated Nix runtime prefix, full capture command, and successful helper stdout. The runtime used pinned Nixpkgs `b3d51a0365f6695e7dd5cdf3e180604530ed33b4` and Nickel 1.17, without credentials, SSH agent, or global Git configuration. The captured parentless base is `08dde030f4d6fe23314c03acdfd520b453b40ad9`; the bundle BLAKE3 is `aa84be5371d12ebf14ad274b3755cbb6fc0ffb9eeced03d5fbed1279fd79213a`.

Use a different `--output` directory to capture a separate fresh run. The helper stages the complete drill in restricted external temporary storage and replaces retained outputs only after every scenario succeeds. `replay-commands.json` records the executed Cairn argument vectors with the temporary directory represented as `<scratch>`; transport argument vectors are in `evidence/transport.json`. Fresh ephemeral Ed25519 creator/delegate keys are generated outside the tracked tree under a mode-0700 directory and removed with every temporary store on exit. Keys are never copied to the bundle, inputs, receipts, or output directory. Public actor and operation identities change on fresh runs. Every Git store has repository-local non-authoritative display metadata and no configured remote.

The integrating owner supplied the matching shipped executable and immutable H3
source revision after reporting successful x86_64 upstream qualification and
policy-gated local source-spec synchronization. This capture did not rerun those
checks and does not establish owner completion, archive, upstream main integration,
external publication, or lifecycle closure. `provider-provenance.json` records the
actual executable BLAKE3 and five-file base inventory. Current Root policy was used
without a fixture-local override; its JSON byte BLAKE3 was
`728c4b47495c4469c2cdaa765908fd380732835f489f41a1270b60f36c4ae8e6`.
That byte digest is distinct from Cairn's canonical semantic policy hash in the
receipts. No fallback provider, historical policy, or key-rejection sampling was used.

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
