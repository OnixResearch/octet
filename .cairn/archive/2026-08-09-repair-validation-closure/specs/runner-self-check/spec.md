# Runner Self Check Specification Delta

## ADDED Requirements

### Requirement: Packaged self-check failures expose bounded evidence

r[octet.runner_self_check.failure_diagnostics] The packaged self-check MUST report the failed phase and bounded diagnostic evidence before it exits unsuccessfully.

#### Scenario: Warning baseline drift is visible

- GIVEN the canonical self-check produces current findings outside the reviewed baseline
- WHEN the packaged Nix gate evaluates the positive run
- THEN it MUST print accepted, current, new, and stale identity counts
- AND it MUST print bounded new or stale identity evidence without dumping the complete lint transcript

#### Scenario: Structured artifacts are unavailable

- GIVEN the runner fails before it writes structured status and finding artifacts
- WHEN the packaged Nix gate reports the failure
- THEN it MUST print the available human summary or a bounded stderr tail
- AND it MUST retain the original nonzero exit status

### Requirement: Warning baseline refreshes preserve semantic monotonicity

r[octet.runner_self_check.warning_budget.semantic_refresh] A self-check baseline refresh MUST reject unreviewed candidate semantics and MUST admit only exact reviewed rekeys or removals from the previously accepted baseline.

#### Scenario: Source repair removes candidate warning debt

- GIVEN current source contains a finding absent from the reviewed baseline
- WHEN repair validation compares the final candidate with the accepted baseline
- THEN the source MUST remove the unapproved finding or a reviewed exact mapping MUST classify it
- AND the refresh MUST NOT use lint suppression, weaker limits, or a blanket baseline regeneration

#### Scenario: Repeated accepted identity gains an occurrence

- GIVEN one stable identity can describe findings in more than one checked crate context
- WHEN migration compares the accepted and candidate semantic occurrence counts
- THEN migration MUST reject each candidate semantic group whose count exceeds the accepted count
- AND identity-set membership MUST NOT hide the occurrence growth

#### Scenario: Unreviewed candidate finding remains

- GIVEN the final candidate contains a semantic finding without an accepted predecessor or reviewed exception
- WHEN baseline migration validation runs
- THEN validation MUST fail before replacing the checked-in baseline
- AND the existing baseline MUST remain unchanged
