# Real CodeQL producer fixtures

These two fixture crates retain raw SARIF produced by the pinned local CodeQL
producer profile, not the test-only scanner executable used by the unit tests.

- CodeQL CLI `2.27.2`
- Query pack `codeql/rust-queries@0.1.46`
- Producer profile `codeql-exec-local`
- Profile bytes `standards/sarif/producers.json`
  (`b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd`)
- CodeQL database build mode `none`

`codeql-clean/report.sarif` has zero results. `codeql-violation/report.sarif`
has one `rust/non-https-url` warning at `src/lib.rs:7` from the reqwest
request-url sink. The retained reports are imported by the checked
`codeql-local` intake profile in
`producer_tests::retained_real_codeql_reports_import_with_expected_findings`.

Reproduce with CodeQL on `PATH` and a copy of the checked producer registry in
the fixture root:

```sh
cp standards/sarif/producers.json standards/sarif/fixtures/codeql-violation/
cargo octet evidence collect \
  --rail sarif-import \
  --profile codeql-exec-local \
  --config producers.json \
  --workspace-root standards/sarif/fixtures/codeql-violation \
  --run \
  --artifact-dir artifacts/codeql-violation \
  --output-format json
```

The producer writes `sarif-raw-report.json`; it is retained here as
`report.sarif`. The fixture is a bounded external static observation. It does not
prove complete vulnerability detection, whole-program correctness, or release
eligibility.

The pinned Semgrep `1.180.0` package does not build in this environment's Nix
toolchain, so no Semgrep report is retained. The Semgrep producer profile remains
declarative and is exercised only through the explicitly labeled test-only
fixture executable.
