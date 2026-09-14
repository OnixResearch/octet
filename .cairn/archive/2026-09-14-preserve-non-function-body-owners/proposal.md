# Preserve non-function compiler body owners

## Why

The published collector crashes on an inline constant in Noble's thread-local state control.
Its lazy caller lookup treats every unseen non-closure body as a function and requests `fn_sig`.
Plain Rust accepts the fixture. The crash prevents a valid compiler observation and cannot establish boundary rejection.

## What Changes

- Classify lazy body owners with their actual compiler definition kind.
- Preserve constant, static, closure, and function identities without invented function signatures.
- Add compiler controls for constant forms, macro expansion, caller references, unit memberships, and retained forbidden effects.
- Retain the existing schema, lint catalog, source roles, and complete target scope.

## Impact

Octet owns the rustc adapter and evidence semantics. Noble remains a published, pinned consumer.
The canonical contract is `.cairn/specs/compiler-architecture-ir/spec.md`.
This repair does not establish exhaustive compiler-body inventory, compiler authenticity, refinement, or Noble M1 acceptance.
