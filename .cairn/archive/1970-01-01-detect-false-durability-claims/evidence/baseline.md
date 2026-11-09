# Recorded baseline

Recorded at merge time on the reconciled main base:

- Lint inventory: 75 lints, including the three new durability lints
  (`discarded_sync_result`, `flush_only_durability`, `retry_sync_after_error`).
- UI fixture set: 191 fixtures, including the six durability fixtures.
- Existing safety lint family and standards registry were unchanged except for
  the new durability config/receipt families and the
  `tigerstyle.resilience.durable-persistence-claims` policy.
- Storage-fault assurance profile lives under
  `standards/policies/assurance-profiles/storage-fault/` with current, stale,
  missing-case, producer-failure, and overclaim evidence fixtures.
- The pre-change baseline for these new lints did not exist; this record is the
  first admitted state and the comparison point for later changes.
