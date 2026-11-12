# Octet 0.1.0 public launch plan

## Objective

Publish one qualified source-only Octet release, make the first check easy to
run, and start a design-partner pipeline with teams that need reviewable Rust
release evidence.

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

- public export contains the corrected README, security, contribution, and
  pre-commit links;
- release cohort manifest and generated JSON agree;
- `release_cohort` tests pass;
- `release-candidate-smoke` passes;
- full `nix flake check -L` result is recorded or explicitly marked not run;
- clean-environment install smoke returns `cargo-octet 0.1.0`;
- private vulnerability reporting is enabled on the public repository;
- the public landing surface is the repository README, or an owned custom
  domain resolves;
- final history and secret scan is recorded for the public tree;
- release notes and announcement have a named approver.

### Phase 1: soft launch and design partners (T-0 to T+7)

- Publish the GitHub release and tag.
- Put the pinned install command at the top of the public README.
- Run the first checked patch on the public channel and save the transcript.
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
- Decide whether a second cohort, crates.io packaging, or a binary distribution
  is justified. Default to no until a consumer requires it.
- Keep the public cohort pinned. Do not silently move `main`.

## Channel plan

| Channel | Asset | Timing | Notes |
| --- | --- | --- | --- |
| GitHub release | `release-notes-0.1.0.md` | T-0 | Primary source of truth; pin the release |
| Public repository README | Corrected quick start | T-0 | Must use the pinned public revision |
| Public landing surface | Repository README using the outline in `positioning.md` | T-0 | Do not reference an unowned domain |
| Show HN | `outreach.md` | T+7 | Disclose maintainer; answer with evidence, not slogans |
| `r/rust` | `outreach.md` | T+7 | Follow community self-promotion rules |
| This Week in Rust / Rust Weekly | newsletter blurb | T+7 to T+14 | Submit once; do not resubmit the same item |
| X / LinkedIn | short thread or post | T+7 | Link the announcement, not a bare repository |
| Rust Discord / Zulip / forums | short note | T+7 | Only where project promotion is allowed |
| Design-partner email | `outreach.md` | T-3 onward | Personal, bounded ask |
| Meetups / conferences | talk abstract | T+14 onward | Prefer a demo and a boundary discussion |

## Content calendar

| Day | Asset | Owner |
| --- | --- | --- |
| T-7 | Finalize release cohort and public export | Release owner |
| T-5 | Approve positioning and release notes | Maintainer |
| T-3 | Send design-partner invitations | Founder/maintainer |
| T-1 | Clean-environment install smoke and final checklist | Release owner |
| T-0 | Tag, GitHub release, README update | Release owner |
| T+1 | Publish announcement draft | Content owner |
| T+7 | Show HN, `r/rust`, newsletters | Community owner |
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
| Imported standards prose is redistributed without permission | Legal and reputational harm | Keep `docs/standards/source/` out of the public export until permission is recorded; do not make the development repository public |
| Public README or metadata points at an unsupported selector | Users fail on first command | Regression-check every public link; use the pinned `octet-public` revision only |
| An unowned domain is referenced | Dead link and lost trust | Use the public repository README until a domain is owned; never reference `octet.dev` |
| Dylint/nightly install friction | Low activation | Lead with the pinned Nix command; mark direct Cargo as advanced |
| False-positive load | Users disable the gate | Start warning-only; baseline and promote by family; publish the Aspen bounded example |
| AGPL objection | Lost prospects | State the license plainly; do not imply permissive terms |
| Whole-program proof misreading | Credibility damage | Repeat the evidence tiers and non-claims in every launch asset |
| Security report arrives through a public issue | Disclosure risk | Enable private vulnerability reporting; test the link before launch |
| Public `main` drifts from the qualified cohort | Evidence invalidation | Pin a revision; re-qualify before moving the channel |
| Support load exceeds capacity | Maintainer burnout | Bound design-partner scope; publish support expectations; prefer issues with reproducible commands |

## Go/no-go criteria

Launch only when all are true:

- [ ] The public tree is the reviewed cohort, not an unreviewed `main` head.
- [ ] The public README install command resolves from a clean environment.
- [ ] The public repository carries `NOTICE`, `THIRD_PARTY_NOTICES.md`, and
      `LICENSES/`, and excludes the imported standards prose.
- [ ] `release_cohort` validation passes.
- [ ] `release-candidate-smoke` passes.
- [ ] The full flake-check result is recorded or explicitly marked not run.
- [ ] The artifact verifier passes on the walkthrough bundle.
- [ ] Private vulnerability reporting is enabled and tested.
- [ ] The public landing surface is the repository README, or an owned custom
      domain resolves.
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
- If a public export leaks private material, stop promotion, rotate the affected
  credential, remove the public artifact, and record the incident before
  resuming.

## Post-launch review

At T+30, answer:

1. How many independent teams reached a saved artifact bundle?
2. How many passed `artifact verify`?
3. Which onboarding step caused the most support?
4. Which claims were misunderstood?
5. Did any launch copy overstate the evidence?
6. Is a second cohort, crates.io packaging, or a binary release justified by a
   real consumer request?
7. What should the next public artifact be: a case study, a demo transcript, or
   a packaging improvement?
