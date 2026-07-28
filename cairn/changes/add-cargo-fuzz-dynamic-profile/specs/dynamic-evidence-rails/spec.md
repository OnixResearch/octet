# Dynamic Evidence Rails Specification Delta

## ADDED Requirements

### Requirement: Cargo-fuzz profiles are explicit and bounded

r[octet.dynamic_evidence.cargo_fuzz_profile]

Octet MUST define an opt-in cargo-fuzz profile with an exact engine, target, toolchain, command shape, execution budget, sanitizer posture, artifact limits, and non-claims.

#### Scenario: Valid fuzz profile loads

- GIVEN a reviewed profile names a supported engine, target selector, allowed argv shape, toolchain, budgets, limits, and non-claims
- WHEN Octet validates the profile
- THEN validation MUST produce one deterministic profile identity
- AND the identity MUST bind every behavior-affecting profile field

#### Scenario: Unbounded or arbitrary command fails

- GIVEN a profile omits a required limit or permits arbitrary command text, target selection, environment input, or output path
- WHEN profile validation runs
- THEN validation MUST fail before process execution
- AND Octet MUST NOT infer missing limits from ambient state

### Requirement: Cargo-fuzz receipts bind exact run and corpus evidence

r[octet.dynamic_evidence.cargo_fuzz_receipt]

Octet MUST emit a versioned receipt that binds the selected target, engine, compiler, sanitizer, argv, budgets, corpus, dictionary, result, crash artifacts, logs, and BLAKE3 identities.

#### Scenario: Clean bounded run records final corpus

- GIVEN a selected fuzz target completes its declared execution budget without a crash
- WHEN Octet records the result
- THEN the receipt MUST bind the initial and final canonical corpus roots
- AND it MUST distinguish a bounded clean observation from exhaustive input coverage

#### Scenario: Crash result records reproducible evidence

- GIVEN the fuzz engine discovers a crashing input and produces a minimized reproducer
- WHEN Octet records the result
- THEN the receipt MUST bind both crash artifacts and the exact replay command
- AND the failed status MUST remain reviewable without being rendered as a successful gate

#### Scenario: Stale or unsafe artifact fails

- GIVEN a corpus, dictionary, crash input, or reproducer has a stale hash, unsafe path, excessive size, or missing required role
- WHEN receipt validation runs
- THEN validation MUST fail with a deterministic artifact diagnostic
- AND artifact validation MUST NOT publish a passing receipt

### Requirement: Fuzz evidence remains bounded test evidence

r[octet.dynamic_evidence.cargo_fuzz_boundary]

Cargo-fuzz receipts MUST remain scoped generated-execution evidence and MUST NOT claim exhaustive coverage, panic freedom, memory safety, formal proof, or release eligibility.

#### Scenario: Passing run remains transcript evidence

- GIVEN a fuzz receipt reports no crash under its selected budget
- WHEN Octet classifies the receipt
- THEN the evidence tier MUST remain `transcript_assertion` or `manual_review_required`
- AND the summary MUST state the selected budget and non-claims

#### Scenario: Claim promotion fails

- GIVEN a fuzz receipt requests `formal_proof` or claims exhaustive input coverage, whole-program correctness, or verifier soundness
- WHEN Octet validates the receipt
- THEN validation MUST fail with a tier or overclaim diagnostic

### Requirement: Named dynamic profiles retain command isolation

r[octet.dynamic_evidence.named_profile_regression]

Octet MUST retain positive and negative consumer fixtures for each named Miri, Loom, and cargo-fuzz dynamic profile.

#### Scenario: Declared command is accepted

- GIVEN a fixture selects one named profile and its exact allowed command shape
- WHEN profile command resolution runs
- THEN Octet MUST construct the declared argv deterministically
- AND no other named profile command MUST be selected

#### Scenario: Cross-profile command is rejected

- GIVEN a fixture selects one profile but supplies another profile's command or artifact shape
- WHEN profile command resolution or receipt validation runs
- THEN Octet MUST fail with a deterministic profile-mismatch diagnostic

### Requirement: Cargo-fuzz verification covers positive and negative paths

r[octet.dynamic_evidence.cargo_fuzz_verification]

The implementation MUST include deterministic positive and negative coverage for profile validation, process classification, corpus identity, crash evidence, limits, command isolation, artifact validation, and default-path independence.

#### Scenario: Focused fixture matrix completes

- GIVEN the clean, crash, malformed, stale, unsafe, over-limit, tool-failure, and command-mismatch fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
