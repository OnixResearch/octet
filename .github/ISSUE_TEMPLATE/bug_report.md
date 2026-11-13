---
name: Bug report
about: Report a reproducible Octet installation, lint, policy, or evidence problem
---

## Before reporting

Use this public template only for non-sensitive problems in the canonical
OnixResearch/octet repository. For exploitable security details, follow
[SECURITY.md](https://github.com/OnixResearch/octet/blob/main/SECURITY.md) and
request a private route without disclosing the exploit if GitHub private
reporting is unavailable. Support is best-effort; no response SLA is promised.

Please complete every section below. These Markdown prompts request information;
they are not server-enforced validation. Redact credentials, private source,
personal paths, private receipts, and consumer data before posting.

## Octet revision and acquisition

Provide the full Octet commit, version output, and exact Nix flake reference or
local source acquisition path. Say whether this is the pending 0.1.1-rc.1
candidate or another revision. Do not treat a moving branch as a reproducible pin.

## Platform and toolchain

Provide OS, architecture, Nix version, and relevant nightly Rust/Dylint identity.
Initial release support is source-only Nix on x86_64-linux; identify experimental
platforms or direct Cargo setups explicitly.

## Exact command and reproduction

Paste the exact command, working-directory role, prerequisites, and minimal
reproduction steps. Say whether Cargo.lock was committed and remained unchanged.
Do not include proprietary source or secrets.

## Configured scope and policy

Describe selected packages/targets, Cargo arguments, relevant dylint.toml and
workspace metadata, selected profiles, and any baseline or suppression. Redact
private values without hiding the setting needed to reproduce the issue.

## Expected result

Describe the expected finding, exit status, artifact, or behavior and why.

## Observed result

Describe the actual finding, exit status, artifact, or behavior. Distinguish
integration failure from a lint finding and identify any incomplete run.

## Redacted evidence

Attach the smallest safe command output and relevant status/summary/finding or
artifact-verifier result. Include evidence scope and source identity. Do not
upload an entire private bundle, keys, credentials, or unredacted consumer source.
