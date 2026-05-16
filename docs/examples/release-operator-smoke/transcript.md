# Release operator smoke transcript

This executable transcript is the release-facing smoke index for operator surfaces that should be discoverable from README/help before a release.

```slotcar-transcript
$ cargo tigerstyle check --cache --artifact-dir artifacts/slotcar --output-format json
=> effect-summary output is written in artifacts/slotcar/summary.txt
=> lint-cache status is recorded per finding as cache_status=miss|hit|bypassed

$ cargo tigerstyle fingerprint --check --review-manifest artifacts/fingerprint-review.json src/lib.rs
=> fingerprint records stable semantic identity and BLAKE3 review hashes

$ cargo tigerstyle migrate start --manifest artifacts/slotcar-migration.json
=> migrate writes a deterministic manifest for safe operator review

$ cargo tigerstyle transcript docs/examples/release-operator-smoke/transcript.md
=> executable transcript validation checks command blocks and expected-output assertions

$ nix build .#checks.x86_64-linux.docs-examples --no-link -L
=> release smoke transcript is covered by the docs-example gate
```
