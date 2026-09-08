# Verification

## Scope

Isolated fix in the `octet-anonymous-body-20260908` worktree (branch `fix/anonymous-body-collector-20260908`) on top of fresh `origin/main` (`42dc32a6fe921d35182d7aef03c2a83999d55771`).

The repair reuses the two narrow corrections already present in the source-ownership work at revision `28abbc8b6d50bc1defe4d8671cfae17ffdfdb197` (recorded in `anon-reuse-revision.txt`): `caller_item_id` fallback routes body owners through `def_kind_to_item_kind`, and `collect_item` records anonymous/inline constant bodies as private without an unsupported visibility query. No schemas, contract catalogs, dependency pins, or lint policy changed.

## Compiler-backed controls

All runs executed the real all-target Cargo fixture through the pinned toolchain `nightly-2026-03-21` (rustc `ac7f9ec7da74d37fd28667c86bf117a39ba5b02a`) with `OCTET_UI_LIBRARY` and `OCTET_UI_DRIVER` wired through the Nix derivation `checks.x86_64-linux.compiler-architecture-targets`.

| Run | Result | Evidence |
| --- | --- | --- |
| Baseline fixture on unchanged producer | PASS | `anon-baseline-compiler.log` |
| Expanded fixture on unchanged producer | FAIL | `anon-red-compiler.log` — rustc ICE `unexpected sort of node in fn_sig(): AnonConst(…anonymous_body::{constant#0})` at `rustc_hir_analysis/src/collect.rs:1046:13`, exit 101 |
| Expanded fixture with both guards | PASS | `anon-green2-compiler.log` — all three controls pass: anonymous, inline, closure; `anon-final-checks.log` reproduces the same green result after the exact-format restoration, plus `actual Cargo target coverage: 10 units` |
| Signature-guard mutation removed (fallback always queries `fn_sig`) | FAIL | `anon-signature-mutation.log` — same `fn_sig(): AnonConst` ICE |
| Visibility-guard mutation removed (always queries `tcx.visibility`) | FAIL | `anon-visibility-mutation.log` — separate ICE `tcx.visibility(…) is not supported for this key` at `rustc_middle/src/query/plumbing.rs:657:5` |

The four scenarios in the specification map one-to-one to these controls, including the closure-vs-function distinction from the expanded fixture's closure body with a resolved call, and the existing shard-coverage complete-coverage check preserved in the fixture.

## Producer checks

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo test -p octet -p cargo-octet -p octet-architecture-ir --lib` | 638 passed, 0 failed, 8 ignored | `anon-library-tests.log` |
| `cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings` | PASS | `anon-local-clippy.log` |
| `nix build .#checks.x86_64-linux.compiler-architecture-targets` | PASS | `anon-final-checks.log` |
| `nix build .#checks.x86_64-linux.fmt` | PASS | `anon-final-checks.log` |
| `nix build .#checks.x86_64-linux.project-architecture-policy` | PASS | `anon-policy-checks.log` |
| `nix build .#checks.x86_64-linux.cairn-traceability` | PASS | `anon-policy-checks.log` |
| `cairn validate --root .` plus proposal/design/tasks gates | PASS | `anon-cairn-baseline.log` and `pueue log --full 5` |
| `cargo clippy` (workspace all-targets, `-D warnings`) | FAIL, pre-existing | `anon-producer-checks.log`, `anon-base-clippy.log` — 21 errors in untouched `crates/octet-architecture-ir` (`missing_errors_doc`, `missing_panics_doc`, `too_many_lines`, …). The published baseline `42dc32a6` fails identically. Not introduced by this change. |

## Consumer replay

Replayed Seaglass's architecture checker against this worktree producer via
`OCTET_ARCHITECTURE_PACKAGE=/home/brittonr/git-worktrees/octet-anonymous-body-20260908#cargo-octet`
on the unchanged `drain-architecture-20260908` consumer source (with its fail-closed wrapper and unchanged lint levels).

Result (`anon-consumer-replay.log`): the collector ran the full Seaglass workspace **without any compiler panic**. Final summary `Status: integration-failure`, `Findings: 2148`, `Warnings: 141`, `Errors: 2007`, dominated by `seaglass-pipeline` debt from the in-progress module split/rename. No occurrence of `unexpected sort of node`, `AnonConst`, or `tcx.visibility` anywhere in the 26k-line transcript. The fail-closed wrapper rejected the findings exit as required (recorded `anon-consumer-replay.exit` = 2).

This consumer replay is not architecture acceptance: the remaining findings are genuine Seaglass WIP debt to be cleared in the consumer change.
