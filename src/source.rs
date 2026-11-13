//! Compiler-owned source provenance for file-level lint passes.

// r[impl octet.lint_signal_quality.source_ownership]
pub(crate) fn is_local(file: &rustc_span::SourceFile) -> bool {
    !file.is_imported()
}

/// Visits each local file represented by parsed Rust syntax, once. The source
/// map also owns `include_str!` and `include_bytes!` payloads, so locality alone
/// does not establish that a file is source code.
pub(crate) fn for_each_parsed_file(
    source_map: &rustc_span::source_map::SourceMap,
    krate: &rustc_ast::Crate,
    visit: impl FnMut(&rustc_span::SourceFile),
) {
    struct Files<'a, F> {
        source_map: &'a rustc_span::source_map::SourceMap,
        seen: std::collections::BTreeSet<rustc_span::BytePos>,
        last_file: Option<(rustc_span::BytePos, rustc_span::BytePos)>,
        visit: F,
    }

    impl<F: FnMut(&rustc_span::SourceFile)> Files<'_, F> {
        fn record(&mut self, span: rustc_span::Span) {
            let pos = span.lo();
            if self.last_file.is_some_and(|(start, end)| start <= pos && pos <= end) {
                return;
            }
            let file = self.source_map.lookup_source_file(pos);
            self.last_file = Some((file.start_pos, file.end_position()));
            if is_local(&file) && self.seen.insert(file.start_pos) {
                (self.visit)(&file);
            }
        }
    }

    impl<'ast, F: FnMut(&rustc_span::SourceFile)> rustc_ast::visit::Visitor<'ast> for Files<'_, F> {
        fn visit_item(&mut self, item: &'ast rustc_ast::Item) {
            // Item spans retain parsed `include!` input, while an out-of-line
            // module's own item span is its declaration in the parent file.
            self.record(item.span);
            if let rustc_ast::ItemKind::Mod(_, _, rustc_ast::ModKind::Loaded(_, _, spans)) = &item.kind {
                self.record(spans.inner_span);
            }
            rustc_ast::visit::walk_item(self, item);
        }

        fn visit_expr(&mut self, expr: &'ast rustc_ast::Expr) {
            // Parsed expression-position `include!` input has source spans.
            // In contrast, `include_str!` manufactures an expanded literal
            // whose span points into the data file, not parsed Rust syntax.
            if !expr.span.from_expansion() {
                self.record(expr.span);
            }
            rustc_ast::visit::walk_expr(self, expr);
        }
    }

    let mut files = Files {
        source_map,
        seen: std::collections::BTreeSet::new(),
        last_file: None,
        visit,
    };
    files.record(krate.spans.inner_span);
    rustc_ast::visit::walk_crate(&mut files, krate);
}

pub(crate) fn source_file_path(file: &rustc_span::SourceFile) -> std::path::PathBuf {
    std::path::PathBuf::from(file.name.prefer_local_unconditionally().to_string_lossy().into_owned())
}

pub(crate) fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Cargo runs rustc from the workspace root. Files outside it, such as build
/// script output under a custom `CARGO_TARGET_DIR`, are not workspace source.
pub(crate) fn workspace_root() -> std::path::PathBuf {
    std::env::current_dir().map(|path| normalize_path(&path)).unwrap_or_default()
}

pub(crate) fn is_workspace_source_path(path: &std::path::Path, workspace_root: &std::path::Path) -> bool {
    if path.is_absolute() {
        return path.starts_with(workspace_root);
    }

    workspace_root.join(path).is_file()
}

/// Normalizes one configured workspace-relative source scope the way
/// `cargo octet` does: trims whitespace, strips leading `./` and trailing `/`.
pub(crate) fn normalize_scope(raw: &str) -> String {
    let mut scope = raw.trim();
    while let Some(rest) = scope.strip_prefix("./") {
        scope = rest.trim_start_matches('/');
    }
    scope.trim_end_matches('/').to_owned()
}

/// Whether a workspace-relative source path is one of `scopes` or lies below
/// one. Matching is by whole path components, so `crates/core` covers
/// `crates/core/src/lib.rs` but not `crates/core-shell/src/lib.rs`.
pub(crate) fn path_in_scopes(path: &std::path::Path, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| !scope.is_empty() && path.starts_with(scope))
}

/// Where a normalized source scope points once resolved on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScopeResolution {
    /// The scope's canonical path, relative to the canonical workspace root.
    Inside(String),
    /// The workspace root itself.
    Root,
    /// A path outside the workspace root.
    Outside,
    /// No existing file or directory.
    Missing,
}

/// Resolves a normalized scope against the canonical workspace root,
/// following symlinks and `..` the way `cargo octet` validates scopes.
pub(crate) fn resolve_scope(canonical_root: &std::path::Path, scope: &str) -> ScopeResolution {
    let Ok(resolved) = canonical_root.join(scope).canonicalize() else {
        return ScopeResolution::Missing;
    };
    match resolved.strip_prefix(canonical_root) {
        Ok(relative) if relative.as_os_str().is_empty() => ScopeResolution::Root,
        Ok(relative) => ScopeResolution::Inside(relative.to_string_lossy().into_owned()),
        Err(_) => ScopeResolution::Outside,
    }
}

/// Whether `span` comes from a file inside one of the normalized `scopes`.
/// Cargo runs rustc from the workspace root, so an absolute file name is made
/// relative to rustc's working directory before matching.
pub(crate) fn span_in_scopes(sess: &rustc_session::Session, span: rustc_span::Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let file = sess.source_map().lookup_source_file(span.lo());
    let path = normalize_path(&source_file_path(&file));
    let relative = sess
        .opts
        .working_dir
        .local_path()
        .map(normalize_path)
        .and_then(|root| path.strip_prefix(root).ok().map(std::path::Path::to_path_buf));
    path_in_scopes(relative.as_deref().unwrap_or(&path), scopes)
}

/// Whether the source text at `ident`'s span spells `ident`, so the user wrote
/// that token. Attribute proc macros such as `#[tokio::main]` build code with
/// `quote_spanned!` at a user token: the generated `.expect` then has no
/// expansion context, yet its span covers other user text, such as a `;`.
pub(crate) fn ident_is_spelled_at_its_span(sess: &rustc_session::Session, ident: rustc_span::Ident) -> bool {
    sess.source_map().span_to_snippet(ident.span).is_ok_and(|snippet| {
        let spelled = snippet.strip_prefix("r#").unwrap_or(&snippet);
        spelled == ident.as_str()
    })
}

/// The comment lines that document an item from above, nearest first: comments
/// between its outer attributes and the contiguous comment lines above the
/// earliest attribute. `prefix` is the source text before the item. A comment
/// line must touch the attribute, comment, or item below it; attributes may be
/// separated by blank lines.
///
/// Attributes are recognised from source text because HIR cannot report the
/// span of every parsed attribute, such as `#[inline]` or `#[repr(C)]`.
pub(crate) fn leading_comment_lines(prefix: &str) -> impl Iterator<Item = &str> {
    let mut rest = prefix;
    std::iter::from_fn(move || {
        loop {
            let code = rest.trim_end();
            let line_start = code.rfind('\n').map_or(0, |index| index + 1);
            let last_line = code[line_start..].trim_start();
            if last_line.starts_with("//") {
                let has_blank_line_below = rest[code.len()..].matches('\n').nth(1).is_some();
                if has_blank_line_below {
                    return None;
                }
                rest = &code[..line_start];
                return Some(last_line);
            }
            rest = &code[..outer_attribute_start(code)?];
        }
    })
}

/// Byte offset of the `#[` that opens the outer attribute ending `code`.
fn outer_attribute_start(code: &str) -> Option<usize> {
    if !code.ends_with(']') {
        return None;
    }
    let mut depth = 0_usize;
    for (index, byte) in code.bytes().enumerate().rev() {
        match byte {
            b']' => depth += 1,
            b'[' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    let hash = index.checked_sub(1)?;
                    return (code.as_bytes()[hash] == b'#').then_some(hash);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether any of [`leading_comment_lines`] for the item at `item_span`
/// satisfies `predicate`.
pub(crate) fn any_leading_comment_line(
    source_map: &rustc_span::source_map::SourceMap,
    item_span: rustc_span::Span,
    predicate: impl FnMut(&str) -> bool,
) -> bool {
    let file = source_map.lookup_source_file(item_span.lo());
    let offset = rustc_span::Pos::to_usize(&(item_span.lo() - file.start_pos));
    file.src
        .as_deref()
        .and_then(|src| src.get(..offset))
        .is_some_and(|prefix| leading_comment_lines(prefix).any(predicate))
}

#[cfg(test)]
mod tests;
