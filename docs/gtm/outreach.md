# Outreach copy

Use these variants only after [launch-checklist.md](launch-checklist.md) is
green. Disclose that you maintain Octet. Do not ask for votes, stars, or
upvotes. Do not astroturf a channel or post the same text repeatedly.

Replace `<PINNED_URL>` with:

```text
https://github.com/OnixResearch/octet-public/blob/becae9621fdf96803c7859e83d303cf535f20b15
```

Replace `<SITE_URL>` with an owned live product site, or omit it while the
repository README is canonical. Do not reference an unowned domain such as
`octet.dev`.

## Show HN

**Title**

```text
Show HN: Octet – evidence-backed checks for Rust systems code
```

**Body**

```text
I maintain Octet, a checked path for Rust systems code. It runs pinned Dylint
lint families and checked-in policy, then writes local evidence that a CI or
release gate can review.

The problem I kept running into: a green build and a wall of lint output do not
tell a reviewer what was checked, against which policy, on which source scope,
or whether the saved artifacts still verify.

Octet runs on existing Rust without a rewrite. A check can write status.json,
summary.txt, results.sarif or results.jsonl, and provenance.jsonl to a local
artifact directory. `cargo octet artifact verify` recomputes the saved sidecar
identities and fails closed on stale or malformed evidence.

The public release is source-only and pinned through Nix:
  nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version

What it is not: a new language, a Rust replacement, a whole-program verifier, a
certification, or a hosted service. It is `AGPL-3.0-or-later`, and it currently
uses Dylint plus a pinned nightly toolchain. Direct Cargo use is an advanced
path; Nix is the supported public install.

I would value feedback from people who maintain a real Rust release gate:
- Does the checked path match how you review patches?
- Is the artifact bundle useful, or is it more evidence than you want?
- Where does the install or setup break?

Docs: <PINNED_URL>/README.md
Trust boundary: <PINNED_URL>/docs/trust-boundary-evidence.md
```

**First comment**

```text
Maintainer here. A few boundaries up front so the discussion is useful:

- The default path is static lint and policy evidence, not proof of correctness.
- Optional Verus/Lean/Kani rails exist, but they are scoped and opt-in.
- The public cohort is source-only; there is no crates.io or binary release yet.
- The full flake check was not claimed for the reviewed cohort; the release
  checklist names exactly which receipts ran.
- Starter noise is real. The archived Aspen phase-1 rollout produced six
  ambient_clock findings on three starter packages. That is bounded historical
  evidence, not a universal noise guarantee.

Happy to answer questions about the evidence format, Dylint tradeoff, or the
release boundaries.
```

## r/rust

**Title**

```text
Octet 0.1.0: an evidence-backed checked path for Rust systems code
```

**Body**

```text
I maintain Octet, a Rust-compatible checked path for teams that need a release
gate to point at reviewable evidence.

It applies 77 Dylint lints across safety, naming, performance, structure,
purity, and concurrency; reads checked-in `dylint.toml` and workspace policy;
and writes local artifacts (`status.json`, SARIF or JSONL, provenance, and a
summary). A separate command verifies the saved bundle and fails closed on
stale or inconsistent evidence.

It is not a new language or a whole-program verifier. It is also not a
replacement for Clippy, Miri, Kani, or Verus. The claim is narrower: the
selected checks ran, against the selected policy and scope, and the evidence can
be reviewed.

The supported public channel is source-only and pinned through Nix:
  nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version

The project is `AGPL-3.0-or-later` and uses Dylint plus a pinned nightly
toolchain. Direct Cargo use is documented as advanced.

I am especially interested in feedback from people who maintain CI gates for
Rust systems code: is the artifact model useful, and where does setup get in the
way?

Repository: <PINNED_URL>
Trust boundary: <PINNED_URL>/docs/trust-boundary-evidence.md
```

## This Week in Rust / Rust Weekly

**Submission blurb**

```text
Octet 0.1.0 is an evidence-backed checked path for Rust systems code. It runs
77 Dylint lints across six domains plus checked-in policy, writes local SARIF or
JSONL evidence with provenance, and verifies the saved artifact bundle before a
CI or release gate uses it. The public channel is a pinned source-only export
and the project is AGPL-3.0-or-later. Optional proof rails stay scoped; the
default path does not claim whole-program correctness.
<PINNED_URL>
```

## X / Twitter thread

**Post 1**

```text
Rust review is the bottleneck.

A green build does not tell a reviewer what ran, against which policy, or
whether the evidence still verifies.

Octet 0.1.0 is an evidence-backed checked path for existing Rust.
<PINNED_URL>
```

**Post 2**

```text
The path:

Rust source
-> cargo octet check
-> checked-in policy
-> evidence artifacts
-> artifact verification
-> CI or release gate

77 Dylint lints, six domains, three strict profiles.
```

**Post 3**

```text
A run can write:

- status.json
- summary.txt
- results.sarif or results.jsonl
- provenance.jsonl

Then `cargo octet artifact verify` recomputes the saved identities and fails
closed on stale or malformed evidence.
```

**Post 4**

```text
What it is not:

- not a new language
- not a Rust replacement
- not a whole-program verifier
- not a certification
- not a hosted service

Optional proof rails are scoped and opt-in.
```

**Post 5**

```text
Public channel is source-only and pinned:

nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version

AGPL-3.0-or-later. Dylint plus a pinned nightly toolchain. Nix is the supported
public install; direct Cargo is advanced.
```

## LinkedIn

```text
Rust teams are reviewing more patches than ever, including patches produced with
coding agents. The hard question is not "did the build pass?" It is "what did we
check, against which policy, on which scope, and can a reviewer verify the
evidence?"

Octet 0.1.0 is an evidence-backed checked path for existing Rust systems code.
It runs 77 Dylint lints across six domains plus checked-in policy, writes local
SARIF/JSONL evidence with provenance, and verifies the saved artifact bundle
before a CI or release gate uses it.

It is not a new language, a whole-program verifier, a certification, or a hosted
service. The default path is static and policy evidence; optional proof rails
stay scoped and opt-in.

The public channel is a pinned source-only export and the project is
AGPL-3.0-or-later.

Repository: <PINNED_URL>
```

## Design-partner email

**Subject**

```text
A bounded Rust release-evidence pilot?
```

**Body**

```text
Hi <name>,

I maintain Octet, an evidence-backed checked path for Rust systems code. I am
looking for a small number of teams with a real release gate to test it on one
crate, warning-only, with a bounded scope.

The evaluation would be:

1. Run the pinned public channel against one crate.
2. Save the artifact bundle and verify it.
3. Review the findings with you and decide whether the evidence is useful.
4. Stop there. No migration commitment and no public case study without your
   approval.

What I need from you: a crate, its current CI command, and 30 minutes to review
the first result. What you get: a bounded read on whether the checked path fits
your release review, plus a direct channel for setup problems.

The project is AGPL-3.0-or-later and currently uses Nix, Dylint, and a pinned
nightly toolchain. If that is a hard blocker, tell me and I will not push it.

<name>
<PINNED_URL>
```

## Warm-introduction note

```text
<name> suggested I reach out. I maintain Octet, a checked path that turns Rust
lint and policy runs into reviewable local evidence for a release gate. I am
looking for one or two teams willing to run it warning-only on a single crate.
No migration commitment; the goal is a bounded read on whether the evidence is
useful. Would that be worth 30 minutes?
```

## Meetup / conference talk abstract

**Title**

```text
From Green Build to Reviewable Evidence: A Checked Path for Rust Releases
```

**Abstract**

```text
A green build is a weak release argument. Reviewers still need to know what ran,
against which policy, on which source scope, and whether the saved evidence
still verifies.

This talk walks through an evidence-backed checked path for existing Rust: pinned
static checks, checked-in policy, local artifact bundles, provenance, and
artifact verification. It shows a first checked patch, a clean and a failing
run, and the boundary between static lint evidence, policy evidence, and scoped
formal-proof receipts.

The goal is not another linter pitch. It is a practical model for making release
claims smaller, inspectable, and harder to overstate.
```

**Takeaways**

- A release gate should point at an artifact, not a terminal memory.
- Static lint evidence, policy evidence, and formal proof are different tiers.
- Existing Rust can adopt a checked path without a language rewrite.
- The Dylint/nightly tradeoff and the AGPL license are real adoption constraints;
  name them early.

## GitHub launch discussion prompt

```text
After the blocked cohort is replaced and re-qualified, invite users to try
Octet 0.1.0 and include:

- the pinned public revision;
- platform and Nix version;
- the exact command and configured scope;
- expected and observed behavior;
- whether `cargo octet artifact verify` passed.

Please keep vulnerability details out of public issues and use the security
policy instead.
```

## Follow-up sequence

| Timing | Action |
| --- | --- |
| T+1 | Thank early testers and ask for the exact command, not a general impression |
| T+3 | Fix or document the first reproducible onboarding blocker |
| T+7 | Publish a short "what we learned" note only if it has measured evidence |
| T+14 | Ask design partners whether a second session is useful; do not ask for a testimonial |
| T+30 | Close the loop with every design partner, including a "not a fit" outcome |
