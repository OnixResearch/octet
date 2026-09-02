# Proof Robustness Evidence Specification Delta

## ADDED Requirements

### Requirement: Proof robustness profiles are typed and bounded

r[octet.proof_robustness.profile]

Octet MUST define an optional typed Nickel proof-robustness profile that binds selected proof rails, ordered unique solver seeds, provider metric identities, run limits, classification policy, required audit facts, artifact policy, and non-claims.

#### Scenario: Complete profile exports deterministically

- GIVEN a profile has valid rail selectors, a non-empty ordered seed set, supported metrics, named limits, complete classifications, and required non-claims
- WHEN Nickel validation and deterministic export run
- THEN the generated runtime profile MUST match the checked-in artifact byte-for-byte
- AND repeated export from equal inputs MUST produce equal BLAKE3 identity

#### Scenario: Profile contains unsafe or ambiguous input

- GIVEN a profile has no seeds, duplicate seeds, an unknown metric, an empty rail selector, an unnamed limit, an unknown classification, or a missing non-claim
- WHEN profile validation runs
- THEN validation MUST fail before any verifier executes

### Requirement: Robustness execution uses existing bounded proof shells

r[octet.proof_robustness.execution]

Octet MUST execute every selected robustness run through the existing typed proof-command model with exact argv, declared seed injection, pinned verifier identity, bounded environment, deterministic working-directory policy, timeout, and bounded output capture.

#### Scenario: Selected seed matrix executes

- GIVEN a valid profile and selected proof rail with a supported seed adapter
- WHEN proof robustness collection runs
- THEN the shell MUST execute one child proof run for every declared seed
- AND each child receipt MUST bind its seed, command, source, verifier, solver, toolchain, result, metrics, and artifacts

#### Scenario: One child run is incomplete

- GIVEN a selected child run cannot start, times out, exceeds output limits, reports malformed metrics, or lacks a proof receipt
- WHEN robustness classification runs
- THEN the matrix MUST be classified `incomplete` or `unsupported` as declared by policy
- AND Octet MUST NOT fabricate a stable result

### Requirement: Robustness classifications are deterministic

r[octet.proof_robustness.classification]

The pure core MUST classify a complete run set as `stable`, `outcome-unstable`, `resource-unstable`, `incomplete`, or `unsupported` from explicit typed observations and policy.

#### Scenario: Outcomes and resource policy agree

- GIVEN every declared seed produces the same terminal proof outcome and every comparable resource metric satisfies policy
- WHEN the pure classification core runs
- THEN the result MUST be `stable`
- AND equal observations MUST produce byte-identical ordered summaries

#### Scenario: A proof outcome changes by seed

- GIVEN at least one declared seed produces a different terminal proof outcome
- WHEN the pure classification core runs
- THEN the result MUST be `outcome-unstable`
- AND resource agreement MUST NOT hide the outcome change

#### Scenario: Provider metrics are not comparable

- GIVEN metric identity, provider identity, toolchain identity, or source identity differs across child runs
- WHEN classification runs
- THEN the result MUST be `incomplete` or `unsupported`
- AND Octet MUST NOT compare the metric values

### Requirement: Assumption audit facts remain explicit

r[octet.proof_robustness.assumption_audit]

A robustness receipt MUST keep policy assumptions, accepted trust boundaries, source-scan findings, no-cheating mode, provider-reported axiom use, and unknown audit classes as separate typed facts.

#### Scenario: Audited boundary is visible

- GIVEN a selected proof rail declares an accepted trust boundary and the source scan finds the matching boundary
- WHEN audit facts are assembled
- THEN the receipt MUST preserve both the declared and observed facts with their identities
- AND it MUST NOT describe the boundary as proven

#### Scenario: Declared and observed assumptions differ

- GIVEN a source scan finds an undeclared boundary, a declared boundary is absent, no-cheating mode differs, or an expected axiom-use fact is missing
- WHEN audit validation runs
- THEN the receipt MUST report a deterministic mismatch or unknown diagnostic
- AND a strict profile MUST fail closed

### Requirement: Proof-use coverage is never inferred

r[octet.proof_robustness.proof_use]

Every provider MUST label proof-use coverage as `complete`, `axiom-usage-only`, or `unsupported`. Octet MUST preserve provider-reported dependency facts without converting missing facts into unused dependencies.

#### Scenario: Verus reports axiom use

- GIVEN a Verus child run emits supported axiom-usage facts for the selected proof
- WHEN the adapter normalizes those facts
- THEN the receipt MUST label coverage `axiom-usage-only`
- AND the facts MUST retain provider, source, proof, and child-receipt identity

#### Scenario: Consumer requests complete dependency coverage

- GIVEN the provider supplies only axiom-use facts or no supported dependency facts
- WHEN a profile requires complete proof dependency coverage
- THEN robustness collection MUST report `unsupported`
- AND it MUST NOT emit an empty complete dependency set

### Requirement: Robustness receipts preserve parent identity and role

r[octet.proof_robustness.receipt]

Octet MUST emit a Rust-owned proof-robustness receipt that binds the profile, selected rail, source, proof, verifier, solver, toolchain, child receipts, run observations, audit facts, proof-use facts, classification, diagnostics, artifact identities, and non-claims.

#### Scenario: Complete receipt validates

- GIVEN a robustness receipt and every parent artifact have matching BLAKE3 identities and admitted roles
- WHEN artifact validation runs
- THEN validation MUST accept the receipt as supplemental recorded evidence
- AND any underlying formal-proof role MUST remain owned by its matching child proof receipt

#### Scenario: Parent or role is stale

- GIVEN a parent receipt, source, profile, verifier, metric, audit fact, artifact, role, or non-claim is missing, stale, substituted, or promoted
- WHEN artifact validation runs
- THEN validation MUST fail with a deterministic diagnostic naming the rejected field

### Requirement: Dafny remains a design reference until separate adoption

r[octet.proof_robustness.boundary]

This change MUST treat `dafny-lang/dafny` release `v4.11.0` at commit `fcb2042d6d043a2634f0854338c08feeaaaf4ae2` as a clean-room design reference only. It MUST NOT add Dafny, Boogie, .NET, Dafny-generated Rust, `.doo`, or `.dtr` artifacts to ordinary Octet execution.

#### Scenario: Ordinary Octet paths run

- GIVEN no robustness profile is selected
- WHEN starter checks, strict profiles, or ordinary proof rails run
- THEN they MUST NOT require robustness artifacts, Dafny, Boogie, .NET, extra solver runs, or new environment authority

#### Scenario: Robustness result is promoted to correctness

- GIVEN a receipt or report claims solver independence, verifier soundness, proof completeness, semantic equivalence, whole-program correctness, or release eligibility
- WHEN claim-boundary validation runs
- THEN validation MUST fail

### Requirement: Proof robustness validation is adversarial

r[octet.proof_robustness.verification]

The implementation MUST include deterministic positive and negative coverage for profiles, plans, seeds, metrics, classifications, assumption audit, proof-use coverage, identities, limits, roles, artifacts, default independence, and non-claims.

#### Scenario: Complete fixture matrix runs

- GIVEN stable, unstable, incomplete, unsupported, stale, malformed, timed-out, role-confused, and overclaiming fixtures
- WHEN focused proof-robustness validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail at its declared boundary
