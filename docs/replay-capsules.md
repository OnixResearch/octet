# Replay capsules

Octet replay capsules are content-addressed receipts for `cargo octet check`
and `cargo octet fingerprint` evidence. They preserve enough normalized data to
inspect and replay a run without replacing JSON, SARIF, baseline, or fingerprint
outputs.

## Capture

Capture a prepared check manifest:

```sh
cargo octet roadmap capsule capture --input check-capsule-input.json --capsule check-capsule.json
```

Capture a prepared fingerprint manifest the same way:

```sh
cargo octet roadmap capsule capture --input fingerprint-capsule-input.json --capsule fingerprint-capsule.json
```

The input manifest records `command_argv`, `run_kind`, `octet_binary_identity`,
`config_hash`, `input_hashes`, `output_hashes`, and an integer
`expected_exit_code`; capture rejects a missing field or a non-integer
`expected_exit_code`. Before hashing, capture drops local-only and secret-like
fields. Field names are split into lowercase words at punctuation and `camelCase`
boundaries; a field is dropped when its words are `absolute path`, `timestamp`,
`username`, or `env dump`, when any word is `secret(s)`, `password(s)`,
`passwd`, `credential(s)`, `apikey`, or `privatekey`, when two consecutive words
are `api key` or `private key`, or when the last word is `token`. Any whole
string value that is an absolute path (Unix `/…` without whitespace, Windows
`C:\…` or `C:/…`, or UNC `\\host\…`), such as a `--manifest-path` entry in
`command_argv`, becomes `<absolute-path>`, so the same run on different
machines yields the same `capsule_hash`. Replay normalizes the observed run the
same way. Secrets and paths embedded inside longer strings are not detected.
The same normalization applies to `cargo octet roadmap artifact store`.

## Inspect and replay

Inspect a capsule:

```sh
cargo octet roadmap capsule inspect --capsule check-capsule.json
```

Replay against observed run metadata:

```sh
cargo octet roadmap capsule replay --capsule check-capsule.json --observed observed-check.json
```

Replay accepts exact matches. It also accepts expected failures when the capsule
records a non-zero `expected_exit_code` and the observed run matches that exit
code. Any command, input, config, binary identity, output hash, or exit-code drift
(including a field missing from the observed run) is reported as `drifted` and
fails closed with exit code 1.

Replay also refuses to evaluate an untrustworthy capsule and exits with code 2:
the capsule must contain an object `manifest` with every compared field and an
integer `expected_exit_code`, and the `capsule_hash` recomputed from that
manifest must equal the recorded `capsule_hash`. Editing a captured manifest,
such as its `output_hashes`, is therefore detected instead of replayed.

## Limits

A replay capsule proves identity of captured evidence and replay inputs. It does
not prove that a finding is correct, safe, verified, or semantically complete.
Operators should treat capsules as reproducibility receipts around Octet
outputs, not as a language runtime or proof system.
