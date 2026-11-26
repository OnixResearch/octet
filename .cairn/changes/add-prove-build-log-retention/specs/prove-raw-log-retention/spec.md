# Prove Raw Log Retention Delta

## ADDED Requirements

### Requirement: Opt-in build-log retention keeps raw tool output out of the bundle

r[octet.prove_raw_log_retention.retention] `cargo octet prove` MUST accept
`--raw-log-retention bundle|build-log` and MUST default to `bundle`. In `bundle`
mode the written bundle MUST stay byte-identical to the previous behavior. In
`build-log` mode the raw stage stdout/stderr MUST be written to cargo-octet's
stderr, framed with a stable header naming the rail, the stage and the stream;
the bundle MUST contain no `proofs/*.stdout|stderr` files and no
`evidence-log-manifest.json`; nothing in the bundle MUST hash the raw text; and
`status.json` and the human summary MUST record `raw_log_retention: build-log`
and a non-claim that the raw tool logs are in the build log and are not bound by
the bundle identity.

#### Scenario: Build-log run writes no bundle log artifacts

- GIVEN a prove run selecting a rail that emits stdout and stderr
- WHEN it runs with `--raw-log-retention build-log`
- THEN no `proofs/*.stdout|stderr` file and no `evidence-log-manifest.json` exist
  in the bundle
- AND `status.json` and `summary.txt` record the mode and the non-claim
- AND the same run with the default `bundle` mode still writes and manifests the
  raw logs.

#### Scenario: Capture consumers still decide the stage verdict

- GIVEN a `lake build` capture containing a declaration-uses-`sorry` warning, or
  an axiom report naming an unadmitted axiom
- WHEN the run uses `--raw-log-retention build-log`
- THEN the stage MUST still fail on that in-memory capture
- AND the failed status and diagnostics MUST remain in the receipt and status.

### Requirement: Verification matches the declared retention shape

r[octet.prove_raw_log_retention.verification] `cargo octet artifact verify` MUST
accept a prove bundle whose `status.json` declares `raw_log_retention:
build-log` and that contains no raw log artifacts and no evidence-log manifest,
and MUST reject a bundle that declares `build-log` while containing a manifest
or a `proofs/*.stdout|stderr` artifact. It MUST leave `bundle`-mode verification
unchanged.

#### Scenario: Valid build-log bundle is accepted

- GIVEN a `build-log` prove bundle with its receipts, cards and provenance but no
  raw logs or manifest
- WHEN `cargo octet artifact verify` runs
- THEN the bundle is valid with no diagnostics.

#### Scenario: Contradictory build-log bundle is rejected

- GIVEN a bundle that declares `raw_log_retention: build-log`
- WHEN it also contains `evidence-log-manifest.json` or a
  `proofs/*.stdout|stderr` artifact
- THEN verification fails with a diagnostic naming the contradictory artifact.

### Requirement: Build-log mode removes timing variance from bundle identity

r[octet.prove_raw_log_retention.reproducibility] Two prove runs that differ only
in tool progress and timing text MUST produce byte-identical bundles when both
use `--raw-log-retention build-log`; the same two runs in `bundle` mode MUST
differ.

#### Scenario: Timing text changes only the build log

- GIVEN two runs of the same rail whose tool output differs only in progress
  order and durations
- WHEN both use `build-log` retention
- THEN every bundle file is byte-identical
- AND the same pair of runs with `bundle` retention differs.

### Requirement: The Mantle consumer job uses build-log retention

r[octet.prove_raw_log_retention.consumer_job] The Aeneas consumer job derivation
MUST invoke `cargo octet prove --raw-log-retention build-log`, and its failure
path MUST keep the raw stage output visible through the build log while
retaining receipts and reports for diagnostics.

#### Scenario: Consumer job derivation selects build-log retention

- GIVEN the consumer job derivation inputs
- WHEN the derivation is evaluated and its prove invocation is inspected
- THEN it passes `--raw-log-retention build-log`
- AND its failure diagnostics read the raw logs from the build log rather than
  from bundle files that no longer exist.
