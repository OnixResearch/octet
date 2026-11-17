# Octet launch kit

This is the public-safe working set for published candidate **0.1.1-rc.1**.
Announcement and outreach drafts still require their own exact approval. The kit
contains no private contacts, customer names or unverified activation metrics.

## Release state

| Item | State |
| --- | --- |
| Canonical source, release, support, and landing home | <https://github.com/OnixResearch/octet> |
| Candidate version | `0.1.1-rc.1` |
| Published immutable tag | [`v0.1.1-rc.1`](https://github.com/OnixResearch/octet/releases/tag/v0.1.1-rc.1), source `4aba4019f77f95cb9c3e25ff27964898d708cef0` |
| Qualification | Required technical rails, native lifecycle closure and exact owner publication approval recorded. See [release-candidate.md](../release-candidate.md). |
| Distribution | Source-only Nix on `x86_64-linux`; no crates.io or binary release |
| Toolchain | Pinned nightly Rust/Dylint; direct Cargo is an advanced developer path |
| License | `AGPL-3.0-or-later`; preserve project and third-party notices |
| Hosted service or custom website | Neither is part of this release |
| Support | Canonical GitHub issues, best-effort; no response SLA or historical-pin LTS promise |

`OnixResearch/octet` is the sole canonical home. The old `octet-public`
repository and `becae962` cohort remain blocked and unsupported, not an alternate
installation route. Do not move or reuse the existing `v0.1.0` tag.

## Evidence and open launch gates

Frozen canonical source `4aba4019f77f95cb9c3e25ff27964898d708cef0` passed
anonymous acquisition, real consumer captures, the complete local rail and
GitHub `core`/`verify` run `43665436695`. Native lifecycle closure is recorded.
The owner approved the exact source/tag/body/public-assets and bounded main
integration. The immutable prerelease is published; anonymous tag/version and
all four public asset downloads were verified. Earlier export/staging receipts
remain historical, not substituted acceptance for this source.

The canonical [hosting observation](../release-candidate.md#verified-hosting-settings)
records owner-approved, API-confirmed private vulnerability reporting, secret
scanning/push protection, dependency alerts, and security-update PRs on
2026-11-12, with a prepublication API readback on 2026-11-15. Approved branch
protection is a separate operation after verified main integration; retain its
actual readback. A dated observation is not a response SLA or security guarantee.

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
| [release-notes-0.1.1-rc.1.md](release-notes-0.1.1-rc.1.md) | Exact approved, published GitHub prerelease body; do not silently rewrite its source or evidence |
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
- Candidate commands select the published immutable tag. The capture helper
  requires the full qualified source revision, not a tag or moving branch.

## Use and approval

Read positioning and the published release body first. The scoped RC approval
does not authorize announcement or outreach: review the exact message and
recipient/channel separately. A new version or changed cohort needs new
evidence and authority; never silently move a tag.
