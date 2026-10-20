# Aeneas consumer contract

<!-- r[impl octet.aeneas_consumer_contract.boundaries] -->

This contract lets an external Rust project use the pinned Octet Aeneas rail.
It keeps provider authority and consumer authority separate.

The checked profile is:

- `standards/proof-rails/aeneas-lean/consumer-profile.ncl`
- `standards/proof-rails/aeneas-lean/generated/consumer-profile.json`
- profile identity `b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879`

## Scope

The contract binds these inputs:

1. The exact provider profile and compatibility matrix.
2. One exact Rust source file and selected item.
3. The selected item span and durable source identity.
4. The LLBC output, translation manifest, and generated Lean source.
5. One maintained theorem and its requirement mapping.
6. The declared resource policy, trust closure, and non-claims.

A transient Charon `def_id` is recorded as an observation. It is not part of the
durable item identity.

The contract does not prove:

- Aeneas or Charon translation soundness.
- Lean or verifier soundness.
- Whole-program correctness.
- Semantic equivalence outside the named theorem.
- Release eligibility.

## Use the checked profile

Pin an immutable Octet revision. Do not discover an ambient sibling checkout.
Export Octet policy artifacts from that pinned revision:

```console
nix run .#octet-policy-artifacts -- artifacts/policy
```

A selected consumer keeps these files in one project root:

```text
Cargo.toml
src/lib.rs
Theorem.lean
```

The checked fixture is at:

`standards/proof-rails/aeneas-lean/fixtures/external-consumer/`

Run its exact rail with the pinned provider closure:

```console
cargo octet prove \
  --policy-artifacts-dir artifacts/policy \
  --source src/lib.rs \
  --rail octet.aeneas_consumer_contract.external-fixture#rail-0 \
  --artifact-dir artifacts/aeneas-consumer
cargo octet artifact verify --artifact-dir artifacts/aeneas-consumer
```

The proof receipt contains a typed `consumer` binding. The binding includes the
profile, provider, source, item, generated stages, theorem, requirement, resource
policy, trust closure, outcome, and receipt identity.

## Outcomes

The contract uses three closed outcomes:

- `supported`: all exact bindings match and the named theorem passes.
- `blocked`: a required provider capability or compatibility condition is absent.
- `failed`: execution completed but the theorem or checked contract failed.

A blocked result is not a successful proof. An incomplete blocker is invalid.
A failed result is not a blocked result.

## Reject drift

The checker rejects these changes:

- Provider profile or compatibility-matrix substitution.
- Source, selected-item, span, or theorem changes.
- Path escape or symbolic-link substitution.
- Ambiguous selected-item matches.
- Missing LLBC, translation, generated Lean, or theorem bindings.
- Resource-policy or environment-allowlist drift.
- Empty trust boundaries or non-claims.
- Unknown Nickel or JSON fields.
- Receipt identity or stage-artifact tampering.

Review each provider or profile upgrade as a new cohort. Run the compatibility
matrix and regenerate the consumer profile. Do not update one identity alone.

## Ownership and deferred translators

Octet owns the consumer rail, execution policy, and receipt construction.
The consumer owns theorem intent, requirement mapping, and theorem campaigns.
Trellis owns reusable verified logic and its proof corpus when a consumer uses it.
A Trellis proof does not gain Octet consumer-proof authority without an Octet rerun.

Valence owns evidence identity and linkage. Mantle owns bounded build evidence.
Cairn owns lifecycle gates and release-readiness decisions. Workspace membership
does not transfer authority between these owners.

Eurydice and Scylla remain deferred translation references. This consumer
contract does not execute them or claim compatibility with their outputs.

## Downstream profiles

A downstream project can define another closed Nickel profile with the same
contracts. Load its exact exported bytes with `aeneas_consumer::load_profile`.
Pass the returned measured identity to `build_binding` and
`validate_receipt_against`.

The profile must select its own source paths, items, theorems, requirements,
resource limits, trust boundaries, and non-claims.

Octet does not infer these declarations from repository membership. It does not
grant proof authority because a project uses the same workspace or provider.

## Mantle producer handoff

Octet owns the executable handoff in
`standards/proof-rails/aeneas-lean/mantle-job.ncl`
(`octet-mantle-producer-job/v1`). It selects this existing consumer/provider
profile and reviewed external-fixture rail, their resource bounds, exact
commands, denied-network execution and output contract. It introduces no
alternative theorem admission policy. This selected Lean rail has no SMT solver.

`checks.x86_64-linux.mantle-producer-export` exercises graph admission and exports
this typed job with the pinned Nickel evaluator; it does not execute the proof.

`packages.x86_64-linux.octet-mantle-aeneas-consumer-job` realizes that contract
as an ordinary derivation: it runs actual `cargo-octet prove` and
`cargo-octet artifact verify`, requires successful owner verification, and
preserves the original report, envelope and full artifact payload tree. Cached
output bytes record their original acceptance; reuse is not fresh proof work.

`tools/mantle-producer/export.py --help` describes acquisition-only export from
an explicit immutable committed flake revision. It retains the exact job,
recursive Nix derivation JSON, flake metadata/lock, selected root/revision and
all declared source objects. Legacy flat and explicit Nix v4 reference shapes
are admitted separately; the original JSON graph bytes are never rewritten or
hash-relabelled as another format. Unknown/mixed versions, incomplete edges and
dynamic outputs are rejected. The exporter requires `--revision COMMITTED_SHA`
and resolves the immutable flake before exporting Nickel from its locked path,
including imported consumer/provider contracts. It refuses to substitute a
mutable checkout job or an old fixture pin. Graph size, derivation/source count
and source bytes are bounded. Acquisition does not build tools or execute a proof.
The complete graph includes declared bootstrap/binary acquisition inputs; it
does not claim source-only or seed-free bootstrap.

Mantle may consume that frozen graph through its existing build/source/store
boundaries and preserve Octet's full external acceptance report. Mantle does
not gain profile, theorem, proof admission, signature trust or release authority
by doing so. Real execution/replay evidence must identify the actual route;
importing a prebuilt closure is not evidence of source-built tools.
