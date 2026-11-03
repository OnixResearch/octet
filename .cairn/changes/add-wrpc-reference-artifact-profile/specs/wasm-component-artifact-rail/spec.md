# wasm-component-artifact-rail delta

## ADDED Requirements

### Requirement: wrpc reference artifacts have an exact profile

r[octet.wasm_component.wrpc_reference_profile] Octet MUST define a recorded-only `wrpc-reference` profile that admits exactly the normalized wrpc reference hello component: the exact server host import `component@3::import-func-hello::Func(0)` and client handler instance import `component::wrpc-examples:hello/handler::Instance(0)`, the bulk-memory and padded-indirect-call proposals, two import slots and two tables, and no other imports, while the portable baseline remains closed against the same artifact.

#### Scenario: Normalized wrpc component is admitted
- GIVEN the normalized wrpc reference server and client components and the `wrpc-reference` profile
- WHEN artifact evidence is collected
- THEN Octet MUST return no findings with recorded-only verification and MUST keep the baseline profile's denial available as negative evidence.

#### Scenario: Another import or proposal is refused
- GIVEN a component with a different import identity or an unlisted proposal still evaluated under `wrpc-reference`
- WHEN artifact evidence is collected
- THEN Octet MUST fail closed with ordered `disallowed-import` or `disallowed-proposal` findings and no runtime claim.
