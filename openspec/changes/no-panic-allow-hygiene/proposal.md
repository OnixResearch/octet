## Why

`unjustified_no_panic_allow` now preserves the `no_panic` escape-hatch contract
for justified `todo!()` stubs, but it only fires when a panic-family macro is
still present under the allow. After a cleanup removes the panic macro, an
orphaned `#[allow(tigerstyle::no_panic)]` can remain in production code with no
warning.

That leaves dead panic escapes behind and weakens review of the migration's new
companion-lint path.

## What Changes

- Audit `#[allow(tigerstyle::no_panic)]` even when no panic-family macro remains
  under the annotated item or block
- Warn on orphaned panic escapes so dead `allow` attrs do not accumulate after
  cleanup
- Add UI proof for orphaned item-level and block-level escapes

## Capabilities

### New Capabilities
- `no-panic-allow-hygiene`: detect orphaned `#[allow(tigerstyle::no_panic)]`
  after the panic-family code it once covered is gone

## Impact

- **Files**: `src/safety/no_panic.rs`, `ui_tests/no_panic_todo_allow.rs` or new
  dedicated fixtures, `openspec/specs/safety/spec.md`
- **APIs**: no new public lint names; tighter auditing of the existing escape
  hatch
- **Testing**: `cargo test --lib`, `cargo test --test ui -- --ignored --nocapture`
