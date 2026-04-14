## ADDED Requirements

### Requirement: Per-Lint Configuration Support

The project MUST support disabling individual lints and overriding lint
severity through the consumer-owned `dylint.toml`.

The configuration MUST include:

- a `disabled_lints` list that prevents named lints from registering
- a `lint_levels` table that maps lint names to `allow`, `warn`, or `deny`
- backward compatibility with existing `dylint.toml` files that lack the
  new fields
- a warning (not error) for unknown lint names in config

#### Scenario: Disabling a noisy lint

- GIVEN `disabled_lints = ["no_unwrap"]` in `dylint.toml`
- WHEN the lint suite runs
- THEN `no_unwrap` produces no diagnostics and no pass overhead

#### Scenario: Old config without new fields

- GIVEN a `dylint.toml` with only `function_length_max = 100`
- WHEN the lint suite loads config
- THEN all lints are enabled at their default levels
