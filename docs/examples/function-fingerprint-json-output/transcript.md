# Function fingerprint JSON output transcript

This transcript is executable documentation for script-facing JSON output. Commands are run from this directory.

## Initial JSON update writes the marker

```console
$ cargo tigerstyle fingerprint --update --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "0",
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "status": "written",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Clean JSON check reports one match

```console
$ cargo tigerstyle fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "status": "match",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Drift JSON check reports one mismatch

```console
$ cargo tigerstyle fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "status": "mismatch",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 1
```

## Accepted JSON update rewrites the marker

```console
$ cargo tigerstyle fingerprint --update --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "update",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "status": "written",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```

## Accepted JSON check reports one match

```console
$ cargo tigerstyle fingerprint --check --output-format=json reviewed.rs
{
  "schema_version": 1,
  "mode": "check",
  "records": [
    {
      "path": "reviewed.rs",
      "name": "capped_retries",
      "stored_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "computed_hash": "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
      "status": "match",
      "review": "json-script-example",
      "evidence": "docs/examples/function-fingerprint-json-output/transcript.md",
      "macro_calls": false,
      "macro_warning": false
    }
  ]
}
# exit status: 0
```
