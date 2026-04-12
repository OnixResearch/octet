# Naming Specification

## Purpose

Tiger Style naming requirements for lint enforcement and agent-facing skill
coverage: boolean prefixes, acronym capitalization, unit suffixes, and
names that make obligations explicit.

## Requirements

### Requirement: Behavioral Naming Coverage In Skill

The Tiger Style skill MUST include the omitted naming guidance from the
reference.

The skill MUST cover:

- behavioral naming that signals ownership or lifecycle obligations
- avoiding name overloading across unrelated concepts
- using option/config structs when multiple parameters share a type
- documenting the "why" rather than restating the "what"
- proper comment style as complete sentences

#### Scenario: Ambiguous same-type parameters

- GIVEN a function with multiple `u64` parameters
- WHEN an agent follows the skill
- THEN it is prompted to prefer a named options struct over positional ambiguity

#### Scenario: Ownership-signaling names

- GIVEN a parameter that transfers ownership
- WHEN an agent names it under Tiger Style
- THEN the name can signal the ownership contract, such as `owned_buffer`
