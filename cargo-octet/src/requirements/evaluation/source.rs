pub(in crate::requirements) fn snapshot_hash(
    contract: &super::super::contract::Definition,
    declared: &[super::super::contract::SourceBinding],
    source_root: &std::path::Path,
    diagnostics: &mut Vec<String>,
) -> String {
    let mut bindings = declared
        .iter()
        .map(|binding| (binding.locator.as_str(), std::borrow::Cow::Borrowed(binding.blake3.as_str())))
        .collect::<std::collections::BTreeMap<_, _>>();
    for clause in &contract.clauses {
        for binding in &clause.source_bindings {
            let path = super::super::resolve_path(source_root, &binding.locator);
            match std::fs::read(&path) {
                Ok(bytes) => {
                    let actual = super::super::hash_bytes(&bytes);
                    if actual != binding.blake3 {
                        diagnostics.push(format!(
                            "clause `{}` source `{}` changed: expected {}, actual {actual}",
                            clause.clause_id, binding.locator, binding.blake3
                        ));
                    }
                    if let Some(declared_hash) = bindings.get(binding.locator.as_str()) {
                        if declared_hash.as_ref() != actual {
                            diagnostics.push(format!(
                                "evidence source_snapshot `{}` does not match the checked source bytes",
                                binding.locator
                            ));
                        }
                    } else {
                        bindings.entry(binding.locator.as_str()).or_insert(std::borrow::Cow::Owned(actual));
                    }
                }
                Err(error) => diagnostics.push(format!(
                    "clause `{}` source `{}` is missing or unreadable: {error}",
                    clause.clause_id, binding.locator
                )),
            }
        }
    }
    let canonical = serde_json::to_vec(&bindings).unwrap_or_default();
    super::super::hash_bytes(&canonical)
}
