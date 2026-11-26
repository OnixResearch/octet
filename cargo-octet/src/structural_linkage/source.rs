//! Bounded non-symlink source verification separating raw-file freshness from normalized item
//! identity across formatting-only vs item-body changes.

/// Computes canonical token-stream BLAKE3 digest for one parsed `syn::Item`.
pub(crate) fn normalized_item_hash(item: &syn::Item) -> String {
    let tokens = quote::quote!(#item).to_string();
    super::blake3_hex(tokens.as_bytes())
}

/// Inspects admitted workspace source files for every active record that carries a `rust_item_v1`
/// anchor.
pub(crate) fn verify_records(
    records: &[super::LinkRecord],
    source_root: &std::path::Path,
) -> (Vec<super::SourceMatchOutcome>, Vec<String>) {
    let Ok(root) = crate::cap_fs::SourceRoot::open_existing(source_root) else {
        return (Vec::new(), vec![format!(
            "source root `{}` cannot be opened as a directory",
            source_root.display()
        )]);
    };
    let mut outcomes = Vec::with_capacity(records.len());
    let mut diagnostics = Vec::with_capacity(records.len());
    for record in records {
        let Some(anchor) = &record.anchor else {
            continue;
        };
        let outcome = verify_one_record(&root, record, anchor);
        if outcome.status != super::SOURCE_STATUS_CURRENT {
            diagnostics.push(outcome.explanation.clone());
        }
        outcomes.push(outcome);
    }
    (outcomes, diagnostics)
}

fn verify_one_record(
    root: &crate::cap_fs::SourceRoot,
    record: &super::LinkRecord,
    anchor: &super::ParsedAnchor,
) -> super::SourceMatchOutcome {
    let Ok(rel) = crate::cap_fs::RootRelativePath::new(&record.source_path) else {
        return failed_outcome(
            record,
            anchor,
            super::SOURCE_STATUS_UNREADABLE,
            format!("unsafe relative source path `{}`", record.source_path),
        );
    };
    let bytes = match root.read_bounded(&rel, super::MAX_INTAKE_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            let status = if error.contains("not found") || error.contains("No such file") {
                super::SOURCE_STATUS_MISSING
            } else {
                super::SOURCE_STATUS_UNREADABLE
            };
            return failed_outcome(
                record,
                anchor,
                status,
                format!("source file `{}` cannot be read: {error}", record.source_path),
            );
        }
    };
    let actual_file_hash = super::blake3_hex(&bytes);
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return failed_outcome(
            record,
            anchor,
            super::SOURCE_STATUS_UNREADABLE,
            format!("source file `{}` is not valid UTF-8", record.source_path),
        );
    };
    let has_marker = file_contains_marker(text, &record.relation_kind, &record.requirement_id);
    let actual_norm = match locate_item_token_hash(text, &anchor.item_kind, &anchor.scoped_locator) {
        Ok(hash) => hash,
        Err(reason) => {
            return super::SourceMatchOutcome {
                locator: anchor.scoped_locator.clone(),
                expected_file_blake3: record.source_file_blake3.clone(),
                actual_file_blake3: actual_file_hash,
                expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
                actual_normalized_item_blake3: String::new(),
                status: super::SOURCE_STATUS_BODY_CHANGED.to_owned(),
                explanation: reason,
            };
        }
    };
    classify_match(record, anchor, actual_file_hash, actual_norm, has_marker)
}

fn classify_match(
    record: &super::LinkRecord,
    anchor: &super::ParsedAnchor,
    actual_file_hash: String,
    actual_norm: String,
    has_marker: bool,
) -> super::SourceMatchOutcome {
    let is_file_same = actual_file_hash == record.source_file_blake3;
    let is_item_same = actual_norm == anchor.normalized_item_blake3;
    if !has_marker {
        return super::SourceMatchOutcome {
            locator: anchor.scoped_locator.clone(),
            expected_file_blake3: record.source_file_blake3.clone(),
            actual_file_blake3: actual_file_hash,
            expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
            actual_normalized_item_blake3: actual_norm,
            status: super::SOURCE_STATUS_BODY_CHANGED.to_owned(),
            explanation: format!(
                "source `{}` is missing marker `r[{} {}]`",
                record.source_path, record.relation_kind, record.requirement_id
            ),
        };
    }
    if is_file_same && is_item_same {
        return super::SourceMatchOutcome {
            locator: anchor.scoped_locator.clone(),
            expected_file_blake3: record.source_file_blake3.clone(),
            actual_file_blake3: actual_file_hash,
            expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
            actual_normalized_item_blake3: actual_norm,
            status: super::SOURCE_STATUS_CURRENT.to_owned(),
            explanation: format!("source bytes and normalized item identity both match `{}`", anchor.scoped_locator),
        };
    }
    if !is_file_same && is_item_same {
        return super::SourceMatchOutcome {
            locator: anchor.scoped_locator.clone(),
            expected_file_blake3: record.source_file_blake3.clone(),
            actual_file_blake3: actual_file_hash.clone(),
            expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
            actual_normalized_item_blake3: actual_norm.clone(),
            status: super::SOURCE_STATUS_FORMATTING_ONLY.to_owned(),
            explanation: format!(
                "formatting-only edit on `{}` preserved normalized_item_blake3 `{actual_norm}`, but exact source_file_blake3 changed from `{}` to `{actual_file_hash}`; exact-source receipt is invalidated",
                anchor.scoped_locator, record.source_file_blake3
            ),
        };
    }
    super::SourceMatchOutcome {
        locator: anchor.scoped_locator.clone(),
        expected_file_blake3: record.source_file_blake3.clone(),
        actual_file_blake3: actual_file_hash,
        expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
        actual_normalized_item_blake3: actual_norm.clone(),
        status: super::SOURCE_STATUS_BODY_CHANGED.to_owned(),
        explanation: format!(
            "item body changed for `{}`: expected normalized_item_blake3 `{}`, actual `{actual_norm}`",
            anchor.scoped_locator, anchor.normalized_item_blake3
        ),
    }
}

fn failed_outcome(
    record: &super::LinkRecord,
    anchor: &super::ParsedAnchor,
    status: &str,
    explanation: String,
) -> super::SourceMatchOutcome {
    super::SourceMatchOutcome {
        locator: anchor.scoped_locator.clone(),
        expected_file_blake3: record.source_file_blake3.clone(),
        actual_file_blake3: String::new(),
        expected_normalized_item_blake3: anchor.normalized_item_blake3.clone(),
        actual_normalized_item_blake3: String::new(),
        status: status.to_owned(),
        explanation,
    }
}

fn file_contains_marker(text: &str, relation_kind: &str, requirement_id: &str) -> bool {
    let expected = format!("r[{relation_kind} {requirement_id}]");
    text.lines().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!"))
            && trimmed.contains(&expected)
    })
}

fn locate_item_token_hash(text: &str, item_kind: &str, scoped_locator: &str) -> Result<String, String> {
    let ast = syn::parse_file(text).map_err(|error| format!("parse Rust source: {error}"))?;
    let target_ident = extract_locator_ident(scoped_locator)
        .ok_or_else(|| format!("cannot derive Rust item identifier from `{scoped_locator}`"))?;
    let mut matches = Vec::new();
    collect_matching_items(&ast.items, item_kind, target_ident, &mut matches);
    match matches.as_slice() {
        [] => Err(format!("Rust {item_kind} `{target_ident}` (`{scoped_locator}`) is missing from parsed source")),
        [single] => Ok((*single).clone()),
        _ => Err(format!(
            "ambiguous Rust {item_kind} `{target_ident}` (`{scoped_locator}`) matched {} items",
            matches.len()
        )),
    }
}

fn extract_locator_ident(scoped_locator: &str) -> Option<&str> {
    let without_disambiguator = scoped_locator.split('#').next()?;
    let after_colon = without_disambiguator.rsplit(':').next()?;
    if after_colon.is_empty() {
        None
    } else {
        Some(after_colon)
    }
}

fn collect_matching_items(root_items: &[syn::Item], item_kind: &str, ident: &str, out: &mut Vec<String>) {
    let mut stack = vec![root_items];
    while let Some(items) = stack.pop() {
        for item in items {
            match (item_kind, item) {
                ("fn", syn::Item::Fn(func)) if func.sig.ident == ident => out.push(normalized_item_hash(item)),
                ("struct", syn::Item::Struct(st)) if st.ident == ident => out.push(normalized_item_hash(item)),
                ("enum", syn::Item::Enum(en)) if en.ident == ident => out.push(normalized_item_hash(item)),
                ("trait", syn::Item::Trait(tr)) if tr.ident == ident => out.push(normalized_item_hash(item)),
                ("const", syn::Item::Const(cn)) if cn.ident == ident => out.push(normalized_item_hash(item)),
                ("static", syn::Item::Static(st)) if st.ident == ident => out.push(normalized_item_hash(item)),
                ("type", syn::Item::Type(ty)) if ty.ident == ident => out.push(normalized_item_hash(item)),
                (_, syn::Item::Mod(module)) => {
                    if let Some((_, inner)) = &module.content {
                        stack.push(inner);
                    }
                }
                _ => {}
            }
        }
    }
}
