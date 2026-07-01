## Contract

- [ ] [serial] Specify canonical and compatibility namespace behavior for source attrs, group attrs, emitted metadata, and config keys. r[octet.namespace_compatibility.canonical]
- [ ] [serial] Specify compatibility alias retention and deprecation boundaries. r[octet.namespace_compatibility.aliases]
- [ ] [serial] Specify escape-hatch preservation for lints with custom allow semantics. r[octet.namespace_compatibility.escape_hatches]

## Implementation

- [ ] [serial] Align lint declarations, group constants, runner metadata, docs, and templates with the selected namespace decision. r[octet.namespace_compatibility.canonical]
- [ ] [serial] Add or update compatibility alias registration for supported legacy namespaces. r[octet.namespace_compatibility.aliases]
- [ ] [serial] Preserve no-todo/no-panic and other custom allow semantics during any tool-lint namespace migration. r[octet.namespace_compatibility.escape_hatches]

## Verification

- [ ] [serial] Add positive UI fixtures for canonical lint attrs and domain group attrs. r[octet.namespace_compatibility.canonical]
- [ ] [serial] Add compatibility UI fixtures for supported legacy attrs and negative fixtures for unsupported spellings. r[octet.namespace_compatibility.aliases]
- [ ] [serial] Add targeted tests proving custom escape-hatch justification logic is not bypassed by namespace migration, then run UI and Cairn gates. r[octet.namespace_compatibility.escape_hatches]
