# Function fingerprint human summary example

This example documents the human-output path for `cargo octet fingerprint`.
It shows two reviewed functions, a one-function drift, the non-zero check result,
and the repair update that rewrites exactly one marker.

The executable regression checks matching and drifted function counts and the
repair transition, including the resulting `reviewed.rs` source. Transcript
wording is not a test-suite oracle.

[`check-output.json`](check-output.json) is the observed JSON check output from
the preflight `cargo-octet 0.1.1-rc.1` runner. Replay the JSON view with:

```sh
cargo octet fingerprint --check --output-format=json reviewed.rs
```

The capture preserves the checked-in source bodies, marker hashes, and review
metadata. Version-bound semantic IDs changed; the scratch repair still produced
the checked-in `reviewed.rs` bytes exactly. These example observations are not
source-owner approval or final release qualification.
