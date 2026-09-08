# Tasks

- [x] [serial] [task:baseline] Record the existing compiler fixture and the expanded-fixture panic. r[octet.compiler_architecture_ir.body_owner_queries]
- [x] [serial] [task:classification] [after:baseline] Reuse the narrow body-kind and private-visibility corrections. r[octet.compiler_architecture_ir.body_owner_queries]
- [x] [serial] [task:controls] [after:classification] Verify anonymous, inline, function, closure, and missing-shard controls. r[octet.compiler_architecture_ir.body_owner_queries]
- [x] [serial] [task:verification] [after:controls] Run producer checks and the Seaglass diagnostic replay. r[octet.compiler_architecture_ir.body_owner_queries]
- [ ] [serial] [task:close] [after:verification] Preserve evidence, sync the accepted requirement, and archive the verified change. r[octet.compiler_architecture_ir.body_owner_queries]

## Verification Coverage

| Scenario | Control |
| --- | --- |
| Anonymous constant contains a resolved call | Real Cargo fixture and signature-guard mutation |
| Inline constant contains a resolved call | Real Cargo fixture and visibility-guard mutation |
| Function and closure evidence remains distinct | Item/signature/call assertions in the real fixture |
| A compiler observation is missing | Existing shard-removal rejection |
