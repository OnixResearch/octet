# Octet Cairn COB pilot fixture

This fixture retains **fresh local observations generated on 2026-11-12**, not the historical pilot run. Its provider is the actual Cairn source revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`. The exact newly generated base, actor, object, operation, content, state, and namespace identities are recorded in `manifest.ncl` and its deterministic JSON export.

The newer Octet flake provider `695124d459574ba7aeba6097310d237f393c243c` cannot execute this signed-update scenario: its neutral collaboration planner rejects genuine Radicle `z6Mk…` creator identities with `invalid collaboration.creator: identity_prefix_unsupported`, because nominal `ActorId` admission requires `actor:` identities. This fixture therefore uses the original provider revision rather than relabelling receipts, changing signed identities, or weakening successful-update coverage. It uses that provider's native `cairn/changes/` layout and the original Octet generated policy bytes retained in the bundle's minimal base tree.

`manifest.ncl` is the typed source of truth; `manifest.json` is generated using `nickel export --format json`. `evidence/` retains untouched Cairn stdout for preview/executed import, preview/executed signed update, secondary status, bundle reload, preview/executed checkout, missing-changes-directory negative checkout, and conflict inspection. `transport.json` is separately produced by `regenerate.py` from actual explicit local Git fetch, ref, remote, tag, and file observations; it is not a Cairn-issued receipt.

The new Git root has no parent and tracks **only** `cairn-policy/generated/cairn-policy.json`. The four original safe lifecycle artifacts are recovered as data from the signed initialization operation, not by importing the original repository history. They are materialized outside the base tree and retained in the newly signed COB operations. `octet-cob.bundle` contains only `refs/heads/main` plus the signed namespace ref; it has no dependency on the old Octet Git ancestry. Replaying starts from a newly initialized Git root again, not a full-history clone. No standards source snapshots, agent-local configuration, environment files, or private keys are imported into the new history.

Every regeneration creates a fresh Ed25519 key using `ssh-keygen` in a mode-0700 temporary directory outside the repository, with umask 077. The private key is never printed, tracked, bundled, or retained; temporary roots and keys are removed on normal exit and exceptions. New keys intentionally produce new actor/signature/object identities; this is scenario replay, not a claim of byte-identical historical cryptographic output. The primary and secondary Git roots have no configured remotes, and local transport does not fetch tags.

The transport artifact-tree digest is `blake3-` plus the base64-encoded BLAKE3 of a compact, key-sorted JSON array of `{path, bytes, blake3_hex}` records, sorted by relative artifact path. Source and checked-out destination inventories must match before that observation is emitted. Receipt and bundle manifest digests are BLAKE3 of exact retained file bytes.

Regeneration requires Python 3, Git, OpenSSH `ssh-keygen`, `b3sum`, Nickel, and the exact original-provider executable. The replay helper accepts its explicit path with `--cairn`; it does not build a provider, change Octet refs, configure remotes, or publish anything. It recovers only the current clean bundle's safe initialization/update payloads and policy, then replaces the fixture outputs after every scenario has succeeded (including the expected negative checkout failure).

Exact replay command, run from the Octet repository root:

```sh
python3 fixtures/cairn-cob-pilot/regenerate.py --cairn /nix/store/09nvvq1mz46vsa9kgxx126zl93ypq6dv-cairn-0.1.0/bin/cairn
```

This executable was built from `github:OnixResearch/cairn/70db8fe114910feb5d162fea5db8ad4ffdb6f176#cairn`, derivation `/nix/store/msshvni34279shl7aaqy423bqpgz7419-cairn-0.1.0.drv`. It is the fixture-specific historical provider, not the current Octet product-provider output. The exact replay command above generated the retained observations from the clean bundle; the first clean bundle was bootstrapped by extracting only the four lifecycle artifact values and original generated policy bytes from the quarantined original fixture.

Validate the fixture:

```sh
git bundle verify fixtures/cairn-cob-pilot/octet-cob.bundle
nix develop -c cargo test --test cairn_cob_pilot
nix develop -c nickel export --format json fixtures/cairn-cob-pilot/manifest.ncl \
  > target/cairn-cob-pilot-manifest.json
cmp fixtures/cairn-cob-pilot/manifest.json target/cairn-cob-pilot-manifest.json
```

This evidence proves local observations for the pinned inputs only. It does not prove Radicle runtime participation, remote publication, authentication, authorization, availability, durability, synchronization, convergence, lifecycle validity, evidence truth, source correctness, whole-program correctness, or release eligibility.
