---
name: Feature or documentation proposal
about: Propose a bounded improvement with a concrete consumer use case
---

## Before proposing

Use the canonical OnixResearch/octet issue tracker. Support and review are
best-effort, with no response SLA or implementation commitment. For security
vulnerabilities follow
[SECURITY.md](https://github.com/OnixResearch/octet/blob/main/SECURITY.md), not a
public proposal. Please complete each section; these are Markdown prompts, not
server-enforced validation. Redact private source, credentials, and consumer data.

## Octet revision and acquisition

Provide the full commit, version, and acquisition reference used to assess the
current behavior. If no command was run, say so explicitly and identify the
reviewed source revision instead. Candidate 0.1.1-rc.1 is not qualified by this
report, and the blocked octet-public channel is not an alternate support route.

## Platform and toolchain

Provide OS, architecture, Nix/toolchain identity, and any adoption constraint.
Initial release support is source-only Nix on x86_64-linux with pinned nightly
Rust/Dylint. Other platforms and direct Cargo setups are experimental.

## Command or workflow

Provide the exact existing command/workflow and prerequisites, or explain why
there is no existing command. Include minimal reproduction steps when applicable.

## Configured scope and policy

State affected crates/targets, source scope, selected profiles, relevant config,
lockfile state, and any baseline/suppression. Identify who owns the proposed
consumer decision; do not include private contacts.

## Expected result and proposed change

Describe the bounded desired behavior, consumer value, and acceptance criterion.
State what is outside the request. Do not assume a new lint or proof rail can
establish whole-program correctness, security, or release eligibility.

## Observed result and alternatives

Describe current behavior and why existing mechanisms or documentation do not
meet the need. If based on source review rather than execution, label that fact.

## Redacted evidence

Include minimal safe output, docs/source links at the reviewed revision, or
artifact excerpts supporting the proposal. If no execution evidence exists, say
so explicitly; do not fabricate observations or publish private bundles.
