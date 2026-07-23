# Tasks

- [x] [serial] V1 Confirm CLI and canonical self-check baselines; record the 1,114-occurrence, 372-identity inventory and reject build-shell and domain-format candidates after old-baseline runs exposed unrelated identity churn. r[octet.runner_self_check.warning_budget.reduction]
- [x] [depends:V1] I1 Replace the private `ExitCode` import with explicit owner-qualified paths without changing line count, lint policy, exceptions, suppression state, or CLI behavior. r[octet.runner_self_check.warning_budget.reduction.cli_exit]
- [x] [depends:I1] V2 Compile and test the CLI shell, then require the old baseline to report exactly one stale ID, zero new IDs, 2,669 warnings, and warning-only exit `0`. r[octet.runner_self_check.warning_budget.reduction.old_baseline]
- [x] [depends:V2] I2 Generate and review the candidate baseline as the exact old set minus one ID; update grouped documentation and permanent Nix counts without admitting additions. r[octet.runner_self_check.warning_budget.reduction.refresh]
- [ ] [depends:I2] V3 Run the refreshed positive and one-new-ID negative self-check gates, policy freshness, traceability, formatting, Clippy, Nextest, source-derived fixture checks, and the broad compatible-system flake gate. r[octet.runner_self_check.warning_budget.reduction.regression]
- [ ] [depends:V3] V4 Record bounded evidence, sync the accepted requirement, and archive deterministically with `CAIRN_ARCHIVE_DATE=2026-07-21`. r[octet.runner_self_check.warning_budget.reduction.boundary]
