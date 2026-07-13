# optional-z4-solver-evidence-rail validation

Implemented the optional `z4-solver` rail in `cargo-octet/src/evidence_rails.rs` and provenance checker registration in `cargo-octet/src/evidence.rs`.

Evidence:

- Baseline before edits: `nix develop -c bash -c 'cargo test -p cargo-octet evidence_rails -- --nocapture'` passed.
- Focused after edits: `nix develop -c bash -c 'cargo test -p cargo-octet evidence_rails -- --nocapture'` passed.
- Format: `nix develop -c bash -c 'cargo fmt --check'` passed after applying `cargo fmt`.
- Artifact path: `cargo run -q -p cargo-octet --bin cargo-octet -- evidence collect --rail z4-solver --input cargo-octet/tests/fixtures/evidence-rails/z4-solver-unsat-proof.json --artifact-dir "$artifact_dir" --output-format json` followed by `cargo run -q -p cargo-octet --bin cargo-octet -- artifact verify --artifact-dir "$artifact_dir" --output-format json` passed.
- Cairn validation: `nix run path:/home/brittonr/git/OnixResearch/octet#cairn -- validate --root /home/brittonr/git/OnixResearch/octet` passed.

Proof-card validation is not applicable to this recorded-only z4 evidence rail because the implementation explicitly rejects `formal_proof` promotion and does not emit proof cards.
