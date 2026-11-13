# Octet 0.1.1-rc.1 launch checklist

This checklist is an owner worklist, **not a qualification receipt**. `[ ]`
means pending/not established here. Mark `[x]` only with fresh evidence for the
exact canonical candidate and a recorded reviewer decision. Historical results
cannot close candidate gates. Do not publish a prerelease, announcement, or
broad outreach until all required gates and approvals are complete.

## 1. Source identity and legal review

- [ ] Bind candidate `0.1.1-rc.1` to the exact sanitized canonical source in
      `standards/release/release-cohort.ncl` and its generated export; validate
      contracts, fixtures, inventory, and manifest consistency.
- [ ] Record the exact dependency/toolchain/provider identities and anonymous
      public acquisition evidence, without private sibling paths, credentials,
      operator caches, or SSH access as prerequisites.
- [ ] Review every reachable object, nested Git bundle, archive, and fixture for
      excluded standards prose, private material, operator settings, and secrets.
      Preserve authorship, AGPL, `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`,
      `LICENSES/`, and upstream license terms.
- [ ] Record legal/maintainer distribution approval, including contribution and
      employer permissions where applicable. A clean scan is not legal clearance.
- [x] Record the owner's legacy-channel decision: `octet-public` remains unchanged
      and unsupported; no rewrite or archive was authorized. Historical receipts
      remain historical. Do not route installs/support there or import its history.
- [ ] Preserve the existing `v0.1.0` tag unchanged. The only planned new candidate
      tag is immutable `v0.1.1-rc.1`.

## 2. Candidate acquisition and required gates

- [ ] Match Cargo versions, Nix wrappers, embedded source, notices, policy
      inventory, cohort pins, and generated exports to the exact candidate.
- [ ] Run the required repository/lifecycle gates, including strict gates, on
      the integrated candidate. Record exact commands, revision, environment,
      exit status, artifact paths, and failed/not-run gates. Do not disable
      checks or widen a warning budget to close the checklist.
- [ ] Run `release_cohort` validation and `release-candidate-smoke` on the final
      candidate, not on historical export bytes.
- [ ] Record the required `core` and full `nix flake check -L` outcomes. Partial
      successful checks do not imply the entire gate passed. Any exception needs
      explicit authorized acceptance; this checklist grants none.
- [ ] From a clean `x86_64-linux` environment, build/run the canonical full source
      revision anonymously. Confirm the actual runner version is
      `cargo-octet 0.1.1-rc.1`; retain output rather than copying expected text.
- [ ] Confirm distribution is source-only Nix. No crates.io publication,
      prebuilt binaries, hosted service, custom website, or additional platform
      qualification is included.

## 3. Real consumer evidence

- [ ] Run `tools/public-release-smoke.py` using a matching packaged producer or
      `--flake` with the canonical full 40-hex revision and `#cargo-octet`.
      The helper rejects tags and branches; `v0.1.1-rc.1` is not its proof input.
- [ ] Retain standalone consumer setup and reviewed committed `Cargo.lock`.
      Missing/new/changed lockfiles must remain integration failures (exit `2`).
- [ ] Capture the deliberately denied `unchecked_division` result (exit `1`),
      guarded fix under identical scope/config (exit `0`, zero selected
      findings), JSON bundles, and real Rust artifact verification (exit `0`).
- [ ] Capture tamper rejection on a copy of a real bundle (verify exit `1`).
- [ ] Capture an independent downstream workspace with committed existing
      policy/config and lockfile, bounded `-p PACKAGE -- --lib --locked` scope,
      actual warnings, and artifact verification. No baseline, blanket disable,
      invented zero-noise statement, or relabelled historical receipt.
- [ ] Confirm first-patch instructions and thin CI/pre-commit integration agree
      with actual setup, scope, lockfile, artifacts, and exit behavior.
- [ ] A second person reproduces the first checked patch. Optional unless
      required by the release approval contract.

## 4. GitHub hosting and support

- [ ] Verify <https://github.com/OnixResearch/octet> is the sole canonical source,
      release, issue/support, and landing home, with no live old-public links.
- [ ] Verify branch protection, required checks, secret scanning/push protection,
      and pull-request credential boundaries on the canonical repository.
- [ ] Enable and verify canonical GitHub private vulnerability reporting and the
      `SECURITY.md` route. Its URL alone does not establish a working channel.
      Do not submit a real exploit as a test report.
- [ ] Verify native issue templates request revision, platform, exact command,
      scope, expected/observed results, and redacted evidence. Markdown fields
      request information; they are not server-enforced validation.
- [ ] Publish best-effort support expectations without an SLA, backport, or
      historical-pin LTS promise. Name triage, security, and rollback owners in
      the owner approval record; do not expose private contacts in launch copy.
- [ ] Review repository description/topics and canonical README links to the
      consumer guide, walkthrough, trust boundary, security policy, and candidate
      notes. A website/domain is outside this release.

## 5. Content, privacy, and owner approval

- [ ] Update [release-notes-0.1.1-rc.1.md](release-notes-0.1.1-rc.1.md) with exact
      qualified source identities and receipt/artifact paths, plus failed/not-run
      checks and explicit non-claims. Pending status is not approval.
- [ ] Approve announcement/outreach claim boundaries and maintainer disclosures.
      Keep AGPL, source-only `x86_64-linux` Nix, nightly/Dylint, and scoped evidence
      limits visible. Never present candidate output expectations as observations.
- [ ] Record a redacting final history/tree/archive secret scan with tool
      version, revision, command, and result. Review privacy and source rights
      separately; rotate exposed credentials before resuming publication.
- [ ] Obtain native Cairn lifecycle completion and all required independent
      reviewer, legal/security, maintainer, and release-owner approvals. Record
      approver identity, date, exact source/cohort, evidence, and go/no-go decision
      in the authorized lifecycle/owner record, not in an untracked private plan.

## 6. Owner-only publication sequence

These are ordered responsibilities, not commands this document has executed:

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
