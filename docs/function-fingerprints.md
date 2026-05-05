# Function fingerprints

Function fingerprints are selective review gates for functions whose exact body was
reviewed, verified, or certified. They are not a workspace-wide style rule and they
do not prove correctness.

Supported marker forms:

```rust
// tigerstyle-fingerprint: b3:<hash>; review=safety-critical; evidence=openspec/.../evidence/check.txt
fn checked_transition(input: State) -> State { /* ... */ }
```

```rust
#[fingerprint("b3:<hash>", review = "safety-critical", evidence = "openspec/.../evidence/check.txt")]
fn checked_transition(input: State) -> State { /* ... */ }
```

The initial implementation hashes normalized function tokens and ignores the marker
itself. Formatting and comments do not change the hash; signature or body token
changes do. Macro calls produce a review warning unless the marker records an
explicit macro review allowance.

Commands:

```bash
cargo tigerstyle fingerprint --check path/to/file.rs --output-format=json
cargo tigerstyle fingerprint --write path/to/file.rs
cargo tigerstyle fingerprint --update path/to/file.rs
cargo tigerstyle fingerprint --diff path/to/file.rs
```
