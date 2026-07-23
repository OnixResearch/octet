# Tasks

- [x] [serial] V1 Confirm the embedded-source tests and canonical self-check gate pass; record the 1,114-occurrence, 372-identity `non_trait_imports` inventory and the four-identity build-shell cohort. r[octet.runner_self_check.warning_budget.reduction]
- [ ] [depends:V1] I1 Replace the four non-trait build-shell imports with explicit owner-qualified paths without changing the trait import, lint policy, exceptions, or behavior. r[octet.runner_self_check.warning_budget.reduction.build_shell]
- [ ] [depends:I1] V2 Compile and test the build shell, then require the old baseline to report exactly four stale IDs, zero new IDs, 2,666 warnings, and warning-only exit `0`. r[octet.runner_self_check.warning_budget.reduction.old_baseline]
- [ ] [depends:V2] I2 Generate and review the candidate baseline as the exact old set minus four IDs; update grouped documentation and permanent Nix counts without admitting additions. r[octet.runner_self_check.warning_budget.reduction.refresh]
- [ ] [depends:I2] V3 Run the refreshed positive and one-new-ID negative self-check gates, policy freshness, traceability, formatting, Clippy, Nextest, source-derived fixture checks, and the broad compatible-system flake gate. r[octet.runner_self_check.warning_budget.reduction.regression]
- [ ] [depends:V3] V4 Record bounded evidence, sync the accepted requirement, and archive deterministically with `CAIRN_ARCHIVE_DATE=2026-07-21`. r[octet.runner_self_check.warning_budget.reduction.boundary]
