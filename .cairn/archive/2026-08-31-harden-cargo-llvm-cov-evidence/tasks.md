## Phase 1: Profiles and contracts

- [x] [serial] Add typed Nickel cargo-llvm-cov profiles with exact scope, provider identity, metrics, fixed command shape, thresholds, limits, and non-claims r[octet.coverage_baseline.profile]
- [x] [serial] Define Rust-owned native-report, count, comparison, and receipt DTOs with generated Nickel contracts r[octet.coverage_baseline.receipt]
- [x] [serial] Register profile, current report, baseline report, normalized projection, logs, receipt, and provenance artifact ownership r[octet.coverage_baseline.receipt]

## Phase 2: Core and shell

- [x] [serial] Implement pure profile, native-report, count, path, comparability, threshold, regression, identity, and limit validation r[octet.coverage_baseline.comparison]
- [x] [serial] Add fixed profile-derived cargo-llvm-cov execution with a dedicated capability-rooted report path r[octet.coverage_baseline.profile]
- [x] [serial] Normalize supported native covered and total counts without trusting caller-provided percentages r[octet.coverage_baseline.native_artifacts]
- [x] [serial] Compare current and accepted baseline reports inside the pure core and reject incomparable scopes r[octet.coverage_baseline.comparison]
- [x] [serial] Preserve compatible normalized imports without inventing Octet-calculated regression authority r[octet.coverage_baseline.boundary]

## Phase 3: Verification

- [x] [parallel] Add positive stable, improved, threshold-pass, native-run, and compatible-import fixtures r[octet.coverage_baseline.verification]
- [x] [parallel] Add negative regression, denominator, incomparable-scope, malformed, stale, unsafe-path, limit, and tool-failure fixtures r[octet.coverage_baseline.verification]
- [x] [parallel] Add exact-count and precision fixtures that prevent floating-point policy drift r[octet.coverage_baseline.comparison]
- [x] [parallel] Prove starter checks and unrelated evidence rails remain independent from cargo-llvm-cov r[octet.coverage_baseline.boundary]

## Phase 4: Documentation and closeout

- [x] [parallel] Document profiles, metric counts, baseline comparability, thresholds, path privacy, costs, and non-claims r[octet.coverage_baseline.boundary]
- [x] [serial] Run focused tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.coverage_baseline.verification]
- [x] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.coverage_baseline.profile]
