# Octet release candidate checklist

## Candidate and publication authority

**0.1.1-rc.1 is published**, source-only Nix on **x86_64-linux**.
<https://github.com/OnixResearch/octet> is the sole canonical source, release,
support and landing home. The immutable
[prerelease](https://github.com/OnixResearch/octet/releases/tag/v0.1.1-rc.1)
selects `4aba4019f77f95cb9c3e25ff27964898d708cef0`. Do not move or reuse
`v0.1.0` or the published candidate tag. Stable `0.1.1` is not qualified here.

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

`standards/release/release-cohort.ncl` and its generated JSON record reviewed
cohort `octet-canonical-4aba4019-2026-11-15-review`. The tested package source is
**`4aba4019f77f95cb9c3e25ff27964898d708cef0`**, with source NAR
**`sha256-vII/3vYmz19lbQ2ZSwj9c3d5tc9moLXrYp74uOAyYWQ=`**. Its 77-name
inventory, Rust toolchain and locked dependency graph bind that same revision.
The evidence identity is
`b3:eb490acfd4404d2894630505250c54f1ce43f1ab5aa0dd6b13df06201289de4b`.

| Exact-source observation | Result | Retained evidence |
| --- | --- | --- |
| New anonymous store, public substitutes only, no credentials/SSH/private builders or sibling checkouts | All five primary outputs built; actual runner reported `cargo-octet 0.1.1-rc.1` | `acquisition/` |
| Complete locked Nix input archive | All 32 inputs acquired anonymously | `acquisition/all-inputs-archive.stdout.json` |
| Fresh crate denied division, guarded fix, real bundle verification and copied-bundle tamper rejection | Required exit sequence observed; unchanged scope/config | `consumer/` |
| Public Valence `e250f498a86ac1e62266b8df749ade021a4b4043`, `valence-shared-identity-pilot --lib --locked` | 44 warnings, zero errors; artifact verification passed; tracked inputs unchanged | `consumer/bundles/valence-warning-inventory` |
| Reserved cold Mantle producer and complete Aeneas/Lean compatibility matrix, followed by the complete local flake rail | All stages exit `0`, including release-candidate smoke; no gate, deadline, theorem or warning-budget waiver | `local/full-flake.log` |
| [GitHub run 43665436695](https://github.com/OnixResearch/octet/actions/runs/43665436695) on this exact source | `core` and `verify` both successful | `github/run-43665436695.json` |
| Native structural/traceability/proposal/design/tasks gates, followed by explicitly authorized sync/archive after GitHub success | Passed; actual native lifecycle mutation receipts retained | `native-cairn/`, `native-lifecycle/` |

These paths name the source-bound public
[evidence archive](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-evidence.tar.gz),
not files implicitly present in every checkout. Its preapproval manifest remains
intentionally blocked. The separate final
[cohort](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-cohort.json)
is `qualified`, with all 18 original execution receipts unchanged. The
[owner decision](../standards/release/rc1-owner-decision.txt) records the actual
scoped interactive approval, not a model inference or cryptographic signature.

GitHub release `448037904` was published at `2026-11-17T03:00:59Z`; the API
reports `draft: false`, `prerelease: true`, `immutable: true`. All four uploaded
assets matched local bytes before publication. A subsequent credential-free
check resolved the published tag to the exact full source and NAR above,
ran its packaged command with actual `cargo-octet 0.1.1-rc.1` stdout, and
downloaded all four public assets with identical bytes. That check reused the
recorded public-only store; it is not another new-empty-store acquisition.

The archive has 198 regular files, 555291 compressed bytes, and BLAKE3
`2c786d70e4fdb57711ce8f53bc82cf5d31203eafc656f98f84f9f5fe86116209`.
See the [asset checksums](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-assets.b3).
The tagged source deliberately retains its tested prepublication docs and older
cohort; these later lifecycle, documentation and qualification bytes are not
relabeled as rebuilt package source. Earlier staging/export observations remain
historical. No announcement, outreach or stable release was approved.

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

The owner separately approved repository-local
[release immutability](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases),
and the GitHub API confirmed `enabled: true`, `enforced_by_owner: false`.
The published RC API reports `immutable: true`: its assets and associated tag
are locked, and its tag name cannot be reused after release deletion. Titles
and release notes remain editable. Existing `v0.1.0` is unchanged; changing
immutable bytes requires a newly qualified version.

The approved `main` protection is a separate post-integration operation:
up-to-date `core` and `verify` checks, one approving review, stale-approval
dismissal, enforcement for administrators, and no force push or deletion.
Retain the actual GitHub API readback when applying it; this static source
paragraph alone does not establish live hosting settings.

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
  [owner H2 decision](../.cairn/archive/1970-01-01-resolve-open-assurance-issues/evidence/rc1-import-clean-owner-approval.txt)
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

The cold Mantle producer and Aeneas/Lean compatibility matrix build first with
`--max-jobs 1 --cores 0` so their selected Mathlib compilation does not compete
with unrelated derivations on the runner. The matrix depends on the complete
live compatibility check. The full flake check follows, reusing those exact
source-bound outputs. The reviewed 45-minute producer deadlines and all proof
obligations are unchanged; a timeout remains a failed gate.

Update the cohort's inventory/toolchain/lock fixtures from the selected source,
export the Nickel manifest, and recompute its evidence identity. Source,
inventory, notice, anonymous-dependency, receipt freshness, evidence-digest,
and unsupported-profile negative controls must still reject invalid inputs.
A truthful blocked manifest can validate without being release-ready.

### Observed native Cairn gates and closure

The shipped H3 provider and generated Octet policy
`454b429ad8c48acf7144c1c42d810f0425f23867c2c83b8625af3d42a6895b4c`
passed `cairn validate --strict`, selected `octet-default` traceability, and
the proposal/design/tasks gates for `resolve-open-assurance-issues` on frozen
source `4aba4019`. These remain bounded lifecycle/declared-profile observations,
not implementation-correctness or publication authority.

After exact-source GitHub success, the owner's conditional authorization allowed
native `sync --execute` and `archive --execute` for that change. Both reported
`blocked: false`, `dry_run: false`, `mutated: true`. The nine requirements were
synced and the change moved to
`.cairn/archive/1970-01-01-resolve-open-assurance-issues/`; that directory label
is the provider's output, not the execution date. Native receipt identities:

- Sync: `d669b57b23fbf81d6a994f0341282e3dc5bb45958257201f5af2e5316aad70ca`.
- Archive: `b7e2175400750ff3a8eb2a72af60ffdca348055182508e78e6753637f8a5279b`.

This is metadata closure after testing, not a rebuilt later package source.
It does not change the earlier separate composite Onix `release-readiness`
failure caused by absent `fixtures/agent-completion/contract.json` and
`results.json`. No demonstration completion data was supplied to force that
check green, and this record does not claim it passed.

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
