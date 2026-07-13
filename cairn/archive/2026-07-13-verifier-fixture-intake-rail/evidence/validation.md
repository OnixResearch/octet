# verifier-fixture-intake-rail validation

Implemented `verifier-fixture` intake as a recorded-only optional evidence rail in `cargo-octet/src/evidence_rails.rs`.

Evidence:

- Baseline before edits: `nix develop -c bash -c 'cargo test -p cargo-octet evidence_rails -- --nocapture'` passed.
- Focused after edits: `nix develop -c bash -c 'cargo test -p cargo-octet evidence_rails -- --nocapture'` passed.
- Format: `nix develop -c bash -c 'cargo fmt --check'` completed after applying `cargo fmt`.
- Artifact path: `cargo run -q -p cargo-octet --bin cargo-octet -- evidence collect --rail verifier-fixture --input cargo-octet/tests/fixtures/evidence-rails/verifier-fixture-positive.json --artifact-dir "$artifact_dir" --output-format json` followed by `cargo run -q -p cargo-octet --bin cargo-octet -- artifact verify --artifact-dir "$artifact_dir" --output-format json` passed.

Proof-card validation is not applicable to this fixture-intake rail because the implementation explicitly records regression/UX fixtures only and rejects proof-soundness, verifier-parity, source-correctness, and release-eligibility claims.
