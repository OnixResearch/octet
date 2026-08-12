## Phase 1: Policy and compiler facts

- [ ] [serial] Extend architecture policy with exhaustive outbound effect classifications and protected-effect protocols r[octet.authorized_effect_protocol.classification]
- [ ] [serial] Add exact decision, witness, constructor, executor, plan, observation, receipt, cardinality, and non-claim fields r[octet.authorized_effect_protocol.schema]
- [ ] [serial] Extend Architecture IR with witness visibility, derives, conversions, constructors, executor signatures, and selected MIR provenance facts r[octet.authorized_effect_protocol.witness]
- [ ] [serial] Record unsupported or ambiguous witness provenance as a typed blocking unknown r[octet.authorized_effect_protocol.origin]

## Phase 2: Pure protocol evaluation

- [ ] [serial] Reject unclassified effectful adapters and malformed protected protocols r[octet.authorized_effect_protocol.classification]
- [ ] [serial] Enforce private witness fields, restricted constructors, forbidden derives, forbidden conversions, and use cardinality r[octet.authorized_effect_protocol.witness]
- [ ] [serial] Require witness provenance from an approved domain decision output r[octet.authorized_effect_protocol.origin]
- [ ] [serial] Require every protected executor and wrapper path to preserve the witness parameter r[octet.authorized_effect_protocol.executor]
- [ ] [serial] Enforce distinct plan, attempt, observation, and receipt types and reject receipt construction without an observation r[octet.authorized_effect_protocol.receipts]
- [ ] [serial] Preserve denied decisions as no-witness and no-protected-executor outcomes r[octet.authorized_effect_protocol.denial]

## Phase 3: Evidence and guidance

- [ ] [serial] Add protocol results, findings, unknowns, and type identities to the architecture receipt r[octet.authorized_effect_protocol.evidence]
- [ ] [parallel] Add a typed consumer example with application-owned policy, port, witness, adapter, observation, and receipt types r[octet.authorized_effect_protocol.schema]
- [ ] [parallel] Document authority ownership, replay risks, observation boundaries, and explicit non-claims r[octet.authorized_effect_protocol.evidence]

## Phase 4: Verification

- [ ] [parallel] Add positive fixtures for authorized execution, denial without effects, move-only witnesses, scoped reusable authority, observed success, and observed failure r[octet.authorized_effect_protocol.verification]
- [ ] [parallel] Add negative fixtures for missing classification, missing witness, public fields, `Default`, `Deserialize`, broad conversion, cloning, and replay r[octet.authorized_effect_protocol.verification]
- [ ] [parallel] Add negative fixtures for unguarded wrappers, unknown provenance, pre-decision execution, receipt-from-plan, and false success receipts r[octet.authorized_effect_protocol.verification]
- [ ] [serial] Run focused policy, compiler, evaluator, receipt, documentation, Cairn, gate, and traceability checks r[octet.authorized_effect_protocol.verification]