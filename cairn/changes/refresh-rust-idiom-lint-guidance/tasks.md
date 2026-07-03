## Contract

- [ ] [serial] Specify guidance-freshness behavior for narrative docs that name shipped lints or enforcement tiers. r[octet.lint_guidance_freshness.enforcement_tiers]
- [ ] [serial] Specify that shipped lints MUST NOT be described as future-only candidates. r[octet.lint_guidance_freshness.shipped_lints]

## Implementation

- [ ] [serial] Update `docs/rust-idiom-guidance.md` so `Borrowed argument type` references the shipped `borrowed_argument_types` warning lint. r[octet.lint_guidance_freshness.shipped_lints]
- [ ] [serial] Add a small deterministic docs freshness helper or test fixture for guidance rows naming live lint identities. r[octet.lint_guidance_freshness.enforcement_tiers]

## Verification

- [ ] [serial] Add a positive test for the current guidance row and a negative stale-wording fixture. r[octet.lint_guidance_freshness.shipped_lints]
- [ ] [serial] Run focused docs freshness tests, `cargo test --lib`, and Cairn validation/gates. r[octet.lint_guidance_freshness.enforcement_tiers]
