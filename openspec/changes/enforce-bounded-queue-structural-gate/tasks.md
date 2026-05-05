## Phase 1: Implementation

- [ ] I1 Extend bounded primitive mechanism contracts and generated artifact export fields for approved APIs, forbidden APIs, capacity witnesses, and backpressure behavior
- [ ] I2 Update `tigerstyle-standards` resource checks to consume bounded queue obligations from generated policy artifacts
- [ ] I3 Add compliant and noncompliant bounded queue structural fixtures
- [ ] I4 Update downstream/Aspen documentation for the hard structural bounded queue gate

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Bounded queue artifact exports checker obligations" with policy artifact export assertions; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/policy-artifact-export.txt`
- [ ] V2 Verify scenario "Artifact-backed checker rejects configured unbounded constructors" with standards fixture check; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- [ ] V3 Verify scenario "Bounded construction with witness passes" with standards fixture check; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- [ ] V4 Verify scenario "Missing capacity witness remains a structural finding" with standards fixture check; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- [ ] V5 Verify scenario "Downstream mutation fails the policy artifact consumer check" with Aspen scratch mutation; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/aspen-policy-artifact-mutation.txt`
- [ ] V6 Verify focused Rust and Nix checks for the implementation; save output at `openspec/changes/enforce-bounded-queue-structural-gate/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`git -C /home/brittonr/git/aspen status --short --branch`; input=`Aspen checkout after scratch mutation proof`; asserts=`Aspen checkout has no retained mutation from the negative proof`; evidence=`openspec/changes/enforce-bounded-queue-structural-gate/evidence/aspen-clean-status.txt`

## Change Status
- Status: in-progress
- Open tasks: I1, I2, I3, I4, V1, V2, V3, V4, V5, V6, H1
## Verification Coverage

- `Scenario: Bounded queue artifact exports checker obligations` -> `V1` -> `openspec/changes/enforce-bounded-queue-structural-gate/evidence/policy-artifact-export.txt`
- `Scenario: Artifact-backed checker rejects configured unbounded constructors` -> `V2` -> `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- `Scenario: Bounded construction with witness passes` -> `V3` -> `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- `Scenario: Missing capacity witness remains a structural finding` -> `V4` -> `openspec/changes/enforce-bounded-queue-structural-gate/evidence/standards-fixtures.txt`
- `Scenario: Downstream mutation fails the policy artifact consumer check` -> `V5` -> `openspec/changes/enforce-bounded-queue-structural-gate/evidence/aspen-policy-artifact-mutation.txt`
