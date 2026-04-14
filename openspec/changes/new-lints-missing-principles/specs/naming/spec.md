## ADDED Requirements

### Requirement: Negated Predicate Detection

The lint MUST warn on function names and boolean variable names containing
negative prefixes: `is_not_`, `has_no_`, `lacks_`, `not_`, `cant_`,
`doesnt_`, `isnt_`, `wasnt_`, `wont_`.

The lint MUST suggest a positive reformulation.

The lint MUST NOT fire on private helper functions named `not()` (boolean
negation utility).

#### Scenario: Negated function name

- GIVEN `fn is_not_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it warns: "negated predicate name — prefer positive form like `is_ready`"

#### Scenario: Positive function name

- GIVEN `fn is_ready(state: &State) -> bool`
- WHEN the lint runs
- THEN it does NOT fire
