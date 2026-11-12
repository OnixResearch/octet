# Octet launch kit

This directory is the working set for the Octet `0.1.0` public release and
go-to-market motion. It is intentionally public-safe: it contains no private
contacts, unreleased roadmap, customer names, or unverified metrics.

## Release state

| Item | State |
| --- | --- |
| Product version | `0.1.0` |
| Supported public channel | `github:OnixResearch/octet-public` pinned at `becae9621fdf96803c7859e83d303cf535f20b15` |
| Reviewed cohort | `octet-public-becae962-2026-11-11`, status `blocked` after nested-history audit |
| Distribution | Source-only public export plus the pinned Nix flake; no crates.io or binary release |
| License | `AGPL-3.0-or-later` |
| Hosted service | None required; checks run locally or in consumer CI |
| Launch status | Blocked. Clean nested Git histories and re-qualify before publication. |

The public channel is a source-only export. The development repository is not
the supported public install path, and `main` is not a supported pin.

The earlier `core` receipt below predates history sanitization and the policy
source migration. It is not verification of this sanitized tree; this repair
reran the focused Rust, policy, and COB gates, not the full core/flake suite.

## Open launch blockers

The 2026-11-12 nested-history audit supersedes the earlier qualification claim:
both public fixture bundles retain old development history containing the
excluded Adam Murray prose and local settings. Clean the separate public
repository and publish a newly qualified revision before using these drafts.
Other hosting observations below date from 2026-11-11 unless stated otherwise.

- the public tree still points its quick start at the unsupported
  `github:OnixResearch/octet` selector; regenerate the export with this tree;
- no custom domain is owned. The public repository README is the canonical
  landing surface; do not reference `octet.dev` in metadata, release notes, or
  outreach unless ownership is established;
- the public repository has no `v0.1.0` tag or GitHub release;
- private vulnerability reporting is disabled, so the `SECURITY.md` link is not
  yet a working private channel;
- `main` is unprotected, and secret scanning/push protection are disabled;
- the public repository has no topics;
- the full `nix flake check -L` gate has not been run for this change;
  `release-candidate-smoke` passed on 2026-11-11;
- the expected-green `core` gate passed on 2026-11-12: workspace clippy, fmt,
  UI, compiler-architecture, profile, rail, smoke, and `nextest` checks all
  completed. `nextest` ran **1856 tests: 1856 passed, 0 failed, 48 skipped**.
  The repairs along the way were a coverage-provider version match, a
  standalone-build vendor copy, a missing-subcommand expectation, and the
  walkthrough's zero-guard wording;
- the pinned public install command was not rebuilt end to end in this change;
  flake resolution passed, and the reviewed cohort's `cargo-octet 0.1.0`
  receipt is recorded in `docs/release-candidate.md`;
- the final secret/history scan has not been rerun against the final export.

Both repositories are already public. The development-history repair removes
the imported prose, machine-local agent data, and unsafe embedded bundles;
it does not erase public-export, fork, or hosting-cache copies. The
`becae962` export remains blocked until its own history is repaired.

## Assets

| Asset | Use | Primary reader |
| --- | --- | --- |
| [positioning.md](positioning.md) | Message hierarchy, audience, differentiation, objections, landing-page outline | Founder, maintainer, content author |
| [launch-plan.md](launch-plan.md) | Phases, channels, metrics, risks, and go/no-go criteria | Release owner |
| [release-notes-0.1.0.md](release-notes-0.1.0.md) | GitHub release body and release-note source | Release owner, consumer |
| [announcement.md](announcement.md) | Long-form launch post or blog draft | Content author |
| [outreach.md](outreach.md) | Channel-specific copy for Show HN, Reddit, newsletters, social, and design-partner email | Community owner |
| [launch-checklist.md](launch-checklist.md) | Operational checklist with verified and open items | Release owner |

## Canonical sources

Do not invent product claims in launch copy. Use these sources:

- product entry points and scope: [`README.md`](../../README.md);
- evidence tiers and non-claims: [`docs/trust-boundary-evidence.md`](../trust-boundary-evidence.md);
- public story cards, examples, and FAQ: [`docs/adoption-story.md`](../adoption-story.md);
- consumer setup and rollout: [`docs/consumer-guide.md`](../consumer-guide.md);
- release cohort, receipts, and publication prerequisites: [`docs/release-candidate.md`](../release-candidate.md);
- lint inventory: [`docs/lint-reference.md`](../lint-reference.md).

If a sentence in this kit conflicts with one of those sources, the source wins.
The kit is copy, not evidence.

## Claim discipline

The launch is credible because the boundaries stay visible:

- say "checked path" and "reviewable evidence", not "proof of correctness";
- say "77 Dylint lints across six domains", not "77 guarantees";
- say "optional scoped formal-proof rails", not "formally verified Rust";
- say "local or consumer CI", not "hosted verification service";
- say "pinned source-only public channel", not "stable crates.io release";
- keep the Dylint/nightly tradeoff explicit;
- keep `AGPL-3.0-or-later` explicit;
- never present historical Aspen evidence as a claim about current HEAD or
  every consumer.

## How to use this kit

1. Read [positioning.md](positioning.md) before writing any external copy.
2. Use [release-notes-0.1.0.md](release-notes-0.1.0.md) for the GitHub release.
3. Use [announcement.md](announcement.md) when a long-form post is needed.
4. Use [outreach.md](outreach.md) for channel-specific variants.
5. Run [launch-checklist.md](launch-checklist.md) before publishing anything.
6. After launch, keep the release notes and checklist current. Do not rewrite
   the announcement into a different claim set.
