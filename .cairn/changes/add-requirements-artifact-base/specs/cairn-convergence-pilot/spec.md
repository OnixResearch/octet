# Cairn Convergence Pilot Delta

## ADDED Requirements

### Requirement: Location-independent proof-deck locators

r[octet.cairn_convergence_pilot.locators] `cargo octet requirements check` and
`cargo octet requirements verify` MUST accept an optional `--artifact-base PATH`
that resolves relative `proof_decks[].artifact_root` and relative
`proof_decks[].deck` locators inside the producer output instead of under
`--source-root`, MUST refuse a relative locator that escapes that base with a
clear diagnostic, MUST keep absolute locator resolution unchanged, and MUST NOT
record the base path in the decision so identical producer-output bytes in
different directories produce byte-identical decisions.

#### Scenario: Identical proof cards at different locations produce one decision

- GIVEN one proof-card deck and artifact tree copied to two different
  directories and one evidence bundle using relative locators
- WHEN `requirements check` runs once per directory with `--artifact-base`
  naming that directory
- THEN both runs MUST pass with byte-identical decision bytes and the same
  `decision_blake3`.

#### Scenario: Relative locator escaping the artifact base is refused

- GIVEN an evidence bundle whose relative `deck` or `artifact_root` contains a
  `..` component or whose absolute path inside the base strips to an escape
- WHEN `requirements check` runs with that base
- THEN the clause MUST fail with a diagnostic naming the locator and the
  containment rule
- AND no foreign file outside the base MUST be read as proof evidence.

#### Scenario: Absolute locators keep their pre-existing resolution

- GIVEN an evidence bundle that names `deck` and `artifact_root` by absolute
  paths, as the pilot evidence did before this change
- WHEN `requirements check` runs with or without `--artifact-base`
- THEN the evaluation MUST read those absolute paths exactly as before and
  MUST NOT change the decision of existing evidence.

#### Scenario: Verify recomputes the recorded decision

- GIVEN a recorded decision and the same contract, evidence, and artifact base
- WHEN `requirements verify` runs
- THEN it MUST recompute the identical `decision_blake3` and report `passed`
- AND when the base holds different proof-card bytes it MUST report a
  non-passing status with the recomputed identity.
