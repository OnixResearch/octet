## Context

The README names Octet as the canonical product and says Rust source attrs and domain groups use `octet::...`. The lint declarations and group constants currently expose `tigerstyle::...` in several implementation and fixture paths. Prior notes also warn that naïve tool-lint migration can break source-attribute activation and no-todo/no-panic escape-hatch contracts.

## Decisions

### 1. Make one namespace decision explicit

**Choice:** Record a release-scoped decision for the canonical source-attribute namespace, emitted group namespace, and compatibility aliases before code churn.

**Rationale:** Consumers need a clear answer even if the final migration is phased. Documentation must reflect live behavior rather than aspirational state.

### 2. Preserve compatibility with tests

**Choice:** Keep legacy namespaces and bare config keys working for a defined compatibility window unless the migration explicitly removes them.

**Rationale:** Existing downstream consumers use legacy attrs. Breaking them requires evidence, migration docs, and UI fixtures.

### 3. Migrate no-todo/no-panic carefully

**Choice:** Treat lints with custom escape-hatch semantics as special cases during namespace migration.

**Rationale:** Tool-scoped rustc allows can suppress lints before custom justification logic runs. The migration must not silently weaken escape-hatch enforcement.

### 4. Align runner output with docs

**Choice:** Emitted group names in JSON/SARIF and docs should match the selected canonical namespace, with compatibility aliases described separately.

**Rationale:** Release evidence should present one product identity and avoid surprising consumers with old names.

## Risks / Trade-offs

- Full migration can trigger UI snapshot churn. Mitigate by batching fixtures and preserving compatibility tests.
- Canonical `octet::...` attrs may require careful `declare_tool_lint!` conversion. Mitigate by moving one family at a time and validating source attrs in standalone fixtures.
- Deferring migration while docs say `octet::...` creates user confusion. Mitigate by updating docs immediately if deferral is chosen.

## Validation

- Positive: canonical namespace attrs and groups work in standalone UI fixtures.
- Negative: unsupported namespace spellings fail with clear rustc or Octet diagnostics.
- Compatibility: supported legacy namespace attrs and bare config keys continue to work until the documented removal boundary.
