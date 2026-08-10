## Phase 1: Restore observable validation

- [x] [serial] Add bounded positive and negative failure diagnostics to the packaged self-check. [depends:octet.runner_self_check.failure_diagnostics]
- [x] [serial] Restore README lint total, namespace freshness, and checked-path documentation contracts. [depends:octet.lint_catalog_freshness.docs_match_inventory]

## Phase 2: Repair warning drift

- [x] [serial] Remove all unapproved candidate self-check identities without lint suppression or weaker limits. [depends:octet.runner_self_check.warning_budget.semantic_refresh]
- [x] [serial] Prove the final baseline transition contains only reviewed semantic rekeys, removals, and non-growing occurrence groups. [depends:octet.runner_self_check.warning_budget.semantic_refresh]

## Phase 3: Validate and archive

- [x] [serial] Run focused positive and negative tests, lifecycle gates, strict checks, and full Nix validation. [depends:octet.runner_self_check.failure_diagnostics]
- [x] [serial] Record bounded evidence, sync accepted requirements, and archive the completed change. [depends:octet.runner_self_check.warning_budget.semantic_refresh]
