# Design: Bounded four-version semantic merge verification rail

## Context

Octet is the evidence and policy owner for checked Rust rails, but merge
correctness is a language-level behavioral question. Noble owns its language
relation and verifier integration, Valence owns typed evidence identity and
linkage, and Kiln owns scheduling and CI interpretation. Octet's Rust lane must
not become the authority for arbitrary Noble program behavior.

This change therefore produces a reviewed contract plus a bounded, executable
pure-core experiment. It does not admit a production merge verifier.

## Required relation

For role-tagged O/A/B/M programs, a declared observable set, and an admitted
input tuple, let `O(i)`, `A(i)`, `B(i)`, and `M(i)` be observations:

- `A(i) != O(i)` implies `M(i) == A(i)`;
- `B(i) != O(i)` implies `M(i) == B(i)`;
- `A(i) == O(i)` and `B(i) == O(i)` implies `M(i) == O(i)`.

When `A(i) != O(i)`, `B(i) != O(i)`, and `A(i) != B(i)`, the relation is
unsatisfiable at `i`. The rail reports that incompatibility rather than choosing
a variant or reconstructing intent. Abort observations are ordinary values for
this comparison.

## Admitted subset

The prototype's smallest useful deterministic subset is pure scalar functions
over `i64` arguments:

- literals, positional arguments, add/sub/mul/div/rem/neg;
- `if` on integer conditions, with zero false and nonzero true;
- direct calls to mapped logical items.

The subset excludes unsafe code, raw pointers, FFI, concurrency, opaque macros,
dynamic and trait dispatch, external effects, floating point, panics,
nontermination, memory aliasing, and exceptional control flow. Unrepresentable
constructs produce `unsupported`.

Arithmetic is an explicit profile assumption:

- `checked` reports overflow and division/remainder by zero as abnormal
  `overflow` and `divide-by-zero` observations;
- `wrapping` uses two's-complement wrapping for add/sub/mul/div/rem/neg and
  still reports division/remainder by zero as abnormal.

Preconditions are conjunctions of per-argument integer comparisons over a
bounded candidate value set. The observed input set is the cartesian product of
candidate values up to an explicit case bound, filtered by the precondition.
Any observable with no admitted input yields `unknown`; an empty domain must
never produce a vacuous `verified`.

The memory and aliasing model is `pure-scalar`: arguments in, scalar result out,
no memory operation, mutation, pointer, or alias. Extending the model requires a
new reviewed profile and explicit aliasing assumptions.

## Rust frontend

The rail includes a bounded `syn`-based frontend for the admitted scalar
subset. It maps every admitted function exactly once, rejects unsupported
constructs fail-closed, and emits a correspondence report with the exact source
text identity, emitted program identity, function inventory, obligation, and
non-claims. The mapping is syntactic; it does not prove Rust semantics,
compiler correctness, whole-program behavior, or the absence of effects outside
the parsed items. The merge receipt's `typed-scalar-ir` frontend identity is
unchanged for hand-authored fixtures.

## Observables and dependency closure

An observable is a logical item and arity with a bounded input domain. The
observable set is assumed complete only for the declared reviewed closure
(`declared-closure-only`); the receipt carries that assumption as a non-claim
about unlisted behavior.

The dependency closure is the transitive set of logical items reachable from
the observables. Every reachable dependency must have the same semantic
identity in all four variants, unless it is itself a declared observable, in
which case it participates in the four-version relation. A reachable dependency
set that differs by role, or a shared dependency whose identity differs, is
`unsupported` (`changed-dependency`). Cycles are `unsupported` (`recursion`)
rather than an implicit fixed-point claim.

## Item and interface matching

Every logical item has a role-tagged name mapping. Renames use distinct
per-role names in one mapping; signature changes must agree with the declared
arity and every call site; changed dependencies must either be declared
observables or fail closed; missing items fail closed. Textual similarity,
function hashes, lint outcomes, and Charon comparison rows are not
correspondence evidence.

The prototype's typed IR is authored by hand. The source-to-IR correspondence
is therefore explicitly unproven: each variant binds the BLAKE3 identity of its
reviewed source text and the canonical identity of its role-local typed program,
but nothing in this change checks that the latter faithfully represents the
former.

## Direct checking versus summaries

Direct four-program checking is selected for the prototype. Each variant is
evaluated independently on the same admitted inputs, then the relation is
checked per observable and input. The trusted base is the evaluator for the
admitted integer operations, the explicit arithmetic profile, the relation
checker, and canonical BLAKE3 serialization.

Shared-fragment summaries and product construction are not implemented. A later
design may use them only when every summary binds checked premises, the exact
observable or function contract it represents, and exact dependency identities.
If those premises cannot be justified, the rail must fall back to `unknown`.
A symbolic-model proof, if added, would remain distinct from Rust extraction,
IR correspondence, and runtime behavior.

## External Lean backend

The optional `LeanBackend` computes the expected observation with the in-memory
evaluator and asks the pinned Lean kernel to check a concrete theorem binding
the encoded program result to that observation. It supports checked arithmetic
only. The trusted base is the Lean kernel, the fixed generated encoding, and the
pinned Lean executable. Evidence is bounded to the supplied input; it is not a
universal relation proof or a Rust compiler-correctness claim. Lean rejection,
timeout, spawn failure, or nonzero exit maps to `tool-error`, never a semantic
conflict.

## Outcomes

- `verified`: the relation held for every declared observable and admitted
  input in the bounded profile.
- `counterexample`: a deterministic observation, input, and failure class
  (`incompatible`, `lost-a-change`, `lost-b-change`, `interference`) records the
  violation with O/A/B/M values.
- `unknown`: the checker could not justify a verdict, for example a vacuous
  precondition or an unjustifiable dependency summary.
- `unsupported`: the request is outside the admitted subset or lacks an exact
  mapping; this includes unsupported constructs, missing observables or items,
  binding and role mismatch, signature mismatch, changed dependencies, and
  domain overflow.
- `timeout`: an explicit evaluation or step budget interrupted the run.
- `tool-error`: the injected backend failed.

A solver warning, timeout, or failure to prove is never itself a semantic
conflict. Structural admission failures (missing or duplicate roles or
bindings, zero budgets, empty observable sets, malformed domains) produce a
deterministic admission error and no semantic verdict.

## Identity and receipts

Each admission identity binds the profile and claim scope, assumptions, all four
role-tagged variants, declared bindings, item mappings, observables and domains,
budget, and optional provenance. Each receipt additionally binds:

- frontend, model, verifier, backend, and optional solver identities and
  versions;
- role-tagged source and semantic-subject identities;
- observable coverage and domain identities, plus shared dependency identities;
- the exact command or library invocation label and the deterministic work
  report (evaluation and relation-check counts, not wall-clock time);
- the deterministic outcome and all consumer non-claims.

Receipt validation recomputes the receipt identity, checks the current admission
identity, verifies role/binding inventories, canonical ordering, identity
syntax, budgets, command, non-claims, work arithmetic, and outcome consistency.
A stale receipt or any payload tamper is rejected.

Pijul native change identities are optional recorded-only provenance. Clean
application, change algebra, and equal trees are not behavioral evidence.

## Functional core / imperative shell split

- **Pure core**: canonical admission, item mapping, binding checks, dependency
  closure, bounded domain enumeration, evaluation of the typed IR, relation
  classification, receipt construction, and receipt validation.
- **Imperative shell**: future Rust extraction, file and command access,
  artifact persistence, and any external solver. The prototype has no shell; it
  consumes typed requests and returns receipts.

The backend is injected. A future symbolic or external verifier can implement
the same evaluation contract only after its trusted base and proof obligations
are reviewed.

## Go/no-go decision

- **GO** for this bounded pure-core prototype and executed fixture corpus, with
  the profile, outcome semantics, receipts, and non-claims above.
- **NO-GO** for production admission, universal verification, and any claim of
  Rust source-to-IR correspondence. Before reconsideration, a separately scoped
  change must provide a reviewed Rust frontend, an explicit correspondence
  obligation and evidence, a verifier with a stated trusted base, coverage and
  solver-failure semantics, and consumer integration through Valence and Kiln.
  Charon comparisons, function hashes, lint results, and tests may not be
  relabeled as merge proofs.

## Consumer non-claims

A verdict does not provide automatic merge admission, source mutation,
whole-program correctness, semantic equivalence, developer-intent recovery,
release authority, or provider trust. Same-input and same-receipt determinism
establishes reproducibility of the bounded runner, not correctness of the
admitted profile beyond its checked facts.

## Risks and trade-offs

- Bounded input domains can miss violations outside the declared values. The
  receipt states the domain and never labels the result universal.
- Hand-authored IR can diverge from Rust source. The correspondence is declared
  unproven, and no extraction claim is made.
- Strict identity checks reject renamed or changed dependencies without an
  explicit mapping, trading convenience for fail-closed behavior.
- Per-observable preservation can reject intentional integrations. That is the
  reviewed criterion, not a defect.
- Dependency-set equality is stricter than per-call checking and intentionally
  reports `unsupported` for unreviewed closure changes.
- Counterexample receipts are bounded; truncation is explicit when the bound is
  reached, so a truncated list is not a complete conflict inventory.
