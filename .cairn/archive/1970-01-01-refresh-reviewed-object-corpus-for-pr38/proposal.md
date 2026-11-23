# Proposal: Admit the owner-approved object-corpus refresh for PR #38

## Why

PR #38 (`20045f39`, branch `reproducible-consumer-job-and-artifact-base`) adds
`--raw-log-retention` and `--artifact-base` to `cargo-octet/src/cli.rs`. That
file is one of the 40 selected reviewed object-corpus sources, so the frozen
candidate/reviewed manifests and the verifier anchors no longer match the
branch bytes. CI `verify` fails at
`checks.x86_64-linux.thir-supplement-inventory-gate` with
`reviewed-v2-source: REJECT selected source byte drift: cargo-octet/src/cli.rs`;
every object rail runs `tools/verify-reviewed-object-source.py` first, so the
whole object-rail family is blocked.

## What Changes

- Re-freeze the selected `cargo-octet/src/cli.rs` byte hash in both the
  candidate and reviewed source manifests and rebind the reviewed manifest's
  `candidate_manifest` to the new candidate file.
- Pin the exact refreshed candidate and reviewed manifest bytes; retain the
  original audit envelope, producer/source binding, historical approval and
  Valence binding without changing their claims or bytes.
- Re-capture the three affected references (`internal-corpus-receipt.json`,
  `expanded-internal-corpus-receipt.json`,
  `expanded-internal-corpus-caveat-taxonomy.json`) with the immutable
  `cargo-octet 0.1.1-rc.1` producer, after proving that producer reproduces all
  four earlier references byte-for-byte on `origin/main`. The corpus gains
  exactly two objects (`RawLogRetention::as_str` and `RawLogRetention::parse`):
  1155 -> 1157 and 1764 -> 1766. `positive-v2-receipt.json` is unchanged.
- Require the pinned PR #38 owner decision in the verifier before using the
  reviewed manifest's old-to-new mappings. The decision binds the exact two
  manifest digests and does not alter them.

## Non-claims

The owner's direct decision admits the selected source/reference tuple, not
future changes. It does not establish a new independent audit, correctness,
semantic equivalence, an owner signature or release eligibility. The original
independent audit covered the earlier source and reference bytes only.
