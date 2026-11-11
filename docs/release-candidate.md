# Octet release candidate checklist

This document is the pre-tag release candidate receipt for Octet. It is intentionally a local verification checklist; it does not tag, publish, sign, or deploy artifacts.

## Candidate identity

- Candidate version: `0.1.0`
- Canonical Cargo package: `cargo-octet`
- Compatibility Cargo binaries: `cargo-slotcar`, `cargo-tigerstyle`
- Lint library package output: `octet` with compatibility aliases `slotcar` and `tigerstyle`

## Public release cohort

The supported public channel is the source-only export pinned at
`github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15`.
That was also the observed `main` head on 2026-11-11. `main` is a moving ref and
is not a supported pin. The reviewed cohort is `octet-public-becae962-2026-11-11`.

`standards/release/release-cohort.ncl` is the reviewed manifest. It is exported
to `standards/release/generated/release-cohort.json` and re-checked offline by
`octet_standards::release_cohort` against `tests/fixtures/release/`. Regenerate
the export and rerun the checker with:

```bash
nickel export --format json standards/release/release-cohort.ncl \
  > standards/release/generated/release-cohort.json
cargo test -p octet-standards --lib release_cohort
```

The manifest binds:

- source: public `becae9621fdf96803c7859e83d303cf535f20b15`
  (`narHash sha256-5qsxu3ncbLCNYRLdryjVQDWPwTkIZ7dQ/F561lcmKCQ=`), an
  independently rooted export derived from development
  `972b6a5a55093283e6fd3571f6f24a4da56f25ed`.
- export composition: independent git root, private settings and archival
  transcripts withheld, operator paths in ordinary guidance illustrative.
- toolchain: Rust `nightly-2026-03-21`, Nickel `1.17.0`
  (`1320a983e6c3d1e2fb53dd2464b084b4903b1426`), Dylint baseline
  `0e0a71eefe6f01563d11acc6e1d7af1d505934a9`.
- dependencies: the exact pins in `tests/fixtures/release/public-flake.lock`,
  including `cairn-public`, `valence-public`, `bounded-exec-public`,
  `onix-artifact-public`, and the three Radicle inputs. Every pin was
  retrievable anonymously at review with no private cache.
- packages: `cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet`,
  `slotcar`, `tigerstyle`, `dylint-driver`, `octet-standards`,
  `slotcar-standards`, `tigerstyle-standards`, `octet-policy-artifacts`,
  `octet-policy-export`, and `octet-assurance-profile-onixos-lifecycle`.
- notices: `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/` are required
  members of a qualified public export.
- acceptance: fresh receipts bound to one evidence digest.

### Capability inventory

The public cohort ships 77 Dylint lints. `tests/fixtures/release/public-lint-inventory-names.txt`
is the reviewed public inventory and matches `tests/fixtures/lint_inventory_names.txt`
at export origin `972b6a5a`. Capability claims are checked, not copied: the
release-cohort test rejects a manifest whose inventory names differ from the
fixture, whose source revision differs from the reviewed public revision, or
whose documented count differs.

### Qualification status: qualified for this cohort

The reviewed cohort carries `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`,
excludes `docs/standards/source/` from the public source tree while preserving
its attribution, and exports the 77-lint inventory and the guarded first-patch
walkthrough. The manifest records no blocking reasons. This is a bounded
qualification of one public source pin and its fresh receipts; it is not a legal
conclusion about the imported prose or a release tag.

### Fresh acceptance receipts for this cohort

These ran on 2026-11-11 in an isolated environment (`HOME`, `XDG_CACHE_HOME`,
and `XDG_CONFIG_HOME` under a scratch directory, `substituters` limited to
`cache.nixos.org`, empty `builders`, no credentials, no pre-populated private
cache):

| Receipt | Command | Outcome |
| --- | --- | --- |
| flake resolution | `nix flake metadata --json github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15` | resolved the pinned revision and nar hash |
| package build | `nix build ...#cargo-octet ...#octet ...#dylint-driver ...#octet-standards ...#octet-policy-artifacts --no-link --json` | all five outputs built from public inputs |
| runner version | `nix run <flake-ref>#cargo-octet -- --version` | `cargo-octet 0.1.0` |
| policy artifacts | `octet-policy-artifacts <dir>` | five generated policy artifacts plus manifest |
| walkthrough | `cargo octet check --artifact-dir artifacts/octet --output-format sarif` | finding, corrected fix, clean run |
| artifact verification | `cargo octet artifact verify --artifact-dir artifacts/octet` | `status=valid` |
| tamper rejection | same verifier on a rewritten `results.sarif` | `status=invalid`, exit 1 |

The walkthrough observed `unchecked_division` at `src/lib.rs:2`. The reviewed
public revision publishes an early `return Err(...)` fix, which still reports
the finding at `src/lib.rs:14` under that cohort. A guarded
`if completed_operations > 0` with an `else` branch produces a clean run: 0
findings, exit 0. The development walkthrough now publishes the guarded
`if`/`else` form, so this reason clears when the next public export carries that
document; the reviewed public bytes remain divergent. A consumer without a
committed `Cargo.lock` also sees exit 2, because the runner restores the lock it
generated; pre-generating the lock gives exit 0.

Historical and development receipts are non-evidence for this cohort. The
manifest lists them under `acceptance.non_evidence`; they cannot be relabelled.

The full `nix flake check -L` gate for this cohort was **not** executed. Only
the source-resolution, package-build, runner, policy-artifact, walkthrough, and
artifact-verification receipts above ran, and only for
`x86_64-linux`.

## Open-source publication prerequisites

The maintenance owner is the Octet maintainer publishing the selected revision.
These checks prepare a repeatable source release; they do not authorize a tag,
repository visibility change, binary upload, or Git history rewrite. The
existing license grant is `AGPL-3.0-or-later`, not a new relicensing of imported
material. Keep `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`
with source distributions.

Before making the repository or a release public:

- Confirm the right to distribute original contributions, including employer
  permissions where applicable. Git authorship is not a copyright assignment.
- Resolve the Adam Murray standards-prose permission entry in
  [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md). Do not publish uncertain
  imported material on the assumption that the root AGPL notice covers it.
  The qualified public export excludes `docs/standards/source/`, keeps the
  attribution, and carries `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`
  as recorded in `standards/release/generated/release-cohort.json`.
- Fetch every exact Cargo and flake input from a clean, credential-free
  environment. On 2026-10-15 anonymous requests for the pinned Valence,
  Bounded Exec, Cairn, and Cairn artifact sources returned HTTP 404. This does
  not distinguish private, missing, or renamed sources. On 2026-11-11 the
  public cohort's pins did resolve anonymously, so the check is now a
  reproducible pass for this cohort rather than an open blocker; rerun it for
  every new pin.
- Review every branch/tag intended for publication and scan reachable history
  and the final tracked tree with a redacting secret scanner. Rotate/revoke
  any real credential before considering an approved history repair. Review
  personal paths, consumer data, and historical evidence separately; a scanner
  finding no credentials is not a privacy or publication clearance.
- Enable GitHub private vulnerability reporting and verify the channel in
  [SECURITY.md](../SECURITY.md); configure branch protection, review policy,
  and available secret-scanning/push-protection settings. Do not supply private
  deploy keys to pull-request-controlled builds.

The preparation scan used Gitleaks 8.30.1:

```bash
gitleaks git --redact=100 --no-banner --log-opts=--all .
gitleaks git --staged --redact=100 --no-banner .
```

At revision `4a540dff99aa2040e267e42bbafe09144f276572`, the history scan reported
1,316 commits and no findings; the staged scan also reported no findings.
`git rev-list --all --count` counted 1,362 reachable commits; scanner and Git
counts differ, so do not substitute one for the other. Rerun against the final
candidate. Unreachable objects, reflog-only history, external LFS/attachments,
and repository hosting settings were not cleared by these scans.

### Local Nix verification limit

On 2026-10-16, `nix flake check --no-write-lock-file -L` timed out waiting
for another Nix process to finish the pinned Bounded Exec fetch. A rerun
with an isolated `XDG_CACHE_HOME`, Git credential prompts disabled, and
empty Nix access tokens exited 1 while fetching
`https://github.com/OnixResearch/bounded-exec.git` at revision
`44eb74eb639194665ee820562d5b2ce2c3ff6a94`.

That isolated run also logged an `onix-artifact` fetch over SSH while
evaluating Cairn. The root artifact flake input uses HTTPS, but that does not
prove every transitive build source uses anonymous transport. Resolve this
path and exact source availability before claiming a credential-free build.
No full Nix package build or passing full flake gate is claimed.

### Binary and Nix distribution

Publish the exact matching corresponding source and build/install inputs,
including lockfiles, manifests, generated-input sources, build scripts, and
all required non-system dependency sources. Provide equivalent source access
and directions beside downloadable binaries as required by AGPL section 6.
An embedded lint-source subset is not the complete corresponding source for
the runner and its dependencies. Modified network-interactive versions also
need the source offer required by section 13.

Inventory license/notice obligations for the actual locked Rust and native/Nix
closure before uploading binaries. Octet packages install legal files under
`share/licenses/octet/`; Dylint driver and UI-driver outputs carry their own
license directories. These files are not an exhaustive closure audit. In
particular, review notice inclusion for the separately packaged Lean/leantar
tool outputs and any other redistributed tools before a binary release.

The supported distribution paths are a full source checkout and the pinned
Nix flake. Crates.io publication is not prepared: workspace path dependencies,
Git dependencies, and the runner's repository-wide embedded build inputs need
a separate packaging decision. Do not treat `cargo package --list` as proof
that a standalone crate can be built or published.

## Required pre-tag gates

Release operators should also review `docs/trust-boundary-evidence.md` for the
controlled-deployment posture, provenance fields, evidence tiers, and explicit
scope boundary behind release evidence claims.

Run these from the repository root before tagging:

```bash
nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link
nix flake check -L
cargo octet check --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
cargo test -p octet-standards --lib release_cohort
```

The last gate revalidates the public release cohort and its negative controls;
it fails on a source/inventory mismatch, an omitted notice, unresolved imported
content presented as cleared, a warm-cache dependency, a copied or
source-mismatched receipt, changed evidence, or an unsupported profile presented
as verified.

## Install smoke commands

The release candidate smoke check verifies these installed binary surfaces from the current flake build:

```bash
nix run .#cargo-octet -- --version
nix run .#cargo-slotcar -- --version
nix run .#cargo-tigerstyle -- --version
nix build .#cargo-octet .#cargo-slotcar .#cargo-tigerstyle .#octet .#slotcar .#tigerstyle --no-link
```

For the supported public cohort, install from the pinned channel rather than a
local checkout. The command below is the copyable form; `main` is not a
supported pin.

```bash
nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- --version
nix run github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15#cargo-octet -- check
```

Expected version output:

```text
cargo-octet 0.1.0
```

## Release notes checklist

Before tagging, update release notes or the GitHub release body with:

- the candidate version (`0.1.0` for this candidate),
- the package outputs that shipped (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet`, `slotcar`, `tigerstyle`),
- the successful `release-candidate-smoke`, `nix flake check -L`, and local artifact verifier receipts,
- any known compatibility aliases or non-goals,
- the controlled-deployment and scope-boundary caveats for source/artifact locality and evidence claims.

## Acceptance signal

The candidate is ready to tag only after `checks.x86_64-linux.release-candidate-smoke` and `nix flake check -L` pass from a clean checkout and the OpenSpec evidence bundle records the exact commands and outputs.
