## Implementation

- [x] [serial] Add conservative primitive HIR body eligibility and free-function guard without changing other classifier policies. r[octet.purity_effect_lints.missing_const_fn.hir_subset]
- [x] [serial] Extend the existing const fixture with non-const string parsing, calls, overloaded operators, and trait-method negative cases while retaining the arithmetic positive case. r[octet.purity_effect_lints.missing_const_fn.hir_subset]
- [x] [serial] Update lint reference and changelog; preserve publication hold and other six consumer obligations. r[octet.purity_effect_lints.missing_const_fn.hir_subset]

## Verification

- [x] [serial] Exercise the complete existing const UI fixture in the pinned shell and compile the fixture directly; report observed results or exact environment blocker. r[octet.purity_effect_lints.missing_const_fn.hir_subset]
- [x] [serial] Hand parent exact focused replay commands, native gate receipts, and construction-policy decision blocker. r[octet.purity_effect_lints.missing_const_fn.hir_subset]

## Observed verification boundary

Actual full owning library and existing owning driver source builds passed offline in separate fresh targets in the retained exact owning Nix environment. The complete existing const fixture executed against that new library/driver and matched its unchanged snapshot exactly: one arithmetic advisory and no negative-case findings. Main authorized a transient HTTPS source replacement to the same verified exact-revision retained vendor bytes/checksums; no package/version/revision patch. See `evidence/verification.md`, `evidence/owning-proof-commands.txt`, and the three owning execution receipts for exact scope and non-claims.

The current origin/main base was re-verified in `evidence/2026-11-09-owning-verification/`: the owning library compiles and the full maintained UI rail passes with one arithmetic advisory and no negative-case findings. The external HWR deny-all replay was not rerun against its pinned consumer checkout. Full-suite, downstream, archive and publication gates remain parent-owned. Final native gate receipts are structural admission only.
