# Octet Cairn COB pilot fixture

This pilot's corrected-current-provider capture completed successfully with Octet's actual shipped Cairn executable `/nix/store/srgd4wpm2ad3hz72iwlbv9h1wny6gibg-cairn-0.2.0/bin/cairn`, source revision `2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b`. The retained observations are fresh signed CLI outputs, not relabelled historical receipts. `provider-provenance.json` records executable BLAKE3 `fc0e74108e3dd38c5dc4c14b902971a711eaa21d8a508cd8e54185bd4b356b3a` and the actual parentless minimal base inventory. This is bounded local pilot capture evidence, not upstream main integration, completion of the entire Cairn lifecycle, or release approval.

The capture helper now uses native `.cairn/changes/` paths and the current generated Octet policy. It accepts an explicit qualified executable and source revision rather than building or silently selecting a historical provider. Canonical Ed25519 creator identities remain the exact signed `z6Mk…` strings; they are never rewritten into portable actor labels.

`manifest.ncl` is the typed source of truth; `manifest.json` is generated using `nickel export --format json`. `evidence/` retains untouched Cairn stdout for preview/executed import, preview/executed signed update, secondary status, bundle reload, preview/executed checkout, missing-changes-directory negative checkout, and conflict inspection. `transport.json` is separately produced by `regenerate.py` from actual explicit local Git fetch, ref, remote, tag, and file observations; it is not a Cairn-issued receipt.

Each capture creates a new parentless Git root tracking **only** `cairn-policy/generated/cairn-policy.json` from the current checkout. The four safe lifecycle artifacts are recovered as data from the existing clean bundle's signed initialization operation, not by importing old repository history. They are materialized outside the base tree and retained in newly signed COB operations. The output bundle contains only `refs/heads/main` plus the signed namespace ref, with no dependency on quarantined Octet ancestry. No standards snapshots, agent-local configuration, environment files or private keys are imported into the new history.

Every regeneration creates a fresh Ed25519 key using `ssh-keygen` in a mode-0700 temporary directory outside the repository, with umask 077. The private key is never printed, tracked, bundled, or retained; temporary roots and keys are removed on normal exit and exceptions. New keys intentionally produce new actor/signature/object identities; this is scenario replay, not a claim of byte-identical historical cryptographic output. The primary and secondary Git roots have no configured remotes, and local transport does not fetch tags.

The transport artifact-tree digest is `blake3-` plus the base64-encoded BLAKE3 of a compact, key-sorted JSON array of `{path, bytes, blake3_hex}` records, sorted by relative artifact path. Source and checked-out destination inventories must match before that observation is emitted. Receipt and bundle manifest digests are BLAKE3 of exact retained file bytes.

Regeneration requires Python 3, Git, OpenSSH `ssh-keygen`, `b3sum`, Nickel and the parent-qualified matching executable. The helper does not build a provider, change Octet refs, configure remotes or publish anything. It recovers only the clean bundle's safe initialization/update payloads, then replaces outputs after every scenario succeeds, including expected negative checkout. `provider-provenance.json` is emitted on successful current-provider capture with the actual executable BLAKE3 and minimal base inventory; a supplied source string is a binding supplied by the integrating owner, not independent source qualification.

Exact replay command, run from the Octet repository root:

```sh
python3 fixtures/cairn-cob-pilot/regenerate.py \
  --cairn /nix/store/srgd4wpm2ad3hz72iwlbv9h1wny6gibg-cairn-0.2.0/bin/cairn \
  --cairn-source-revision 2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b
```

The command above ran through the exact isolated SDK environment recorded as the first command in `capture-current-policy.log`: an empty initial environment, isolated Nix store/home/cache, disabled global/system Git configuration and SSH credential routes, pinned Nixpkgs `b3d51a0365f6695e7dd5cdf3e180604530ed33b4`, and explicit Nickel 1.17. The log retains the complete replay prefix, actual command output, and exit status 0. `capture.log` separately preserves the earlier genuine policy-compatibility failure; it is not successful capture evidence. The successful capture followed the parent-owned current policy integration (generated policy file BLAKE3 `728c4b47495c4469c2cdaa765908fd380732835f489f41a1270b60f36c4ae8e6`), without falling back to historical policy or changing exact signed Ed25519 identities. All pilot scenarios completed before output replacement: import preview/execution, non-noop signed update preview/execution, explicit local transport, secondary status, expected missing-directory negative checkout, checkout preview/execution, conflict-free inspection, and clean bundle reload. Other fixtures and parent consumer/Nix verification remain outside this pilot capture's completion claim.

Validate the fixture:

```sh
git bundle verify fixtures/cairn-cob-pilot/octet-cob.bundle
nix develop -c cargo test --test cairn_cob_pilot
nix develop -c nickel export --format json fixtures/cairn-cob-pilot/manifest.ncl \
  > target/cairn-cob-pilot-manifest.json
cmp fixtures/cairn-cob-pilot/manifest.json target/cairn-cob-pilot-manifest.json
```

This evidence proves local observations for the pinned inputs only. It does not prove Radicle runtime participation, remote publication, authentication, authorization, availability, durability, synchronization, convergence, lifecycle validity, evidence truth, source correctness, whole-program correctness, or release eligibility.
