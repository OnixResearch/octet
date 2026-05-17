# Roadmap agent surfaces transcript

This executable transcript documents the Unison-inspired agent-facing roadmap surfaces. It is intentionally command-shaped: operators can copy each command into a scratch directory with the named JSON fixtures and expect deterministic JSON receipts.

```slotcar-transcript
$ cargo slotcar roadmap artifact store --kind prompt --input prompt.json --store-dir store
=> stores normalized content, redacts local/secret fields, and returns a stable b3: artifact hash

$ cargo slotcar roadmap artifact inspect --hash b3:<artifact> --store-dir store
=> reports artifact_kind, dependencies, and replay prerequisites without re-reading local paths

$ cargo slotcar roadmap artifact replay --hash b3:<artifact> --store-dir store
=> fails closed when dependencies are missing and replays only when prerequisite hashes exist

$ cargo slotcar roadmap ability enforce --manifest ability-manifest.json --trace effect-trace.json
=> denies effects not declared by the ability manifest and reports denied_effects

$ cargo slotcar roadmap pointer set --registry registry.json --name done-review --kind prompt --hash b3:<artifact> --receipt pointer-receipt.json
=> records name-to-hash updates with an old_hash rollback receipt

$ cargo slotcar roadmap pointer rollback --registry registry.json --receipt pointer-receipt.json
=> restores the prior immutable hash from the rollback receipt

$ cargo slotcar roadmap session check --input session.json
=> verifies typed/versioned session records with identity and redaction metadata

$ cargo slotcar roadmap session migrate --input session-v0.json --output session-v1.json
=> writes a schema_version migration receipt without changing the original session identity

$ cargo slotcar roadmap session todo --input session.json
=> summarizes structured todo kinds, statuses, sources, next actions, and blockers

$ cargo slotcar roadmap pure-cache check --key cache-key.json --store-dir cache --output-hash b3:<output>
=> reports miss for new deterministic keys and hit when command identity, input hashes, config hash, tool revision, and environment allowlist match
```
