## Phase 1: Test-context suppression

- [x] Add shared test-context detection helper used by `no_unwrap` and `no_panic`
- [x] Teach `tests/ui.rs` to pass per-fixture `--cfg test` for cfg-gated test-context fixtures
- [x] Keep `tests/ui.rs` compiling standalone fixtures with `--edition=2024` so async precision fixtures match the crate edition
- [x] `no_unwrap`: skip `#[test]` functions
- [x] `no_unwrap`: skip `#[cfg(test)]` modules / `#![cfg(test)]` items
- [x] `no_unwrap`: still warn for `#[cfg(not(test))]` production code
- [x] `no_panic`: skip `#[test]` functions
- [x] `no_panic`: skip `#[cfg(test)]` modules / `#![cfg(test)]` items
- [x] `no_panic`: still warn for `#[cfg(not(test))]` production code
- [x] UI proof: `ui_tests/no_unwrap.rs` shows production warning + test silence
- [x] UI proof: `ui_tests/no_unwrap_cfg_not.rs` shows `#[cfg(not(test))]` still warns
- [x] UI proof: `ui_tests/no_panic.rs` shows production warning + test silence
- [x] UI proof: `ui_tests/no_panic_cfg_not.rs` shows `#[cfg(not(test))]` still warns

## Phase 2: Pattern-based suppression

- [x] `no_unwrap`: skip `.unwrap()` on compile-time-known `Some` / `Ok` and literal-string `.parse().unwrap()`, including const/static-style initializer contexts
- [x] UI proof: `ui_tests/no_unwrap_compile_time_known.rs` stays silent for compile-time-known `Some` / `Ok` and literal-string `.parse().unwrap()` cases while unsafe `expect` / production cases still warn elsewhere
- [x] `no_unwrap`: skip `.unwrap()` after `.is_some()` / `.is_ok()` guard in same branch
- [x] UI proof: `ui_tests/no_unwrap_guarded.rs` stays silent in guarded branch and still warns in unguarded `else` path
- [x] `no_panic`: skip `unreachable!()` in exhaustive trailing-wildcard enum matches for local, non-`#[non_exhaustive]` enums
- [x] UI proof: `ui_tests/no_panic_exhaustive.rs` stays silent for exhaustive wildcard `unreachable!()` and still warns for incomplete variant coverage
- [ ] `no_panic`: skip `todo!()` when an enclosing function or block has `#[allow(tigerstyle::no_panic)]` plus an adjacent `//` justification comment
- [ ] UI/setup proof: add a tool-registered fixture for justified `todo!()` allow, and keep an unjustified / still-warning case proving blank lines or missing `//` comments do not suppress
- [x] `ignored_result`: skip `let _ = expr` inside `Drop::drop` impls
- [x] `ignored_result`: detect real `Drop::drop` via lang-item-backed trait impl matching, not trait-name text
- [x] UI proof: `ui_tests/ignored_result_drop.rs` stays silent in `Drop::drop` and still warns for an inherent `drop` method
- [ ] `ignored_result`: skip `let _ = tx.send(msg);` only for `std::sync::mpsc::Sender<_>` / `tokio::sync::mpsc::Sender<_>` in functions or methods named exactly `shutdown` or `close`
- [ ] UI proof: add positive/negative send fixtures so `shutdown` / `close` stay silent and ordinary `publish`-style sends still warn

## Phase 3: Bounds-check recognition

- [x] `unchecked_narrowing`: recognize `x.min(T::MAX as u64) as T`
- [x] UI proof: `ui_tests/unchecked_narrowing_min.rs` stays silent for `min(... ) as T` and still warns for a plain unchecked cast
- [ ] `unchecked_narrowing`: recognize `T::try_from(x).unwrap_or(T::MAX)`
- [ ] UI proof: add positive/negative fixture for `try_from(...).unwrap_or(...)`
- [ ] `unchecked_narrowing`: recognize an earlier same-block `assert!(value <= T::MAX as WIDE)` before `value as T`
- [ ] UI proof: add positive/negative fixture for same-block `assert!`
- [ ] `unbounded_loop`: recognize `loop { tokio::select! { ... } }` and imported Tokio `select!` loops as bounded
- [ ] UI proof: add positive/negative async fixtures for Tokio `select!` loops
- [ ] `unbounded_loop`: recognize `while let Some(_) = rx.recv().await` only for `tokio::sync::mpsc::Receiver<_>`
- [ ] UI proof: add positive/negative async fixtures that distinguish bounded `Receiver<_>` from `UnboundedReceiver<_>`

## Phase 4: Downstream validation

- [ ] Re-run against Aspen and record before/after finding counts
- [ ] Confirm no real findings were lost in the precision changes
- [ ] Update rollout playbook noise-level notes if any family becomes promotable
