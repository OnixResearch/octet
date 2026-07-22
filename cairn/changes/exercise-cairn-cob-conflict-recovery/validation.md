# Validation

## Outcome

The downstream conflict-recovery drill passed its bounded completion contract. One signed root embedded creator `z6MkrYePae54PSMayuv3YJJcPV65GE6Mhvuq2537cmJybFCW` and delegate `z6Mkgvh5Qq4bjh8BZUDwPB9ZPNoQvHGMu1wvPX2KRyPNUDtq`. Their separate stores produced tasks operations `2ba95e8204bd05dd566d93f7f5f9ac68f7126904` and `e73bf24a150c12c5cbeda410c40d113d002ef259` from the same root basis `0702d3bd89a7be698cd24ec343323d7786a99d6d`.

Explicit local Git ref transport exposed both actor refs without configured remotes. Default conflict inspection retained metadata with `shown_head: null`; explicit selectors revealed each exact retained tasks value separately. Ordinary update returned a blocked, non-mutating failure and left both refs unchanged.

Resolution preview selected the creator operation and bound both divergent operations in its action basis. Executed operation `3669f3e1001bba8878d9ad4bcaf874d21207703b` has both operations as direct Git parents. The delegate ref still targets the losing operation, and both divergent operations remain ancestors of the resolution operation. Bundle reload into a third store reported no current artifact conflict and preview-first checkout materialized the exact creator-selected tasks bytes rather than the delegate bytes.

The final lifecycle observation is intentionally `stale`: content selection does not silently replace the root lifecycle subject hash. The drill proves explicit content-conflict recovery, not lifecycle review, evidence truth, or release readiness.

## Retained evidence

- `fixtures/cairn-cob-conflict-pilot/manifest.ncl` and generated `manifest.json`
- `fixtures/cairn-cob-conflict-pilot/octet-cob-conflict.bundle`
- exact creator and delegate tasks values under `fixtures/cairn-cob-conflict-pilot/heads/`
- signed import, update, inspection, refusal, resolution, reload, checkout, and transport receipts under `fixtures/cairn-cob-conflict-pilot/evidence/`
- positive and complete-basis-tampering-negative validation in `tests/cairn_cob_conflict_pilot.rs`

No private key is retained. The temporary creator/delegate keystores and mutable stores remain only under ignored `target/` and confer no production authority.

## Checks

- Pre-change focused `cairn_cob_pilot` test — passed.
- Focused `cairn_cob_conflict_pilot` test — 2 passed, including positive and tampering-negative cases.
- `cargo fmt --all -- --check` — passed.
- Focused Clippy with warnings and pedantic lints denied — passed.
- Nickel typecheck/export freshness and manifest byte comparison — passed.
- Git bundle verification, exact three-head import, no-remote checks, and direct two-parent ancestry — passed.
- `nix run .#octet-policy-validate` — passed.
- `nix build .#checks.x86_64-linux.cairn-cob-conflict-pilot --no-link -L` — passed.
- `nix build .#checks.x86_64-linux.nextest --no-link -L` — 230 tests passed, 2 skipped.
- `nix flake check -L --option secret-key-files ''` — all 59 compatible-system checks passed; incompatible non-`x86_64-linux` systems were omitted.
- Native Cairn proposal, design, tasks, sync, and strict validation gates — passed.

## Accepted traceability

The typed Cairn policy now includes `cairn/specs/cob-collaboration-conflict-recovery/spec.md` as an accepted requirement source. Every `octet.cob_conflict_recovery.*` ID is linked from `tests/cairn_cob_conflict_pilot.rs`.

The final report preserves the pre-existing baseline exactly: 32 unrelated `octet.evidence_provenance.*` IDs remain missing and `octet.effect_closure.cli.check_failure` remains dangling. The refreshed current policy hash is `e32547d27f02daf26e94ecaa0a361fbe76cc7149a73e740515624203fe9d5103`. Historical signed fixture receipts remain bound to mutation-time policy hash `0d5e71092b45e2a8cfbaed4edffe6b01cd6cd0e4dc71f9c86fc1d0bf82cc70fc`; they are not rewritten to claim current-policy status.

## Bounded blocker

`nix run .#cargo-octet -- check` still fails before running checks because the root `Cargo.toml` lacks the pre-existing runner `[metadata]` table. This is the same inherited blocker recorded by the earlier adoption pilot and is not attributed to conflict recovery. The focused runner tests, policy gate, Nextest, Clippy, and full flake gate passed.

## Adversarial audit

An advisory independent review identified false-completion risks around same-namespace overwrite, stale previews, path-only assertions, loss of the losing revision, and generic conflict checks. Deterministic repository checks closed those risks by requiring distinct actor refs, same input state, exact selector receipts, unchanged refs after ordinary-update refusal, matching preview/execute plans, direct ancestry from both divergent operations, retained losing ref reachability, and exact selected checkout bytes.

## Non-claims

This evidence is scoped to the pinned Cairn library cohort, exact Octet base, static signed bundle, and local operations exercised here. It does not prove semantic merge correctness, Radicle node/profile participation, publication, remote authentication or authorization, availability, durability, synchronization, distributed convergence, cross-version compatibility, source correctness, whole-program correctness, or release eligibility.

Development-shell evaluation reused the pinned cached Dylint revision after reporting the unavailable Git HTTPS helper. Nix also reported the configured remote builder unavailable and completed the requested checks locally.
