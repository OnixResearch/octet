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

The token, `pass`/`pwd`, and qualified-key rules below first skip trailing
numbering and environment words: any all-digit word and `prod`, `production`,
`dev`, `development`, `staging`, `stage`, `test`, `testing`, `qa`, `uat`, `ci`,
`local`, `live`, `sandbox`, `preview`, `old`, `new`, `primary`, `secondary`,
and `backup`. The words left are the core words, so `GITHUB_TOKEN_2`,
`NPM_TOKEN_PROD`, and `DB_PASS_PROD` end in `token`, `token`, and `pass`.

- Any word is `secret(s)`, `passwd`, `credential(s)`, `authorization`, `auth`,
  `bearer`, `cookie(s)`, `jwt`, or `sessionid`, any word contains `password`
  (`PGPASSWORD`), the words include `session id`, or the last core word is
  `pass` or `pwd`; and no word is one of the metadata words `count`, `has`,
  `is`, `min`, `max`, `length`, `len`, `policy`, `helper`, `enabled`,
  `required`, `name`, `expiry`, `expires`, or `rotation`. So `client_secret`,
  `dbPassword`, `PGPASSWORD`, `MYSQL_PWD`, `DB_PASS`, `SMTP_PASS`,
  `DB_PASS_PROD`, `Authorization`, `cookie`, `session_id`, and `jwt` are
  dropped, while `secret_count`, `has_secrets`, `password_min_length`,
  `credential_helper`, `auth_required`, `cookie_name`, and `pass` or `pwd`
  followed by another word, such as the proof-rail field `pass_signal` or
  `pass_rate`, stay.
- The last core word is `token` or `tokens`, possibly followed by digits
  (`GITHUB_TOKEN2`), and the field is not exempt. Either form is exempt when
  any word is one of the count words `max`, `min`, `num`, or `total`, or when
  the word before the token is one of the vocabulary, position, pagination, and
  coordination words `eos`, `bos`, `eot`, `eod`, `pad`, `unk`, `sep`, `cls`,
  `mask`, `start`, `stop`, `end`, `special`, `first`, `last`, `prev`,
  `previous`, `page`, `pagination`, `next`, `continuation`, `resume`, `cursor`,
  `cancellation`, `sync`, or `count`. Only the plural `tokens` is also exempt
  after the usage-count and word-list words `prompt`, `completion`, `input`,
  `output`, `cached`, `reasoning`, `capability`, `normative`, or
  `placeholder`; a singular `token` after them is a credential, because
  `INPUT_TOKEN` is how GitHub Actions passes a `token` input. So `token`,
  `tokens`, `access_token`, `GITHUB_TOKEN`, `VAULT_TOKEN`, `HF_TOKEN`,
  `CI_JOB_TOKEN`, `CARGO_REGISTRIES_<NAME>_TOKEN`, `INPUT_TOKEN`,
  `cached_token`, and `capability_token` are dropped, while `eos_token`,
  `stop_token`, `last_token`, `resume_token`, `next_page_token`,
  `PaginationToken`, `cancellation_token`, `max_tokens`, `max_new_tokens`,
  `prompt_tokens`, `input_tokens`, and `capability_tokens` stay. Fields where
  `token` is not the last core word, such as `token_count` or `token_limit`,
  also stay.
- Any word is `apikey` or `privatekey`; or `key` anywhere follows `private`
  or a word ending in `api` (`api_key`, `x-api-key`, `private_key`,
  `private_key_path`); or `key` is the last core word or is followed by `id`
  and follows a word other than the lookup, schema, and public-key words
  `primary`, `foreign`, `composite`, `unique`, `sort`, `partition`, `shard`,
  `index`, `row`, `group`, `dedup`, `cache`, `map`, `dict`, `hash`, `lookup`,
  or `public`. So `STRIPE_KEY`, `SIGNING_KEY`, `STRIPE_KEY_PROD`, and
  `AWS_ACCESS_KEY_ID` are dropped, while `key`, `key_id`, `primary_key`,
  `cache_key`, `public_key`, and `changed_key_components` stay.

Capture then normalizes local path presentation. A string value that starts
with a local root, followed by a path separator or nothing, has that root
replaced by a placeholder and keeps the relative remainder. The local roots are
the current directory as `<workspace>` (run the command from the workspace
root), `HOME` as `<home>`, and the temporary directory as `<temp>`; the longest
matching root wins. Each root is matched in every spelling a recorded path is
likely to use: the current directory as the operating system resolves it and,
when the shell's `PWD` is absolute and resolves to that same directory, as `PWD`
spells it through symlinks; `HOME` and the temporary directory both as given and
with symlinks resolved. So a checkout under a symlinked directory, such as
macOS `/tmp` resolving to `/private/tmp`, normalizes in either spelling. So `--manifest-path /home/alice/repo/Cargo.toml` captured
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
