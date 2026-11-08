# Policy package customs gate

The local customs gate checks policy/config/template/proof-rail/generated-contract packages before promotion.

A package manifest uses `standards/policy-packages/contracts.ncl` and declares package ID, kind, owner, source paths, generated artifacts, registry family IDs, generation and validation commands, fixtures, provenance, license notes, non-claims, and manual review actions.

`octet-standards customs-gate` validates the manifest against:

- registry coverage in `standards/policies/registry.ncl`
- positive and negative fixture coverage
- generated artifact freshness through a BLAKE3 manifest
- checked-in source and fixture inventory
- no remote fetches during validation
- no raw secret material in package sources, positive and negative fixtures, or generated artifacts; a `secret_exclusion_rationale` excuses secret-shaped text only for `non-candidate` packages, so a package whose fixtures need secret-shaped text cannot be promoted

The report is deterministic and local. Passing customs does not trust package authors, prove package safety, prove vulnerability freedom, or replace repository review.
