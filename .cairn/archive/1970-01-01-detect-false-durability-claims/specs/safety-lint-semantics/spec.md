# Safety Lint Semantics Delta

## ADDED Requirements

### Requirement: Declared durability boundary profile
r[octet.durability_claims.boundary_profile] Octet MUST analyze durability claims only within consumer-declared source scopes and exact boundary surfaces.

#### Scenario: No durability boundary is declared
- GIVEN a consumer without a durability-boundary profile
- WHEN Octet checks ordinary `flush` or commit calls
- THEN it does not invent a product durability claim

### Requirement: Synchronization result handling
r[octet.durability_claims.sync_result] Octet MUST report direct discard, `let _ =`, empty error handling, or unconditional error-to-success mapping for declared synchronization and commit results.

#### Scenario: Sync result is discarded
- GIVEN a declared boundary that uses `let _ = file.sync_all()`
- WHEN source analysis runs
- THEN Octet reports the configured durability finding

#### Scenario: Sync result is handled
- GIVEN a declared boundary that maps synchronization failure to explicit uncertainty and quarantine
- WHEN source analysis runs
- THEN this rule does not report result discard

### Requirement: Flush-only durability finding
r[octet.durability_claims.flush_only] `Write::flush()` MUST NOT satisfy a declared durable-persistence observation by itself.

#### Scenario: Declared boundary uses flush only
- GIVEN a declared durable boundary that ends after handled `flush()`
- WHEN source analysis runs
- THEN Octet reports missing durable observation

### Requirement: Retry after synchronization error
r[octet.durability_claims.retry_after_error] Octet MUST report the bounded same-receiver retry shape when no intervening write, reopen, replacement handle, or reconciliation appears in the analyzed block.

#### Scenario: Failed sync is retried directly
- GIVEN a handled sync error followed by another sync on the same visible receiver
- WHEN no admitted recovery operation appears between them
- THEN Octet reports the retry shape

#### Scenario: Failed sync is retried after rewrite
- GIVEN a handled sync error, an explicit rewrite, and a later handled sync
- WHEN source analysis runs
- THEN the direct-retry rule does not report that shape

### Requirement: Durability finding guidance
r[octet.durability_claims.guidance] Diagnostics MUST explain explicit uncertain outcomes, quarantine, reopen, rewrite, and reconciliation without claiming one universal repair.

#### Scenario: Finding is rendered
- GIVEN one durability finding
- WHEN Octet renders guidance
- THEN it states the source-shape scope and durability non-claims

### Requirement: Durability lint validation
r[octet.durability_claims.validation] Every new durability rule MUST include positive and negative UI, policy, and evidence fixtures.

#### Scenario: Invalid profile is supplied
- GIVEN a durability profile with an unknown observation class
- WHEN policy validation runs
- THEN validation fails closed
