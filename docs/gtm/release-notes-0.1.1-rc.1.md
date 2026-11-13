# Octet 0.1.1-rc.1 — candidate draft

**Pending qualification and publication.** This is the planned GitHub
prerelease body for immutable tag `v0.1.1-rc.1`, not a current qualified release
announcement. Do not publish until the required gates, exact source-bound
receipts, lifecycle decisions, and owner approvals are recorded in
[release-candidate.md](../release-candidate.md) and the
[launch checklist](launch-checklist.md). Do not move the existing `v0.1.0` tag.

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. Existing Rust runs through pinned static checks, checked-in
policy, and optional scoped proof rails, then produces local evidence a CI or
release gate can review. <https://github.com/OnixResearch/octet> is the sole
canonical source, release, support, and landing home.

## Candidate acquisition

Initial support is **source-only Nix on `x86_64-linux`**, with pinned nightly
Rust and Dylint. After the release owner qualifies and publishes the planned
immutable candidate tag, the evaluation commands are:

```bash
nix run github:OnixResearch/octet/v0.1.1-rc.1#cargo-octet -- --version
nix run github:OnixResearch/octet/v0.1.1-rc.1#cargo-octet -- check
```

The version must actually report `0.1.1-rc.1`; no observed version output is
claimed in this draft. Record the tag's resolved full revision. The reproducible
capture helper requires the canonical full 40-hex source revision rather than
a tag or branch. Before publication, use a reviewed local checkout and a
matching packaged producer; do not invent source identities or receipts.

The old `octet-public` repository and `becae962` cohort remain blocked and publicly
unchanged. They are not an alternate installation/support route. Historical
package, walkthrough, and pre-sanitization `core` receipts remain tied to their
original source and do not qualify this candidate.

## First checked patch

Follow the [complete walkthrough](../examples/first-checked-rust-patch/README.md)
and [consumer guide](../consumer-guide.md), rather than treating a successful
version command as onboarding evidence:

1. Copy the consumer-owned `dylint.toml` and configure
   `[workspace.metadata.octet]` for a bounded owned package scope and `--lib`.
2. Generate, review, and commit the consumer's `Cargo.lock` before checking.
   Missing/new/changed lockfiles remain integration failures (exit `2`).
3. Retain the deliberately denied `unchecked_division` run (exit `1`), apply the
   guarded fix with identical scope/config, and capture a clean selected run
   (exit `0`). These are required observations, not results claimed here.
4. Retain and verify the real JSON bundles, then reject a tampered copy with
   the Rust artifact verifier. Capture an independent consumer's actual
   warnings without broad disables, a baseline, or a zero-noise claim.

Runner exit codes are `0` for clean/warning-only, `1` for error-level findings
or unaccepted baseline IDs, and `2` for integration/configuration/Cargo failures.
A warning-only exit `0` is not a claim of zero findings.

## Highlights

- 77 Dylint lints across safety, naming, performance, structure, purity, and
  concurrency; qualification must confirm the exact candidate inventory.
- Rust-compatible checked path with no language rewrite or new syntax.
- Local `status.json`, `summary.txt`, SARIF or JSONL findings, and
  `provenance.jsonl`; `cargo octet artifact verify` checks saved identities.
- Reviewed strict profiles: `robust`, `bounded-systems`, `deterministic-core`.
- Consumer-owned lint and workspace policy plus Nickel-authored standards.
- Optional scoped proof/evidence rails, separate from the starter lint path.
- Local or consumer-CI execution; no Octet-operated upload service required.
- Canonical source/support cutover, honest best-effort support expectations,
  and native issue templates requesting reproducible, redacted reports.

## Qualification status

No candidate acceptance is established by this draft. The release owner must
bind the actual full source revision, dependency and toolchain identities,
notice/source inventory, and artifacts in the cohort and qualification record.
Required open evidence includes:

- anonymous acquisition and actual packaged runner version;
- exact integrated repository gates, including strict gates and explicit
  failed/not-run results rather than inherited historical checkmarks;
- fresh standalone and independent consumer captures, artifact verification,
  and tamper rejection;
- final reachable-history/nested-bundle, privacy, secret, and rights review;
- verified canonical GitHub security settings/private reporting route;
- native lifecycle completion and required owner/reviewer publication approval.

Keep this section pending until those real records exist. Then record their
exact source/artifact paths and outcomes; do not substitute expected output or
an older cohort's receipts. This document grants no qualification authority.

## Requirements and limits

- No crates.io publication, binary distribution, custom website, hosted
  verification service, or additional supported platform is included.
- Dylint depends on pinned nightly compiler internals; stable-Rust-only is not
  a supported setup. Direct Cargo is an advanced developer path, not the
  supported release installation.
- Preserve AGPL, project/third-party notices, upstream terms, and exact source
  provenance. Running Octet does not relicense the code checked.
- Static lint cleanliness and artifact integrity are not whole-program
  correctness, certification, security scores, or release eligibility.
- Proof receipts cover only their declared property/model, verifier, source,
  assumptions, and artifact; lint/policy evidence is not formal proof.
- Aspen phase-1 evidence is historical and scoped to its archive, not current
  HEAD or a finding/noise guarantee for other consumers.
- Support is best-effort, with no response SLA, backport commitment, or LTS
  window for historical pins. A release candidate may change before final.

## Links and reporting

- Canonical repository: <https://github.com/OnixResearch/octet>
- [First checked patch](../examples/first-checked-rust-patch/README.md)
- [Consumer guide](../consumer-guide.md)
- [Trust boundary and evidence tiers](../trust-boundary-evidence.md)
- [Release qualification record](../release-candidate.md)
- Bugs/proposals: <https://github.com/OnixResearch/octet/issues>; include full
  revision, platform, exact command, scope, expected/observed result, and
  redacted evidence. Markdown templates request fields but do not enforce them.
- [Security policy](../../SECURITY.md): canonical private vulnerability reporting
  was enabled and API-verified on 2026-11-12; see the
  [hosting observation](../release-candidate.md#verified-hosting-settings).
  If that route is unavailable, follow the policy's private fallback; never
  disclose exploit details through public issues.

## License

Octet's original material is `AGPL-3.0-or-later`. See `LICENSE`, `NOTICE`,
`THIRD_PARTY_NOTICES.md`, and `LICENSES/`. Dependencies and imported material
retain their own terms.
