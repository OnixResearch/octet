# Function fingerprint lifecycle smoke

This artifact shows the complete operator lifecycle for one reviewed function:
initial marker repair, clean check, source drift, diff diagnosis, accepted update,
and final clean check.

The fixture was created in a scratch directory, then the final repaired source was
copied into `reviewed.rs`. Commands were run from the scratch directory with the
repo-local binary:

```bash
TOOL='cargo run -q --manifest-path /home/brittonr/git/tigerstyle/cargo-tigerstyle/Cargo.toml --'
```

## Initial fixture

```rust
#[rustfmt::skip]
// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    50_u32.saturating_mul(attempt.saturating_add(1))
}
```

`#[rustfmt::skip]` keeps this deliberately long single-line marker intact for the
example. Production code can use shorter `review` and `evidence` values instead.

## Initial update

```bash
$ $TOOL fingerprint --update reviewed.rs
```

```text
tigerstyle fingerprint update
reviewed.rs bounded_retry_delay_ms written stored=0 computed=b3:c5707c7c97cc9590933c9b45064bcfc39d19ef7ad553e5b9ec1d751cc242b48c review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

## Clean check

```bash
$ $TOOL fingerprint --check reviewed.rs
```

```text
tigerstyle fingerprint check
reviewed.rs bounded_retry_delay_ms match stored=c5707c7c97cc9590933c9b45064bcfc39d19ef7ad553e5b9ec1d751cc242b48c computed=b3:c5707c7c97cc9590933c9b45064bcfc39d19ef7ad553e5b9ec1d751cc242b48c review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Drift mutation

The reviewed function body was changed from:

```rust
50_u32.saturating_mul(attempt.saturating_add(1))
```

to:

```rust
75_u32.saturating_mul(attempt.saturating_add(1))
```

## Diff after drift

```bash
$ $TOOL fingerprint --diff reviewed.rs
$ echo $?
1
```

```text
tigerstyle fingerprint diff
reviewed.rs bounded_retry_delay_ms mismatch stored=c5707c7c97cc9590933c9b45064bcfc39d19ef7ad553e5b9ec1d751cc242b48c computed=b3:7deb37887bd38ad0b5e586a9700eb9d34f61153df85ab06649c3f468f6816114 review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=1 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Accepted update after review

```bash
$ $TOOL fingerprint --update reviewed.rs
```

```text
tigerstyle fingerprint update
reviewed.rs bounded_retry_delay_ms written stored=c5707c7c97cc9590933c9b45064bcfc39d19ef7ad553e5b9ec1d751cc242b48c computed=b3:7deb37887bd38ad0b5e586a9700eb9d34f61153df85ab06649c3f468f6816114 review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=0 mismatch=0 missing=0 written=1 macro_warnings=0 metadata_missing=0
```

## Final clean check

```bash
$ $TOOL fingerprint --check reviewed.rs
```

```text
tigerstyle fingerprint check
reviewed.rs bounded_retry_delay_ms match stored=7deb37887bd38ad0b5e586a9700eb9d34f61153df85ab06649c3f468f6816114 computed=b3:7deb37887bd38ad0b5e586a9700eb9d34f61153df85ab06649c3f468f6816114 review=safety-critical evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md macro_warning=false
summary total=1 match=1 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0
```

## Final fixture

The checked-in `reviewed.rs` file is the final repaired state. The important
operator assertions are:

- clean checks report `match=1 mismatch=0 written=0`;
- drift diff exits `1` and reports `mismatch=1`;
- accepted update reports exactly one `written` marker;
- final check returns to `match=1 mismatch=0 written=0`.
