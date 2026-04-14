# Clippy triage notes

Current-nightly `checks.clippy` failures were mechanical style rewrites, not
policy disputes.

Buckets:

- collapse nested `if` / `if let` chains with let-chains
- replace `iter().any(...)` with `contains(...)` where Clippy requested it
- remove unnecessary same-type casts
- replace a single-pattern `match` with `if let`
- run repo formatting so `checks.fmt` stays green after source edits

No lint semantics changed. The cleanup only rewrote internal implementation
style in existing lint code so the repo passes its own current-nightly clippy
profile.
