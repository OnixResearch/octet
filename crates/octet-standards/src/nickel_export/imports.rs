//! Static Nickel import observation for export dependency closures.
//!
//! The shell supplies exact source and dependency texts. This pure scanner
//! reports every `import` target reachable from the source through those
//! texts, so the standalone core can compare the observed closure with the
//! declared dependencies. Comments and string contents are skipped, while
//! string interpolations are scanned as code. Quoted targets resolve against
//! the importing file's directory; package imports keep their bare name.

use std::collections::BTreeSet;

use super::NamedText;

const IMPORT_KEYWORD: &[u8] = b"import";
/// Extensions Nickel imports as data rather than Nickel source.
const DATA_EXTENSIONS: &[&str] = &[".json", ".yaml", ".yml", ".toml", ".txt"];

/// Lexical scanner state for one nesting level.
enum Mode {
    /// Nickel code; `braces` counts open `{` inside an interpolation.
    Code { braces: usize },
    /// Standard `"…"` string.
    Standard,
    /// Multiline or symbolic string delimited by `percents` `%` characters.
    Multiline { percents: usize },
}

/// Return the sorted, deduplicated import closure observed from `source_path`.
///
/// Imports are followed through dependency texts whose normalized path matches
/// a resolved Nickel import. The source itself is never reported.
pub(super) fn observed_import_closure(
    source_path: &str,
    source_text: &str,
    dependencies: &[NamedText<'_>],
) -> Vec<String> {
    let source = normalize_path(source_path);
    let mut observed = BTreeSet::new();
    let mut pending = vec![(source.clone(), source_text)];
    while let Some((importer, text)) = pending.pop() {
        for target in import_targets(text) {
            let resolved = match target {
                ImportTarget::Path(path) => resolve_import(&importer, &path),
                ImportTarget::Package(name) => name,
            };
            if resolved == source || !observed.insert(resolved.clone()) {
                continue;
            }
            if let Some(dependency) = dependencies.iter().find(|dependency| normalize_path(dependency.path) == resolved)
                && is_nickel_source(&resolved)
            {
                pending.push((resolved, dependency.text));
            }
        }
    }
    observed.into_iter().collect()
}

/// One `import` target as written in source.
#[derive(Debug, Eq, PartialEq)]
enum ImportTarget {
    /// Quoted file path.
    Path(String),
    /// Bare package identifier.
    Package(String),
}

fn import_targets(text: &str) -> Vec<ImportTarget> {
    let bytes = text.as_bytes();
    let mut targets = Vec::new();
    let mut stack = vec![Mode::Code { braces: 0 }];
    let mut pos = 0;
    while pos < bytes.len() {
        let nested = stack.len() > 1;
        pos = match stack.last_mut() {
            Some(Mode::Code { braces }) => match bytes[pos] {
                b'#' => skip_line(bytes, pos),
                b'"' => {
                    stack.push(Mode::Standard);
                    pos + 1
                }
                b'%' if pos > 0 && is_identifier_byte(bytes[pos - 1]) => {
                    let percents = run_length(bytes, pos, b'%');
                    if bytes.get(pos + percents) == Some(&b'"') {
                        stack.push(Mode::Multiline { percents });
                        pos + percents + 1
                    } else {
                        pos + percents
                    }
                }
                b'{' => {
                    *braces += 1;
                    pos + 1
                }
                b'}' => {
                    if *braces > 0 {
                        *braces -= 1;
                    } else if nested {
                        stack.pop();
                    }
                    pos + 1
                }
                byte if is_identifier_start(byte) => {
                    let end = identifier_end(bytes, pos);
                    if &bytes[pos..end] == IMPORT_KEYWORD
                        && let Some((target, next)) = import_target(bytes, end)
                    {
                        targets.push(target);
                        next
                    } else {
                        end
                    }
                }
                _ => pos + 1,
            },
            Some(Mode::Standard) => match bytes[pos] {
                b'\\' => pos + 2,
                b'"' => {
                    stack.pop();
                    pos + 1
                }
                b'%' if bytes.get(pos + 1) == Some(&b'{') => {
                    stack.push(Mode::Code { braces: 0 });
                    pos + 2
                }
                _ => pos + 1,
            },
            Some(Mode::Multiline { percents }) => {
                let percents = *percents;
                match bytes[pos] {
                    b'"' if run_length(bytes, pos + 1, b'%') >= percents => {
                        stack.pop();
                        pos + 1 + percents
                    }
                    b'%' if run_length(bytes, pos, b'%') == percents && bytes.get(pos + percents) == Some(&b'{') => {
                        stack.push(Mode::Code { braces: 0 });
                        pos + percents + 1
                    }
                    b'%' => pos + run_length(bytes, pos, b'%'),
                    _ => pos + 1,
                }
            }
            None => bytes.len(),
        };
    }
    targets
}

/// Read the target after an `import` keyword ending at `pos`.
fn import_target(bytes: &[u8], mut pos: usize) -> Option<(ImportTarget, usize)> {
    loop {
        match bytes.get(pos)? {
            byte if byte.is_ascii_whitespace() => pos += 1,
            b'#' => pos = skip_line(bytes, pos),
            _ => break,
        }
    }
    if bytes[pos] == b'"' {
        let mut value = Vec::new();
        pos += 1;
        loop {
            match *bytes.get(pos)? {
                b'"' => return Some((ImportTarget::Path(String::from_utf8_lossy(&value).into_owned()), pos + 1)),
                b'\\' => {
                    value.push(match *bytes.get(pos + 1)? {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        escaped => escaped,
                    });
                    pos += 2;
                }
                byte => {
                    value.push(byte);
                    pos += 1;
                }
            }
        }
    }
    if is_identifier_start(bytes[pos]) {
        let end = identifier_end(bytes, pos);
        return Some((ImportTarget::Package(String::from_utf8_lossy(&bytes[pos..end]).into_owned()), end));
    }
    None
}

/// Resolve a quoted import against the importing file's directory.
///
/// Absolute targets stay absolute and parent traversal beyond the root stays
/// visible, so the core rejects both as unsafe undeclared dependencies.
fn resolve_import(importer: &str, target: &str) -> String {
    if target.starts_with('/') {
        return target.to_owned();
    }
    let mut components = importer.split('/').filter(|component| !component.is_empty()).collect::<Vec<_>>();
    components.pop();
    push_components(&mut components, target);
    components.join("/")
}

fn normalize_path(path: &str) -> String {
    let mut components = Vec::new();
    push_components(&mut components, path.trim());
    components.join("/")
}

fn push_components<'a>(components: &mut Vec<&'a str>, path: &'a str) {
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." if components.last().is_some_and(|last| *last != "..") => {
                components.pop();
            }
            other => components.push(other),
        }
    }
}

fn is_nickel_source(path: &str) -> bool {
    !DATA_EXTENSIONS.iter().any(|extension| path.ends_with(extension))
}

fn skip_line(bytes: &[u8], pos: usize) -> usize {
    bytes[pos..].iter().position(|byte| *byte == b'\n').map_or(bytes.len(), |offset| pos + offset + 1)
}

fn run_length(bytes: &[u8], pos: usize, expected: u8) -> usize {
    bytes.get(pos..).map_or(0, |rest| rest.iter().take_while(|byte| **byte == expected).count())
}

fn identifier_end(bytes: &[u8], pos: usize) -> usize {
    bytes[pos..]
        .iter()
        .position(|byte| !is_identifier_byte(*byte))
        .map_or(bytes.len(), |offset| pos + offset)
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\'' | b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_reports_code_and_interpolated_imports_only() {
        let text = concat!(
            "# import \"comment.ncl\"\n",
            "let a = import \"a.ncl\" in\n",
            "let b = import # comment between keyword and target\n  \"dir/b.ncl\" in\n",
            "let reimport = 1 in\n",
            "let pkg = import some_package in\n",
            "{\n",
            "  s = \"import \\\"string.ncl\\\" %{ import \"interpolated.ncl\" }\",\n",
            "  m = m%%\"import \"multiline.ncl\" %{ ignored } %%{ import \"multi-interp.ncl\" }\"%%,\n",
            "  r = { nested = { x = 1 } },\n",
            "  after = import \"after.ncl\",\n",
            "}\n",
        );

        let targets = import_targets(text);

        assert_eq!(targets, vec![
            ImportTarget::Path("a.ncl".to_owned()),
            ImportTarget::Path("dir/b.ncl".to_owned()),
            ImportTarget::Package("some_package".to_owned()),
            ImportTarget::Path("interpolated.ncl".to_owned()),
            ImportTarget::Path("multi-interp.ncl".to_owned()),
            ImportTarget::Path("after.ncl".to_owned()),
        ]);
    }

    #[test]
    fn closure_resolves_relative_imports_and_follows_supplied_nickel_dependencies() {
        let dependencies = [
            NamedText {
                path: "./standards/contracts.ncl",
                text: "let data = import \"data/values.json\" in import \"../shared/base.ncl\"",
            },
            NamedText {
                path: "shared/base.ncl",
                text: "import \"../config/source.ncl\"",
            },
            NamedText {
                path: "standards/data/values.json",
                text: "{\"key\": import \"ignored-data.ncl\"}",
            },
        ];

        let observed = observed_import_closure(
            "config/source.ncl",
            "import \"../standards/./contracts.ncl\" & import \"/abs.ncl\" & import \"../../escape.ncl\"",
            &dependencies,
        );

        assert_eq!(observed, vec![
            "../escape.ncl".to_owned(),
            "/abs.ncl".to_owned(),
            "shared/base.ncl".to_owned(),
            "standards/contracts.ncl".to_owned(),
            "standards/data/values.json".to_owned(),
        ]);
    }
}
