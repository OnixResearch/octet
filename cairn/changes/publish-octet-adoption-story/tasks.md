## Contract

- [ ] [serial] Define the canonical adoption-story promise and checked-path stage inventory. r[octet.adoption_story.positioning]
- [ ] [serial] Define required "Built with Octet" case-study fields and evidence boundaries. r[octet.adoption_story.case_studies]
- [ ] [serial] Define required methodology fields for public performance, cache, CI-latency, suppression, or rollout-cost claims. r[octet.adoption_story.methodology]
- [ ] [serial] Define forbidden external-comparison and overclaim wording for adoption docs. r[octet.adoption_story.claim_boundaries]

## Implementation

- [ ] [serial] Add or update the canonical adoption-story docs surface and link it from `README.md`. r[octet.adoption_story.positioning]
- [ ] [serial] Add a case-study template and seed it with existing archived downstream evidence where the required fields can be satisfied. r[octet.adoption_story.case_studies]
- [ ] [serial] If Aspen phase-1 evidence is used, cite `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` as the canonical evidence source. r[octet.adoption_story.case_studies]
- [ ] [serial] Add a methodology section or fixture for public adoption/performance metrics before publishing any metrics. r[octet.adoption_story.methodology]
- [ ] [serial] Add deterministic documentation checks for required fields and forbidden unsupported claims. r[octet.adoption_story.claim_boundaries]

## Verification

- [ ] [serial] Run a positive documentation inventory proving the adoption story contains the promise, checked-path stages, case-study fields, and methodology fields. r[octet.adoption_story.positioning]
- [ ] [serial] Run a positive evidence-reference check proving every published case study names its command/config/scope/artifact boundary. r[octet.adoption_story.case_studies]
- [ ] [serial] Run a negative wording check rejecting mathematical-certainty, Rust-replacement, whole-program-correctness, certification, and verified-runtime overclaims. r[octet.adoption_story.claim_boundaries]
- [ ] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.adoption_story.methodology]
