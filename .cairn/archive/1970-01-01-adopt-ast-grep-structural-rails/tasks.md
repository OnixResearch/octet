## Tasks

- [x] [serial] r[octet.ast_grep_structural_rails.profile] Define the ast-grep structural rail profile, rule-pack layout, severity posture, and promotion lanes.
- [x] [depends:ast-grep-profile] r[octet.ast_grep_structural_rails.rule_tests] Add positive and negative ast-grep rule tests for each promoted rule.
- [x] [depends:ast-grep-profile] r[octet.ast_grep_structural_rails.bundle_identity] Bind ast-grep version, scan scope, and rule bundle BLAKE3 identity into Octet evidence artifacts.
- [x] [depends:ast-grep-bundle-identity] r[octet.ast_grep_structural_rails.findings] Preserve JSON/SARIF findings with spans, metavariables, severity, diagnostics, and explicit non-claims.
- [x] [depends:ast-grep-rule-tests] r[octet.ast_grep_structural_rails.codemods] Keep ast-grep codemods dry-run-first or interactive until scoped validation proves the migration.
- [x] [depends:ast-grep-findings] r[octet.ast_grep_structural_rails.valence_export] Export only candidate source-shape evidence to Valence with recorded-only or boundary roles.
- [x] [depends:ast-grep-valence-export] r[octet.ast_grep_structural_rails.validation] Run ast-grep tests, focused scans, artifact verification, Cairn gates, and the smallest relevant Octet checks.
