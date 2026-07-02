## Contract

- [ ] [serial] Define the card-oriented public story surface and required card inventory. r[octet.adoption_story.public_story_cards]
- [ ] [serial] Define terminal-first quickstart output requirements. r[octet.adoption_story.quickstart_path]
- [ ] [serial] Define before/after example requirements for high-confidence lint families. r[octet.adoption_story.before_after_examples]
- [ ] [serial] Define FAQ topic and boundary requirements. r[octet.adoption_story.faq_boundaries]
- [ ] [serial] Define Salt-inspiration boundary wording. r[octet.adoption_story.salt_inspiration_boundaries]

## Implementation

- [ ] [serial] Update the canonical adoption-story docs surface with checked-path, how-it-works, built-with, examples, quickstart, and FAQ sections. r[octet.adoption_story.public_story_cards]
- [ ] [parallel] Add before/after examples for selected existing lint families without solver-counterexample wording. r[octet.adoption_story.before_after_examples]
- [ ] [parallel] Tighten the first-run quickstart to include expected artifact outputs and artifact verification. r[octet.adoption_story.quickstart_path]
- [ ] [parallel] Add the compact FAQ answers for Clippy comparison, proof scope, Dylint/nightly, data locality, starter noise, and `cargo octet prove`. r[octet.adoption_story.faq_boundaries]
- [ ] [serial] Link the story surface from README and relevant reference docs without duplicating long-form content. r[octet.adoption_story.public_story_cards]
- [ ] [serial] Add deterministic docs checks or fixtures for card inventory, examples, quickstart, FAQ topics, and forbidden Salt-style overclaims. r[octet.adoption_story.salt_inspiration_boundaries]

## Verification

- [ ] [serial] Run the positive story-card inventory check. r[octet.adoption_story.public_story_cards]
- [ ] [serial] Run the positive before/after example inventory check. r[octet.adoption_story.before_after_examples]
- [ ] [serial] Run the positive quickstart command/output inventory check. r[octet.adoption_story.quickstart_path]
- [ ] [serial] Run the positive FAQ topic inventory check. r[octet.adoption_story.faq_boundaries]
- [ ] [serial] Run the negative forbidden-claim wording check. r[octet.adoption_story.salt_inspiration_boundaries]
- [ ] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.adoption_story.public_story_cards]
