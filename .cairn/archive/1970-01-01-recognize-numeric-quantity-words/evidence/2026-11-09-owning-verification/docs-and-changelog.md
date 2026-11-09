# Documentation and claim boundary

The current `CHANGELOG.md` already records that `numeric_units` matches whole
words, so `proportion`, `report_lines`, and `transport` no longer trigger it,
and that `port` no longer asks for a unit suffix. The delta spec was corrected
to match that reviewed behavior: a `port` component names an identifier, not a
unit-bearing quantity. `request_timeout` and `buffer_size` remain positive
quantity controls.

The prior HWR admission evidence (`evidence/admission-before.json`,
`evidence/owning-gates.json`, and `evidence/final-owning-gates.json`) is
retained as historical source-bound evidence. This session did not rerun the
external HWR deny-all replay against its pinned consumer checkout; it re-ran
the owning library, unit, and full UI rails on the current Octet base. The
change does not modify lint levels, policy, severity, or any other lint, so
downstream policy obligations are unchanged. No runtime, publication, or
release authority is claimed.
