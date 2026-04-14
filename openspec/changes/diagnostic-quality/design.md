## Context

Current diagnostics use `diag.warn(message)` or `diag.help(suggestion)`
with ad-hoc formatting. There is no consistent structure across lints.
The downstream rollout kit includes extensive pattern guidance (boundary
playbook, narrow allow rules) that developers must find manually.

## Goals / Non-Goals

**Goals:** Every lint diagnostic includes principle name, approved pattern,
and reference pointer. Lint groups match spec domains.

**Non-Goals:** Interactive help or web-based documentation. The diagnostics
should be self-contained terminal output.

## Decisions

### 1. Standard diagnostic structure

**Choice:** Every lint emits:
1. Warning message: one-line description of the violation
2. Help note: `Tiger Style: <principle>` + approved pattern
3. Note: reference pointer to the relevant guide section

**Rationale:** Consistent structure lets developers learn the pattern.
The principle name becomes searchable across diagnostics.

**Implementation:** Create a shared helper in a `diagnostics` module:

```rust
fn tiger_style_help(diag: &mut Diag<'_, ()>, principle: &str, help: &str, reference: &str) {
    diag.help(format!("Tiger Style ({principle}): {help}"));
    diag.note(format!("see: {reference}"));
}
```

### 2. Lint groups by spec domain

**Choice:** Register five lint groups matching spec domains:
- `tigerstyle::safety`
- `tigerstyle::naming`
- `tigerstyle::structure`
- `tigerstyle::purity`
- `tigerstyle::concurrency`

**Rationale:** Consumers can `#[allow(tigerstyle::safety)]` to suppress
an entire domain during phased rollout, matching the rollout profile
model.

**Implementation:** Use `lint_store.register_group()` with the lint IDs
from each domain.

**Alternative:** No groups, only individual lint IDs — rejected because
group-level suppression is a common request during rollout.

### 3. Keep reference pointers relative

**Choice:** Reference pointers use `Tiger Style guide: <section name>`
rather than URLs.

**Rationale:** The guide lives in the repo at
`references/tigerstyle.md` and in the skill. URLs would go stale.
Section names are stable and grep-able.

**Alternative:** URLs to a hosted doc site — rejected because the project
doesn't have one yet.

### 4. Before/after examples in help text

**Choice:** For lints with concrete fix patterns (`compound_assertion`,
`ambient_clock`, `contradictory_time`), include a one-line before/after
in the help text.

**Rationale:** The developer reading the diagnostic in their terminal
shouldn't have to open a separate file to see what the right pattern
looks like.

**Implementation:** Multi-line help strings in the diagnostic builder.

## Risks / Trade-offs

**[Verbose diagnostics]** → Three-part diagnostics are longer. Mitigation:
the help and note are separate lines that can be collapsed in IDEs.
Developers who have seen the lint before skip the help text.

**[All UI fixtures need updating]** → Every `.stderr` file changes.
Mitigation: update all fixtures in one pass as part of this change.
Depends on `ui-test-coverage` being done first (or simultaneous).
