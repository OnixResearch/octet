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
fields. Field names are split into lowercase words at punctuation, at `camelCase`
boundaries, and before the last capital of an acronym run that starts a
capitalized word (`AWSSecretAccessKey` is `aws secret access key`,
`OpenAIAPIKey` is `open aiapi key`). A field is dropped when any of these hold:

- Its words are exactly `absolute path`, `timestamp`, `username`, or `env dump`.
- Any word is `secret(s)`, `password(s)`, `passwd`, or `credential(s)`, and no
  word is one of the metadata words `count`, `has`, `is`, `min`, `max`,
  `length`, `len`, `policy`, `helper`, `enabled`, `required`, `name`,
  `expiry`, `expires`, or `rotation`. So `client_secret` and `dbPassword` are
  dropped, while `secret_count`, `has_secrets`, `password_min_length`, and
  `credential_helper` stay.
- Its last word is `token`, unless the word before it is one of the
  non-credential words `eos`, `bos`, `pad`, `unk`, `sep`, `cls`, `mask`,
  `start`, `end`, `special`, `page`, `next`, `continuation`, `cancellation`,
  `sync`, `max`, `num`, or `count`. So `token`, `access_token`, `GITHUB_TOKEN`,
  `VAULT_TOKEN`, `HF_TOKEN`, `CI_JOB_TOKEN`, and
  `CARGO_REGISTRIES_<NAME>_TOKEN` are dropped, while `eos_token`, `pad_token`,
  `next_page_token`, and `cancellation_token` stay. Fields where `token` is not
  the last word, such as `max_tokens` or `token_count`, also stay.
- Any word is `apikey` or `privatekey`, or a word that is `private` or ends in
  `api` is followed by `key` (`api_key`, `x-api-key`, `private_key`).

Capture then normalizes local path presentation. A string value that starts
with a local root, followed by a path separator or nothing, has that root
replaced by a placeholder and keeps the relative remainder. The local roots are
the current directory as `<workspace>` (run the command from the workspace
root), `HOME` as `<home>`, and the temporary directory as `<temp>`; the longest
matching root wins. So `--manifest-path /home/alice/repo/Cargo.toml` captured
from `/home/alice/repo` becomes `--manifest-path <workspace>/Cargo.toml`, and
the same project at another checkout path or under another home yields the
same `capsule_hash`. Strings outside those roots, such as HTTP routes, JSON
pointers, or `/nix/store/…` paths, stay verbatim, and values under a field
whose last word is `identity`, such as `octet_binary_identity`, are never
normalized. A changed path inside the workspace or a different binary identity
is therefore still drift. Replay normalizes the observed run the same way,
using the replaying process's roots. Secrets and paths embedded inside longer
strings, such as `--manifest-path=/home/alice/repo/Cargo.toml`, are not
detected. The same normalization applies to `cargo octet roadmap artifact
store`, and both report `redaction_status`
`secret-like-and-local-fields-dropped-local-roots-normalized`.

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
