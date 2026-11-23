# Spec-trace parity rail

The spec-trace parity rail replays pinned, seed-bound traces from a behavioral
model against a target crate through an injected adapter. It is a checked
admission surface, not a model checker: Octet owns identity binding, freshness,
bounded replay, verdict classification, and deterministic receipts. The model
layer owns model production and the target crate owns its replay adapter.

The pure core lives in
`octet_standards::spec_trace_parity`. It does not read files, spawn processes,
inspect a clock, or discover adapters.

## Check surface

A run is bound to six identities:

| Identity | Meaning |
| --- | --- |
| `model_identity` | BLAKE3 identity of the exact behavioral model bytes |
| `checker_identity` | BLAKE3 identity of the model checker build |
| `checker_version` | Declared model checker version |
| `seed` | Explicit replay seed |
| `trace_set_identity` | BLAKE3 identity of the canonical trace set |
| `adapter_identity` | BLAKE3 identity of the adapter declaration, or absent |

The trace set must declare the stack interchange identity
`octet.spec-trace/v1`. Foreign formats are mapped to stack identity by Kamacite
before they reach this rail; the rail rejects an unknown format rather than
guessing.

## Freshness

`admit` runs before any replay step and fails closed on a stale model, checker
identity, checker version, seed, trace-set identity, adapter identity, trace
format, or malformed adapter declaration. Each denial binds the denied layer,
the declared value, and the observed value. The denial receipt binds every
declared pin, so a stale pin is evidence rather than a silent skip.

## Adapter contract

Each target crate supplies its adapter through the injected `TraceAdapter`
port. The adapter must declare:

- a stable adapter identifier;
- the trace format it consumes;
- a state projection naming both state vocabularies and its description; and
- a loss bound, either `Exact` or `Bounded { bound }`.

An adapter that omits its projection or loss bound is rejected before replay.
A bounded (lossy) projection may still report parity, but the receipt records
the projection and loss bound and adds the projection non-claim. Parity under a
lossy projection is never reported as undisclosed parity: divergences merged by
the projection are outside the claim.

## Replay

`plan` records the admitted identities, the adapter projection, the loss bound,
and the declared bounds. `run` replays every trace in declared order with the
admitted seed. The core enforces:

- `max_steps`: a trace set that cannot fit the step budget is denied before any
  step runs;
- `max_millis`: the shell reports cumulative elapsed time with each adapter
  observation, and the core stops after the first observation that exceeds the
  budget.

Identical adapters, traces, seeds, and reported timings produce identical step
sequences and receipt identities. Receipts bind the declared bounds, not raw
clock readings, so a time-budget denial is classified deterministically.

## Verdicts and receipts

The closed verdict vocabulary is `parity`, `divergence`, and
`unsupported-verifier`. A run that fails freshness or exceeds a bound is not a
verdict: its receipt records a deterministic denial instead of claiming a
class.

A receipt binds the model, checker, seed, trace-set, adapter, bounds, verdict
class or denial layers, replayed step count, bounded divergence samples, and
non-claims. `validate_receipt` recomputes the admitted identities, the replay
prefix, the divergence samples, and the verdict class from the observed replay
record; a receipt whose class, identities, step count, or samples disagree with
the observed run fails closed.

## Integration boundary

The rail is a pure checked core with an injected adapter port. There is no
`cargo octet` or `octet evidence` subcommand yet: an adapter is per-crate code,
not data, and Octet has no adapter-loading ABI. The imperative shell that owns a
target crate must:

1. load the model bytes, trace set, pins, and adapter;
2. measure cumulative replay time and pass it in `StepObservation`;
3. call `spec_trace_parity::run`; and
4. persist the returned receipt and replay record.

Wiring the rail into `cargo-octet` is deferred until an adapter loading surface
exists. Until then the core is exercised by the committed positive and negative
fixtures in `spec_trace_parity::tests`.

## Fixtures

Positive fixtures cover fresh parity, deterministic replay, projection and loss
bound recording, seed binding, and opt-in Dive schema-4 pin/report admission.
Negative fixtures cover stale model, stale checker identity and version, seed
drift, tampered traces, unsupported trace format, divergence, adapter loss,
missing adapter projection or loss bound, step and time bound denials,
unsupported verifier, swapped receipt classes, receipt identity drift,
malformed requests, Dive schema/commit pin drift, incomplete Dive schema-4
reports stopping as `Unsupported`, and `DiveReport` unknown-field rejection.

## Non-claims

A passing parity rail proves only that the admitted traces matched the
implementation under the pinned model, checker, seed, trace set, and adapter
projection. It does not prove model validity, checker soundness, behavior
outside the traced set, or release eligibility. A divergence is evidence of one
observed mismatch, not a proof about every execution.

## Optional Dive `validate-trace` pin (`trace-lab#6`)

Octet does not vendor `Chelis-Lang/dive`, does not package Dive, and does not
run `validate-trace`. For the live-Raft trace-validation tracker
(`OnixResearch/trace-lab#6`), Octet records an opt-in, default-off pin in
`octet_standards::spec_trace_parity::external`:

| Field | Pinned value |
| --- | --- |
| Upstream repository | `https://github.com/Chelis-Lang/dive` |
| Tracker | `OnixResearch/trace-lab#6` |
| Pinned `main` commit | `0fc00914b7cd93d88e97e5d7d66aac805c06228b` (observed 2026-11-20) |
| Trace report schema | `4` (`docs/user/traces.md` at pinned commit) |

This pin supersedes Dive `0.1.0` (`94cbbed9ae02ad3d30bbe2191bd2619a839b8795`)
for `OnixResearch/trace-lab#6` only. `aspen#62` retains its own separate
third-checker parity campaign and `0.1.0` pin. If a future Dive release tag
ships schema 4, a follow-up may move the pin to that tag's commit SHA.

Recorded local handoff context (`OnixResearch/trace-lab@e983e6bde13e9afcf24c9b0820ca3bd7ae689509`):
exercised `0fc00914b7cd93d88e97e5d7d66aac805c06228b` with Rust 1.99.0 (local
executable BLAKE3 `2a39fc7ca7c05679c7521c1f390183dc8182c5f9e2e4b7eca2ce50bfe8e7f09c`),
producing schema-4 reports. That observation is an exact local build and report
handoff, not a reusable Octet package or admission of either trace into
`spec-trace-parity`.

### Opt-in admission boundary

`spec_trace_parity::external::admit_dive_report` is selected only by explicit
caller request and does not change default `cargo octet check`, policy, or
existing evidence profiles. It reuses `spec_trace_parity::admit` rather than
inventing a new rail:

- A report whose `schema_version != 4` or `dive_commit_sha != 0fc00914b7cd93d88e97e5d7d66aac805c06228b`
  is denied fail-closed.
- A schema-4 report that lacks any required binding (`model_bytes`, `seed`, or
  mapped `octet.spec-trace/v1` trace set) returns `DiveAdmission::Unsupported`
  with explicit non-claims and stops; that is a complete result.
- A complete schema-4 report with all four identities (`model`, `checker`,
  `seed`, `trace_set`) is checked through `spec_trace_parity::admit` against
  the caller's `ParityPins` (`pins.adapter_identity` must be `None`, because
  `admit_dive_report` passes no `AdapterDeclaration` to `admit`).

This pin does not take over `octet#27` (ChaosControl evidence/replay),
`octet#30` (Cairn/Buoy structural linkage), `aspen#62`, or `compiler-lab#264`.
