# Proposal: Detect false durability claims

## Why

Rust code can discard synchronization errors, use `Write::flush()` as durable persistence, or retry a failed synchronization call as if later success proves earlier bytes reached storage.

Octet can detect bounded source shapes and require external storage-fault evidence. It cannot prove filesystem or device behavior.

## What Changes

- Add a declared durability-boundary profile for consumer-owned persistence surfaces.
- Detect discarded `sync_all`, `sync_data`, and configured commit results.
- Reject `Write::flush()` as the only evidence on a declared durable boundary.
- Detect bounded same-handle synchronization retry shapes without an intervening rewrite or reopen.
- Add guidance for explicit uncertain outcomes, quarantine, reopen, and reconciliation.
- Add a storage-fault assurance profile that links saved ChaosControl and Cairn evidence.
- Add positive and negative UI, policy, and evidence fixtures.

## Impact

- **Policy**: Nickel declarations for durability boundaries and assurance evidence.
- **Lints and standards**: bounded source-shape diagnostics.
- **Evidence**: exact source, profile, finding, external receipt, and non-claim bindings.

## Non-goals

- Do not infer arbitrary durability semantics from function names.
- Do not require direct I/O for every application.
- Do not prove that `sync_all`, Redb commit, a filesystem, or hardware is correct.
- Do not convert campaign evidence into release authority.
