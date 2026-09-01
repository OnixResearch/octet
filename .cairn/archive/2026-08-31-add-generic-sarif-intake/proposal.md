## Why

Octet emits SARIF but does not provide a generic intake boundary for SARIF from external Rust analyzers.

A local import rail can preserve CodeQL, Semgrep, and other producer findings without bundling those engines or requiring hosted services.

## What Changes

- Add an opt-in `sarif-import` evidence rail for a bounded SARIF `2.1.0` subset.
- Add a typed Nickel producer profile for accepted tool identities, scope rules, severity mappings, limits, and non-claims.
- Preserve exact input bytes beside deterministic normalized findings and provenance.
- Keep source checker severity separate from Octet gate policy.

## Impact

- **Specs**: `external-lint-reporting`
- **Files**: SARIF profile Nickel, generated runtime data, parser core, import shell, receipt DTOs, fixtures, artifact validation, and consumer documentation
- **Testing**: clean and finding reports, several producers, malformed data, unsafe locations, unsupported versions, stale identities, limit failures, and hosted-service independence

## Non-goals

- Do not run CodeQL, Semgrep, SonarQube, or a hosted scanning service.
- Do not treat external findings as Octet lint identities.
- Do not infer missing rule, producer, scope, severity, or source-location data.
- Do not claim complete vulnerability detection or whole-program correctness.

## Completion Evidence

The change is complete when supported local SARIF imports produce deterministic receipts and every malformed, unsafe, stale, unsupported, or over-limit fixture fails closed.
