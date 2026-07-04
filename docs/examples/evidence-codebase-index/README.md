# Evidence codebase index example

Build a deterministic local index over existing artifacts:

```sh
cargo octet evidence-index codebase build --artifact-dir artifacts/octet --output artifacts/octet/evidence-codebase-index.json
```

Query by a stable identity:

```sh
cargo octet evidence-index codebase query --index artifacts/octet/evidence-codebase-index.json --identity 'r[octet.evidence_codebase_index.query.lookup]'
```

Render a static review page from the same model:

```sh
cargo octet evidence-index codebase html --index artifacts/octet/evidence-codebase-index.json --output artifacts/octet/evidence-codebase-index.html
```

The generated index is review/provenance infrastructure only: it proves artifact identity and linkage, not correctness or semantic equivalence.
