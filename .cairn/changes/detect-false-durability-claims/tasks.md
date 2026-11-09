# Tasks: Detect false durability claims

## Policy and analysis

- [ ] [serial] Record current safety lint, standards, assurance profile, UI fixture, and evidence baselines. r[octet.durability_claims.boundary_profile]
- [ ] [serial] Add Nickel contracts for declared durability boundaries, required observations, uncertain outcomes, and analyzed scopes. r[octet.durability_claims.boundary_profile]
- [ ] [serial] Add source analysis for discarded synchronization and configured commit results. r[octet.durability_claims.sync_result]
- [ ] [serial] Add declared-boundary analysis that rejects `Write::flush()` as the only durability observation. r[octet.durability_claims.flush_only]
- [ ] [serial] Add bounded same-receiver retry analysis and incomplete-analysis facts. r[octet.durability_claims.retry_after_error]
- [ ] [serial] Add diagnostics for explicit uncertainty, quarantine, reopen, rewrite, and reconciliation. r[octet.durability_claims.guidance]

## Evidence profile

- [ ] [serial] Add storage-fault assurance profile inputs for exact saved producer and Cairn receipt identities. r[octet.durability_claims.assurance_profile]
- [ ] [serial] Keep producer execution outside normal check and evidence verification. r[octet.durability_claims.assurance_profile]
- [ ] [parallel] Add current, stale, missing-case, producer-failure, and overclaim evidence fixtures. r[octet.durability_claims.validation]

## UI and closeout

- [ ] [parallel] Add positive handled-sync, uncertainty, quarantine, reopen, rewrite, and adapter-observation fixtures. r[octet.durability_claims.validation]
- [ ] [parallel] Add negative discard, `let _`, empty-error, flush-only, same-handle-retry, and missing-uncertainty fixtures. r[octet.durability_claims.validation]
- [ ] [serial] Update lint catalog, guidance, standards registry, artifact manifest, and claim boundaries. r[octet.durability_claims.guidance]
- [ ] [serial] Run policy freshness, library and UI tests, ignored UI evidence, formatting, Clippy, Cairn gates, and Nix checks. r[octet.durability_claims.validation]

## Verification Coverage

- `Scenario: Sync result is handled` -> positive UI fixture
- `Scenario: Sync result is discarded` -> negative UI fixture
- `Scenario: Declared boundary uses flush only` -> negative UI fixture
- `Scenario: Failed sync is retried after rewrite` -> positive retry fixture
- `Scenario: Saved campaign receipt is stale` -> assurance rejection fixture
