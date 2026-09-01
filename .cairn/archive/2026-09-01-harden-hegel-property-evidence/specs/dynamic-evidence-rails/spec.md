# Dynamic Evidence Rails Specification Delta

## ADDED Requirements

### Requirement: Hegel run mode uses a typed producer profile

r[octet.dynamic_evidence.hegel_profiled_run]

Hegel run mode MUST construct fixed argv from a validated Nickel producer profile and MUST reject arbitrary command text, executable selection, arguments, environment values, paths, or network authority.

#### Scenario: Named producer profile runs

- GIVEN a reviewed profile binds a producer package, binary target, property catalog, Hegel version, backend, settings, source scope, limits, retention policy, and non-claims
- WHEN the operator selects the profile in Hegel run mode
- THEN Octet MUST construct the declared argv deterministically
- AND the receipt MUST bind the profile identity and exact constructed argv

#### Scenario: Explicit command is supplied

- GIVEN an operator supplies `--command` or a profile contains free-form command text
- WHEN Hegel run preflight executes
- THEN Octet MUST reject the request before process execution

#### Scenario: Hegel is not selected

- GIVEN an operator runs starter checks or another evidence rail
- WHEN command resolution runs
- THEN Octet MUST NOT require or execute a Hegel producer, profile, adapter, or property test

### Requirement: Hegel runs use a dedicated output contract

r[octet.dynamic_evidence.hegel_output_contract]

A profiled Hegel producer MUST write one versioned summary artifact through an explicit capability-rooted output path, while stdout and stderr remain bounded diagnostic logs.

#### Scenario: Dedicated summary passes

- GIVEN the selected producer exits under its limits and writes a complete supported summary to the declared output path
- WHEN Octet evaluates the run
- THEN the pure core MUST validate the summary independently from process logs
- AND the receipt MUST preserve status, settings, property identity, source scope, generator identity, and any counterexample

#### Scenario: Mixed process output cannot become evidence

- GIVEN a producer writes test-harness text, JSON fragments, or unrelated diagnostics only to stdout or stderr
- WHEN Hegel run evaluation executes
- THEN Octet MUST report a missing or malformed summary artifact
- AND it MUST NOT parse the mixed logs as a passing Hegel receipt

#### Scenario: Compatible import remains bounded

- GIVEN an external `hegel-property-summary/v1` input satisfies the accepted import contract
- WHEN import mode runs
- THEN Octet MUST preserve its bounded generated-testing evidence classification
- AND Octet MUST NOT invent hardened run measurements that the input does not contain

### Requirement: Hegel run identities are measured

r[octet.dynamic_evidence.hegel_measured_identity]

Octet MUST remeasure and bind the selected profile, Cargo lock, producer package, source scope, generator inputs, summary artifact, bounded logs, and retained counterexample before publishing a profiled Hegel receipt.

#### Scenario: Declared and measured identities match

- GIVEN the producer summary declares facts that match Octet's measured profile, package, source, generator, output, and counterexample facts
- WHEN receipt validation runs
- THEN Octet MUST publish their BLAKE3 identities and parent links deterministically

#### Scenario: Self-declared identity is stale

- GIVEN a declared version, source hash, generator hash, profile hash, summary hash, or counterexample hash differs from measured input
- WHEN receipt validation runs
- THEN validation MUST fail with a deterministic identity diagnostic
- AND the failed artifact MUST NOT be presented as passing evidence

#### Scenario: Output path is unsafe

- GIVEN a producer output or counterexample uses parent traversal, an absolute path, symlink substitution, an undeclared file, or an over-limit artifact
- WHEN artifact collection runs
- THEN collection MUST fail closed without reading or replacing data outside the capability root

### Requirement: Hegel hardening has positive and negative verification

r[octet.dynamic_evidence.hegel_hardening_verification]

The implementation MUST include deterministic positive and negative coverage for profile authority, output contracts, result classification, counterexamples, replay, identities, limits, paths, compatibility, evidence tiers, and default independence.

#### Scenario: Focused hardening matrix completes

- GIVEN profiled-pass, counterexample, replay, import, command, schema, output, path, identity, limit, timeout, and overclaim fixtures
- WHEN focused validation runs
- THEN every positive fixture MUST pass deterministically
- AND every negative fixture MUST fail for its declared diagnostic class
