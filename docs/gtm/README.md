# Octet launch kit

This is the public-safe working set for candidate **0.1.1-rc.1**, not an
announcement that a release is qualified or published. It contains no private
contacts, customer names, or unverified activation/performance metrics.

## Release state

| Item | State |
| --- | --- |
| Canonical source, release, support, and landing home | <https://github.com/OnixResearch/octet> |
| Candidate version | `0.1.1-rc.1` |
| Planned immutable tag | `v0.1.1-rc.1`; publish only after qualification |
| Qualification | Pending; real source identities, receipts, and approvals belong in [release-candidate.md](../release-candidate.md) |
| Distribution | Source-only Nix on `x86_64-linux`; no crates.io or binary release |
| Toolchain | Pinned nightly Rust/Dylint; direct Cargo is an advanced developer path |
| License | `AGPL-3.0-or-later`; preserve project and third-party notices |
| Hosted service or custom website | Neither is part of this release |
| Support | Canonical GitHub issues, best-effort; no response SLA or historical-pin LTS promise |

`OnixResearch/octet` is the sole canonical home. The old `octet-public`
repository and `becae962` cohort remain blocked and unsupported, not an alternate
installation route. Do not move or reuse the existing `v0.1.0` tag.

## Evidence and open launch gates

The earlier public-export package/walkthrough receipts and pre-sanitization
`core` observations are historical/source-bound only. They are not acceptance
of this candidate. Hosting observations about the old repository are not facts
about canonical repository settings. Private vulnerability reporting, branch
protection, secret scanning, and push protection must be verified by the owner;
this kit does not assert that they are enabled.

Follow [launch-checklist.md](launch-checklist.md). Before publication, bind the
candidate source and dependencies, qualify clean anonymous acquisition, run the
required gates and consumer captures, review legal/privacy and nested histories,
verify GitHub security settings, and obtain required lifecycle and owner
approvals. Pending validation is not completion authority. Never turn an open
gate into a checkmark based only on this documentation.

## Assets

| Asset | Use |
| --- | --- |
| [positioning.md](positioning.md) | Message hierarchy, audience, differentiation, and non-claims |
| [launch-plan.md](launch-plan.md) | Phases, proposed metrics, responsibilities, and go/no-go criteria |
| [release-notes-0.1.1-rc.1.md](release-notes-0.1.1-rc.1.md) | Candidate GitHub prerelease body; qualification remains pending |
| [announcement.md](announcement.md) | Long-form draft; publication requires approval |
| [outreach.md](outreach.md) | Channel-specific drafts with canonical candidate references |
| [launch-checklist.md](launch-checklist.md) | Operational release gates and owner-only sequence |

## Canonical sources

Do not invent claims in launch copy. Use:

- entry points and scope: [README.md](../../README.md);
- evidence tiers and non-claims: [trust-boundary-evidence.md](../trust-boundary-evidence.md);
- consumer setup: [consumer-guide.md](../consumer-guide.md) and the
  [first checked patch](../examples/first-checked-rust-patch/README.md);
- scope promotion: [downstream-rollout.md](../downstream-rollout.md);
- real cohort pins, receipts, and publication prerequisites:
  [release-candidate.md](../release-candidate.md);
- licensing and reporting: [CONTRIBUTING.md](../../CONTRIBUTING.md) and
  [SECURITY.md](../../SECURITY.md).

The kit is copy, not evidence. If it conflicts with the exact source-bound
qualification record, correct the copy rather than relabelling the evidence.

## Claim discipline

- Say "checked path" and "reviewable evidence", not "proof of correctness".
- Say "77 Dylint lints across six domains", not "77 guarantees"; validate the
  exact candidate inventory during qualification.
- Keep optional scoped proof rails separate from lint/policy evidence.
- Keep source-only `x86_64-linux` Nix support, nightly/Dylint, and AGPL explicit.
- No website, binary, crates.io, hosted-service, certification, security-score,
  performance, or whole-program correctness claim is authorized.
- Historical Aspen evidence remains bound to its archived source and scope,
  never current HEAD or a universal noise guarantee.
- Candidate commands describe the planned tag after qualification/publication.
  The capture helper requires a full immutable source revision, not a tag.

## Use and approval

Read positioning first, then update the release-note draft with real candidate
receipts and exact artifact paths. Review the announcement and relevant outreach
variant. Finish the checklist and record named owner approvals before publishing.
A new version or changed cohort needs new evidence; do not silently move a tag.
