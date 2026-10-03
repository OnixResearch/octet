# Boolean API migration inventory

This is the bounded source inventory for `raw_bool_parameter` and its
suppression guard. It records compiled findings and the chosen migrations;
it does not claim that all other Octet lints are clean.

## Octet workspace

The first compiled `cargo check --workspace --all-targets` with the new lint
found seven diagnostics at six callable sites. Each site was migrated rather
than given an `allow` or an expectation:

The [bounded initial compiler excerpt and site manifest](../.cairn/changes/harden-boolean-api-modeling/evidence/octets-initial-bool-inventory.txt)
preserve all seven original diagnostic lines, their positions, the selected
Dylint alias and driver, the public-redacted historical replay shape, and the
original raw output hash. The byte-exact argv is operator-local, not published.
This was an uncommitted source state based on
`b65337f1cd3ce130afb3dd76415b665e91cfd191`; the old library bytes were
not frozen, so the manifest is observed baseline evidence rather than an
exact-source replay.

| Site | Initial finding | Migration |
| --- | --- | --- |
| `crates/octet-standards/src/project_architecture_gate.rs` — `classify_status` | Two private positional flags | Derive integration and policy status from the findings at the classification boundary, preserving integration-failure precedence. |
| `tests/support/consumer_fixture.rs` — `run_internal` | Two private positional flags | Use `LockfileMutation` for the mutually exclusive check/build mutation modes; existing public fixture helpers select the mode. |
| `src/safety/no_todo.rs` — `NoTodo::new` | Two public direct-`bool` parameters | Pass a named `NoTodoLintSelection` record; migrate its library registration caller. |
| `cargo-octet/src/evidence_rails.rs` — `validate_kani_claims` | Three private positional flags | Pass the named `KaniClaimEvidence` record for unwind, contract, and boundary-stub observations. |
| `cargo-octet/src/evidence_rails.rs` — `advisory_database_update` | Two private positional flags | Read the existing `EvaluationInput` at the receipt boundary instead of repeating two flag arguments. |
| `cargo-octet/src/architecture_ir/cargo_fixture_tests.rs` — `run_body_owner_compiler_fixture` | Two private positional flags | Remove the redundant wrapper and route its callers through the existing `BodyFixture::NonFunctionOwners` selector. |

The `classify_status` cutover has one callsite. Before the change, both flags
were derived from the **sorted, post-waiver** finding slice: an open Integration
finding took precedence, followed by open Policy findings. The new classifier
applies those same two predicates to that same slice in the same order. A
waived finding does not count; a stale waiver appends an open Integration
finding. There was no separate diagnostic-only failure flag passed to the old
classifier. The 38 scoped architecture-gate tests pass, including clean and
stale waiver outcomes, incomplete compiler coverage, blocked gate versus
inventory findings, and admission closure outcomes.

The targeted Kani and dependency-audit receipt tests each pass, as do four
lockfile/termination integration scenarios covering check-time and
bootstrap-time mutation. The ignored, real-compiler
`real_cargo_all_targets_preserves_non_function_body_owners` fixture also
passes after its two callers switched to `BodyFixture::NonFunctionOwners`.

After the parsed-attribute and strict guard-scheduling repairs and final
Dylint library rebuild, the fresh pinned-nightly, local-builder-only compiled
inventory, with a new harmless Cargo configuration flag to force every
workspace target to recompile under the final alias:

```sh
RUSTFLAGS='--cfg octet_boolean_api_final_inventory_v3' \
DYLINT_RUSTFLAGS='-D raw_bool_parameter -D raw_bool_parameter_suppression' \
cargo check -j 4 --workspace --all-targets --message-format=short
```

The [bounded current two-lint receipt](../.cairn/changes/harden-boolean-api-modeling/evidence/octets-current-two-lint-check.txt)
records a public-redacted Nix/Dylint/UI-driver replay shape and selected
settings, the original operator-local output hash, zero matching boolean-API
finding lines, and the actual success line. It does not attest a complete
inherited environment or publish byte-exact argv. The
command exited **0**, with no diagnostics from either boolean-API lint;
unrelated existing lint warnings remain. This is a check of the compiled
workspace targets under these two denied rules after the final planner and
metadata changes, not a claim that the whole catalog passes a deny-all gate.

The source inventory is not a warning-budget waiver: other Octet diagnostics
remain outside this change's boolean-API scope. The separate guard checks
attempts to suppress either boolean-API lint rather than treating a blanket
`allow` as a migration.

The [H2 source-bound drift record](../.cairn/changes/harden-boolean-api-modeling/evidence/h2-source-drift-blocker.txt)
identifies the two necessary changed frozen files, their approved and current
hashes, and the three failing object-corpus freshness tests. The other 35
frozen files, including `cargo-octet/src/config.rs` and the restored
`cargo-octet/src/cli.rs`, still match their approved hashes. Historical H2
manifest, receipts, and owner note remain untouched; a new independent
source/reference review and externally frozen approval are required before
current-source admission. A clean two-lint compiler inventory is not that
approval or a passing full workspace test suite.
The earlier targeted local-only Nix `object-corpus-gate` child rejected
`selected source byte drift: cargo-octet/src/engine.rs`
(see [bounded H2 drift note](../.cairn/changes/harden-boolean-api-modeling/evidence/h2-source-drift-blocker.txt);
raw output is operator-local, not publicly accessible);
the planner identity source is outside the H2 selected paths. Neither the
original reviewed manifest nor its receipts, owner approval, or freshness
tests were repinned. The [unapproved current-source review](../.cairn/changes/harden-boolean-api-modeling/evidence/unapproved-h2-current-source-review-20261002.txt)
compares all 37 selected source files (35 match; `engine.rs` and
`evidence_rails.rs` drift). The [unapproved proposed-corpus packet](../.cairn/changes/harden-boolean-api-modeling/evidence/unapproved-h2-proposed-corpus-artifacts-20261002.txt)
records scratch positive/internal/expanded/taxonomy outputs and same-producer
replay, not a new frozen reference or independent owner approval. Current H2
admission remains **FAIL/BLOCKED**; neither packet authorizes archive.

The former `alias_bins` (full pinned all-targets hook) and `build-script-build`
(robust workspace) missing-source-role errors were **historical**, not
current blockers. Target-root roles are now explicit in the
[Nickel policy](../standards/project-architecture/octet-self-architecture.ncl),
[generated JSON](../standards/project-architecture/octet-self-architecture.json),
and [export manifest](../standards/project-architecture/octet-self-architecture.manifest.json).
The strict `cargo-octet/build.rs` proof found byte-identical generated source
and rerun directives and checked that failure paths return errors rather than
panic. No source-role fallback or cap increase was introduced.

The current full pinned deny-all hook
(`--profile robust --workspace -- --all-targets`) instead exits 2 when its
39 selected compiler-architecture units exceed unchanged `max_units=32`.
The real robust `--workspace` run reaches lint evaluation and reports 108
unrelated preexisting strict architecture-IR/metadata findings
(raw output is operator-local, not publicly accessible). The earlier
23-pedantic-error Clippy report was also
historical: the current local-only `nix flake check -L` exits 1 with two
observed `clippy::too_many_lines` errors at
`crates/octet-architecture-ir/src/contract.rs:9` and `core.rs:96`
(raw output is operator-local, not publicly accessible). It evaluated 74
checks but realized only 32: this is
**not** an aggregate PASS or evidence that the remaining checks pass.
The [bounded current-source focused checks](../.cairn/changes/harden-boolean-api-modeling/evidence/boolean-focused-current-source-check-20261002.txt)
cover unit, real UI/snapshot, metadata/profile, Clippy co-reporting,
compiler-fixture and selected-consumer migration paths; they do not override
these gate failures or authorize a Boolean-API finding baseline.

## Same-callable Clippy co-report

The [bounded dual-tool compiler receipt](../.cairn/changes/harden-boolean-api-modeling/evidence/boolean-dual-tool-co-report.txt)
records one disposable four-`bool` public function compiled by both real CLI
rails. Octet emitted four parameter findings; its CLI exited 2 with
`integration-failure` because Cargo exited 101 under the strict profile.
The independent warning-level Clippy profile reported
`fn_params_excessive_bools` at the same source line, and its evidence artifact
verified. The two Clippy rows from the all-targets command are not two
different APIs, and Clippy's receipt does not assign per-row stable IDs. The
tools preserve their own checker, rule, message, and receipt identities
without merging or replacing one another.

## Selected compiled external consumer

The current external consumer is `brittonr/aspen`'s active
`molten-release-policy` binary at commit
`81baf80cb667a22277be22c140d34829e5c7da7f` on `molten`, checked
in a clean isolated clone. The bounded source has no `bool` parameter sites;
there was no consumer source migration or exception to make. Its selected
pinned-toolchain command was
`cargo check -j 4 -p molten-release-policy --bin molten-release-policy --locked`
with the then-local Octet Dylint library, UI driver, and `DYLINT_NO_DEPS=1`.
The build finished successfully (`dev` profile, 39.80 seconds), reporting
45 unrelated warnings and no `raw_bool_parameter` or suppression-guard
finding in that selected binary. This is a compiled selected-target check,
not a claim that all Aspen packages or historical `aspen-time` sources pass.
The four selected package Rust source files had no `bool` spelling on review.
The [bounded selected-consumer receipt](../.cairn/changes/harden-boolean-api-modeling/evidence/aspen-molten-selected-check.txt)
records the source pin, public-redacted command shape and semantic
driver/library paths, observed remote-builder use, and completion excerpt.
It does not publish the byte-exact original argv, private remote endpoint, or
raw output. No consumer API signature, caller, or external-signature exception
changed in this selected target. The old Octet library bytes were not frozen
by the capture.
A separate [current-source selected-target replay](../.cairn/changes/harden-boolean-api-modeling/evidence/boolean-focused-current-source-check-20261002.txt)
recompiled the same pinned binary with the current Octet library and both
Boolean-API lints denied: it exited 0 with 45 unrelated binary warnings.
The non-linted `molten-core` dependency also emitted two unknown-lint
warnings from those propagated flags; this is not an Aspen-wide check.

The Nix development-environment materialization for that particular run
inherited <configured remote builder> and built `octet-standards` remotely;
it was not a local-builder-only build. No remote cleanup,
restart, or other destructive action was requested or performed. Subsequent
Octet checks and the later selected-target replay explicitly set `builders =`
empty, `min-free = 0`, `max-free = 0`, and local job/core limits. Do not
describe the earlier external consumer run as an aspen3-only builder proof.

The [source-bound Radiance style reference](boolean-api-modeling.md#upstream-style-boundary)
records the exact upstream revision, its MIT license, the guidance selected
for Octet, and the unrelated helper/panic guidance excluded from this rule.
The external style reference is not evidence for the compiled inventory or
the selected consumer's behavior.
