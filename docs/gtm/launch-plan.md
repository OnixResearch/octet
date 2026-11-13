# Octet 0.1.1-rc.1 candidate launch plan

## Objective

Publish one qualified source-only Octet release, make the first check easy to
run, and start a design-partner pipeline with teams that need reviewable Rust
release evidence.

This is a plan, not evidence of qualification or completed publication.
`OnixResearch/octet` is the sole canonical source/release/support/landing home.
The planned immutable tag is `v0.1.1-rc.1`, published only after qualification;
never move or reuse the existing `v0.1.0` tag. The old `octet-public` cohort is
blocked and unsupported. Initial support is source-only Nix on
`x86_64-linux`, pinned nightly Rust/Dylint, and `AGPL-3.0-or-later`.

Success is not a star count. Success is:

- a consumer can go from repository landing page to a first checked artifact
  bundle without private help;
- the artifact verifier passes on the saved bundle;
- at least a few teams bring a real workspace into a warning-only rollout;
- inbound issues name a revision, command, scope, and expected/observed result;
- no launch claim outruns the evidence tiers.

## Non-goals

- crates.io publication;
- a binary distribution or hosted service;
- a claim of whole-program correctness, certification, or release eligibility;
- a broad "AI coding safety" claim;
- a language migration story;
- a performance or false-positive benchmark without methodology.

## Launch thesis

Rust teams do not need another generic linter. They need a checked path they can
defend when a reviewer asks what ran, against which policy, on which source
scope, and with what artifact. Octet fits that job because it starts from
existing Rust, keeps policy in source, writes local evidence, and verifies the
saved bundle before promotion.

## Message map

| Audience | Message | Proof to show | CTA |
| --- | --- | --- | --- |
| Rust platform/infra lead | "Add a defensible release gate without rewriting your workspace." | checked path, artifact verifier, exit status, profiles | Start warning-only on one crate |
| Staff engineer using agents | "Every promoted patch follows the same checked path and leaves evidence." | first checked patch, SARIF/JSONL, provenance | Run the walkthrough |
| Safety/formal-methods reviewer | "Proof rails stay scoped, optional, and separate from lint evidence." | evidence tiers, proof cards, artifact verification | Read the trust boundary |
| Engineering manager | "Adopt incrementally; gate by family; no hosted dependency." | warning budget, rollout profiles, local/CI posture | Pick a design-partner scope |
| Skeptical Rust developer | "Use Clippy too. This is a different question." | differentiation table, non-claims | Try the pinned command |

## Phases

### Phase 0: release readiness (T-7 to T-1)

Exit criteria:

- canonical sanitized source contains the corrected README, security,
  contribution, native issue templates, CI, and pre-commit links;
- exact candidate source, dependencies/toolchain, release cohort, and generated
  exports agree, with no private source acquisition prerequisites;
- required repository/lifecycle gates, strict gates, `release_cohort`, and
  `release-candidate-smoke` have real source-bound outcomes; full flake results
  and any failed/not-run checks are explicit and accepted only by authorized
  owners, never implied green by a draft;
- clean anonymous `x86_64-linux` acquisition reports actual
  `cargo-octet 0.1.1-rc.1` output and records the full immutable revision;
- standalone and independent consumer captures meet the walkthrough/helper
  contract: committed lockfile, denied finding, same-config guarded fix,
  saved JSON bundles, Rust verification, and tamper rejection;
- private vulnerability reporting and required GitHub security/protection
  settings are enabled and verified on the canonical repository;
- the canonical repository README is the landing surface; no custom website;
- final reachable-history/nested-bundle, privacy/secret, and legal review is
  recorded, with project and third-party notices preserved;
- native lifecycle completion and required reviewer/owner approvals are
  recorded for the exact candidate. Pending validation is not authority.

### Phase 1: soft launch and design partners (T-0 to T+7)

- After owner approval, publish the immutable `v0.1.1-rc.1` tag and GitHub
  prerelease from the exact qualified canonical source.
- Verify published acquisition resolves to that source and keep the canonical
  README candidate/current status honest.
- Retain the real first-patch transcript and exact source/runner identities;
  the capture helper binds a full revision, not a tag or branch.
- Invite 5-10 design partners individually. Ask for a real workspace and a
  bounded evaluation scope, not a testimonial.
- Open a pinned "launch feedback" discussion or issue with the exact revision
  and the claim boundary.
- Do not post to broad channels until the install path has survived at least one
  external reproduction.

### Phase 2: public launch (T+7 to T+14)

- Publish the long-form announcement.
- Submit a Show HN post and a `r/rust` post; disclose maintainer status.
- Submit to Rust newsletters and community roundups.
- Publish one short demo clip or terminal recording from the first checked patch.
- Ask design partners for permission before naming them. Do not imply adoption
  that did not happen.

### Phase 3: sustain (T+14 to T+30)

- Triage issues by revision, platform, scope, and command.
- Ship one small follow-up that removes a real onboarding blocker.
- Publish a launch retrospective with measured activation and support load.
- Review actual onboarding friction and support load before proposing any new
  distribution scope. Crates.io, binaries, and websites remain out of scope.
- Keep source references immutable. Do not silently move `main` into the role
  of a qualified pin or move a published tag.

## Channel plan

| Channel | Asset | Timing | Notes |
| --- | --- | --- | --- |
| GitHub prerelease | `release-notes-0.1.1-rc.1.md` | T-0 after approval | Candidate body must bind real qualification receipts |
| Canonical repository README | Candidate status and pinned command | T-0 | Sole landing home; exact source revision retained |
| Trust-boundary and walkthrough docs | Existing canonical docs | T-0 | Local artifacts, consumer-owned lockfile/policy, bounded claims |
| Show HN | `outreach.md` | T+7 | Disclose maintainer; answer with evidence, not slogans |
| `r/rust` | `outreach.md` | T+7 | Follow community self-promotion rules |
| This Week in Rust / Rust Weekly | newsletter blurb | T+7 to T+14 | Submit once; do not resubmit the same item |
| X / LinkedIn | short thread or post | T+7 | Link the announcement, not a bare repository |
| Rust Discord / Zulip / forums | short note | T+7 | Only where project promotion is allowed |
| Design-partner email | `outreach.md` | After qualified RC and exact invitation approval | Personal, bounded ask; approved participant, channel, and message |
| Meetups / conferences | talk abstract | T+14 onward | Prefer a demo and a boundary discussion |

## Content calendar

| Day | Asset | Owner |
| --- | --- | --- |
| T-7 | Finalize canonical candidate source/cohort and required gates | Release owner |
| T-5 | Approve positioning and release notes | Maintainer |
| T-1 | Clean-environment install smoke and final checklist | Release owner |
| T-0 | Approved immutable candidate tag, GitHub prerelease, README status | Release owner |
| T+1 | Send approved design-partner invitations for the qualified RC | Founder/maintainer |
| T+7 or later, after actual external reproduction | Publish the approved announcement | Content owner |
| After actual external reproduction and channel-specific approval | Show HN, `r/rust`, newsletters | Community owner |
| T+10 | First launch metrics review | Maintainer |
| T+30 | Retrospective and next-cohort decision | Maintainer |

## Metrics

These are proposed measurements. Replace every target with a baseline after the
first 30 days; do not publish a metric without its command, scope, environment,
artifact, and interpretation boundary.

| Metric | Definition | Instrumentation | Proposed first target |
| --- | --- | --- | --- |
| Activation | A consumer runs `cargo octet check` and saves an artifact bundle | Public install smoke, issue reports, optional opt-in telemetry (none today) | 10 independent runs |
| Verification | A consumer runs `cargo octet artifact verify` on a saved bundle | Issue reports, design-partner check-ins | 5 independent verifications |
| Time to first finding | Time from clone to first reviewed finding | Walkthrough transcript and design-partner notes | Under 30 minutes |
| Retention | A team runs a second check within 14 days | Design-partner check-ins | 3 teams |
| Issue quality | Inbound issue names revision, platform, command, scope, expected/observed | Issue template | 80% of issues |
| Support load | Maintainer minutes per design partner per week | Manual log | Under 60 minutes |
| Claim quality | Public copy contains no unsupported proof/security/performance claim | Launch review checklist | Zero violations |
| Pipeline | Qualified design-partner conversations | CRM or manual list | 10 conversations, 3 pilots |

GitHub stars, forks, and impressions are awareness signals only. Do not use them
as evidence of product value or release readiness.

## Roles

| Role | Responsibility |
| --- | --- |
| Release owner | Cohort, tag, GitHub release, clean-environment smoke, rollback |
| Maintainer | Claim review, legal/security decisions, issue triage |
| Content owner | Announcement, landing page, demo transcript |
| Community owner | Channel posts, moderation, design-partner follow-up |
| Security owner | Private vulnerability reporting, disclosure handling |

One person can hold several roles for this launch. Name the actual person in the
private launch tracker, not in this public document.

## Risks and mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Excluded prose/private material survives in history or bundles | Legal, privacy, and reputational harm | Audit canonical reachable/nested history independently. The owner left the legacy export unchanged; do not imply its copies were erased or reimport material without rights. |
| Public links point at the old export, moving branch, or unqualified source | Users fail on first command or trust stale evidence | Use only canonical candidate instructions after tag publication; helper evidence binds full revision |
| A website/domain is implied | Dead link and unsupported distribution scope | Canonical GitHub README is the sole landing surface for this release |
| Dylint/nightly install friction | Low activation | Lead with source-only Nix on x86_64-linux; direct Cargo is an advanced developer path |
| False-positive load | Users disable the gate | Start warning-only; baseline and promote by family; publish the Aspen bounded example |
| AGPL objection | Lost prospects | State the license plainly; do not imply permissive terms |
| Whole-program proof misreading | Credibility damage | Repeat the evidence tiers and non-claims in every launch asset |
| Security report arrives through a public issue | Disclosure risk | Enable private vulnerability reporting; test the link before launch |
| Public `main` drifts from the qualified cohort | Evidence invalidation | Pin a revision; re-qualify before moving the channel |
| Support load exceeds capacity | Maintainer burnout | Bound design-partner scope; publish support expectations; prefer issues with reproducible commands |

## Go/no-go criteria

Launch only when all are true:

- [ ] Exact canonical sanitized source/cohort is frozen and immutable, not an
      unreviewed branch or blocked old-public cohort.
- [ ] Anonymous acquisition and actual version output are retained.
- [ ] Notices, legal/privacy, reachable-history/nested-bundle, and secret review
      are complete for that source.
- [ ] Required gates and native lifecycle completion have source-bound evidence,
      with failed/not-run results explicit and required owner decisions retained.
- [ ] Standalone and independent consumer captures preserve reviewed lockfiles,
      scope/config, actual warnings, artifact verification, and tamper rejection.
- [ ] Private vulnerability reporting and required canonical GitHub protection
      settings are verified, not merely linked.
- [ ] Canonical README, support expectations, and native issue templates are
      available; no website, binaries, or crates.io launch is implied.
- [ ] Release notes and announcement have a named approver.
- [ ] Rollback owner and channel are named.

If any item fails, delay the launch. A smaller launch with accurate claims is
better than a broad launch that teaches users the evidence path is unreliable.

## Rollback and correction

- If the install path is broken, edit the release notes and README first, then
  publish a patch release or corrected cohort. Do not ask users to trust a
  moving `main`.
- If a claim is wrong, correct it in the original asset and add a dated note. Do
  not silently rewrite history.
- If a security issue appears, follow `SECURITY.md` and keep exploit details out
  of public channels until coordinated review.
- If published source leaks private material, stop promotion, follow the
  security/incident process, rotate affected credentials, and record disclosure
  limits. Retirement does not erase forks or caches; do not claim it does.

## Post-launch review

At T+30, answer:

1. How many independent teams reached a saved artifact bundle?
2. How many passed `artifact verify`?
3. Which onboarding step caused the most support?
4. Which claims were misunderstood?
5. Did any launch copy overstate the evidence?
6. What measured consumer need justifies proposing a separately scoped future
   release or packaging change? Do not add crates.io, binaries, or a website to
   this candidate by implication.
7. What should the next public artifact be: a source-bound case study, demo
   transcript, or onboarding repair?
