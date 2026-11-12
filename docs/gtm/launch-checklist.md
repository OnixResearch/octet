# Launch checklist

Status values:

- `[x]` verified in the repository or observed directly;
- `[ ]` open, not run, or owner action required;
- `[~]` partially verified; the note names what remains.

Do not publish the release, announcement, or broad outreach until every hard
gate below is `[x]`. Soft or optional items are marked.

## 1. Release identity and legal

- [x] Version is `0.1.0` in the Cargo manifests.
- [x] Project license is `AGPL-3.0-or-later`.
- [x] `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/` are required members of
      the qualified public export.
- [x] Imported standards prose is excluded from the qualified public export and
      retained only in the attribution inventory.
- [x] The public cohort manifest is `standards/release/release-cohort.ncl` and
      its generated export is `standards/release/generated/release-cohort.json`.
- [x] The manifest and generated export now report the `qualified` state and
      agree on the public revision and 77-lint inventory.
- [ ] Legal/maintainer: if the development repository will also become public,
      recover and record the upstream source and license, or written permission,
      for the Adam Murray standards prose before changing visibility.
- [ ] Legal/maintainer: confirm the right to distribute original contributions,
      including employer permissions where applicable.
- [ ] Release owner: decide whether to publish source only or also publish
      binaries. The current recommendation is source only.

## 2. Public channel

- [x] `OnixResearch/octet-public` is publicly visible.
- [x] `main` resolved to `becae9621fdf96803c7859e83d303cf535f20b15` during the
      2026-11-11 audit.
- [x] The reviewed cohort is `octet-public-becae962-2026-11-11`.
- [x] The public channel is source-only; direct Cargo publication is not
      supported.
- [x] Export-transform pitfalls identified: `.envrc`, `.cairn/archive/**`,
      `openspec/changes/**/evidence/**`, `docs/standards/source/**`,
      `standards/release/**`, `tests/fixtures/release/**`,
      `crates/valence-core/**`, and
      `crates/octet-standards/src/release_cohort.rs` are withheld; private
      Cargo/flake URLs become the public mirrors. The public tree also carries
      selected later changes, so a plain archive copy is wrong.
- [ ] Public export: regenerate the export from the development tree that
      contains this launch kit and the corrected README/metadata links.
- [ ] Public export: verify the README quick start uses only the pinned
      `octet-public` revision and never `main` or `github:OnixResearch/octet`.
- [x] The current public tree at `becae962` uses the HTTPS
      `valence-public` and `bounded-exec-public` mirrors in
      `cargo-octet/Cargo.toml` (observed 2026-11-11).
- [ ] Public export: re-verify the HTTPS public mirrors after the new export is
      generated.
- [x] The current public tree at `becae962` carries `NOTICE`,
      `THIRD_PARTY_NOTICES.md`, and `LICENSES/` (observed 2026-11-11).
- [x] The current public tree at `becae962` has no `docs/standards/source/`
      path (observed 2026-11-11).
- [ ] Public export: re-verify the notices and the excluded prose after the new
      export is generated.
- [ ] Release owner: re-pin the cohort to the new public revision and regenerate
      `release-cohort.json`.
- [ ] Release owner: run the fresh acceptance receipts for the new revision.
- [ ] Release owner: create the `v0.1.0` tag and GitHub release only after the
      go/no-go review.
- [ ] Maintainer: enable branch protection. `main` was unprotected when observed
      on 2026-11-11.
- [ ] Maintainer: enable secret scanning and push protection. Both were disabled
      when observed on 2026-11-11.
- [ ] Maintainer: enable private vulnerability reporting. It was disabled when
      observed on 2026-11-11, so the `SECURITY.md` link is not yet a working
      private channel.
- [ ] Maintainer: verify the public repository has no stale `github:OnixResearch/octet`
      install or issue links after the export.

## 3. Install and evidence smoke

- [x] The development README now shows the pinned public channel instead of the
      unsupported `github:OnixResearch/octet` selector.
- [x] `CONTRIBUTING.md` and `SECURITY.md` now point at the public repository.
- [x] The consumer pre-commit example and
      `templates/pre-commit-config.yaml` now point at the public repository.
- [x] The trust-boundary controlled-deployment text now names the public pin path.
- [x] Cargo `repository` and `homepage` metadata now point at the public
      repository.
- [x] Run `cargo test -p octet-standards --lib release_cohort` after
      regenerating the cohort export. All three tests passed in an isolated
      target directory.
- [x] Pre-commit template assertions passed for the public repository, revision
      placeholder, hook id, and optional args.
- [x] `nix flake metadata --json github:OnixResearch/octet-public/becae9621fdf96803c7859e83d303cf535f20b15`
      resolved the pinned revision and nar hash from a clean environment.
- [x] Run `nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link`.
      Passed on 2026-11-11 after overriding the local Nix configuration:
      `NIX_CONFIG=$'substituters = https://cache.nixos.org\nbuilders =\nmin-free = 0\nmax-free = 0'`.
      A first attempt failed in the broken remote build hook before the check
      ran; that was an environment failure, not a source failure.
- [ ] Run `nix flake check -L` and record the result, including any known
      non-blocking failures.
- [~] Run the pinned public channel in a clean environment:
      `nix run github:OnixResearch/octet-public/<pinned-revision>#cargo-octet -- --version`.
      The full local build did not finish in this verification window; the
      reviewed cohort's fresh `cargo-octet 0.1.0` receipt is recorded in
      `docs/release-candidate.md`. Rerun before tagging.
- [~] Run the first checked patch walkthrough on the pinned public channel and
      save the transcript. The reviewed cohort's walkthrough receipt is
      recorded; rerun before tagging.
- [~] Run `cargo octet artifact verify --artifact-dir artifacts/octet` on the
      walkthrough bundle. The reviewed cohort's `status=valid` receipt is
      recorded; rerun before tagging.
- [~] Run the tamper-rejection check on a rewritten sidecar and confirm
      `status=invalid`, exit 1. The reviewed cohort's receipt is recorded; rerun
      before tagging.

## 3a. Code quality gates

- [x] `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`.
      The first `core` attempt exposed 72 `cargo-octet` pedantic warnings.
      Mechanical fixes and scoped allows are applied, and the workspace recheck
      passed. Re-run the Nix `clippy` check before tagging.
- [x] `cargo fmt --all -- --check` from the final tree.
- [x] `cargo test -p cargo-octet --lib`: 939 passed, 0 failed, 27 ignored.
- [~] `cargo test --workspace` or the repository `nextest` rail from the final
      tree. The `core` gate rerun covers this; record its result before tagging.
- [x] `coverage-profiles` check. The first `core` rerun exposed a provider
      version mismatch: the check expected `cargo-llvm-cov 0.9.0` while the
      pinned nixpkgs provider and the profile registry are `0.8.3`. The check
      now matches `0.8.3`, and the focused check passed.
- [x] Full expected-green `core` gate passed on 2026-11-12:
      `nix build .#checks.x86_64-linux.core -L --no-link` completed, including
      workspace clippy, fmt, UI, compiler-architecture, profile, rail, smoke,
      and `nextest` checks. `nextest` ran **1856 tests: 1856 passed, 0 failed,
      48 skipped**. The gate also covers `release-candidate-smoke` and the
      coverage-profile check. The wrapper was run with the local Nix
      configuration overridden to avoid the broken remote builder/cache:
      `NIX_CONFIG=$'substituters = https://cache.nixos.org\nbuilders =\nmin-free = 0\nmax-free = 0'`.

## 4. Public surfaces

- [ ] The public landing surface is the repository README, or an owned custom
      domain resolves. `octet.dev` is not owned and must not be referenced in
      metadata, release notes, or outreach.
- [ ] The landing surface follows the outline in
      [positioning.md](positioning.md#landing-page-outline).
- [ ] The landing surface links the pinned install command, first checked patch,
      trust boundary, and security policy.
- [ ] Public README has a working install command above the fold.
- [ ] Public README links the release notes and consumer guide.
- [ ] GitHub repository description and topics are set:
      `rust`, `static-analysis`, `dylint`, `release-engineering`,
      `software-supply-chain`, `evidence`. The repository had a description but
      no topics when observed on 2026-11-11. The homepage field is optional; if
      set, it must be an owned URL.
- [ ] Social preview image and repository avatar are present. Optional.
- [ ] Support expectations and issue template are published. Optional but
      recommended.
- [ ] Private vulnerability reporting is enabled on the public repository and
      the `SECURITY.md` link resolves.
- [ ] Issue templates request revision, platform, command, configured scope, and
      expected/observed behavior.

## 5. Launch assets

- [x] Positioning and message brief drafted.
- [x] Launch plan drafted.
- [x] v0.1.0 release notes drafted.
- [x] Announcement draft written.
- [x] Outreach copy written.
- [ ] Release notes updated with the final tag's receipts and exact artifact
      paths.
- [ ] Announcement reviewed by the maintainer for claim discipline.
- [ ] A named approver signs off on the release notes and announcement.
- [ ] Demo transcript or terminal recording prepared. Optional but recommended.
- [ ] Landing page copy approved. Optional until the site is live.
- [ ] All external copy discloses maintainer status where required.

## 6. Security and privacy

- [ ] Run a redacting secret scan against the final public tree and reachable
      history. Record the scanner version, command, revision, and result.
- [ ] Review personal paths, consumer data, and historical evidence separately.
      A clean secret scan is not privacy clearance.
- [ ] Confirm no private settings, operator paths, or archival transcripts are in
      the public export.
- [ ] Confirm no credentials or private receipts are in release notes, demo
      transcripts, or issue templates.
- [ ] Confirm the public repository does not expose private deploy keys to
      pull-request-controlled builds.
- [ ] Rotate any credential that the scan or review finds before resuming.

## 7. Community and support

- [ ] Name a support owner for the first 30 days.
- [ ] Create a launch feedback issue or discussion and pin it.
- [ ] Prepare a triage label set: `launch`, `install`, `evidence`, `lint`,
      `security`, `docs`.
- [ ] Prepare a canned response for "does this prove correctness?" and "why
      nightly?" that links the trust boundary.
- [ ] Confirm the maintainer can reproduce the first checked patch on a clean
      machine.
- [ ] Decide the response time expectation and publish it in the issue template
      or release notes.

## 8. Owner-only publish sequence

Run these only after the go/no-go review. Substitute the actual public worktree
and revision. Stop on the first failure.

```bash
# 1. Resolve the new public pin.
nix flake metadata --json github:OnixResearch/octet-public/<revision> \
  | jq '{rev: .locked.rev, narHash: .locked.narHash}'

# 2. Update the cohort manifest and fixtures, then validate.
nickel export --format json standards/release/release-cohort.ncl \
  > standards/release/generated/release-cohort.json
cargo test -p octet-standards --lib release_cohort

# 3. Run the release gates.
nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link
cargo octet check --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet

# 4. Update the release notes qualification block with this tag's receipts.
# 5. Tag and publish from the public worktree.
git -C <public-worktree> tag -a v0.1.0 -m "Octet 0.1.0"
git -C <public-worktree> push origin v0.1.0
gh release create v0.1.0 \
  --repo OnixResearch/octet-public \
  --title "Octet 0.1.0" \
  --notes-file docs/gtm/release-notes-0.1.0.md

# 6. Enable the repository security settings.
gh api --method PATCH \
  repos/OnixResearch/octet-public/private-vulnerability-reporting \
  -f enabled=true
gh repo edit OnixResearch/octet-public \
  --add-topic rust \
  --add-topic static-analysis \
  --add-topic dylint \
  --add-topic release-engineering \
  --add-topic software-supply-chain \
  --add-topic evidence
```

Configure branch protection and secret scanning/push protection in the
repository settings; verify the `SECURITY.md` private-reporting link after
enabling it.

## 9. Go/no-go review

Hard gates:

- [ ] Public export is reviewed and pinned.
- [ ] `release_cohort` validation passes.
- [ ] `release-candidate-smoke` passes.
- [ ] Install smoke returns `cargo-octet 0.1.0` from a clean environment.
- [ ] Artifact verifier passes on the walkthrough bundle.
- [ ] Tamper rejection fails closed.
- [ ] Private vulnerability reporting is enabled and tested.
- [ ] The public landing surface is the repository README, or an owned custom
      domain resolves.
- [ ] Release notes and announcement have a named approver.
- [ ] Rollback owner and channel are named.

Soft gates:

- [ ] Full `nix flake check -L` passes. If not run, record that fact in the
      release notes and do not claim it.
- [ ] A second person reproduces the first checked patch.
- [ ] Landing page is live.
- [ ] Demo recording is ready.

Record the final go/no-go decision, date, approver, and evidence links in the
private launch tracker. Do not publish the tracker.
