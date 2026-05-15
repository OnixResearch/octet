# Function fingerprint warning summary transcript

This transcript is executable documentation for `macro_warnings` and `metadata_missing` summary counts. Commands are run from this directory.

## Initial update writes markers and reports warning counts

```console
$ cargo slotcar fingerprint --update reviewed.rs
slotcar fingerprint update
reviewed.rs unreviewed_macro_path written stored=0 computed=b3:f00700706a371a7cf92aa0ce17350dfc5d349e561a1f491d1102d5f17688b35e review=macro-warning-example evidence=docs/examples/function-fingerprint-warning-summary/transcript.md macro_warning=true
reviewed.rs missing_gate_metadata written stored=0 computed=b3:70e5f9ca896cf2aa8f305b1ea6cdd35ce59ead7f7c8cfc5fe14ff9c7db7d76f4 review=<missing> evidence=<missing> macro_warning=false
summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1
# exit status: 0
```

## Clean check fails on warning counts

```console
$ cargo slotcar fingerprint --check reviewed.rs
slotcar fingerprint check
reviewed.rs unreviewed_macro_path match stored=f00700706a371a7cf92aa0ce17350dfc5d349e561a1f491d1102d5f17688b35e computed=b3:f00700706a371a7cf92aa0ce17350dfc5d349e561a1f491d1102d5f17688b35e review=macro-warning-example evidence=docs/examples/function-fingerprint-warning-summary/transcript.md macro_warning=true
reviewed.rs missing_gate_metadata match stored=70e5f9ca896cf2aa8f305b1ea6cdd35ce59ead7f7c8cfc5fe14ff9c7db7d76f4 computed=b3:70e5f9ca896cf2aa8f305b1ea6cdd35ce59ead7f7c8cfc5fe14ff9c7db7d76f4 review=<missing> evidence=<missing> macro_warning=false
summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1
# exit status: 1
```
