# Octet 0.1.1-rc.1 launch checklist

This checklist is an owner worklist, **not a qualification receipt**. `[ ]`
means pending/not established here. Mark `[x]` only with fresh evidence for the
exact canonical candidate and a recorded reviewer decision. Historical results
cannot close candidate gates. The RC is now published; announcement and broad
outreach remain unauthorized until their own exact approvals are complete.

## 1. Source identity and legal review

- [x] Bind candidate `0.1.1-rc.1` to frozen canonical source
      `4aba4019f77f95cb9c3e25ff27964898d708cef0`, source NAR, 77-name inventory,
      locked inputs and generated cohort. The Rust offline check finds a
      consistent manifest and only the absent final owner decision blocks admission.
- [x] Record all 32 locked public inputs and actual anonymous acquisition from
      a new empty store, without credentials, SSH, private builders/cache or
      sibling checkouts. All five primary outputs built.
- [x] Record the bounded frozen-source/history/archive audit: 50 mirrored local
      refs, no intersection with the supplied prohibited identities and no
      forbidden history paths. Raw scanner exits were `1`/`1`, reviewed fixture
      markers `2`/`3`, additional findings zero; this is not zero-findings or
      legal-clearance evidence. Notices and excluded-prose boundaries remain.
- [x] Record the actual final owner confirmation of distribution rights,
      contribution/employer permissions where applicable and retained terms in
      [rc1-owner-decision.txt](../../standards/release/rc1-owner-decision.txt).
      A clean scan is not legal clearance.
- [x] Record the owner's legacy-channel decision: `octet-public` remains unchanged
      and unsupported; no rewrite or archive was authorized. Historical receipts
      remain historical. Do not route installs/support there or import its history.
- [x] Preserve existing `v0.1.0` unchanged; create the new immutable
      `v0.1.1-rc.1` once at frozen source `4aba4019`.

## 2. Candidate acquisition and required gates

- [x] Match candidate package versions, wrappers, source/toolchain/dependency
      identities and 77-name inventory to the frozen source. Later lifecycle,
      locator, documentation and receipt edits are not relabelled package source.
- [x] Run the complete required local rail, including strict gates,
      release-candidate smoke, reserved cold Mantle producer and complete
      Aeneas/Lean compatibility matrix. Actual stages exited `0`; no deadline,
      theorem, policy or warning-budget waiver.
- [x] Run the cohort unit negative controls and offline current-manifest
      consistency check. Before final owner approval, admission remains blocked.
- [x] Record exact-source GitHub run `43665436695`: `core` and full `verify`
      both succeeded. Earlier staging results remain historical only.
- [x] From a new anonymous `x86_64-linux` store, acquire/build the full canonical
      pin and retain actual `cargo-octet 0.1.1-rc.1` version output.
- [x] Limit distribution to source-only Nix; no registry/binary, hosted service,
      website or additional-platform qualification is included.

## 3. Real consumer evidence

- [x] Run `tools/public-release-smoke.py` with the matching source-bound
      packaged producer. Parent acquisition binds the full immutable source;
      `--runner` alone does not establish anonymous acquisition or source identity.
- [x] Retain fresh consumer setup, committed lockfile, and missing/new/changed
      lockfile refusals. Integration failures remain exit `2`.
- [x] Capture denied division (exit `1`), guarded fix (exit `0`, zero selected
      findings), original Rust bundles and verification (exit `0`), and copied
      bundle tamper rejection (exit `1`), with unchanged scope/config.
- [x] Capture public Valence `e250f498`, owned
      `valence-shared-identity-pilot --lib --locked`: 44 actual warnings, zero
      errors, verified artifacts and unchanged tracked inputs. No baseline,
      blanket disables, zero-noise claim or human-pilot substitution.
- [x] Keep consumer, first-patch and CI/pre-commit instructions aligned with
      the exercised setup, scope, committed lockfile, artifacts and exit behavior.
- [ ] A second person reproduces the first checked patch. Optional unless
      required by the release approval contract.

## 4. GitHub hosting and support

- [x] Observe canonical `OnixResearch/octet` description, homepage, topics,
      issues and private support/security route. The API readback is retained
      under `hosting/prepublication-readback.json`.
- [x] API-verify private vulnerability reporting, secret scanning/push
      protection, dependency alerts and enabled/unpaused security-update PRs.
      No exploit report, paid security feature or legal acceptance was submitted.
- [ ] Apply the separately approved `main` protection only after verified,
      approved release integration; record actual required checks, review,
      administrator enforcement and force-push/deletion restrictions.
- [x] Retain native issue templates requesting revision, platform, command,
      scope, expected/observed result and redacted evidence; not server enforcement.
- [x] Publish best-effort support with no SLA, backport or historical-pin LTS
      promise. The owner confirmed OnixResearch / brittonr for public triage,
      security and rollback; no private contact details are published.
- [x] Review canonical description/topics and landing links to the consumer
      guide, walkthrough, trust boundary, security policy and candidate notes.
      A website/domain remains outside this release.

## 5. Content, privacy, and owner approval

- [x] Publish the exact approved release body with source identities, receipt
      locations, actual warning counts, limits and explicit non-claims.
      All four immutable data assets matched local bytes before publication.
- [ ] Before announcement/outreach, approve each exact message and recipient
      or channel, disclose maintainer status and preserve claim boundaries.
      Prerelease publication alone never authorizes outreach.
- [x] Retain the exact-source redacted scan/audit projection, tool versions,
      scope, raw exits and reviewed fixture-marker counts. Privacy/rights and
      external erasure are separate; no universal secret-detection claim.
- [x] Obtain scoped independent model reviews, resolve the launch-kit security
      status inconsistency, and execute the separately authorized native Cairn
      sync/archive after source-bound GitHub success. Models are not authority.
- [x] Obtain the direct final exact source/tag/body/public-assets/main decision,
      including distribution rights and named triage/security/rollback owners.
      The bounded public owner record is also an immutable release asset.

## 6. Owner-only publication sequence

Steps 1–4 below are complete for this RC; the source-bound release record retains
actual API publication, anonymous tag/version and public-asset observations.
The sequence remains the replay contract for a newly qualified candidate:

1. Freeze the reviewed canonical source and matching cohort/receipt identities.
   Stop on missing or failed gates; do not claim qualification from a draft.
2. Obtain the explicit publication decision for that exact source and scope.
3. Create `v0.1.1-rc.1` once on the approved source in `OnixResearch/octet` and
   publish a GitHub **prerelease** using the approved candidate notes. Do not
   recreate, force-update, or move `v0.1.0` or any published candidate tag.
4. Resolve the published tag to the approved full revision, verify anonymous
   acquisition, and record the post-publication observation. Tag discovery
   commands may use `github:OnixResearch/octet/v0.1.1-rc.1`; reproducible helper
   evidence continues to bind the full revision.
5. Update candidate/current status only from those actual results. Publish
   approved outreach only after the required consumer reproduction and approvals.

## 7. Correction and retirement

If acquisition, evidence, or a claim is wrong, stop promotion and label the
release status accurately. Correct README/notes with a dated explanation, follow
`SECURITY.md` for exploit details, and publish a newly qualified version when
needed. Never silently move tags, reuse historical acceptance, or route users
back to the blocked `octet-public` channel. Retiring the old repository does not
erase forks, caches, or prior disclosure; record those limits honestly.
