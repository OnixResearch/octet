# Agent Notes

## Downstream rollout evidence
- Use `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical Aspen phase-1 starter-rollout reference. Current `/home/brittonr/git/aspen` scripts may already include later scope and lint promotions. Replay helpers and checkpoint notes must cite that archive path explicitly, not just a local Aspen worktree.
- Consumer rollout guidance now lives in `docs/downstream-rollout.md`, with copyable templates in `templates/tigerstyle-check.sh` and `templates/dylint.toml`.
- Template default `../tigerstyle` only works when the consumer repo is a direct sibling of `tigerstyle`; nested consumers like `mc/valence` need script-side auto-detection (for example `../../tigerstyle`) or an explicit `TIGERSTYLE_PATH`.
- To reproduce Aspen phase-1 evidence, use Aspen commit `909e5e9fa`. A naive temp worktree fails because Aspen has sibling path dependencies (`aspen-dns`, `aspen-wasm-plugin`); use a sandbox parent with `aspen/` worktree plus sibling symlinks.
- If a change archives helper scripts and transcripts, regenerate the `.txt` outputs from the archived location before committing so saved commands point at `openspec/changes/archive/...`, not the deleted active change path.
- If the exact replay proof uses a helper script, document that helper command as the exact replay path and prepend the same command line to the saved transcript.
- If a saved replay narrows the enabled lint set through per-lint config, make the transcript self-describing by patching the copied wrapper banner for that run or saving the applied `dylint.toml` snapshot alongside the log.
- Saved check helpers must assert every printed summary claim, including zero-warning claims like `aspen-hlc: 0 starter findings`.
- If a transcript starts with `$ bash ...`, store and document the capture helper or exact shell wrapper that emits that preamble; do not claim a quieter inner helper produced it.
- When a reviewable fix depends on `cargo test --test ui -- --ignored --nocapture`, save that ignored-run transcript under the change evidence path; a generic `cargo test --lib --test ui` artifact is not enough for newly added UI fixtures.
- If supplementary workspace reruns are non-blocking because regressions stay in vendored or otherwise out-of-scope paths, say that acceptance explicitly in the archived tasks/evidence files before calling the change done.

## Build / test
- Run Rust builds inside the nix devshell: `nix develop -c bash -c '...'`. Plain `cargo` outside the shell can miss OpenSSL/pkg-config and rustc-private setup needed by Dylint.
- When shared `~/.cargo-target` lock contention or unrelated repo builds interfere, use a temporary `CARGO_TARGET_DIR` with trap-cleanup for reviewable `cargo test --lib` / `cargo test --test ui` runs.
- After adding or removing Cargo dependencies, run `cargo generate-lockfile` before trusting fresh `--locked` Nix builds or `nix flake check`.
- When review needs proof of specific UI cases here, make the saved UI transcript print each fixture name and save a separate artifact that prints snapshot contents; a bare `test ui ... ok` transcript is not enough.
- The custom `tests/ui.rs` harness does not compile `#[cfg(test)]` fixture code unless that fixture gets an explicit `--cfg test`; add per-fixture rustc args when you need to exercise cfg-gated test modules.
- Standalone UI fixtures can use bare lint attrs like `#[allow(no_panic)]`, but tool-scoped forms like `#[allow(tigerstyle::no_panic)]` need `#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]` or rustc rejects them before the lint runs.
- Test-context suppression must treat only cfg predicates that require `test`; a token match on `test` is too broad and wrongly suppresses cases like `#[cfg(not(test))]`.
- Early-pass line-scan fallback for test context must recognize both outer `#[cfg(test)]` and inner `#![cfg(test)]` attributes when recovering stripped cfgs from source text.
- For same-branch unwrap suppression, a raw condition-snippet search is too broad: `if x.is_some() || other { x.unwrap(); }` is still unsafe. Use condition structure so `&&` can imply the guard, while `||` only counts when both sides require it.
- When a lint needs to recognize `Drop::drop`, compare the enclosing trait impl against `tcx.lang_items().drop_trait()`; matching a trait name string like `Drop` is too loose.
- For `unchecked_narrowing`, scanning an entire enclosing block for `assert`/`MAX` is too broad and will suppress casts guarded only for other values or later statements; inspect only prior same-block assertions that match the casted expression.
- In late HIR, `assert!(cond)` expands to an `if !cond { ... }`; same-block assert recognition must unwrap that leading `!` instead of looking for a bare binary condition.
- In late HIR, `expr.await` lowers to an `AwaitDesugar` match around `IntoFuture::into_future(<method call>)`; async ignored-result rules should inspect the nested method call rather than looking for a top-level `MethodCall`.
- Tokio `sync::mpsc` public re-exports resolve through internal def paths like `tokio::sync::mpsc::bounded::Sender` and `...::bounded::Receiver`; `get_def_path` matching must use the definition path, not just the public re-export spelling.
- Late-pass `unbounded_loop` must ignore expansion/desugaring-generated loops or it will falsely warn on compiler-inserted `.await` polling loops.
- `tests/ui.rs` can support async/Tokio standalone fixtures by building a sibling fixture-local `Cargo.toml` once, then passing `-L dependency=...` and `--extern tokio=...` while still compiling the actual linted fixture with `--edition=2024`.
- Nix `checks.ui` only vendors dependencies present in the repo root `Cargo.lock`; if a fixture-local support crate depends on `tokio`, add matching root `dev-dependencies` so the vendored sandbox can resolve it.
- The custom `tests/ui.rs` harness now supports snapshot refresh via `BLESS=1 cargo test --test ui -- --ignored --nocapture`; use it inside `nix develop` when updating `.stderr` files.

## Lint registration / config
- Do not implement per-lint severity by registering cloned `Lint` descriptors with patched `default_level`; passes still emit the original static lint IDs, so `allow`/`deny` overrides silently fail.
- In this repo, per-lint severity overrides must keep original lint statics registered and feed effective levels through rustc lint opts before lint-level computation.
- Do not route pass registration through a runtime function-pointer inventory table here; it breaks source-attribute activation for allow-by-default lints in standalone UI fixtures. Keep `LINT_INVENTORY` as the source of truth, but generate explicit registration code from it.
- Tool-scoped group attrs like `#![allow(tigerstyle::safety)]` do not work while the lints are plain `declare_lint!` statics. A naive switch to `declare_tool_lint!` makes those group attrs viable, but it breaks the current per-lint config path that feeds bare names through `sess.opts.lint_opts` and also invalidates existing bare source-attr fixtures.
- Spike proof on 2026-04-15: one lint (`no_unwrap`) can move to `declare_tool_lint!(pub tigerstyle::NO_UNWRAP)` while preserving bare `dylint.toml` keys and bare lint opts by registering a one-member bare alias with `lint_store.register_group(true, "no_unwrap", None, vec![LintId::of(NO_UNWRAP)])`; standalone tool-scoped attrs still need `#![feature(register_tool)]` plus `#![register_tool(tigerstyle)]`.
- Mixed-phase side effect: once any `tigerstyle::...` tool lint is registered, standalone `#![register_tool(tigerstyle)]` attrs for still-plain lints stop taking the old silent MissingTool path and start warning `unknown lint: tigerstyle::<lint>` with suggestions to the bare name; expect snapshot churn like `ui_tests/no_panic_todo_allow.stderr` until those lints migrate too.
- Hard blocker found during bulk conversion on 2026-04-15: `no_panic` cannot keep its current `#[allow(tigerstyle::no_panic)]` + adjacent-comment semantics once it becomes a real tool lint; rustc `allow` suppresses the entire lint before custom justification logic runs. Preserving that contract needs a redesign (for example split `todo!()` into its own lint/marker) or an explicit exception from full tool-lint migration.
