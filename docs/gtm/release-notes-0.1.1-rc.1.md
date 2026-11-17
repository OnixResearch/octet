# Octet 0.1.1-rc.1 — release candidate

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. Existing Rust runs through pinned static checks, checked-in
policy and optional scoped proof rails, producing local evidence for consumer
CI and release gates. <https://github.com/OnixResearch/octet> is the sole
canonical source, release, support and landing home.

Initial support is **source-only Nix on `x86_64-linux`**, with pinned nightly
Rust and Dylint. This is a release candidate, not stable `0.1.1`, a binary
release, a hosted service or a correctness certificate.

## Install and first checked patch

The immutable candidate tag is `v0.1.1-rc.1`, selecting full source revision
`4aba4019f77f95cb9c3e25ff27964898d708cef0`. Do not substitute `main` or the
blocked historical `octet-public` export.

```bash
nix run github:OnixResearch/octet/v0.1.1-rc.1#cargo-octet -- --version
nix run github:OnixResearch/octet/v0.1.1-rc.1#cargo-octet -- check
```

Recorded packaged stdout from the **full source pin** was:

```text
cargo-octet 0.1.1-rc.1
```

Before checking, follow the
[first checked patch](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/docs/examples/first-checked-rust-patch/README.md)
and [consumer guide](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/docs/consumer-guide.md):

1. Copy the consumer-owned `dylint.toml` and configure
   `[workspace.metadata.octet]` for a bounded owned package scope and `--lib`.
2. Generate, review and commit the consumer's `Cargo.lock` before checking.
   Missing/new/changed lockfiles remain integration failures, not bootstrap.
3. Retain the denied `unchecked_division` run, apply the guarded fix under the
   same scope/config, verify the real bundles, and reject a tampered copy.
4. Inventory a real downstream's actual warnings before promoting policy;
   warning-only exit `0` does not mean zero findings.

Runner exits are `0` for clean/warning-only, `1` for error-level findings or
unaccepted baseline IDs, and `2` for integration/configuration/Cargo failures.
The capture helper requires the full immutable source revision, not a tag.

## What is included

- 77 Dylint lints across safety, naming, performance, structure, purity and
  concurrency; Rust-compatible checks without a new language or syntax.
- Consumer-owned lint/workspace policy and Nickel-authored standards.
- Local `status.json`, `summary.txt`, SARIF or JSONL findings, and
  `provenance.jsonl`; `cargo octet artifact verify` checks saved identities.
- Strict-profile controls `robust`, `bounded-systems` and `deterministic-core`;
  selecting one is not evidence that its complete assurance contract passed.
- Optional scoped proof/evidence rails, separate from the starter lint path.
- `cargo-octet`, lint library `octet`, `dylint-driver`, `octet-standards` and
  `octet-policy-artifacts` Nix outputs. Existing `cargo-slotcar`,
  `cargo-tigerstyle`, `slotcar` and `tigerstyle` remain compatibility outputs.

## Exact-source evidence

The data-only [evidence archive](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-evidence.tar.gz)
contains the approved public captures and their explicitly preapproval manifest.
The separate [final cohort](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-cohort.json)
and [owner decision](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-owner-decision.txt)
record final qualification and publication authority. The
[BLAKE3 checksums](https://github.com/OnixResearch/octet/releases/download/v0.1.1-rc.1/octet-0.1.1-rc.1-assets.b3)
bind those asset bytes, not execution truth or a cryptographic owner signature.
The tagged source retains its frozen prepublication documentation and historical
cohort; these attachments distinguish it from later qualification metadata.
Publication requires a separate owner decision, not an inference from checks.

- Source NAR: `sha256-vII/3vYmz19lbQ2ZSwj9c3d5tc9moLXrYp74uOAyYWQ=`.
- Cohort evidence identity:
  `b3:eb490acfd4404d2894630505250c54f1ce43f1ab5aa0dd6b13df06201289de4b`.
- [GitHub run 43665436695](https://github.com/OnixResearch/octet/actions/runs/43665436695)
  passed `core` and complete `verify` on the exact source above.

| Observation | Actual result and scope |
| --- | --- |
| Anonymous acquisition from a new empty Nix store | Five primary outputs built; all 32 locked inputs archived; only public substitutes, no credentials/SSH/private builders or sibling checkout |
| First checked patch and artifact checks | Denied division, guarded fix, original bundle verification and copied-bundle tamper rejection observed under unchanged scope/config |
| Independent public Valence workspace | `e250f498a86ac1e62266b8df749ade021a4b4043`, `valence-shared-identity-pilot --lib --locked`: 44 warnings, zero errors; artifacts verified and tracked inputs unchanged |
| Complete local repository rail | Reserved cold Mantle producer and complete Aeneas/Lean compatibility matrix, then full flake check, all passed; no deadline, theorem, warning-budget or gate waiver |
| Native lifecycle | Selected structural/traceability/change gates passed; separately authorized sync/archive executed after exact-source GitHub success |

The Valence capture is automated downstream evidence, **not a human external
pilot**, strict-profile admission or a universal noise guarantee. Earlier export
and staging receipts are historical, not current-source acceptance.

## Limits and known gaps

- Only source-only Nix on `x86_64-linux` is supported here. Stable-Rust-only,
  crates.io, prebuilt binaries, other platforms, a custom website and an
  Octet-operated upload service are not release paths.
- Dylint depends on pinned nightly compiler internals. Direct Cargo is an
  advanced developer path that must reproduce the same cohort.
- Lint cleanliness and artifact integrity do not prove whole-program
  correctness, certification, security scores or a consumer's release eligibility.
  Proof receipts cover only their declared property/model, source, assumptions,
  verifier and artifacts.
- The separate composite Onix `release-readiness` check was not established:
  this checkout lacks that tool's agent-completion fixture pair. Native Cairn
  closure is not a substitute or a claim that the composite passed.
- The old `octet-public` channel remains unchanged and unsupported. Canonical
  history repair does not erase forks, caches, prior clones or disclosure.
- An actual human pilot and its feedback remain outstanding. Stable `0.1.1`
  needs its own frozen source, qualification and pilot acceptance; this RC must
  not be renamed or its receipts reused for changed source.

## Support, security and license

Use <https://github.com/OnixResearch/octet/issues> for bugs and proposals.
Include full revision, platform, exact command, scope, expected/observed result,
and redacted evidence. Support is best-effort, without a response SLA, backport
commitment or historical-pin LTS promise.

For vulnerabilities use the canonical repository's private reporting route and
[security policy](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/SECURITY.md).
Private reporting, secret scanning/push protection, dependency alerts and
security-update PRs were API-verified; no exploit report was submitted as a test.
Never disclose exploit details in a public issue. If private reporting is
unavailable, use the policy's private fallback.

Octet's original material is `AGPL-3.0-or-later`. Preserve
[LICENSE](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/LICENSE),
[NOTICE](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/NOTICE),
[THIRD_PARTY_NOTICES.md](https://github.com/OnixResearch/octet/blob/4aba4019f77f95cb9c3e25ff27964898d708cef0/THIRD_PARTY_NOTICES.md)
and `LICENSES/`. Dependencies and imported material retain their own terms.
Running Octet does not relicense the code checked.
