# Contributing to Octet

Read the [contributor guide](docs/contributor-guide.md) for the source map,
Nix environment, and focused verification commands. Read the
[README](README.md) for supported entry points and evidence boundaries.

## Changes

- Open an issue for a bug or proposal at
  <https://github.com/OnixResearch/octet/issues>. Use the issue template and
  include the full pinned Octet revision, platform, exact command, configured
  scope, expected and observed result, and a small reproducible example with
  redacted evidence.
- Keep pull requests focused. Explain the consumer-visible behavior, the
  exercised checks, and any remaining limits. Update affected docs and the
  changelog; preserve generated-file and lifecycle conventions in the guide.
- Run Rust commands inside `nix develop`. Do not use private sibling checkouts,
  credentials, or local caches as evidence that a public checkout works.
- Do not include credentials, proprietary consumer source, raw private
  receipts, or machine-specific permissions. Use [SECURITY.md](SECURITY.md)
  rather than a public issue for exploitable security details.

## Support expectations

`OnixResearch/octet` is the sole canonical source, release, and support home.
The old `octet-public` channel remains blocked and unsupported, not an alternate
support route. Initial release support is source-only Nix on `x86_64-linux`,
with the pinned nightly Rust/Dylint cohort. Direct Cargo use and other platforms
are developer experiments, not supported release installations.

Support and contribution review are best-effort, with no promised response SLA,
backport policy, or long-term support window for historical pins. Candidate
`0.1.1-rc.1` is pending qualification; do not describe candidate commands or
historical receipts as a qualified release. Review the
[release checklist](docs/release-candidate.md) before making release claims.


## Contribution license

Submit original contributions under `AGPL-3.0-or-later`, the existing project
license in [LICENSE](LICENSE). You retain your copyright. Submit only material
you have the right to contribute, including any required employer permission.

For third-party material, preserve its original license and attribution,
identify the immutable upstream revision and local modifications, include the
required license texts, and update [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
Do not present an upstream author's work as an original Octet contribution.
