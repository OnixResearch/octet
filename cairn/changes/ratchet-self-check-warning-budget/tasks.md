# Tasks

- [x] [serial] V1 Confirm existing baseline positive/rename/incomplete-run tests pass; capture the 2,671-warning inventory and the structured-output absolute-path failure. r[octet.runner_self_check.inventory]
- [ ] [depends:V1] I1 Normalize in-workspace Cargo diagnostic paths through a pure repository-relative core and retain fail-closed outside-root/traversal rejection. r[octet.runner_self_check.inventory.paths]
- [ ] [depends:I1] I2 Replace the single `negated_predicate` finding with behavior-preserving positive naming and no lint suppression. r[octet.runner_self_check.warning_budget.cleanup]
- [ ] [depends:I2] I3 Generate the reviewed post-cleanup baseline and grouped inventory documentation. r[octet.runner_self_check.warning_budget.baseline]
- [ ] [depends:I3] I4 Extend the hermetic self-check gate with exact positive baseline assertions and a one-new-warning negative fixture. r[octet.runner_self_check.warning_budget.regression]
- [ ] [depends:I4] V2 Run focused path, baseline, cleanup, canonical structured-output, and Nix self-check validations. r[octet.runner_self_check.inventory.paths] r[octet.runner_self_check.warning_budget.regression]
- [ ] [depends:V2] V3 Run policy freshness, traceability, formatting, repository Clippy, Nextest, and the broad compatible-system flake gate. r[octet.runner_self_check.warning_budget]
- [ ] [depends:V3] V4 Record bounded evidence and archive deterministically with `CAIRN_ARCHIVE_DATE=2026-07-21`. r[octet.runner_self_check.warning_budget.boundary]
