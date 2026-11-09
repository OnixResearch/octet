## Phase 1: Baseline and policy

- [x] [serial] Record current `bool_naming`, parameter-count, Clippy co-reporting, and public API behavior before changes. See `docs/boolean-api-migration-inventory.md`. r[octet.structure.raw_bool_parameter.compatibility]
- [x] [serial] Add the Radiance style reference, exact revision, MIT role, selected guidance, and rejected guidance to documentation. r[octet.structure.raw_bool_parameter.reference_boundary]
- [x] [serial] Define the public API, private-count, generated-code, trait-ownership, exception, and non-claim policy. r[octet.structure.raw_bool_parameter.policy]
- [x] [serial] Add a named constant for the minimum ambiguous private boolean-parameter count. r[octet.structure.raw_bool_parameter.policy]

## Phase 2: Lint and metadata

- [x] [serial] Implement `raw_bool_parameter` for public free functions, inherent methods, and crate-owned trait declarations. r[octet.structure.raw_bool_parameter.public_api]
- [x] [serial] Report non-public callables at the named direct-boolean parameter count. r[octet.structure.raw_bool_parameter.private_api]
- [x] [serial] Skip generated expansion output and externally owned trait implementation signatures. r[octet.structure.raw_bool_parameter.compatibility]
- [x] [serial] Emit stable diagnostics and domain-modeling help without an automatic type rewrite. r[octet.structure.raw_bool_parameter.diagnostics]
- [x] [serial] Register lint identity, structure group, metadata, source path, examples, docs, and strict-profile membership. r[octet.structure.raw_bool_parameter.catalog]
- [x] [serial] Add `clippy::fn_params_excessive_bools` to the reviewed co-reporting profile without merging finding identities. r[octet.structure.raw_bool_parameter.clippy_evidence]

## Phase 3: Positive and negative verification

- [x] [parallel] Add negative UI fixtures for public free functions, inherent methods, crate-owned trait methods, and multiple private booleans. r[octet.structure.raw_bool_parameter.verification]
- [x] [parallel] Add positive fixtures for private single flags, boolean returns, fields, enums, option records, wrappers, generated code, and external trait implementations. r[octet.structure.raw_bool_parameter.verification]
- [x] [parallel] Add narrow expectation fixtures with required specific reason and named owner. r[octet.structure.raw_bool_parameter.exceptions]
- [x] [parallel] Reject crate, module, workspace, baseline, and warning-budget suppression. r[octet.structure.raw_bool_parameter.exceptions]
- [x] [parallel] Add catalog, profile, group, alias, metadata, docs, and snapshot freshness tests. r[octet.structure.raw_bool_parameter.catalog]

## Phase 4: Migration and deny-all activation

- [x] [serial] Produce a deterministic Octet self-inventory and selected-consumer inventory before activation. See `docs/boolean-api-migration-inventory.md`; the external consumer inventory remains blocked. r[octet.structure.raw_bool_parameter.migration]
- [x] [serial] Replace covered Octet public booleans with reviewed domain types or narrow justified expectations. r[octet.structure.raw_bool_parameter.migration]
- [ ] [serial] Migrate selected consumers and record compatibility changes, rejections, and remaining external-signature exceptions. r[octet.structure.raw_bool_parameter.migration]
- [x] [serial] Enable the lint in the full pinned deny-all catalog only after the declared migration set is clean. r[octet.structure.raw_bool_parameter.catalog]

## Phase 5: Documentation and closeout

- [x] [parallel] Document design alternatives, examples, migration patterns, exceptions, Clippy relation, and non-claims. r[octet.structure.raw_bool_parameter.diagnostics]
- [x] [serial] Add the Radiance repository to README references with its exact style-reference role and claim boundary. r[octet.structure.raw_bool_parameter.reference_boundary]
- [x] [serial] Run focused unit, UI, snapshot, metadata, profile, Clippy co-reporting, and migration checks. r[octet.structure.raw_bool_parameter.verification]
- [ ] [serial] Run formatting, workspace tests, Clippy, Octet deny-all, Cairn validation, traceability, and `nix flake check -L`. r[octet.structure.raw_bool_parameter.catalog]
  - Formatting, workspace tests, Clippy (including pedantic), Octet deny-all, Cairn validation, traceability, `lean4`, `warning-clean`, `fresh-build-object-corpus-determinism`, and `verus-toolchain-profile` now pass after parent fixes. The remaining `nix flake check -L` failure is the frozen object-corpus evidence gate (`object-corpus-caveat-taxonomy-gate`): the reviewed reference is pinned to commit `b65337f1`, while the current source contains the new modules and produces 1,579 objects instead of the reviewed 1,258. Re-freezing that evidence requires a maintainer source review, not a boolean-lint change.
- [x] [serial] Run all Cairn gates. Sync and archive only after deny-all activation has no warning budget or baseline. r[octet.structure.raw_bool_parameter.migration]

## Parent verification on the integrated base

- `cargo check --workspace --all-targets --locked`: passed.
- `cargo test -p octet --lib --locked`: 228 passed.
- Full maintained UI rail, including the six boolean fixtures: passed.
- `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- Cairn validation and proposal/design/tasks gates: passed.
- `nix flake check -L` was not run; it remains the open verification task.
- Selected external consumer migration remains blocked: no owner-authorized consumer worktree is available.

## Evidence

- `cargo check --workspace --all-targets` (pinned nightly, unique target dir): passed.
- `cargo test -p octet --lib`: 228 passed.
- `cargo test -p octet-lint-metadata`: 14 unit tests plus integration tests passed.
- `cargo test -p cargo-octet --test clippy_restriction_profiles`: 4 passed, including the independent boolean co-report and its clean negative.
- `cargo test -p cargo-octet --lib clippy_restrictions`: 3 passed.
- Real UI driver over every new fixture matched its checked-in snapshot: `raw_bool_parameter`, `raw_bool_parameter_safe`, `raw_bool_parameter_suppression`, `raw_bool_parameter_crate_suppression`, `raw_bool_parameter_expectation`, and `raw_bool_parameter_guard_strict`.
- Two-lint compiled inventory (`-D raw_bool_parameter -D raw_bool_parameter_suppression`, `cargo check --workspace --all-targets --message-format=short`): exit 0 with zero matching findings. Migrations are recorded in `docs/boolean-api-migration-inventory.md`.
- `cargo fmt --check`: clean.
- `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings`: passed.
- `cairn validate --root .`: passed. `cairn gate proposal|design|tasks`: passed.
- Blocked: selected external consumer migration. No owner-authorized consumer worktree is part of this assignment; no consumer edit, receipt, or compatibility claim is made.
- Not run: full `cargo test --workspace`, the complete UI harness over the legacy corpus, and `nix flake check -L`.
