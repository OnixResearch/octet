# Octet 0.1.1-rc.1 outreach drafts

**Not approved for publication.** Use these variants only after the required
[launch checklist](launch-checklist.md), lifecycle, and owner approvals. The
planned immutable candidate tag is `v0.1.1-rc.1`; candidate qualification and
publication are not established here. Do not move or reuse `v0.1.0`.

Disclose that you maintain Octet. Follow each channel's self-promotion rules;
do not ask for votes, stars, or repeat-post the same announcement. All links
use the sole canonical source/release/support home, `OnixResearch/octet`.
The old `octet-public` cohort remains blocked and unsupported, not an alternate
route. No custom website/domain is part of this release.

## Show HN

**Title**

```text
Show HN: Octet – reviewable evidence for Rust checks
```

**Body**

```text
I maintain Octet, an evidence-backed checked path for agent-written and
human-written Rust systems code. It runs pinned Dylint checks and checked-in
policy, then writes local evidence a CI or release gate can review.

A check can write status.json, summary.txt, SARIF or JSONL findings, and
provenance.jsonl. A separate artifact verifier checks saved identities; it does
not turn lint cleanliness into proof of whole-program correctness.

The 0.1.1-rc.1 candidate targets source-only Nix on x86_64-linux, with pinned
nightly Rust/Dylint and AGPL-3.0-or-later licensing. There is no crates.io,
prebuilt binary, custom website, or required hosted verification service.
Optional proof rails remain scoped and separate from starter lint evidence.

Start with one owned crate, a reviewed committed Cargo.lock, and the first
checked patch walkthrough. Capture actual warnings and artifacts rather than
assuming a warning-only exit means zero findings. I would value feedback on
setup friction and whether the saved evidence helps your review.

Canonical source, qualification status, and walkthrough:
https://github.com/OnixResearch/octet
Trust boundary:
https://github.com/OnixResearch/octet/blob/v0.1.1-rc.1/docs/trust-boundary-evidence.md
```

**First comment**

```text
Maintainer here. The claim boundaries matter:

- Static lint/policy evidence is not a correctness, security, or certification claim.
- Optional proof receipts cover only their declared property and source/model.
- Candidate evidence must bind the actual full source revision and toolchain;
  old public-export receipts do not qualify this source.
- Historical Aspen rollout results are scoped archive observations, not a
  guarantee about current Aspen or another consumer's finding count.
- Support is best-effort, with no response SLA or historical-pin LTS promise.

Please report redacted reproduction details at:
https://github.com/OnixResearch/octet/issues
For vulnerabilities, use SECURITY.md and do not disclose exploits publicly.
```

## r/rust

**Title**

```text
Octet 0.1.1-rc.1: a checked path from Rust source to reviewable evidence
```

**Body**

```text
I maintain Octet, a Rust-compatible checked path for teams that want a CI gate
to point at inspectable evidence rather than only a terminal log.

It combines 77 Dylint lint checks across six domains with checked-in policy,
local artifact bundles, and an artifact verifier. It is not a new language,
a Rust replacement, or a whole-program verifier. Use Clippy, Miri, and scoped
verifiers for their own questions; Octet does not replace them.

The 0.1.1-rc.1 candidate is source-only Nix on x86_64-linux, with pinned nightly
Rust/Dylint and AGPL-3.0-or-later licensing. Direct Cargo is an advanced developer
path, not the supported release install. Candidate qualification status and
source-bound receipts live in the canonical repository.

Start with the walkthrough: commit the reviewed consumer lockfile, capture a
deliberately denied unchecked division, apply the guarded fix under identical
scope/config, verify the saved bundles, and reject a tampered copy. Then try one
owned package in an independent workspace without hiding actual warnings.

Repository and current qualification status:
https://github.com/OnixResearch/octet
Walkthrough after candidate tag publication:
https://github.com/OnixResearch/octet/blob/v0.1.1-rc.1/docs/examples/first-checked-rust-patch/README.md
```

## This Week in Rust / Rust Weekly

```text
Octet 0.1.1-rc.1 is a candidate checked path for existing Rust systems code:
pinned Dylint lint checks, checked-in policy, local findings/provenance, and
artifact verification. It targets source-only Nix on x86_64-linux with pinned
nightly Rust and AGPL-3.0-or-later licensing. Proof rails are optional/scoped;
static cleanliness is not whole-program correctness. See the canonical
repository for qualification status and the first checked patch:
https://github.com/OnixResearch/octet
```

## Short social post

```text
I maintain Octet. The 0.1.1-rc.1 candidate checks existing Rust with pinned lint
and policy inputs, then writes local evidence your CI/reviewer can inspect.
Source-only Nix, x86_64-linux, nightly/Dylint, AGPL-3.0-or-later. Bounded claims,
not whole-program proof. Qualification status and walkthrough:
https://github.com/OnixResearch/octet
```

## Professional-network post

```text
I maintain Octet, a checked path for agent-written and human-written Rust.
The review question is not only "did the build pass?" It is "which checks ran,
against which policy and scope, and can we verify the saved evidence?"

The 0.1.1-rc.1 candidate combines pinned Dylint checks, checked-in policy, local
SARIF/JSONL evidence and provenance, and an artifact verifier. Initial support
is source-only Nix on x86_64-linux with pinned nightly Rust/Dylint and
AGPL-3.0-or-later licensing. No hosted upload service is required.

It does not establish whole-program correctness, certification, or release
eligibility. I am interested in bounded evaluations on one owned crate, with
actual findings and an inspectable artifact bundle—not a testimonial or a
zero-noise promise. Support is best-effort.

Canonical source, qualification status, and walkthrough:
https://github.com/OnixResearch/octet
```

## Design-partner invitation

**Subject:** A bounded Rust release-evidence evaluation?

```text
I maintain Octet, an evidence-backed checked path for Rust systems code. I am
looking for teams willing to evaluate the 0.1.1-rc.1 candidate on one owned crate
with an existing release gate, committed policy/config, and reviewed lockfile.

The evaluation is bounded: run the pinned canonical source, retain actual
findings, verify the saved bundle, and decide whether the evidence is useful.
No migration commitment and no public case study or naming without permission.
No baseline or blanket lint disable will be used to present a cleaner result.

The supported candidate setup is source-only Nix on x86_64-linux, pinned
nightly Rust/Dylint, and AGPL-3.0-or-later. If those constraints do not fit, this
is not the right cohort. Review is best-effort, with no response SLA.

Would a short review of one scoped result be useful? Please do not send private
source or receipts through a public issue. Qualification status and setup:
https://github.com/OnixResearch/octet
```

Personalize greetings and signatures outside the published draft; no private
recipient or contact list belongs in this repository.

## Meetup / conference abstract

**Title:** From Green Build to Reviewable Evidence: A Checked Path for Rust

```text
A green build is a weak release argument. Reviewers still need to know what
ran, against which policy, on which source scope, and whether the saved
artifacts retain their identity.

This talk demonstrates a bounded checked path for existing Rust: pinned static
checks, checked-in policy, local artifact bundles, provenance, and artifact
verification. It distinguishes lint findings, policy validation, and optional
scoped proof receipts. A real first-patch capture supplies the example; no
historical export receipt is relabelled as a fresh candidate run.

The goal is smaller, inspectable claims—not a whole-program verification pitch.
The source-only x86_64-linux Nix setup, nightly/Dylint tradeoff, and AGPL license
are explicit adoption constraints.
```

## GitHub feedback prompt

```text
What happened on your first bounded Octet check? Please use the canonical issue
template and provide full revision, platform, command, configured scope,
expected/observed result, and redacted evidence. State any incomplete run or
lockfile failure rather than reporting it as lint cleanliness.

https://github.com/OnixResearch/octet/issues

Support is best-effort. For vulnerabilities, follow SECURITY.md; do not post
exploit details or private artifacts publicly.
```
