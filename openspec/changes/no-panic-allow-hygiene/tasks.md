## Phase 1: Escape-hatch hygiene

- [ ] Extend `src/safety/no_panic.rs` so `unjustified_no_panic_allow` can detect orphaned item-level and block-level `#[allow(tigerstyle::no_panic)]` attrs even when no panic-family macro remains
- [ ] Add UI coverage for orphaned item-level and block-level allows while keeping justified live `todo!()` escapes quiet
- [ ] Run `cargo test --lib`
- [ ] Run `cargo test --test ui -- --ignored --nocapture`
