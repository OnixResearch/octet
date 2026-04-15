## Context

Current diagnostics use ad-hoc wording like `diag.warn(message)` plus a
lint-specific `diag.help(...)`. There is no shared contract for help text,
reference notes, or principle naming. The repo currently registers 25 lints
across five spec domains, but the change artifacts do not yet carry one
authoritative lint -> principle -> domain mapping. That gap makes it easy to
miss a lint, drift group membership, or update snapshots with inconsistent
wording.

The existing standalone UI harness already compiles fixtures with
`--edition=2024` and prints each fixture name. Tool-scoped crate attrs like
`#![allow(tigerstyle::safety)]` still require `#![feature(register_tool)]`
and `#![register_tool(tigerstyle)]` inside the fixture crate.

## Goals / Non-Goals

**Goals:**
- Give every lint the same warning/help/note structure
- Keep principle names, guide sections, and domain membership in one source
  of truth
- Prove group-level `#[allow]` and `#[deny]` behavior in standalone UI tests
- Update snapshots once, after the contract is settled

**Non-Goals:**
- Hosted documentation links or interactive diagnostics
- New lints or behavior changes to lint detection logic
- Automatic fix suggestions; those stay in `autofix-suggestions`

## Decisions

### 1. Canonical diagnostic wording

**Choice:** Every lint emits:
1. One warning message describing the violation
2. One help note in the exact form `Tiger Style (<principle>): <guidance>`;
   when a lint has a concrete approved pattern, `<guidance>` includes an
   appended `Example: <approved pattern>` clause in the same help note
3. One note in the exact form `see Tiger Style guide: <section>`

**Rationale:** The exact prefixes are now part of the contract. Snapshot
updates, downstream docs, and future autofix suggestions all build on the
same wording.

**Implementation:** Add `src/diagnostics.rs` with one wrapper that owns all
Tiger Style help/note emission, for example:

```rust
pub(crate) fn emit_tiger_style_lint(
    cx: &impl LintContext,
    lint: &'static Lint,
    span: Span,
    message: impl Into<String>,
    lint_name: &'static str,
) {
    let meta = diagnostic_meta(lint_name);
    cx.opt_span_lint(lint, Some(span), DiagDecorator(|diag| {
        diag.primary_message(message.into());
        let help = match meta.example_mode {
            ExampleMode::Required => format!(
                "Tiger Style ({}): {} Example: {}",
                meta.principle,
                meta.guidance,
                meta.example_text.expect("required example text")
            ),
            ExampleMode::None => format!("Tiger Style ({}): {}", meta.principle, meta.guidance),
        };
        diag.help(help);
        diag.note(format!("see Tiger Style guide: {}", meta.guide_section));
    }));
}
```

Lint modules provide only their primary warning message and `lint_name`; they
must not emit `help` or `note` text directly. Required examples are emitted
inside the same help note after `Example:` so the spec still has one canonical
help line and one canonical note line per diagnostic.

**Alternative:** Keep free-form help strings in each lint. Rejected because
it keeps the current drift problem.

### 2. One metadata inventory for all 25 lints

**Choice:** Define one `LintDiagnosticMeta` inventory in `src/diagnostics.rs`
covering every registered lint.

**Rationale:** This inventory is the authoritative mapping for completeness,
review, and group registration. If a lint is missing from the table, tests
fail before snapshots drift.

Principle names, guide sections, guidance strings, example mode, and example
text are allowed only in this inventory and its appendix-derived snapshot, not
as ad-hoc literals in lint modules.

**Implementation:** Each entry stores:
- lint name
- principle name
- guide section
- domain group
- help guidance
- example mode plus concrete example text when that mode is `Required`

The canonical guidance and example strings live in the appendix table below and
are copied verbatim into the metadata inventory.

Add two deterministic checks:
- a unit test that the metadata inventory names exactly match `LINT_INVENTORY`
- a snapshot test over the full serialized metadata inventory so `principle`,
  `guide section`, `domain group`, `guidance`, and example text cannot drift
  silently

**Alternative:** Scatter principle names inside 25 lint modules and maintain
separate group lists in `src/lib.rs`. Rejected because it duplicates the same
mapping in multiple places.

### 3. Domain groups derive from the same inventory

**Choice:** Register the five domain groups from the same metadata inventory:
- `tigerstyle::safety` (11 lints)
- `tigerstyle::naming` (4 lints)
- `tigerstyle::structure` (6 lints)
- `tigerstyle::purity` (3 lints)
- `tigerstyle::concurrency` (1 lint)

**Rationale:** Group membership must stay aligned with the lint matrix and the
spec domains. Reusing the same source of truth prevents silent drift.

**Implementation:** Keep the original lint statics registered, then feed the
matching static lint IDs into `lint_store.register_group(...)` from the
inventory-derived domain lists.

**Alternative:** Hand-maintain five group arrays next to registration code.
Rejected because review already showed that completeness is easy to lose.

### 4. Reference pointers stay as guide sections, not URLs

**Choice:** Diagnostic notes point to stable Tiger Style section names such as
`Functional Core / Imperative Shell` or `Assertion Density`.

**Rationale:** This repo does not ship one canonical hosted guide URL. Section
names stay readable in terminal output and stable across repo/skill moves.

**Implementation:** Metadata stores section names only. Proposal, design, and
spec all use the same note form: `see Tiger Style guide: <section>`.

**Alternative:** File paths or web URLs. Rejected because they are more likely
to drift than section names.

### 5. Example policy is explicit per lint

**Choice:** The metadata inventory records whether a lint needs a concrete
before/after example in help text.

**Rationale:** Review asked for a lint-by-lint map, including example/no-example.
A few lints need short guidance only; others should show the approved pattern
inline.

**Implementation:** Use an enum such as `ExampleMode::{Required,None}` plus
an `example_text: Option<&'static str>` field. A unit test asserts every
`Required` entry has concrete example text; `None` entries keep guidance
concise.

### 6. Standalone UI fixtures must opt into tool-scoped attrs correctly

**Choice:** Group allow/deny verification uses dedicated fixture crates that
compile on the repo's supported nightly toolchain and include:
- `#![feature(register_tool)]`
- `#![register_tool(tigerstyle)]`
- `#![allow(tigerstyle::<domain>)]` or `#![deny(tigerstyle::<domain>)]`

**Rationale:** This lint crate already depends on nightly rustc internals via
Dylint and `rustc_private`. Standalone rustc fixtures reject tool-scoped attrs
without `register_tool`. Make the nightly + `register_tool` consumer contract
explicit instead of rediscovering it during snapshot updates.

**Implementation:** Reuse the current UI harness, which already passes
`--edition=2024` under the supported nightly toolchain. Keep group fixtures
standalone and self-contained.

### 7. Verification happens at four levels

**Choice:** Verification covers metadata completeness, full metadata content,
full group registration, and emitted diagnostics.

**Rationale:** Updating `.stderr` files alone does not prove that all 25 lints
were wired, that wording stayed consistent, or that every required group is
registered with the right members.

**Implementation:**
1. **Metadata completeness:** `cargo test --lib` checks that the diagnostic
   metadata inventory names match `LINT_INVENTORY` exactly.
2. **Metadata contract snapshot:** `cargo test --lib` compares a serialized
   snapshot of every metadata field for all 25 lints: lint name, principle,
   guide section, domain group, guidance, example mode, and example text.
   The same test asserts every `ExampleMode::Required` entry has concrete
   example text.
3. **Group registration snapshot:** `cargo test --lib` compares a serialized
   snapshot of all five exact consumer-visible group names and their ordered
   lint memberships: `tigerstyle::safety`, `tigerstyle::naming`,
   `tigerstyle::structure`, `tigerstyle::purity`, and
   `tigerstyle::concurrency`.
4. **Shared-helper enforcement:** `cargo test --lib` source-scans lint modules
   and fails if any file outside `src/diagnostics.rs` contains `Tiger Style (`,
   `see Tiger Style guide:`, `.help(`, `.note(`, `span_help(`, or `span_note(`.
   This keeps all help/note rendering routed through the shared wrapper instead
   of ad-hoc strings and enforces exactly one shared help line plus one shared
   note line per diagnostic.
5. **Emission coverage snapshot:** maintain a deterministic `lint -> fixture`
   coverage manifest for all 25 lints and test that the manifest names exactly
   match the metadata inventory. Each mapped fixture must be an emitting case,
   not a suppressed case. Maintain a second deterministic fixture-set artifact
   listing the full existing UI fixture corpus plus the new group allow/deny
   fixtures so review can see exactly which `.stderr` files were refreshed.
6. **Diagnostic wording and attribute behavior:** refresh all existing UI
   `.stderr` fixtures plus the new group allow/deny fixture snapshots. The
   mapped emitting fixtures prove per-lint canonical warning/help/note output;
   the fixture-set artifact proves the full refreshed corpus. Also add
   standalone fixtures proving:
   - `#![allow(tigerstyle::safety)]` suppresses safety diagnostics while a
     non-safety lint in the same crate can still warn
   - `#![deny(tigerstyle::naming)]` escalates a naming lint to an error
7. **End-to-end verification:** run `cargo test --test ui -- --ignored --nocapture`
   and `nix flake check` after snapshot refresh.

## Lint Matrix

| Lint | Domain group | Principle | Guide section | Example in help |
| --- | --- | --- | --- | --- |
| `compound_assertion` | `tigerstyle::safety` | Assertion Quality | Assertion Density | Required |
| `assertion_density` | `tigerstyle::safety` | Assertion Density | Assertion Density | None |
| `unbounded_loop` | `tigerstyle::safety` | Bounded Control Flow | Bounded Control Flow | Required |
| `ignored_result` | `tigerstyle::safety` | Error Handling | Error Handling | Required |
| `unchecked_narrowing` | `tigerstyle::safety` | Safe Narrowing | Safe Narrowing / Conversions | Required |
| `unchecked_division` | `tigerstyle::safety` | Explicit Division | Explicit Division | Required |
| `raw_arithmetic_overflow` | `tigerstyle::safety` | Overflow Protection | Overflow Protection | Required |
| `catch_all_on_enum` | `tigerstyle::safety` | Exhaustive Matching | Exhaustive Matching | Required |
| `no_unwrap` | `tigerstyle::safety` | Error Handling | Error Handling | Required |
| `no_panic` | `tigerstyle::safety` | Fail Fast Correctly | Error Handling / Panics | Required |
| `no_recursion` | `tigerstyle::safety` | Simple Control Flow | Simple Control Flow | Required |
| `bool_naming` | `tigerstyle::naming` | Positive Predicates | Positive Predicates | Required |
| `acronym_style` | `tigerstyle::naming` | Acronym Capitalization | Acronym Capitalization | Required |
| `numeric_units` | `tigerstyle::naming` | Unit Suffixes | Unit Suffixes | Required |
| `float_for_currency` | `tigerstyle::naming` | Integer Currency | Integer Currency | Required |
| `function_length` | `tigerstyle::structure` | Function Length | Function Length | None |
| `compound_condition` | `tigerstyle::structure` | Condition Decomposition | Condition Decomposition | Required |
| `ambiguous_params` | `tigerstyle::structure` | Explicit Interfaces | Explicit Interfaces | Required |
| `platform_dependent_cast` | `tigerstyle::structure` | Explicit Types | Explicit Types | Required |
| `unjustified_allow` | `tigerstyle::structure` | Documented Suppression | Documented Suppression | Required |
| `nested_conditionals` | `tigerstyle::structure` | Simple Control Flow | Simple Control Flow | Required |
| `ambient_clock` | `tigerstyle::purity` | Functional Core | Functional Core / Imperative Shell | Required |
| `contradictory_time` | `tigerstyle::purity` | Functional Core | Functional Core / Imperative Shell | Required |
| `verified_purity` | `tigerstyle::purity` | Verified Module Purity | Verified Module Purity | Required |
| `multi_lock_ordering` | `tigerstyle::concurrency` | Lock Ordering | Lock Ordering | Required |

## Canonical Guidance and Examples

This appendix is the source of truth for emitted `guidance` and `example_text`
strings.

### Safety

- `compound_assertion`
  - guidance: split compound assertions so the failing clause is obvious
  - example: `assert!(a > 0); assert!(b > 0);`
- `assertion_density`
  - guidance: add enough assertions to make local invariants visible near the logic they defend
  - example: none
- `unbounded_loop`
  - guidance: bound long-running loops with an exit condition, timeout, or bounded input
  - example: `while let Some(msg) = rx.recv().await { ... }`
- `ignored_result`
  - guidance: handle fallible results explicitly with `?`, `match`, or an intentional logged drop
  - example: `tx.send(msg)?;`
- `unchecked_narrowing`
  - guidance: narrow integers with checked conversions instead of `as`
  - example: `let small = u32::try_from(value)?;`
- `unchecked_division`
  - guidance: guard divisors before division so zero is impossible
  - example: `assert!(divisor != 0); let q = value / divisor;`
- `raw_arithmetic_overflow`
  - guidance: use checked or saturating arithmetic when overflow would change behavior
  - example: `let total = a.checked_add(b)?;`
- `catch_all_on_enum`
  - guidance: match enum variants explicitly so new variants cannot slip through `_`
  - example: `match state { State::Open => ..., State::Closed => ... }`
- `no_unwrap`
  - guidance: propagate or handle absence explicitly instead of panicking with `unwrap` or `expect`
  - example: `let value = maybe?;`
- `no_panic`
  - guidance: return errors or prove exhaustiveness instead of panicking in production code
  - example: `return Err(Error::invalid_state());`
- `no_recursion`
  - guidance: prefer explicit iteration or work queues over self-recursive control flow
  - example: `while let Some(item) = stack.pop() { ... }`

### Naming

- `bool_naming`
  - guidance: name booleans as predicates so call sites read like questions
  - example: `let is_ready = status == Ready;`
- `acronym_style`
  - guidance: capitalize acronyms like ordinary words in type and variant names
  - example: `RpcHandler`
- `numeric_units`
  - guidance: put the unit in the name of numeric quantities
  - example: `timeout_ms`
- `float_for_currency`
  - guidance: store money as scaled integers, not floats
  - example: `let price_micros: i64 = 125_000;`

### Structure

- `function_length`
  - guidance: split long functions into named helpers at invariant or phase boundaries
  - example: none
- `compound_condition`
  - guidance: break multi-part conditions into named predicates or nested guards
  - example: `if is_leader { if has_quorum { ... } }`
- `ambiguous_params`
  - guidance: replace same-type parameter clusters with named fields or an options struct
  - example: `fn connect(opts: ConnectOptions) -> ...`
- `platform_dependent_cast`
  - guidance: use explicit-width types at boundaries instead of `usize` or `isize` conversions
  - example: `let len: u32 = value.try_into()?;`
- `unjustified_allow`
  - guidance: every lint suppression needs a local reason tied to the code shape
  - example: `#[allow(no_panic, reason = "ffi boundary aborts process")]`
- `nested_conditionals`
  - guidance: flatten control flow with guards, early returns, or helper predicates
  - example: `if !is_ready { return; }`

### Purity

- `ambient_clock`
  - guidance: pass time into core logic or isolate clock reads in a shell-boundary helper with a narrow allow reason
  - example: `#[allow(tigerstyle::ambient_clock, reason = "shell boundary clock read")] fn read_now() -> Instant { Instant::now() }`
- `contradictory_time`
  - guidance: choose one source of time for a code path and thread it through consistently
  - example: `fn expires_at(now: Instant, ttl: Duration) -> Instant { now + ttl }`
- `verified_purity`
  - guidance: keep verified modules free of runtime-only side effects and shell concerns
  - example: `pub fn next_state(input: Input) -> Output`

### Concurrency

- `multi_lock_ordering`
  - guidance: document and preserve one lock acquisition order across the codebase
  - example: `// Lock order: state -> peer -> metrics`

## Risks / Trade-offs

**[Verbose diagnostics]** -> More lines in stderr snapshots. Mitigation: keep
one canonical help line and one canonical note line, not extra prose blocks.

**[Large snapshot churn]** -> Every existing UI `.stderr` file changes.
Mitigation: settle wording first, then refresh snapshots once.

**[Group fixture pitfalls]** -> Tool-scoped attrs fail without
`register_tool`. Mitigation: make the fixture boilerplate an explicit design
rule and keep using the harness's `--edition=2024` path.
