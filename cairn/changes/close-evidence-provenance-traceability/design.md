# Design

## Success contract

Goal: make all 32 accepted evidence-provenance requirements discoverable from semantically matching maintained evidence without weakening the accepted specs.

Observable completion evidence:

- both Nickel-owned traceability profiles name the same exact additional source files;
- generated policy JSON is fresh;
- focused positive and negative tests for all four families pass;
- Cairn's external function-address stack smoke passes at the recorded revision;
- Cairn traceability reports `missing: []` and only `octet.effect_closure.cli.check_failure` as dangling;
- strict Cairn validation and broad repository gates pass.

False completion includes deleting or weakening accepted requirements, adding lifecycle/archive directories as evidence, broadening the evidence root to all of `cargo-octet/src`, using task completion as implementation evidence, linking an ID to unrelated code, or claiming that marker coverage proves behavior.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| exact-source semantic linkage | Add four exact Rust modules and two exact docs, then place markers on matching cores/tests/docs | active | Existing implementations and positive/negative tests map directly to 31 IDs; external stack smoke maps the remaining validation ID |
| accepted-requirement deletion | Remove IDs until the report is green | falsified | Implementations and accepted contracts remain current; deletion would weaken policy |
| broad source discovery | Scan all `cargo-octet/src` or the whole repository | rejected | It obscures ownership and allows unrelated future markers to satisfy policy accidentally |
| lifecycle evidence reuse | Scan archived tasks and historical validation prose | rejected | Task completion and historical prose are not maintained executable evidence |
| source-only unit linkage for stack smoke | Link the cross-stack requirement to the local function-address unit test | rejected | The requirement names Octet, Kamacite, Valence, Mantle, and Cairn; Cairn's maintained stack smoke is the bounded external evidence |

## Selected design

The typed source of truth remains `cairn-policy/default.ncl`; generated JSON is regenerated, never hand-edited. Both profiles receive identical exact source rows for:

- `cargo-octet/src/evidence_chain.rs`
- `cargo-octet/src/function_address.rs`
- `cargo-octet/src/formal_proof_valence_export_fixture.rs`
- `cargo-octet/src/preserves_receipt_export.rs`
- `docs/formal-proof-valence-export-fixture.md`
- `docs/preserves-receipt-export.md`

Existing `cargo-octet/src/artifact.rs`, `README.md`, and `docs/trust-boundary-evidence.md` remain the Hegel and stack-boundary evidence roots.

Markers are attached by role: `impl` to pure validators/builders, `verify` to positive/negative tests and bounded report assertions, and `depends` to documentation that names the external stack smoke. Test-only modules are acceptable only for requirements that explicitly define frozen fixtures or fixture validation; they are not presented as public runtime commands.

## Cross-repository boundary

Cairn owns the maintained `function_address_stack_smoke` fixture and test. Octet records its exact revision and command as dependency evidence. Passing it proves the selected fixture crosses the modeled stack with preserved identity/linkage/non-claims; it does not prove arbitrary versions, production deployment, behavioral correctness, or release eligibility.

## Search budget

Authority is bounded to accepted Octet specs, current implementation/tests/docs, typed Nickel policy, Cairn's maintained smoke, and deterministic validators. Retrieval budget: twelve targeted file/search passes, one advisory review, three implementation/audit rounds, and no network retrieval. Allowed outcomes are validated, blocked on a semantically unproven requirement, exhausted, or user-decision-required.
