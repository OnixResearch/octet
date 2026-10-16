## Phase 1: Provider implementation

- [x] [serial] Declare all 15 exact Molten domain aliases/fields and explicit wire compatibility using the central Nickel catalog. r[octet.type_driven_domain_modeling.molten_adoption.catalog]
- [x] [serial] Admit checked marker-generic/private-fallible policy in Nickel and Rust while preserving marker-only and rejecting private-infallible; add matching admission fixtures. r[octet.type_driven_domain_modeling.molten_adoption.catalog]
- [x] [serial] Enforce local macro primitive aliases at the configured core callsite without heuristic macro noise or external-macro expansion. r[octet.type_driven_domain_modeling.molten_adoption.macros]
- [x] [serial] Register compiler UI success/rejection fixtures and config-family coverage, regenerate source-owned catalog JSON, and document scope/non-claims. r[octet.type_driven_domain_modeling.molten_adoption.compatibility]

## Phase 2: Integration-owner verification

- [x] [serial] Generate and review only new Molten UI diagnostics, then replay the UI rail without blessing; prove all 15 direct/macro raw aliases and raw core fields fail while nominal generic and explicit compatibility pass. r[octet.type_driven_domain_modeling.molten_adoption.macros]
- [x] [serial] Run nominal catalog freshness and focused library/metadata checks; inspect exact fixture diagnostics rather than accepting a generic suite pass. r[octet.type_driven_domain_modeling.molten_adoption.catalog]
- [x] [serial] Run real Molten clean and alias-to-String mutation smokes through the integrated Octet provider; preserve the macro and original source after the mutation. r[octet.type_driven_domain_modeling.molten_adoption.macros]
- [x] [serial] Obtain required review/lifecycle gates and approved immutable upstream provider availability before changing the consumer pin or claiming sync/archive readiness. r[octet.type_driven_domain_modeling.molten_adoption.compatibility]

## Verification Coverage

| Scenario | Required observation | Owner |
| --- | --- | --- |
| Exact Molten catalog | Source-owned export equals checked-in catalog; all 15 rows retain exact core and compatibility selectors | Integration owner |
| Local macro String regression | Clean real macro aliases admitted; resolved String mutation denied at configured user/core invocation | Integration owner |
| Direct core String regression | Direct/optional registered core fields rejected for all declared domains | Integration owner |
| Nominal generic compatibility | Typed macro aliases/core fields and explicit raw wire shapes admitted without nominal diagnostics | Integration owner |
| Scope and non-claims | No borrowed/generic-internal/external-macro/heuristic-only enforcement or publication claim inferred | Integration owner |
