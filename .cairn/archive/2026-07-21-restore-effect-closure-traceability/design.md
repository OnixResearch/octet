# Design

## Success contract

Goal: restore the entire accepted effect-closure capability to Cairn's maintained traceability profile and permanently gate missing/dangling regressions.

Observable completion evidence:

- both profiles select `cairn/specs/effect-closure-admission/spec.md` and the same exact effect-closure evidence roots;
- every accepted `octet.effect_closure.*` ID has semantically matching maintained evidence;
- Cairn traceability exits zero with `missing: []` and `dangling: []`;
- a hermetic negative copy with both `octet.effect_closure.cli.check_failure` markers removed exits nonzero and reports exactly that missing ID;
- policy freshness, focused effect-closure tests, Nextest, strict validation, and the broad flake gate pass.

False completion includes deleting the dangling marker, selecting only a fragment of the accepted spec, broadening evidence discovery to all source/docs, scanning lifecycle archives, manually editing generated JSON, accepting coverage with newly introduced missing IDs, or treating marker coverage as behavioral proof.

## Approach registry

| Family | Mechanism | State | Evidence / blocker |
| --- | --- | --- | --- |
| complete accepted-source restoration | Register the full accepted spec and exact maintained core/shell/docs roots | active | Existing 30-ID spec maps to current `effect_closure.rs`, `admit.rs`, `cli.rs`, integration tests, and operator docs |
| dangling-marker deletion | Remove the two direct-check failure markers | falsified | The fail-closed path is implemented and its integration test passes; deletion hides evidence |
| one-scenario source fragment | Create a synthetic requirement source containing only `cli.check_failure` | rejected | It forks accepted authority and leaves the rest of the current capability outside coverage |
| broad source scan | Select all `cargo-octet/src` and all docs | rejected | It weakens ownership review and can admit unrelated future markers |
| archive/task scan | Treat the original archived change as maintained evidence | rejected | Historical completion is provenance, not executable current evidence |

## Exact source ownership

The requirement source is `cairn/specs/effect-closure-admission/spec.md`. Evidence roots added to both profiles are:

- `cargo-octet/src/admit.rs` for the two-stage imperative shell, direct-check refusal, receipt emission, and verification;
- `cargo-octet/src/cli.rs` for CLI parsing and integration behavior;
- `cargo-octet/src/effect_closure.rs` for pure policy validation, inference, roles, dependency summaries, fixed-point closure, evidence construction, and positive/negative tests;
- `docs/effect-closure-admission.md` for opt-in boundaries, non-claims, and bounded Valence dogfood.

The existing `tests` evidence root retains the direct-check failure integration test. Three parent IDs—boundary non-claims, dogfood, and verification—gain markers beside their already maintained child evidence.

## Permanent gate

`checks.<system>.cairn-traceability` uses the pinned Cairn flake input. The positive case scans the immutable repository source and requires exit zero plus empty `missing` and `dangling` arrays. The negative case copies that source to a temporary writable directory, removes both implementation and integration markers for the direct-check failure ID, requires nonzero exit, and asserts exactly one missing ID with no dangling IDs. The gate never rewrites the checked-in source or generated policy.

## Functional boundary and claims

Cairn remains the traceability decision core and filesystem shell. Octet's Nix gate only supplies explicit source snapshots and asserts the typed JSON result. Coverage establishes current ID linkage, not runtime reachability, inference completeness, dependency correctness, whole-program correctness, verifier soundness, or release eligibility.

## Search budget

This second slice uses one accepted-spec read, four exact evidence-root audits, one flake integration pass, one negative-fixture audit, and two validation rounds. Authority is limited to accepted specs, typed Nickel policy, current implementation/tests/docs, and the pinned Cairn checker. Allowed outcomes are validated, blocked, exhausted, or user-decision-required.
