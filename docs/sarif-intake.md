# Local SARIF intake

Octet imports a bounded SARIF 2.1.0 subset from an explicit local file. It does not run CodeQL, Semgrep, or a hosted scanner.

## Import

Select one checked producer profile:

```console
cargo octet evidence collect \
  --rail sarif-import \
  --profile codeql-local \
  --config standards/sarif/profiles.json \
  --input target/codeql.sarif \
  --artifact-dir artifacts/sarif-codeql
```

Use `semgrep-local` for a Semgrep report. Octet rejects `--run` and `--command` for this rail.

The profile binds the driver name, supported version, required rule identity, source-level mapping, local path policy, evidence tier, and resource limits. The checked JSON profile has BLAKE3 identity `b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7`.

## Supported subset

A report contains one local run. The run contains a declared driver, a `tool.driver.rules` array, bounded results, messages, and one physical location per result. The run's `results` must be an array: SARIF treats absent or `null` results as a tool that failed to start, so Octet rejects that run, and it rejects a run whose `invocations` report `executionSuccessful: false`. `tool.extensions`, `artifacts`, `invocations`, and each extension's `rules` are optional, but each one present must be an array.

Rules can be declared in `tool.driver.rules` or in a `tool.extensions[].rules` table, where CodeQL's `--sarif-group-rules-by-pack` places them; a rule ID declared twice anywhere is rejected. A result's `ruleIndex`, and the `index` of its `rule` reference, must be in range of the rule table of the referenced tool component (`rule.toolComponent.index` into `tool.extensions`, otherwise the driver) and name the result's `ruleId`. A `rule.id` must equal `ruleId`, and a `ruleIndex` and `rule.index` that are both present must agree. A tool component referenced only by name or GUID is rejected. `-1` is SARIF's "no index" default and counts as absent.

A location can contain a safe relative URI, a valid index into the run artifact table, or both; when both are present, the URI and the indexed artifact's URI must name the same path. Octet percent-decodes each URI and applies the path policy to the decoded path, which it also stores, so `src/my%20module.rs` becomes `src/my module.rs` and `src/%2e%2e/x.rs` is rejected as traversal. A malformed escape, a decoded path that is not UTF-8 or contains a control character, a colon, or a backslash-separated `..`, and a raw `?` or `#` (a query or fragment) are rejected. A URI without `uriBaseId`, or with the `%SRCROOT%` base CodeQL emits, is relative to the workspace root; Octet does not read `originalUriBaseIds`, and any other `uriBaseId` is rejected. Each region contains a positive start line and a positive start column; an absent `startColumn` means column 1, the SARIF default that CodeQL omits.

Octet preserves the driver, rule ID, source level, message, path, line, and column. The normalized finding stores source level separately from Octet gate severity.

Octet follows SARIF 2.1.0 result `kind` and `level` defaults. A result without `kind` is a `fail` result. A `fail` result without `level` takes the declared rule's `defaultConfiguration.level`, then `warning`. Octet does not import `pass`, `notApplicable`, or `informational` results and does not gate on them; such a result must omit `level` or set it to `none`. Octet rejects reports with undecided `review` or `open` results and with unknown kinds, because it cannot decide whether they should gate.

Octet rejects malformed JSON, unsupported versions, unknown drivers, missing, `null`, or non-array results and rule tables, unsuccessful tool executions, undeclared or duplicate rules, rule indexes that are out of range or name another rule, unsupported levels or kinds, remote URIs, absolute paths, traversal (including percent-encoded traversal), malformed percent escapes, URI queries and fragments, unsupported `uriBaseId` values, bad artifact indexes, URIs that disagree with their artifact index, missing regions, duplicate normalized findings, and over-limit input.

Genuine CodeQL CLI 2.27.2 output imports under `codeql-local`: the checked fixtures `cargo-octet/tests/fixtures/evidence-rails/sarif-codeql-cli.json` and `sarif-codeql-cli-grouped.json` are trimmed `codeql database analyze --format=sarif-latest` reports for a small Rust crate, without and with `--sarif-group-rules-by-pack`. CodeQL reports columns in UTF-16 code units (`columnKind: utf16CodeUnits`); Octet preserves the reported column number unchanged.

## Artifacts and privacy

The bundle contains the exact raw report, a deterministic normalized projection, a receipt, a summary, and provenance. The projection links to the raw report. Artifact verification remeasures both BLAKE3 identities.

SARIF messages and paths can contain sensitive data. Review the local report before import. The intake does not redact source content and does not contact a network service.

## Non-claims

Imported SARIF findings remain external static observations. They do not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.
