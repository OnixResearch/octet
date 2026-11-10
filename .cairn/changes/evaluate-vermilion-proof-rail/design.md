# Design: Evaluate Vermilion alongside Verus and Aeneas proof rails

## Context

The existing proof rails bind one provider, a reviewed profile, an exact
command, source paths, trust boundaries, and a receipt. The Aeneas consumer
contract adds exact source/item, LLBC, translation manifest, generated Lean,
maintained theorem, resource policy, and trust closure, and classifies execution
as `supported`, `blocked`, or `failed`.

Vermilion reuses the Verus front/middle end and emits Lean theorem obligations.
Its trusted surface therefore shares a front end with Verus and a kernel with
Aeneas. A comparison must keep those shared edges visible.

On 2026-11-09 the pinned Vermilion/Verus/Lean/cvc5 closure was built and
exercised: Lean 4.33.0, the Mathlib cache, the Vermilion Lean libraries, the
Rust workspace, and the Verus fork at `4860083` built successfully. A live
`m1-pipeline` canary and a live u64 checked-increment pilot both passed Lean
kernel checking with zero `sorry`. The pinned Aeneas/Charon closure also built
and passed four live compatibility tests. The Aeneas u64 pilot bridge is
checked with a universal Lean theorem over the generated model; the Vermilion
plain-Rust bridge and the Verus plain-Rust refinement theorem remain open.

## Decisions

### Decision: Pin the provider as an exact cohort

The profile binds the Vermilion revision, the Verus fork branch and revision,
vstd, translator revision and argv, Lean toolchain, Mathlib and every
`lake-manifest.json` dependency, the checker argv, environment allowlist,
resource bounds, allowed axioms, proof-hole markers, and non-claims.
`lake-manifest.json` bytes have their own BLAKE3 identity, so a missing or
substituted dependency audit fails closed.

Changing one pin requires a fresh compatibility run. The compatibility matrix
names live and pure canaries. The pinned closure was built on 2026-11-09; the
live `m1-pipeline` canary and the live u64 checked-increment pilot are recorded
under `standards/proof-rails/vermilion-lean/evidence/`. The Aeneas/Charon
closure and its live compatibility matrix are recorded there as well.

### Decision: One common semantic contract, explicit route bridges

The pilot subject is `increment_checked(value: u64) -> Option<u64>` with
`value < u64::MAX` implied by checked behavior and `None` at `u64::MAX`. The
statement bytes and subject bytes have stable BLAKE3 identities.

Each route declares the obligations it can actually discharge and one
refinement-bridge identifier. The Verus route checks the u64 checked contract
model with a production proof rail; the Aeneas route checks the extracted u64
model with a universal Lean theorem; the Vermilion route checks the live
generated value and overflow obligations. The Vermilion plain-Rust bridge and
the Verus plain-Rust refinement theorem remain open; the Aeneas bridge is
checked. Open bridges stay recorded as obligations, not hidden in prose.

### Decision: Receipts are typed and fail closed

Receipts bind every field required by the issue, plus artifact roles for each
generated source and proof twin. Validation derives required artifact roles
from the obligations, compares generated and twin identities, checks obligation
identity/kind set equality against the contract, checks status and theorem
coverage, checks axiom inventory against assumptions and the reviewed
allowance, checks checker completeness and exit code, and recomputes the
receipt identity.

The checker result must not be empty, interrupted, unavailable, or incomplete.
A `sorry` or `admit` in an assumption or theorem name fails admission.

### Decision: Synthetic fixtures cannot become property evidence

The positive fixture is a `synthetic_contract_fixture`. Its contract outcome is
`supported`, but its evidence disposition must be `recorded_only`. A synthetic
receipt requesting `property` evidence fails with `evidence_overclaim`. Only a
complete `provider_run` receipt can request property evidence.

### Decision: Comparison is a separate closed vocabulary

Comparison accepts route claims and outcomes. Different property identities
produce `incomparable_claims`; same identity with different width, overflow,
precondition, result, or statement produces `semantic_mismatch`; bridge gaps
produce `incomparable_claims`; proof failure, blocked, timeout, and tool error
are distinct. Every category carries an explicit mapping to the existing Aeneas
and Vermilion outcome spellings. `evidence_merge` is always false, including
for `comparable_agreement`.

### Decision: Keep downstream integration out of scope

The module is pure and opt-in. It does not add a proof rail to the policy
registry, a proof card, an artifact bundle role, or a Valence export. Those
surfaces require a measured provider run and a separate reviewed change.

## Test Design

Pure tests cover the exact profile, exact contract, actual pilot source and
statement bytes, the sealed positive fixture, absence of the toolchain, route
comparison categories, outcome mappings, and agreement without authority
merging.

The negative matrix applies typed mutations to the positive fixture and
reseals the receipt projection except for the deliberate tamper case. Each case
asserts the derived outcome and primary diagnostic class only; no negative
fixture may admit property evidence.

## Claim Boundary

Passing these tests proves only that the typed contract rejects the declared
invalid shapes and accepts the declared complete synthetic shape under the
reviewed pins. It does not prove that Vermilion or Lean accepts any obligation,
that any route refines the executable Rust function, or that the three
providers agree semantically.
