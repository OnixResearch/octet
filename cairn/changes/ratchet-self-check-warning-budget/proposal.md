# Ratchet the Octet self-check warning budget

## Why

Octet's canonical self-check completes with 2,671 warning findings, but the repository does not check in an acceptance baseline or fail when a new stable finding appears. Existing `cargo-octet` baseline semantics already provide exact stable-ID comparison and report stale resolved debt without blocking cleanup.

The first inventory attempt also exposed a prerequisite defect: Cargo emits absolute workspace paths for Octet's own build, while structured JSON/SARIF validates finding locators as repository-relative. Human output succeeds, but machine-readable inventory exits with integration code `2`. Scraping human stderr or weakening the domain contract would make the budget less trustworthy.

## What Changes

- Normalize Cargo diagnostic paths under the discovered workspace root into validated repository-relative finding locators; continue to reject paths outside the workspace.
- Capture the canonical warning set as a checked-in `cargo-octet` baseline with reviewable finding context.
- Add a deterministic inventory document grouped by lint and domain.
- Remove the single `negated_predicate` self-check finding through a behavior-preserving positive-name refactor before establishing the final baseline.
- Extend the hermetic self-check gate so the real workspace has zero new/stale baseline IDs and a source mutation introducing one new warning fails with exit `1` and exactly one new ID.

## Impact

The canonical warning-only exit contract remains unchanged. The self-check Nix gate becomes a no-new-warning ratchet while allowing resolved baseline entries to be pruned deliberately. Finding-domain validation remains fail closed, and the baseline does not convert accepted warnings into correctness, completeness, or release-eligibility evidence.
