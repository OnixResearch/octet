## Contract

- [ ] [serial] Specify broadened discard semantics for `ignored_result` and `drop_must_use`. r[octet.safety_lint_semantics.discards]
- [ ] [serial] Specify method/direct recursion coverage and advisory boundaries for mutual recursion. r[octet.safety_lint_semantics.recursion]
- [ ] [serial] Specify structural loop-bound recognition for `unbounded_loop`. r[octet.safety_lint_semantics.loop_bounds]

## Implementation

- [ ] [serial] Extend `ignored_result` and `drop_must_use` to cover additional typed discard forms while preserving shutdown/drop-context exceptions. r[octet.safety_lint_semantics.discards]
- [ ] [serial] Extend `no_recursion` to method self-recursion and document or implement mutual-recursion advisory behavior. r[octet.safety_lint_semantics.recursion]
- [ ] [serial] Replace source-marker loop checks with structural bound/cancellation recognition where type facts are available. r[octet.safety_lint_semantics.loop_bounds]

## Verification

- [ ] [parallel] Add positive and negative UI fixtures for every new discard shape. r[octet.safety_lint_semantics.discards]
- [ ] [parallel] Add method recursion and non-recursive trait/default-method guardrail fixtures. r[octet.safety_lint_semantics.recursion]
- [ ] [parallel] Add bounded-loop, shutdown-channel, and runaway-loop fixtures, including async/Tokio cases. r[octet.safety_lint_semantics.loop_bounds]
- [ ] [serial] Run targeted safety UI rail, `cargo test --lib`, and Cairn validation/gates. r[octet.safety_lint_semantics.loop_bounds]
