# Design

## Success contract

Goal: remove the first bounded `non_trait_imports` cohort from Octet's canonical self-check while proving monotonic baseline reduction and unchanged build-shell behavior.

Observable completion evidence:

- `cargo-octet/build.rs` retains only the required `std::fmt::Write as _` trait import and uses explicit owner-qualified paths for environment, filesystem, and path values;
- the old baseline reports exactly four stale identities, zero new identities, and 2,666 warning occurrences;
- the reviewed replacement baseline is exactly the old stable-ID set minus those four identities, contains 1,816 unique identities, and produces zero new or stale identities;
- `non_trait_imports` decreases from 1,114 to 1,110 occurrences without lint suppression, exception expansion, aliasing, or lint implementation changes;
- embedded-source checks, the permanent positive/negative self-check gate, formatting, Clippy, Nextest, policy freshness, traceability, and the broad compatible-system flake gate pass.

False completion includes deleting baseline records before observing stale IDs, accepting any new identity, introducing an alias that merely evades the lint, changing lint policy or exceptions, suppressing findings, modifying generated-source content or traversal behavior, relying only on aggregate counts, or claiming that four resolved warnings establish broader correctness.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| owner-qualified build shell | Remove four imports and qualify every corresponding use site | active | Directly follows the lint guidance; build script is isolated and compilation exercises all names |
| one-import domain slice | Qualify only `std::fmt` in `domain_types.rs` | deferred | Valid but proves only one identity and provides weaker first-tranche evidence |
| local type/module aliases | Replace imports with aliases declared in the file | rejected | Hides owner paths and evades rather than satisfies the rule |
| lint exception or source allow | Add configuration or attributes | rejected | Leaves the warning debt semantically accepted and weakens the policy |
| repository-wide rewrite | Qualify all 1,114 occurrences at once | rejected | Excessive review surface and high incidental-regression risk |

An advisory secondary review selected the build-shell mechanism but supplied non-authoritative API examples; only compiler, self-check, baseline-set, and repository-gate evidence is accepted.

## Functional core and imperative shell

No decision core changes. `cargo-octet/build.rs` remains the imperative shell that reads Cargo environment, walks the reviewed source directories, hashes bytes, and writes deterministic embedded-source Rust. Replacing lexical shorthands with `std::...` paths changes name resolution spelling only.

The existing pure stable-ID set comparison remains authoritative. The reduction is observed first with the old immutable baseline, then the runner writes a candidate baseline. A deterministic set-difference check must show no additions and exactly the four reviewed removals before replacement.

## Validation sequence

1. Preserve pre-change embedded-source and permanent self-check results.
2. Compile and test the qualified build shell.
3. Run the canonical self-check against the old baseline and require four stale IDs, no new IDs, exit `0`, and warning-only status.
4. Generate a candidate baseline, compare identity sets, and update exact documentation and Nix assertions only after review.
5. Re-run positive and one-new-ID negative gates, then repository-wide validation.

## Risks and claim boundary

Fully qualified paths can be missed in signatures or macro-adjacent expressions; compilation and a zero-finding source audit discriminate that failure. Line shifts can affect rendered positions, so stable-ID set comparison rather than line-number assumptions controls refresh acceptance. Embedded source identity will legitimately change because maintained source bytes changed, but generated file membership and traversal semantics must remain unchanged.

The bounded search budget is three mechanism families, one independent advisory audit, one old-baseline observation, one candidate refresh, and three validation rounds. Allowed outcomes are validated, blocked with an exact new/stale/build discrepancy, exhausted, or user-decision-required.

The result proves only removal of four stable warning identities from the pinned workspace-library self-check. It does not prove the remaining warnings correct, build-script correctness, lint completeness, runtime safety, whole-program correctness, or release eligibility.
