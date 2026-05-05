# Function fingerprint downstream smoke

The flake exposes a scratch-consumer smoke for the checked fingerprint docs examples:

```bash
nix build .#checks.x86_64-linux.fingerprint-docs-downstream-smoke --no-link -L
```

The check creates a temporary Cargo crate, copies every checked
`docs/examples/function-fingerprint-*/reviewed.rs` fixture into that crate, and
runs the real `cargo-tigerstyle fingerprint` binary over the consumer `src/`
directory. It asserts that:

- all seven documented records are discovered outside the Tigerstyle checkout;
- `macro_rules!` generated functions are still absent from the record set;
- warning and missing-metadata examples keep their nonzero operator-warning status;
- JSON drift, update, and repaired-check behavior work on a copied downstream file.

The reusable script is also exposed as:

```bash
nix run .#fingerprint-docs-downstream-smoke
```
