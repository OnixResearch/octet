# Function fingerprint source-scanner transcript

This transcript documents the source-scanner boundary: `macro_rules!` generated functions are not fingerprint records, while explicit source wrappers are. Commands are run from this directory. The first update starts from the same source with the marker temporarily set to `b3:0`; the checked-in `reviewed.rs` is the final repaired state.

## Update fingerprints only the explicit source wrapper

```console
$ cargo slotcar fingerprint --update reviewed.rs
slotcar fingerprint update
reviewed.rs explicit_generated_boundary written stored=0 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
# exit status: 0
```

## JSON check reports one explicit source function

```console
$ cargo slotcar fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "explicit_generated_boundary",
      "stored_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
      "computed_hash": "eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42",
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
$ cargo slotcar fingerprint --check reviewed.rs
slotcar fingerprint check
reviewed.rs explicit_generated_boundary match stored=eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 computed=b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42 review=source-scanner-boundary evidence=docs/examples/function-fingerprint-source-scanner/transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
# exit status: 0
```
