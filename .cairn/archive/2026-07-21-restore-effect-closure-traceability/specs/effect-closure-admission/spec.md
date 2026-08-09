# Effect Closure Traceability Delta

## MODIFIED Requirements

### Requirement: Two-stage closure CLI

r[octet.effect_closure.cli] `cargo octet admit --profile deterministic-core --closure` MUST run the existing strict-profile check before attempting effect closure and MUST require successful deny-mode function-address evidence for the selected core paths.

#### Scenario: Direct check and closure are admitted

r[octet.effect_closure.cli.admitted]

- GIVEN the selected strict check passes, function addresses match, all source constructs and calls are resolved, role edges are allowed, and closure effects satisfy the profile
- WHEN the admission command runs
- THEN it MUST emit an admitted effect-closure bundle and exit successfully.
- AND the existing check artifacts MUST remain available as identified parent evidence.

#### Scenario: Direct check failure blocks closure

r[octet.effect_closure.cli.check_failure]

- GIVEN the strict check reports an error, stale function address, compiler failure, or integration failure
- WHEN the admission command runs
- THEN Octet MUST NOT claim or emit an admitted closure.
- AND status evidence MUST identify the failed direct-check phase.
