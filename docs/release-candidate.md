# Octet release candidate checklist

## Candidate and publication authority

The release target is **0.1.1-rc.1**, source-only Nix on **x86_64-linux**.
<https://github.com/OnixResearch/octet> is the sole canonical source, release,
support, and landing home. The planned immutable tag is `v0.1.1-rc.1`; a planned
tag or copyable command is not evidence that it exists or is qualified. Do not
move or reuse `v0.1.0`.

The package is `cargo-octet`; `cargo-slotcar` and `cargo-tigerstyle` are existing
compatibility binaries. The lint library is `octet`, with `slotcar` and
`tigerstyle` compatibility outputs. Direct Cargo installation, crates.io,
prebuilt downloads, other platforms, and a custom website are not release
support claims.

The Octet maintainer owns candidate selection, distribution rights, gates,
source-bound receipts, support, and publication. The immediate consumer is a
Rust maintainer installing the checked path. The durable capability is the
repeatable anonymous acquisition and fail/fix/verify/tamper path, not a one-off
successful build or launch announcement.

Launch assets:

- [Positioning](gtm/positioning.md)
- [Launch plan](gtm/launch-plan.md)
- [Candidate release notes](gtm/release-notes-0.1.1-rc.1.md)
- [Announcement draft](gtm/announcement.md)
- [Outreach drafts](gtm/outreach.md)
- [Launch checklist](gtm/launch-checklist.md)

## Current evidence boundary

The RC is not qualified merely because implementation, static review, or an
older cohort succeeded. Qualification requires the actual frozen candidate,
matching producer, complete required gates, and owner decisions below.

`standards/release/release-cohort.ncl` and its generated JSON name an exact
cohort, not necessarily the current checkout. Until replaced by fresh RC
qualification, their `octet-public-becae962-2026-11-11` record is historical and
blocked. Its package/walkthrough receipts and the earlier full-core receipt do
not qualify this RC. Do not relabel those receipts, manufacture output, or turn
an expected command result into an observation.

The unqualified staging commit
`d96557fe8bea68174dc408df293d5d6aa3cda2da` passed anonymous acquisition on
2026-11-13 from a new empty store, using only `cache.nixos.org` substitutes and
no credentials, SSH, private builders/cache, or sibling checkouts. Its source
NAR identity was `sha256-hdCrd5g2SnQ/HsFMZHgZ1uy3TBeDZufKksKhz2piv0w=`.
All five primary outputs built; the matching `0.1.1-rc.1` runner completed the
fresh-crate and pinned Valence fail/fix/verify/tamper capture. This is not a
qualified release: the complete flake rail rejected stale reviewed-object
admission locks and a `0.1.0` Charon handoff projection. Replacement source
must pass those gates and receive its own source-bound qualification.

Later staging commit `24b1768d8ad990eab6a65dba179d64dd3f0fa14b` passed
the complete local flake rail and the anonymous five-output build. Its real
consumer capture passed after moving the capture directory outside an ambient
ancestor Cargo configuration; that initial refusal is retained, not bypassed.
The pinned Valence scope reported 44 warnings and no errors, with policy,
lockfile, and sources unchanged. All 32 flake inputs were also archived
anonymously. GitHub run `43483770888` passed `core` but failed `verify` when
Bubblewrap could not configure loopback (`RTM_NEWADDR: Operation not permitted`).
That host admission failure still blocks publication of a qualified RC.

The old `octet-public` repository remains unchanged by explicit owner decision.
It is blocked and unsupported, not a fallback install or reporting route. Its
reachable history and nested bundles still need separate cleanup; the owner
did not authorize a rewrite or archive. Canonical history repair does not erase
that repository, the `meta-introspector/octet` fork, Radicle replicas, GitHub
caches, old clones, or prior disclosure.

## Verified hosting settings

On 2026-11-12 the owner approved and the GitHub API confirmed, for
`OnixResearch/octet`:

- canonical description, repository homepage, and Rust/static-analysis topics;
- private vulnerability reporting enabled;
- secret scanning and secret-scanning push protection enabled;
- dependency alerts enabled;
- security-update PRs enabled and not paused.

No report was submitted and no paid security feature or legal terms were
accepted. [SECURITY.md](../SECURITY.md) defines the private reporting route and
safe fallback. API state is not a response-SLA or security guarantee.

`main` protection is approved **after verified release integration**, not claimed
already enabled: passing `core` and `verify`, an up-to-date branch, one approving
review, stale-approval dismissal, enforcement for administrators, and no force
push or deletion. Record the actual API result when applying it.

## Freeze source before acceptance

1. Finish admitted public dependency intake, the corrected Cairn provider, and
   both genuine signed COB fixture drills. Preserve exact upstream revisions,
   original manifests, licenses, notices, and Cargo checksums. Never patch a
   vendored snapshot in place or give PR-controlled builds private keys.
2. Finish implementation and native Cairn change inputs. Run Rust commands from
   the repository's Nix devshell, not an ambient Cargo installation.
3. Select an immutable candidate commit. Record its public repository, complete
   revision, NAR hash, lockfiles, toolchain, package versions, policy identities,
   and exact capability inventory. The inventory currently contains 77 Dylint
   lints; compare actual candidate bytes rather than treating this count as
   qualification.
4. Capture against that source. A later qualification/documentation commit may
   advance a branch without changing the selected source. Keep the source pin
   and its matching receipts explicit; do not claim the later commit was built
   or silently move the tag to it.

A public source branch, tag, release, external message, or history rewrite needs
its exact authorized target and values. A local gate, model review, or this
checklist is not publication authority.

## Anonymous acquisition

Prove the public path in an isolated home, XDG configuration/cache, Cargo home,
and writable target directory, with no SSH agent, credentials, Git URL rewrite,
sibling checkout, or private cache. Record allowed substituters and builders.
Using an existing private store object is not proof of anonymous acquisition.

The existing intake owner is `vendor/dependencies.ncl`; use
`tools/import-public-dependencies.py` for its exact archive/checksum projection.
Preserve historical Cargo source identities while resolving the admitted
public source bytes. Valence's reviewed module/revision and its separate
historical distribution snapshot are distinct inputs, not interchangeable
fallbacks. The packaged Cairn source must be the corrected immutable provider,
not known-broken `695124d459574ba7aeba6097310d237f393c243c`.

Build all five primary outputs from the selected public source: `cargo-octet`,
`octet`, `dylint-driver`, `octet-standards`, and `octet-policy-artifacts`. Retain
actual output paths, commands, tool versions, exits, stdout/stderr, and source
identity. A transport/acquisition failure is a blocker, not a clean check.

## Real consumer proof

Use the matching Nix-store runner and Cargo executable, and a new capture path:

```bash
python3 tools/public-release-smoke.py \
  --runner "$OCTET_RUNNER" --cargo "$OCTET_CARGO" \
  --output "$CAPTURE_DIR" --downstream valence
```

The helper also accepts `--flake` with a full immutable canonical commit and
`#cargo-octet`. Record the resolved producer/source identity; a version string
alone is insufficient. The capture must establish:

- a fresh standalone Rust crate with consumer-owned policy and a committed,
  Cargo-generated lockfile;
- an actual `unchecked_division` finding and its intentional nonzero status;
- a genuine guarded fix and the observed result for the selected lint scope;
- valid original Rust-emitted evidence and rejection of a deliberately tampered
  copy, without altering the retained originals;
- anonymous dependency acquisition and a real warning-only inventory of public
  Valence `e250f498a86ac1e62266b8df749ade021a4b4043`, using its owned
  `valence-shared-identity-pilot` library scope and unchanged policy/lockfile.

Record downstream findings, including residuals. Do not cap or suppress policy,
assert zero noise, substitute a fake workspace, or count an acquisition failure
as a finding. [The walkthrough](examples/first-checked-rust-patch/README.md)
explains lockfile and exit-code behavior. Raw logs are observations; the existing
Rust artifacts/provenance remain the receipt authority.

## Reviewed gates that cannot be mechanically waived

- **Object corpus:** the RC's exact source/reference tuple has a recorded
  [owner H2 decision](../.cairn/changes/resolve-open-assurance-issues/evidence/rc1-import-clean-owner-approval.txt)
  and [reviewed manifest](../cargo-octet/tests/fixtures/object-corpus/source-token-v2/reviewed-source-manifest.json).
  The selection preserves the earlier sources and adds the three extracted
  modules: 40 selected sources, 42 unique source/module/test comparisons, and
  four actual reference counts of 1, 1155, 1764, and 1764. Earlier RC and 0.1.0
  approvals remain historical. The named admission and tamper-control rails
  must pass for the integrated source; this decision does not approve future
  consumer-test code, correctness, lifecycle closure, or release qualification.
- **Fingerprint examples:** use actual RC output for version-bound identities
  and transcripts. Preserve deliberate warning/missing-review negative cases;
  do not hand-edit hashes or insert accepted markers to pass.
- **Self-check:** the owner approved the exact Rust-emitted pre-freeze baseline
  of 3,030 identities and lowered the occurrence expectation to 4,239, after
  disclosure of 36 changed warning contexts. The two new import findings were
  fixed rather than accepted. Configuration and complete collection scope are
  unchanged; no lint was disabled or budget expanded. This is not a frozen-source
  gate result: rerun the accepted-baseline and tamper controls after integration.
- **Cairn:** both complete signed CLI drills passed with selected provider
  `2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b` and the current generated Octet
  policy. The [pilot](../fixtures/cairn-cob-pilot/README.md) and
  [conflict drill](../fixtures/cairn-cob-conflict-pilot/README.md) retain actual
  signatures, creator/delegate authority, conflict refusal, and two-parent
  resolution. The matching pre-freeze Octet consumer/Nix fixture checks passed.
  This does not establish upstream main integration, lifecycle closure, or
  release approval.
- **Valence page:** the retained May-23 page describes its historical payload,
  not the RC. Re-rendering it with a new date/HEAD does not execute its claims.

## Required verification

After integration, run the narrow changed-surface checks and every required
repository gate, retaining actual results. The final release rail includes:

```bash
nix build .#checks.x86_64-linux.release-candidate-smoke --no-link -L
nix flake check -L
nix develop -c cargo test -p octet-standards --lib release_cohort
```

`release-candidate-smoke` runs the packaged commands through a fresh consumer
with an empty Rustup home. It asserts the denied division, guarded fix,
original Rust artifact verification, and tamper rejection, retaining command
logs and native bundles under the check output's `consumer-onboarding/`.
The driver uses the pinned Nix sysroot independently of the consumer home.
This sandboxed smoke does not establish anonymous public-source acquisition,
the separate downstream inventory, or full release qualification.

The full flake rail includes core and owner-dependent checks; a green `core`
job is not a passing full release. Follow repo-required formatting, Clippy,
policy freshness, source/reference parity, signed fixture, and native Cairn
validation/review/traceability gates. Do not remove an actual gate, change owner
policy, or substitute an incidental prose grep for behavior. Pure wording tests
are not release evidence.

The GitHub `verify` job admits user namespaces for the exact Bubblewrap
executable resolved from the locked Nix inputs. Its CI-only AppArmor profile
keeps Ubuntu's global restrictions enabled; it does not remove `--unshare-net`,
run the proof producer as root, or skip a live rail. An immediate namespace
probe fails before the full build if admission does not work. See
[Ubuntu's per-application namespace policy](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces).
Loading this profile changes only the disposable GitHub runner, not consumer
machines. A local full-flake pass does not establish that GitHub's host setup
has passed.

The cold `mantle-aeneas-consumer-job` builds first with `--max-jobs 1 --cores 0`
so its selected Mathlib compilation does not compete with unrelated derivations
on the runner. The complete flake check follows, reusing that exact source-bound
output. Its reviewed 45-minute producer deadline and all proof obligations are
unchanged; a timeout remains a failed gate.

Update the cohort's inventory/toolchain/lock fixtures from the selected source,
export the Nickel manifest, and recompute its evidence identity. Source,
inventory, notice, anonymous-dependency, receipt freshness, evidence-digest,
and unsupported-profile negative controls must still reject invalid inputs.
A truthful blocked manifest can validate without being release-ready.

### Observed local Cairn prechecks

The shipped H3 provider and generated Octet policy
`728c4b47495c4469c2cdaa765908fd380732835f489f41a1270b60f36c4ae8e6`
passed these local, pre-freeze checks:

| Command | Observed result | Boundary |
| --- | --- | --- |
| `cairn validate --strict` | `valid: true`, 93 specs, no issues | Lifecycle structure, not implementation correctness |
| `cairn traceability coverage --profile octet-default --json` | `valid: true` | Selected declared traceability profile only |
| `cairn gate proposal`, `design`, and `tasks` for `resolve-open-assurance-issues` | All three `valid: true`, no issues | Existing change gates; no lifecycle closure or publication authority |

These observations do not qualify an immutable RC. Additional read-only
readiness calls were not green: change readiness was `unavailable` without a
completion observation, and the composite `cairn release-readiness` reported
`valid: false` because this Octet checkout does not provide Cairn's
`fixtures/agent-completion/contract.json` and `results.json`. No demonstration
completion data was copied in to force acceptance, and no policy gate was
disabled. Do not describe these broader readiness results as passing.

## Legal, history, and distribution limits

The canonical history cutover at
`49c0d3ec0c3cce50449dfad13800f5ed702e7ee1` preserved 47 advertised refs, removed
98 original prohibited objects from reachability, rebuilt nested fixtures on
clean roots, migrated 72 policy citations, and passed seven focused Rust tests
and four Nix checks. That is a bounded history-repair observation, not a fresh
RC core/full-flake result. Restricted backups/ref maps remain outside published
source. Do not merge old history back in.

Keep `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, `LICENSES/`, and selected
upstream notices. The removed Adam Murray standards prose has no recorded
redistribution permission; attribution is retained, not permission inferred
from Octet's AGPL grant. Reintroduction requires the original license or written
permission. Confirm rights to original contributions, including employer
permissions where applicable.

Review every final reachable object and nested archive/bundle, not only the
working tree. Run a redacting secret scan and separately review privacy and
source rights. Preserve reviewed negative-test markers; a zero-warning scan is
not legal clearance. Real exposed credentials require rotation/revocation.

No downloadable binary or registry release is authorized by this source-only
qualification. Such a release needs its own corresponding-source, complete
locked dependency/native closure, license/notice, and packaging review. An
embedded lint-source subset is not the runner's complete corresponding source.

## Publication and adoption acceptance

Publish an approved immutable RC only after its exact source, complete gates,
real consumer captures, independent review and owner decisions are recorded.
The GitHub release body must link the actual qualification, list shipped
outputs/support limits/known issues, and use the tested command. Do not invent
an observed version output or mark an unrun gate passed.

An external pilot is a separate human adoption requirement. Obtain approval for
the exact participant/channel/message, collect actual installation and first
patch feedback, and record failures, residual warnings, scope and disposition.
Automated smoke or another model is not an external pilot. Stable `0.1.1` needs
its own frozen source, fresh qualification and actual pilot acceptance; do not
publish it by renaming the RC or reusing the RC's receipts for changed bytes.
