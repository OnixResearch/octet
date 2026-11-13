# Function fingerprint source-scanner transcript

Captured with `cargo-octet 0.1.1-rc.1`, executable BLAKE3 `a8152960f5b536cf72d96f3b299245cf9f308490049aa47cfa5d0c0aa4d43316`. This local pre-freeze capture is not source-owner approval or release qualification. Commands use the Cargo subcommand spelling actually dispatched during capture.

`macro_rules!` generated functions are not fingerprint records; the explicit source wrapper is. Create this initial `reviewed.rs` in a scratch directory. The checked-in file is the final state, not the initial-update input.

```rust
macro_rules! define_generated_boundary {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input.saturating_add(1)
        }
    };
}

define_generated_boundary!(generated_boundary);

// octet-fingerprint: b3:0; review=source-scanner-boundary; evidence=docs/examples/function-fingerprint-source-scanner/transcript.md
pub fn explicit_generated_boundary(input: u32) -> u32 {
    generated_boundary(input).min(64)
}
```

## Update fingerprints only the explicit source wrapper

```console
$ cargo octet fingerprint --update reviewed.rs
octet fingerprint update
reviewed.rs explicit_generated_boundary written semantic_id=b3:40229370be2db35f stored=0 computed=b3:05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d normalized_body_hash=b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## JSON check reports one explicit source function

```console
$ cargo octet fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "explicit_generated_boundary",
      "stored_hash": "05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d",
      "computed_hash": "05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d",
      "normalized_body_hash": "b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88",
      "semantic_id": "b3:40229370be2db35f",
      "dependency_set_hash": "b3:1d4fc535ad0b0fd60b72a85e2e4cc6ed26c80949c16e61c48836697a81111c31",
      "dependency_identity_status": "source-scanner-token-set",
      "tool_version": "0.1.1-rc.1",
      "status": "match",
      "review": "source-scanner-boundary",
      "evidence": "docs/examples/function-fingerprint-source-scanner/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Human check reports one explicit source function

```console
$ cargo octet fingerprint --check reviewed.rs
octet fingerprint check
reviewed.rs explicit_generated_boundary match semantic_id=b3:40229370be2db35f stored=05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d computed=b3:05541578a083b5dd63ab23ac86f243c233dc92310bb773a62d36525e0d132e3d normalized_body_hash=b3:6220aac73b2c4a9f364ca8040b2f3bacdc13b467be12f1dad0f6111c14b78d88 dependency_identity_status=source-scanner-token-set review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
