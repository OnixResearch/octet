# Tasks

- [x] [serial] [task:baseline] Record the existing source-file lint controls. r[octet.lint_signal_quality.source_ownership]
- [x] [serial] [task:ownership] [after:baseline] Restrict source-map scans to compiler-owned local files. r[octet.lint_signal_quality.source_ownership]
- [x] [serial] [task:controls] [after:ownership] Prove local positive and imported-file negative behavior. r[octet.lint_signal_quality.source_ownership]
- [x] [serial] [task:verification] [after:controls] Run library, Clippy, compiler-fixture, and consumer checks. r[octet.lint_signal_quality.source_ownership]
- [x] [serial] [task:close] [after:verification] Record verified evidence and sync the accepted requirement for archive readiness. r[octet.lint_signal_quality.source_ownership]

## Verification Coverage

| Scenario | Control |
| --- | --- |
| Invalid local file remains actionable | Existing local underscore source control |
| Imported dependency files do not contaminate the consumer | SourceFile provenance control and Seaglass replay |
| Valid local file remains accepted | Valid local source control |

See `evidence/verification.md`. The producer repair is verified. The consumer replay still rejects genuine Seaglass findings. Archive and publication receipts remain separate from these implementation controls.
