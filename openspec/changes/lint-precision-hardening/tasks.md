## Phase 1: Test-context suppression

- [x] Add test-context detection helper (shared across lints)
- [x] `no_unwrap`: skip `#[test]` fns and `#[cfg(test)]` modules
- [x] `no_panic`: skip `#[test]` fns and `#[cfg(test)]` modules
- [x] UI fixtures for both: fire in production, silent in test

## Phase 2: Pattern-based suppression

- [x] `no_unwrap`: skip `.unwrap()` on compile-time-known `Some`/`Ok` (literal `.parse()`, `Some(x)`)
- [x] `no_unwrap`: skip `.unwrap()` after `.is_some()`/`.is_ok()` guard in same branch
- [ ] `no_panic`: skip `unreachable!()` in exhaustive match trailing wildcard
- [ ] `no_panic`: skip `todo!()` with `#[allow(tigerstyle::no_panic)]` + justification
- [x] `ignored_result`: skip `let _ = expr` inside `Drop::drop` impls
- [ ] `ignored_result`: skip fire-and-forget `send` on shutdown paths (channel drop patterns)
- [ ] UI fixtures for each suppression case

## Phase 3: Bounds-check recognition

- [ ] `unchecked_narrowing`: recognize `x.min(T::MAX as u64) as T`
- [ ] `unchecked_narrowing`: recognize `T::try_from(x).unwrap_or(T::MAX)`
- [ ] `unchecked_narrowing`: recognize `assert!` within same block (not just preceding line)
- [ ] `unbounded_loop`: recognize `loop { tokio::select! { ... } }` as bounded
- [ ] `unbounded_loop`: recognize `while let Some(_) = rx.recv().await` for bounded receivers
- [ ] UI fixtures for each recognition pattern

## Phase 4: Downstream validation

- [ ] Re-run against Aspen and record before/after finding counts
- [ ] Confirm no real findings were lost in the precision changes
- [ ] Update rollout playbook noise-level notes if any family becomes promotable
