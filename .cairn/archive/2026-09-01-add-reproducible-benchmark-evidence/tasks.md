## Phase 1: Profiles and contracts

- [x] [serial] Add typed Nickel benchmark profiles for workloads, commands, modes, environments, samples, metrics, limits, comparisons, and non-claims r[octet.benchmark_evidence.profile]
- [x] [serial] Define Rust-owned environment, sample, signal, comparison, and receipt DTOs with generated Nickel contracts r[octet.benchmark_evidence.receipt]
- [x] [serial] Register profile, workload, oracle, samples, logs, findings, comparison, receipt, and provenance artifact ownership r[octet.benchmark_evidence.receipt]

## Phase 2: Performance core and shell

- [x] [serial] Implement pure validation for profiles, samples, summaries, environment comparability, thresholds, uncertainty, and deterministic ordering r[octet.benchmark_evidence.comparison]
- [x] [serial] Implement the bounded Rust shell for fixed command IDs, cache preparation, environment capture, process execution, and atomic artifacts r[octet.benchmark_evidence.performance_protocol]
- [x] [serial] Add cold, warm, no-change, and declared incremental workloads without merging mode authority r[octet.benchmark_evidence.performance_protocol]
- [x] [serial] Add baseline and candidate comparison with `improved`, `stable`, `regressed`, `inconclusive`, and `incomparable` results r[octet.benchmark_evidence.comparison]

## Phase 3: Signal benchmark

- [x] [serial] Add an immutable signal corpus and adjudicated oracle with reviewer roles, rationales, source identities, and unresolved outcomes r[octet.benchmark_evidence.signal_oracle]
- [x] [serial] Implement pure finding-to-oracle alignment and true-positive, false-positive, false-negative, suppression, and inconclusive classification r[octet.benchmark_evidence.signal_oracle]
- [x] [parallel] Add selected synthetic and cross-project workload fixtures with explicit representativeness boundaries r[octet.benchmark_evidence.boundary]

## Phase 4: Verification

- [x] [parallel] Add positive stable, improved, regressed, warm, cold, incremental, and signal-classification fixtures r[octet.benchmark_evidence.verification]
- [x] [parallel] Add negative command, workload, sample, timing, environment, oracle, finding, identity, limit, and overclaim fixtures r[octet.benchmark_evidence.verification]
- [x] [parallel] Prove starter checks and unrelated evidence rails remain independent from benchmark execution r[octet.benchmark_evidence.boundary]

## Phase 5: Documentation and closeout

- [x] [parallel] Document benchmark profiles, workload preparation, noise, oracle review, metric interpretation, replay, and non-claims r[octet.benchmark_evidence.boundary]
- [x] [parallel] Require every published benchmark metric to cite a verified receipt and interpretation boundary r[octet.benchmark_evidence.public_claims]
- [x] [serial] Run focused tests, Nickel freshness, artifact validation, formatting, strict first-party lint rails, Cairn validation, and traceability coverage r[octet.benchmark_evidence.verification]
- [x] [serial] Run proposal, design, and tasks gates before sync and archive r[octet.benchmark_evidence.profile]
