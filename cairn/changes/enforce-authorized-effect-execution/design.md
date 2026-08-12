## Context

Hexagonal dependency direction does not prove that a protected effect followed an authority decision. A shell can call a valid adapter before policy approval.

A typed witness can make this ordering visible. A separate observed-outcome type can prevent planned work from becoming execution evidence.

The project architecture gate supplies exact policy, compiler facts, local call edges, type facts, and receipts.

## Decisions

### Decision: Keep product authority application-owned

**Choice:** The project policy declares exact qualified names for its authority inputs, decision outputs, witness types, witness constructors, protected executor methods, plans, observations, and receipts.

Octet supplies a protocol checker. It does not supply product policy or a universal authorization type.

**Rationale:** Domain authority and capability meaning belong to the consuming application.

### Decision: Classify every outbound effect family

**Choice:** Each declared outbound effect family is `unprotected` with a reason or `protected` with an authorization protocol.

An effectful adapter without a classification blocks `gate` and `admission`.

**Rationale:** Omission must not become implicit authorization.

### Decision: Require an opaque authorization witness

**Choice:** A protected protocol declares one witness type. Compiler facts must show:

- all fields are private outside the owner module;
- approved constructors have restricted visibility;
- no `Default`, `Deserialize`, broad `From` or `Into`, public unchecked constructor, or unsafe construction path exists;
- one-shot witnesses are not `Copy` or `Clone`;
- the type carries the protected effect family or plan identity required by policy.

**Rationale:** The easiest invalid witness paths must be unrepresentable or gate-visible.

### Decision: Bind witness origin to a domain decision

**Choice:** Policy lists exact approved decision functions and authorized output types.

Selected compiler facts extend the Architecture IR with local MIR def-use provenance for witness values. A witness reaching a protected executor must originate from an approved decision output.

Missing, ambiguous, merged, or unsupported provenance is `unknown` and blocks strict modes.

**Rationale:** A matching type alone does not prove the application obtained approval.

### Decision: Make use cardinality explicit

**Choice:** Each protocol selects `consume` or `borrow` witness use.

`consume` requires move-only use and rejects replay through cloning. `borrow` requires an explicit reusable-authority rationale and scope.

**Rationale:** One-shot and reusable authority have different replay risks.

### Decision: Require the witness at every protected executor

**Choice:** Each declared protected executor method includes the witness in its compiler-resolved signature.

All adapter implementations preserve that parameter. All selected call sites provide a witness with accepted provenance.

Wrappers cannot erase the witness requirement before the external effect site.

**Rationale:** Protected effects must not have a second unguarded entry point.

### Decision: Separate plan, attempt, observation, and receipt

**Choice:** The project declares distinct types for:

- planned effect;
- authorized effect or witness;
- execution attempt;
- observed outcome;
- execution receipt.

Receipt constructors accept the observed outcome and bind the plan or attempt identity. They do not accept a plan alone.

Broad conversions from plans or attempts into receipts are forbidden.

**Rationale:** Requested work is not observed work.

### Decision: Record denial and non-execution

**Choice:** A denied decision returns no authorization witness. Shell tests must prove that protected executor ports receive no call on denial.

A failed execution can create a failure observation and bounded receipt. It cannot create a success receipt.

**Rationale:** Negative authority and effect paths need explicit evidence.

### Decision: Add protocol evidence to the architecture receipt

**Choice:** The architecture receipt records protected classifications, witness invariants, approved constructors, executor signatures, provenance results, observation boundaries, findings, and non-claims.

**Rationale:** Reviewers need evidence for both type shape and checked flow.

## Required Flow

```text
validated facts + actor authority + state + request
  -> domain decision
  -> denied result or authorized plan with witness
  -> application shell
  -> protected executor port with witness
  -> outbound adapter
  -> external attempt
  -> observed outcome
  -> execution receipt
```

## Functional Core and Imperative Shell

The product domain core owns the authority decision and effect-plan meaning. The product shell executes only an authorized plan.

The Octet protocol evaluator is a pure core over project policy and Architecture IR. The Octet shell only gathers compiler facts and writes receipts.

## Claim Boundaries

A passing protocol check proves only that declared compiler-visible types, constructors, signatures, and selected local provenance satisfy the configured contract.

It does not prove credential validity, revocation freshness, replay storage, actor trust, business-policy correctness, external execution success, or receipt authenticity.

## Risks / Trade-offs

- MIR provenance support can be incomplete. Strict modes fail on unknown provenance rather than infer approval.
- Reusable witnesses can hide broad authority. Borrow mode requires an explicit rationale and scope.
- Product APIs need migration. Inventory and advisory modes can show unguarded executor paths before promotion.
- Private constructors do not prove correct policy. Domain tests and separate authority evidence remain required.

## Validation

Positive fixtures cover authorized execution, denial without effects, move-only witnesses, reusable scoped authority, observed success, and observed failure.

Negative fixtures cover missing classification, missing witness, forged fields, `Default`, `Deserialize`, broad conversion, public construction, cloning, replay, unguarded wrappers, unknown provenance, pre-decision execution, receipt-from-plan, and success receipt from failure observation.