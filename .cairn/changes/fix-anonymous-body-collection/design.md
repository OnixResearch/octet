# Design

## Owner and boundary

`src/architecture_collector.rs` is the rustc adapter. Rustc definition kinds determine which compiler queries are valid. The adapter records private anonymous items through the existing IR contract. Pure normalization and coverage rules remain unchanged.

## Narrow implementation

`caller_item_id` retains its registered-owner lookup and closure-specific identity path. The fallback uses `def_kind_to_item_kind`. Only `ItemKind::Function` requests a signature.

`collect_item` handles `DefKind::AnonConst` and `DefKind::InlineConst` as private before any visibility query. Other item kinds retain their existing visibility behavior.

The two corrections already exist in the separate source-ownership branch at the inspected `28abbc8b` revision. Record the full source revision in verification evidence. Reuse only these corrections. Preserve that branch and its worktree.

## Compiler controls

Extend the existing architecture-target fixture with explicit const-generic calls, inline-constant calls, and a closure call. Assert the emitted owner kind, absent or present signature, private visibility, and call-to-owner links. Do not accept mere process success.

Run the unchanged fixture as a baseline. Run the expanded fixture against unchanged producer code to reproduce the panic. After the repair, repeat the fixture and its existing missing-shard rejection. Mutation controls remove each query guard separately and must fail.

## Verification and non-claims

Run producer library tests, all-target Clippy, formatting, the Nix compiler-target check, the project-policy check, and Cairn traceability. Run a Seaglass replay through the rebuilt producer without changing its lint levels. A remaining consumer lint failure is not producer acceptance failure if the anonymous-body crash is absent and the dedicated compiler controls pass.

Do not claim complete Seaglass coverage from a truncated lint replay. Do not change producer schemas, proof limits, or lint policy. No force-push, hook bypass, or changes to unrelated worktrees.
