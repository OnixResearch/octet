use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::cli::FingerprintArgs;
use crate::cli::FingerprintMode;
use crate::cli::OutputFormat;
use crate::determinism;
use crate::evidence;

const HASH_PREFIX: &str = "b3:";
/// Version tag hashed ahead of the canonical tokens in every marker hash. The
/// untagged v1 hash split literal contents into tokens and dropped code after
/// a quoted `//`; its token text never contains a newline, so no v1 marker can
/// equal a tagged v2 hash.
const MARKER_HASH_DOMAIN: &str = "octet-source-token-fingerprint-v2";
const COMMENT_MARKER: &str = "// octet-fingerprint:";
const LEGACY_SLOTCAR_COMMENT_MARKER: &str = "// slotcar-fingerprint:";
const LEGACY_TIGERSTYLE_COMMENT_MARKER: &str = "// tigerstyle-fingerprint:";
const ATTR_MARKER: &str = "#[fingerprint(";
const REVIEW_KEY: &str = "review=";
const EVIDENCE_KEY: &str = "evidence=";
const MACRO_ALLOW_KEY: &str = "macro=allow";
const FINGERPRINT_PROVENANCE_SOURCE_ID: &str = "octet.fingerprint.review-manifest";
const FINGERPRINT_REVIEW_ACTION: &str = "review fingerprint manifest records before using them as release evidence";

#[derive(Debug, Clone, Serialize)]
struct FunctionRecord {
    path: String,
    name: String,
    stored_hash: Option<String>,
    computed_hash: String,
    normalized_body_hash: String,
    semantic_id: String,
    dependency_set_hash: String,
    dependency_identity_status: String,
    tool_version: &'static str,
    status: FingerprintStatus,
    review: Option<String>,
    evidence: Option<String>,
    macro_calls: bool,
    macro_warning: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum FingerprintStatus {
    Match,
    Mismatch,
    Missing,
    Written,
}

#[derive(Debug, Serialize)]
struct FingerprintReport {
    schema_version: u32,
    mode: &'static str,
    records: Vec<FunctionRecord>,
}

#[derive(Debug, Serialize)]
struct FingerprintReviewManifest {
    schema_version: u32,
    tool_name: &'static str,
    tool_version: &'static str,
    mode: &'static str,
    input_paths: Vec<String>,
    manifest_hash: String,
    records: Vec<ReviewManifestRecord>,
}

#[derive(Debug, Serialize)]
struct ReviewManifestRecord {
    stable_id: String,
    path: String,
    name: String,
    status: FingerprintStatus,
    stored_hash: Option<String>,
    computed_hash: String,
    normalized_body_hash: String,
    semantic_id: String,
    dependency_set_hash: String,
    dependency_identity_status: String,
    tool_version: &'static str,
    review: Option<String>,
    evidence: Option<String>,
    requires_review: bool,
    macro_warning: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionSpan {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) marker: Option<Marker>,
    pub(crate) preceding_marker_in_code: bool,
    pub(crate) macro_calls: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Marker {
    start: usize,
    end: usize,
    stored_hash: String,
    review: Option<String>,
    evidence: Option<String>,
    macro_allowed: bool,
}

impl Marker {
    pub(crate) fn stored_hash(&self) -> &str {
        &self.stored_hash
    }

    pub(crate) fn review(&self) -> Option<&str> {
        self.review.as_deref()
    }

    pub(crate) fn evidence(&self) -> Option<&str> {
        self.evidence.as_deref()
    }
}

#[allow(clippy::too_many_lines)]
pub fn run(args: &FingerprintArgs) -> Result<i32, String> {
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };
    let input_paths: Vec<String> = paths.iter().map(|path| determinism::normalize_path(path)).collect();
    let mut files = Vec::new();
    for path in paths {
        collect_rust_files(&path, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut records = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let spans = find_functions(&source);
        let mut replacements = Vec::new();
        for span in spans {
            let function_source = &source[span.start..span.end];
            let normalized = normalize_tokens(function_source);
            let computed_hash = function_hash(&normalized);
            let normalized_body_hash = determinism::full_hash(&normalized);
            let dependency_set_hash = dependency_set_hash(function_source);
            let dependency_identity_status = dependency_identity_status(function_source).to_owned();
            let semantic_id = determinism::short_hash(&format!(
                "body={normalized_body_hash}\ndeps={dependency_set_hash}\ndep_status={dependency_identity_status}\ntool={}\n",
                env!("CARGO_PKG_VERSION"),
            ));
            let marker = span.marker.clone();
            if !matches!(args.mode, FingerprintMode::Write) && marker.is_none() {
                continue;
            }
            let status = match marker.as_ref().map(|m| m.stored_hash.as_str()) {
                Some(stored) if stored == computed_hash => FingerprintStatus::Match,
                Some(_) => FingerprintStatus::Mismatch,
                None => FingerprintStatus::Missing,
            };
            let macro_warning = span.macro_calls && !marker.as_ref().is_some_and(|m| m.macro_allowed);
            let record = FunctionRecord {
                path: path.display().to_string(),
                name: span.name.clone(),
                stored_hash: marker.as_ref().map(|m| m.stored_hash.clone()),
                computed_hash: computed_hash.clone(),
                normalized_body_hash,
                semantic_id,
                dependency_set_hash,
                dependency_identity_status,
                tool_version: env!("CARGO_PKG_VERSION"),
                status: if matches!(args.mode, FingerprintMode::Write | FingerprintMode::Update)
                    && status != FingerprintStatus::Match
                {
                    FingerprintStatus::Written
                } else {
                    status
                },
                review: marker.as_ref().and_then(|m| m.review.clone()),
                evidence: marker.as_ref().and_then(|m| m.evidence.clone()),
                macro_calls: span.macro_calls,
                macro_warning,
            };
            if matches!(args.mode, FingerprintMode::Write | FingerprintMode::Update)
                && status != FingerprintStatus::Match
            {
                let review = marker.as_ref().and_then(|m| m.review.as_deref()).unwrap_or("TODO");
                let evidence = marker.as_ref().and_then(|m| m.evidence.as_deref()).unwrap_or("TODO");
                let macro_suffix = if marker.as_ref().is_some_and(|m| m.macro_allowed) {
                    "; macro=allow"
                } else if span.macro_calls {
                    "; macro=review-required"
                } else {
                    ""
                };
                let line = format!(
                    "// octet-fingerprint: b3:{computed_hash}; review={review}; evidence={evidence}{macro_suffix}\n"
                );
                if let Some(marker) = marker {
                    replacements.push((marker.start, marker.end, line));
                } else {
                    let (start, line) = marker_insertion(&source, span.start, line);
                    replacements.push((start, start, line));
                }
            }
            records.push(record);
        }
        if !replacements.is_empty() {
            let mut updated = source;
            for (start, end, replacement) in replacements.into_iter().rev() {
                updated.replace_range(start..end, &replacement);
            }
            fs::write(&path, updated).map_err(|error| format!("write {}: {error}", path.display()))?;
        }
    }

    let report = FingerprintReport {
        schema_version: 1,
        mode: mode_name(args.mode),
        records,
    };
    if let Some(path) = args.review_manifest.as_deref() {
        write_review_manifest(path, &report, input_paths)?;
    }
    match args.output_format {
        OutputFormat::Human => print_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("fingerprint does not support SARIF output; use human or json".to_owned()),
    }

    let failing = report.records.iter().any(|record| {
        matches!(args.mode, FingerprintMode::Check | FingerprintMode::Diff)
            && (record.status != FingerprintStatus::Match
                || record.macro_warning
                || record.review.is_none()
                || record.evidence.is_none())
    });
    Ok(i32::from(failing))
}

fn write_review_manifest(path: &Path, report: &FingerprintReport, input_paths: Vec<String>) -> Result<(), String> {
    let mut records: Vec<ReviewManifestRecord> = report
        .records
        .iter()
        .map(|record| ReviewManifestRecord {
            stable_id: fingerprint_record_stable_id(record),
            path: record.path.clone(),
            name: record.name.clone(),
            status: record.status,
            stored_hash: record.stored_hash.clone(),
            computed_hash: record.computed_hash.clone(),
            normalized_body_hash: record.normalized_body_hash.clone(),
            semantic_id: record.semantic_id.clone(),
            dependency_set_hash: record.dependency_set_hash.clone(),
            dependency_identity_status: record.dependency_identity_status.clone(),
            tool_version: record.tool_version,
            review: record.review.clone(),
            evidence: record.evidence.clone(),
            requires_review: record.status != FingerprintStatus::Match
                || record.review.is_none()
                || record.evidence.is_none()
                || record.macro_warning,
            macro_warning: record.macro_warning,
        })
        .collect();
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.name.cmp(&right.name))
            .then(left.stable_id.cmp(&right.stable_id))
    });

    let hash_payload = serde_json::json!({
        "mode": report.mode,
        "input_paths": &input_paths,
        "records": &records,
    });
    let manifest = FingerprintReviewManifest {
        schema_version: 1,
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        mode: report.mode,
        input_paths,
        manifest_hash: determinism::full_hash(&hash_payload.to_string()),
        records,
    };
    let contents =
        serde_json::to_string_pretty(&manifest).map_err(|error| format!("write review manifest JSON: {error}"))?;
    let rendered = format!("{contents}\n");
    fs::write(path, &rendered).map_err(|error| format!("write {}: {error}", path.display()))?;
    write_review_manifest_provenance(path, rendered.as_bytes(), &manifest.input_paths)
}

fn write_review_manifest_provenance(path: &Path, contents: &[u8], input_paths: &[String]) -> Result<(), String> {
    let provenance_path = path.with_extension("provenance.jsonl");
    let record = evidence::ProvenanceRecord {
        schema_version: evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: FINGERPRINT_PROVENANCE_SOURCE_ID.to_owned(),
        checker_id: evidence::CHECKER_OCTET_FINGERPRINT.to_owned(),
        evidence_tier: evidence::EvidenceTier::ManualReviewRequired,
        artifact_locator: path.display().to_string(),
        blake3: evidence::artifact_blake3_hex(contents),
        checked_scope: format!("fingerprint:{}", input_paths.join(",")),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(FINGERPRINT_REVIEW_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    };
    let line = serde_json::to_string(&record).map_err(|error| format!("write fingerprint provenance JSON: {error}"))?;
    fs::write(&provenance_path, format!("{line}\n"))
        .map_err(|error| format!("write {}: {error}", provenance_path.display()))
}

fn fingerprint_record_stable_id(record: &FunctionRecord) -> String {
    record.semantic_id.clone()
}

pub(crate) fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| format!("read dir {}: {error}", path.display()))? {
            let entry = entry.map_err(|error| format!("read dir entry {}: {error}", path.display()))?;
            let child = entry.path();
            if child.file_name().is_some_and(|name| name == "target" || name == ".git") {
                continue;
            }
            collect_rust_files(&child, files)?;
        }
    }
    Ok(())
}

pub(crate) fn find_functions(source: &str) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    // Structure is matched in a same-length view with comments and literals
    // blanked, so documentation examples and quoted text cannot define, hide,
    // or end a function. Spans, markers, and hashed text still use `source`.
    let code = code_view(source);
    let macro_rules_ranges = macro_rules_ranges(&code);
    let mut marker_cursor = MarkerLexicalCursor::new(source);
    let mut cursor = 0;
    while let Some(relative) = code[cursor..].find("fn") {
        let fn_pos = cursor + relative;
        let after_keyword = fn_pos + "fn".len();
        if code[..fn_pos].chars().next_back().is_some_and(identifier_continue)
            || !code[after_keyword..].chars().next().is_some_and(char::is_whitespace)
        {
            cursor = after_keyword;
            continue;
        }
        if macro_rules_ranges.iter().any(|(start, end)| (*start..*end).contains(&fn_pos)) {
            cursor = after_keyword;
            continue;
        }
        let Some((separator, _)) = code[after_keyword..].char_indices().find(|(_, ch)| !ch.is_whitespace()) else {
            break;
        };
        let name_start = after_keyword + separator;
        let ident_start = name_start + if code[name_start..].starts_with("r#") { 2 } else { 0 };
        if !code[ident_start..].chars().next().is_some_and(identifier_start) {
            cursor = ident_start;
            continue;
        }
        let name_end = code[ident_start..]
            .find(|ch: char| !identifier_continue(ch))
            .map_or(code.len(), |offset| ident_start + offset);
        let open_brace = match signature_end(&code, name_end) {
            Some(SignatureEnd::Body(open_brace)) => open_brace,
            Some(SignatureEnd::Declaration(semicolon)) => {
                cursor = semicolon + 1;
                continue;
            }
            None => break,
        };
        let Some(end) = matching_brace(&code, open_brace) else {
            break;
        };
        let item_start = function_item_start(source, &code, fn_pos);
        // A block comment can close on the function's own line (`*/ pub fn`).
        // Its closing bytes precede the first code token, not the item, and
        // cannot contribute to the function identity or marker insertion.
        let first_code = code[item_start..fn_pos]
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map_or(fn_pos, |(offset, _)| item_start + offset);
        let item_start = if source[item_start..first_code].trim().is_empty() {
            item_start
        } else {
            first_code
        };
        let (marker, preceding_marker_in_code) = preceding_marker(source, item_start, &mut marker_cursor);
        let body = &source[open_brace..end];
        spans.push(FunctionSpan {
            name: source[name_start..name_end].to_owned(),
            start: item_start,
            end,
            marker,
            preceding_marker_in_code,
            macro_calls: contains_macro_call(body),
        });
        cursor = end;
    }
    spans
}

fn macro_rules_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("macro_rules!") {
        let macro_start = cursor + relative;
        let Some(open_brace) = source[macro_start..].find('{').map(|offset| macro_start + offset) else {
            break;
        };
        let Some(end) = matching_brace(source, open_brace) else {
            break;
        };
        ranges.push((macro_start, end));
        cursor = end;
    }
    ranges
}

/// How a function signature ends in the code view.
enum SignatureEnd {
    /// Offset of the opening brace of the function body.
    Body(usize),
    /// Offset of the `;` that ends a bodiless declaration.
    Declaration(usize),
}

/// Finds the first top-level `{` or `;` after a function name. Delimiters
/// nested in parentheses, brackets, braces, or generic arguments belong to the
/// signature, as in `fn f(x: [u8; 2]) -> W<{ N }> {`.
fn signature_end(code: &str, name_end: usize) -> Option<SignatureEnd> {
    let bytes = code.as_bytes();
    let mut enclosing = 0_usize;
    let mut generics = 0_usize;
    for (index, byte) in bytes.iter().enumerate().skip(name_end) {
        match byte {
            b'{' if enclosing == 0 && !opens_const_argument(bytes, index, generics) => {
                return Some(SignatureEnd::Body(index));
            }
            b';' if enclosing == 0 => return Some(SignatureEnd::Declaration(index)),
            b'(' | b'[' | b'{' => enclosing += 1,
            b')' | b']' | b'}' => enclosing = enclosing.saturating_sub(1),
            b'<' if enclosing == 0 && opens_generics(code, index) => generics += 1,
            b'>' if enclosing == 0 && closes_generics(bytes, index) => generics = generics.saturating_sub(1),
            _ => {}
        }
    }
    None
}

/// A `{` directly inside generic arguments, as in `W<{ N }>` or `W<A, { N }>`.
fn opens_const_argument(bytes: &[u8], index: usize, generics: usize) -> bool {
    generics > 0
        && matches!(
            bytes.get(..index).and_then(|before| before.iter().rev().find(|byte| !byte.is_ascii_whitespace())),
            Some(b'<' | b',')
        )
}

/// `<` opens generic arguments when it directly follows a name or `::`; a
/// spaced `x < y` or a `<=` is a comparison, for example in a macro's proof
/// clauses.
fn opens_generics(code: &str, index: usize) -> bool {
    let previous = code[..index].chars().next_back();
    previous.is_some_and(|ch| identifier_continue(ch) || ch == ':') && code.as_bytes().get(index + 1) != Some(&b'=')
}

/// `>` closes generic arguments unless it belongs to `->`, `=>`, or `>=`.
fn closes_generics(bytes: &[u8], index: usize) -> bool {
    let previous = index.checked_sub(1).and_then(|before| bytes.get(before));
    !matches!(previous, Some(b'-' | b'=')) && bytes.get(index + 1) != Some(&b'=')
}

/// Returns `source` with comments and string, raw-string, byte, and character
/// literals blanked to spaces. Byte offsets and line breaks are unchanged, so
/// an offset found in the view indexes the same position in `source`.
fn code_view(source: &str) -> String {
    let mut view = String::with_capacity(source.len());
    let mut hidden_end = 0;
    for (offset, ch) in source.char_indices() {
        if offset >= hidden_end {
            hidden_end = non_code_at(source, offset).map_or(offset, |(_, end)| end);
        }
        if offset < hidden_end && ch != '\n' {
            view.extend(std::iter::repeat_n(' ', ch.len_utf8()));
        } else {
            view.push(ch);
        }
    }
    view
}

/// A source region that is not code.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NonCode {
    Comment,
    Literal,
}

/// Kind and end offset of the comment or literal that starts at `start`, if
/// any. `code_view` blanks both kinds; `SourceTokens` drops comments and keeps
/// literals whole.
fn non_code_at(source: &str, start: usize) -> Option<(NonCode, usize)> {
    let bytes = source.as_bytes();
    let next = bytes.get(start + 1).copied();
    match bytes.get(start).copied()? {
        b'/' if next == Some(b'/') => Some((NonCode::Comment, line_comment_end(bytes, start))),
        b'/' if next == Some(b'*') => Some((NonCode::Comment, block_comment_end(bytes, start))),
        b'"' => Some((NonCode::Literal, quoted_literal_end(bytes, start))),
        b'\'' => char_literal_end(source, start).map(|end| (NonCode::Literal, end)),
        b'r' => raw_string_end(source, start).map(|end| (NonCode::Literal, end)),
        _ => None,
    }
}

/// Doc comments become `#[doc = "..."]` tokens inside macro input; unlike
/// ordinary comments, their text is observable by a procedural macro.
fn is_doc_comment(comment: &str) -> bool {
    let bytes = comment.as_bytes();
    bytes.starts_with(b"//!")
        || (bytes.starts_with(b"///") && !bytes.starts_with(b"////"))
        || bytes.starts_with(b"/*!")
        || (bytes.starts_with(b"/**") && !bytes.starts_with(b"/***") && !bytes.starts_with(b"/**/"))
}

/// Line, outer doc, and inner doc comments end before their newline.
fn line_comment_end(bytes: &[u8], start: usize) -> usize {
    bytes
        .get(start..)
        .and_then(|rest| rest.iter().position(|byte| *byte == b'\n'))
        .map_or(bytes.len(), |offset| start + offset)
}

/// Block comments nest in Rust, so `/* /* */ */` is one comment.
fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0_usize;
    let mut index = start;
    while let Some(pair) = bytes.get(index..index + 2) {
        match pair {
            b"/*" => {
                depth += 1;
                index += 2;
            }
            b"*/" => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return index;
                }
            }
            _ => index += 1,
        }
    }
    bytes.len()
}

/// String, byte-string, and C-string literals with backslash escapes.
fn quoted_literal_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while let Some(byte) = bytes.get(index) {
        match byte {
            b'\\' => index += 2,
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

/// Character and byte literals. A quote that starts a lifetime or a loop label
/// is code, so it returns `None`.
fn char_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let content = start + 1;
    let close = if bytes.get(content) == Some(&b'\\') {
        // Escapes include `'\''`, `'\\'`, `'\x7f'`, and `'\u{10FFFF}'`.
        let escaped = content + 2;
        escaped + bytes.get(escaped..)?.iter().position(|byte| matches!(byte, b'\'' | b'\n'))?
    } else {
        content + source.get(content..)?.chars().next()?.len_utf8()
    };
    (bytes.get(close) == Some(&b'\'')).then_some(close + 1)
}

/// Raw string literals such as `r"…"`, `r#"…"#`, `br"…"`, and `cr#"…"#`.
/// Raw identifiers such as `r#type` are code.
fn raw_string_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let prefix_start = match start.checked_sub(1).and_then(|before| bytes.get(before)) {
        Some(b'b' | b'c') => start - 1,
        _ => start,
    };
    if source[..prefix_start].chars().next_back().is_some_and(identifier_continue) {
        return None;
    }
    let hashes = bytes.get(start + 1..)?.iter().take_while(|byte| **byte == b'#').count();
    let body = start + 2 + hashes;
    if bytes.get(body - 1) != Some(&b'"') {
        return None;
    }
    let closing_quote = (body..bytes.len()).find(|index| {
        bytes.get(*index) == Some(&b'"')
            && bytes.get(index + 1..index + 1 + hashes).is_some_and(|tail| tail.iter().all(|byte| *byte == b'#'))
    });
    Some(closing_quote.map_or(bytes.len(), |index| index + 1 + hashes))
}

fn function_item_start(source: &str, code: &str, fn_pos: usize) -> usize {
    let mut prefix_start = fn_pos;
    while let Some(modifier_start) = preceding_function_modifier_start(code, prefix_start) {
        prefix_start = modifier_start;
    }
    let mut item_start = source[..prefix_start].rfind('\n').map_or(0, |index| index + 1);
    let mut line_start = item_start;

    loop {
        let previous_end = line_start.saturating_sub(1);
        if previous_end == 0 {
            return item_start;
        }

        let previous_start = source[..previous_end].rfind('\n').map_or(0, |index| index + 1);
        // Comments and blank lines do not detach Rust outer attributes from
        // their item. Probe across them, but do not include unrelated trivia
        // (or a reviewed marker) in the function's source-token span.
        if code[previous_start..previous_end].trim().is_empty() {
            line_start = previous_start;
            continue;
        }
        if code[previous_start..previous_end].trim_start().starts_with("#[") {
            // The historical fingerprint attribute is a review marker, not
            // one of the attributes hashed as part of the function item.
            if source[previous_start..previous_end].trim_start().starts_with(ATTR_MARKER) {
                return item_start;
            }
            item_start = previous_start;
            line_start = previous_start;
            continue;
        }
        if let Some(attribute_start) = multiline_attribute_start(source, code, previous_end) {
            item_start = attribute_start;
            line_start = attribute_start;
            continue;
        }

        return item_start;
    }
}

fn preceding_function_modifier_start(code: &str, before: usize) -> Option<usize> {
    let end = code[..before].trim_end().len();
    let bytes = code.as_bytes();
    if bytes.get(end.checked_sub(1)?) == Some(&b')') {
        let open = matching_open_delimiter(code, end - 1, b'(', b')')?;
        let word_end = code[..open].trim_end().len();
        let start = ascii_identifier_start(bytes, word_end);
        return (code.get(start..word_end) == Some("pub")).then_some(start);
    }

    let start = ascii_identifier_start(bytes, end);
    matches!(code.get(start..end)?, "pub" | "default" | "const" | "async" | "unsafe" | "extern").then_some(start)
}

fn ascii_identifier_start(bytes: &[u8], end: usize) -> usize {
    let mut start = end;
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    start
}

fn multiline_attribute_start(source: &str, code: &str, line_end: usize) -> Option<usize> {
    let end = code[..line_end].trim_end().len();
    let close = end.checked_sub(1)?;
    if code.as_bytes()[close] != b']' {
        return None;
    }
    let open = matching_open_delimiter(code, close, b'[', b']')?;
    let hash = open.checked_sub(1)?;
    if code.as_bytes()[hash] != b'#' {
        return None;
    }
    let line_start = source[..hash].rfind('\n').map_or(0, |index| index + 1);
    code[line_start..hash].trim().is_empty().then_some(line_start)
}

fn matching_open_delimiter(code: &str, close: usize, open_byte: u8, close_byte: u8) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in code.as_bytes()[..=close].iter().enumerate().rev() {
        if *byte == close_byte {
            depth += 1;
        } else if *byte == open_byte {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn preceding_marker(
    source: &str,
    item_start: usize,
    marker_cursor: &mut MarkerLexicalCursor<'_>,
) -> (Option<Marker>, bool) {
    let prefix = &source[..item_start];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let previous_end = line_start.saturating_sub(1);
    let previous_start = source[..previous_end].rfind('\n').map_or(0, |index| index + 1);
    let line = &source[previous_start..previous_end];
    let previous = line.trim();
    let marker_in_code = marker_cursor.in_code(previous_start + line.len() - line.trim_start().len());
    if !(previous.starts_with(COMMENT_MARKER)
        || previous.starts_with(LEGACY_SLOTCAR_COMMENT_MARKER)
        || previous.starts_with(LEGACY_TIGERSTYLE_COMMENT_MARKER)
        || previous.starts_with(ATTR_MARKER))
        || !marker_in_code
    {
        return (None, marker_in_code);
    }
    let line_end = if previous_end < source.len() {
        previous_end + 1
    } else {
        previous_end
    };
    (parse_marker(previous, previous_start, line_end), marker_in_code)
}

/// Source-ordered functions share one lexical walk. Even if every function has
/// a marker, the total scan is O(source bytes), without per-function storage.
struct MarkerLexicalCursor<'a> {
    source: &'a str,
    visited: usize,
    hidden_end: usize,
}

impl<'a> MarkerLexicalCursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            visited: 0,
            hidden_end: 0,
        }
    }

    fn in_code(&mut self, marker_start: usize) -> bool {
        if marker_start < self.visited
            || marker_start > self.source.len()
            || !self.source.is_char_boundary(marker_start)
        {
            return false;
        }
        for (relative, _) in self.source[self.visited..marker_start].char_indices() {
            let offset = self.visited + relative;
            if offset >= self.hidden_end {
                self.hidden_end = non_code_at(self.source, offset).map_or(offset, |(_, end)| end);
            }
        }
        self.visited = marker_start;
        self.hidden_end <= marker_start
    }
}

/// Insert a standalone marker before the item, preserving its indentation.
/// If a block comment closes on the item's line, a newline must separate
/// that closer from the active marker before the function.
pub(crate) fn marker_insertion(source: &str, item_start: usize, line: String) -> (usize, String) {
    let line_start = source[..item_start].rfind('\n').map_or(0, |index| index + 1);
    if source[line_start..item_start].trim().is_empty() {
        (line_start, line)
    } else {
        (item_start, format!("\n{line}"))
    }
}

fn parse_marker(line: &str, start: usize, end: usize) -> Option<Marker> {
    let hash_start = line.find(HASH_PREFIX)? + HASH_PREFIX.len();
    let hash = line[hash_start..].chars().take_while(char::is_ascii_hexdigit).collect::<String>();
    if hash.is_empty() {
        return None;
    }
    Some(Marker {
        start,
        end,
        stored_hash: hash,
        review: value_after(line, REVIEW_KEY),
        evidence: value_after(line, EVIDENCE_KEY),
        macro_allowed: line.contains(MACRO_ALLOW_KEY),
    })
}

fn value_after(line: &str, key: &str) -> Option<String> {
    let bare_key = key.trim_end_matches('=');
    let start = line.find(bare_key)? + bare_key.len();
    let value_start = line[start..].find('=')? + start + 1;
    let value = line[value_start..]
        .split([';', ',', ')', ']'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_owned();
    (!value.is_empty() && value != "TODO").then_some(value)
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0u32;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + offset + ch.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

/// Marker hash of a function's canonical tokens, separated from v1 hashes by
/// [`MARKER_HASH_DOMAIN`].
fn function_hash(normalized: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(MARKER_HASH_DOMAIN.as_bytes());
    hasher.update(b"\n");
    hasher.update(normalized.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Hash of the sorted, distinct identifiers in code, one per line. Literal
/// contents are not dependencies.
pub(crate) fn dependency_set_hash(function_source: &str) -> String {
    let mut identifiers = SourceTokens::new(function_source)
        .filter(|token| !token.is_literal && token.text.starts_with(identifier_start))
        .map(|token| token.text)
        .collect::<Vec<_>>();
    identifiers.sort_unstable();
    identifiers.dedup();
    let mut lines = String::with_capacity(identifiers.iter().map(|identifier| identifier.len() + 1).sum());
    for identifier in identifiers {
        if !lines.is_empty() {
            lines.push('\n');
        }
        lines.push_str(identifier);
    }
    determinism::full_hash(&lines)
}

pub(crate) fn dependency_identity_status(function_source: &str) -> &'static str {
    if contains_macro_call(function_source) {
        "unsupported-macro-context"
    } else {
        "source-scanner-token-set"
    }
}

/// Canonical token text: tokens joined by single spaces. Whitespace and
/// ordinary comments do not change it. Joint punctuation and lexical literal
/// boundaries stay intact, as does doc-comment text observable inside macros.
/// Literal contents stay verbatim, including any `//` inside them.
pub(crate) fn normalize_tokens(source: &str) -> String {
    let mut normalized = String::with_capacity(source.len());
    let mut previous_end = 0;
    let mut previous_punctuation = false;
    let mut previous_quote = false;
    for token in SourceTokens::new(source) {
        let joint = previous_end == token.start
            && previous_punctuation
            && (token.is_punctuation
                || (previous_quote && !token.is_literal && token.text.starts_with(identifier_start)));
        if !normalized.is_empty() && !joint {
            normalized.push(' ');
        }
        normalized.push_str(token.text);
        previous_end = token.start + token.text.len();
        previous_punctuation = token.is_punctuation;
        previous_quote = token.is_punctuation && token.text == "'";
    }
    normalized
}

/// One canonical source token, borrowed from the source text.
struct SourceToken<'a> {
    text: &'a str,
    is_literal: bool,
    is_punctuation: bool,
    start: usize,
}

/// Canonical tokens of Rust source: identifiers (including raw identifiers),
/// numbers, punctuation, and whole literals (including suffixes and string
/// prefixes). Ordinary comments and whitespace only separate tokens; doc
/// comments remain observable tokens.
struct SourceTokens<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> SourceTokens<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    fn raw_identifier_end(&self, start: usize) -> Option<usize> {
        let rest = self.source.get(start..)?;
        if !rest.starts_with("r#") && !rest.starts_with("k#") {
            return None;
        }
        let name_start = start + 2;
        let first = self.source[name_start..].chars().next()?;
        if !identifier_start(first) {
            return None;
        }
        let mut end = name_start + first.len_utf8();
        while let Some(ch) = self.source[end..].chars().next() {
            if !identifier_continue(ch) {
                break;
            }
            end += ch.len_utf8();
        }
        Some(end)
    }

    /// End of a literal whose prefix is the identifier run `start..run_end`.
    fn prefixed_literal_end(&self, start: usize, run_end: usize) -> Option<usize> {
        match &self.source[start..run_end] {
            "br" | "cr" => raw_string_end(self.source, run_end - 1),
            "b" | "c" => match non_code_at(self.source, run_end)? {
                (NonCode::Literal, end) => Some(end),
                (NonCode::Comment, _) => None,
            },
            _ => None,
        }
    }
}

/// Rust's numeric lexer keeps a fraction, exponent, and immediately attached
/// suffix in one literal token. A dot starting a range or field access is not
/// part of the number. Operates only on source byte offsets; no token copies.
fn numeric_literal_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut end = start + 1;
    let mut hexadecimal = false;
    if bytes[start] == b'0' {
        match bytes.get(end) {
            Some(b'b' | b'o' | b'x') => {
                hexadecimal = bytes[end] == b'x';
                end += 1;
            }
            Some(b'0'..=b'9' | b'_' | b'.' | b'e' | b'E') => {}
            _ => return literal_suffix_end(source, end),
        }
    }
    end = number_digits_end(bytes, end, hexadecimal);

    if bytes.get(end) == Some(&b'.')
        && bytes.get(end + 1) != Some(&b'.')
        && !source[end + 1..].chars().next().is_some_and(identifier_start)
    {
        end += 1;
        if bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end = number_digits_end(bytes, end, false);
            if matches!(bytes.get(end), Some(b'e' | b'E')) {
                end = exponent_end(bytes, end + 1);
            }
        }
    } else if matches!(bytes.get(end), Some(b'e' | b'E')) {
        end = exponent_end(bytes, end + 1);
    }
    literal_suffix_end(source, end)
}

fn number_digits_end(bytes: &[u8], mut end: usize, hexadecimal: bool) -> usize {
    while bytes
        .get(end)
        .is_some_and(|byte| *byte == b'_' || byte.is_ascii_digit() || (hexadecimal && byte.is_ascii_hexdigit()))
    {
        end += 1;
    }
    end
}

fn exponent_end(bytes: &[u8], mut end: usize) -> usize {
    if matches!(bytes.get(end), Some(b'+' | b'-')) {
        end += 1;
    }
    number_digits_end(bytes, end, false)
}

fn literal_suffix_end(source: &str, start: usize) -> usize {
    let mut end = start;
    let Some(first) = source[end..].chars().next() else {
        return end;
    };
    if !identifier_start(first) {
        return end;
    }
    end += first.len_utf8();
    while let Some(ch) = source[end..].chars().next() {
        if !identifier_continue(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

fn rust_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation() && !matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | '_' | '"' | '\\' | '`')
}

impl<'a> Iterator for SourceTokens<'a> {
    type Item = SourceToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ch) = self.source[self.offset..].chars().next() {
            let start = self.offset;
            if let Some((kind, end)) = non_code_at(self.source, start) {
                self.offset = end;
                if kind == NonCode::Literal {
                    self.offset = literal_suffix_end(self.source, end);
                    return Some(SourceToken {
                        text: &self.source[start..self.offset],
                        is_literal: true,
                        is_punctuation: false,
                        start,
                    });
                }
                if is_doc_comment(&self.source[start..end]) {
                    return Some(SourceToken {
                        text: &self.source[start..end],
                        is_literal: true,
                        is_punctuation: false,
                        start,
                    });
                }
                continue;
            }
            self.offset += ch.len_utf8();
            if ch.is_whitespace() {
                continue;
            }
            if ch.is_ascii_digit() {
                self.offset = numeric_literal_end(self.source, start);
                return Some(SourceToken {
                    text: &self.source[start..self.offset],
                    is_literal: true,
                    is_punctuation: false,
                    start,
                });
            }
            if matches!(ch, 'r' | 'k')
                && let Some(end) = self.raw_identifier_end(start)
            {
                self.offset = end;
                return Some(SourceToken {
                    text: &self.source[start..end],
                    is_literal: false,
                    is_punctuation: false,
                    start,
                });
            }
            if !identifier_continue(ch) {
                return Some(SourceToken {
                    text: &self.source[start..self.offset],
                    is_literal: false,
                    is_punctuation: rust_punctuation(ch),
                    start,
                });
            }
            let run_end = self.source[start..]
                .find(|next: char| !identifier_continue(next))
                .map_or(self.source.len(), |len| start + len);
            let literal_end = self.prefixed_literal_end(start, run_end);
            self.offset = literal_end.map_or(run_end, |end| literal_suffix_end(self.source, end));
            return Some(SourceToken {
                text: &self.source[start..self.offset],
                is_literal: literal_end.is_some(),
                is_punctuation: false,
                start,
            });
        }
        None
    }
}

fn contains_macro_call(source: &str) -> bool {
    // Comments and literals are blanked, so quoted or commented text cannot
    // add a macro call and a quoted `//` cannot hide one.
    let code = code_view(source);
    code.match_indices('!').any(|(bang, _)| invokes_macro(&code, bang))
}

/// Strict keywords in every edition plus keywords reserved since Rust 1.0. None
/// can name a macro, so `if !(ready)` or `return !(flag)` is a unary `!`. Words
/// that are keywords only in later editions stay macro names: `try!` is a 2015
/// macro.
const NON_MACRO_KEYWORDS: &[&str] = &[
    "abstract", "as", "become", "box", "break", "const", "continue", "crate", "do", "else", "enum", "extern", "false",
    "final", "fn", "for", "if", "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override",
    "priv", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// Whether the `!` at `bang` invokes a macro: a macro name or path ends right
/// before it, and a `(`, `[`, or `{` group follows it.
fn invokes_macro(code: &str, bang: usize) -> bool {
    let has_group_after = code[bang + 1..].trim_start().starts_with(['(', '[', '{']);
    let before = code[..bang].trim_end();
    let name_start = before
        .char_indices()
        .rev()
        .find(|(_, ch)| !identifier_continue(*ch))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let name = &before[name_start..];
    let is_raw_identifier = before[..name_start].ends_with("r#");
    has_group_after && name.starts_with(identifier_start) && (is_raw_identifier || !NON_MACRO_KEYWORDS.contains(&name))
}

fn identifier_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}

fn identifier_continue(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_continue(ch)
}

fn mode_name(mode: FingerprintMode) -> &'static str {
    match mode {
        FingerprintMode::Check => "check",
        FingerprintMode::Write => "write",
        FingerprintMode::Update => "update",
        FingerprintMode::Diff => "diff",
    }
}

fn print_human(report: &FingerprintReport) {
    println!("octet fingerprint {}", report.mode);
    for record in &report.records {
        println!(
            "{} {} {} semantic_id={} stored={} computed=b3:{} normalized_body_hash={} dependency_identity_status={} review={} evidence={} macro_warning={}",
            record.path,
            record.name,
            status_name(record.status),
            record.semantic_id,
            record.stored_hash.as_deref().unwrap_or("<none>"),
            record.computed_hash,
            record.normalized_body_hash,
            record.dependency_identity_status,
            record.review.as_deref().unwrap_or("<missing>"),
            record.evidence.as_deref().unwrap_or("<missing>"),
            record.macro_warning,
        );
    }

    let match_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Match).count();
    let mismatch_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Mismatch).count();
    let missing_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Missing).count();
    let written_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Written).count();
    let macro_warning_count = report.records.iter().filter(|record| record.macro_warning).count();
    let metadata_missing_count =
        report.records.iter().filter(|record| record.review.is_none() || record.evidence.is_none()).count();
    println!(
        "summary total={} match={} mismatch={} missing={} written={} macro_warnings={} metadata_missing={}",
        report.records.len(),
        match_count,
        mismatch_count,
        missing_count,
        written_count,
        macro_warning_count,
        metadata_missing_count,
    );
}

fn status_name(status: FingerprintStatus) -> &'static str {
    match status {
        FingerprintStatus::Match => "match",
        FingerprintStatus::Mismatch => "mismatch",
        FingerprintStatus::Missing => "missing",
        FingerprintStatus::Written => "written",
    }
}

#[cfg(test)]
mod tests {
    use super::contains_macro_call;
    use super::dependency_set_hash;
    use super::find_functions;
    use super::function_hash;
    use super::normalize_tokens;

    fn source_hash(source: &str) -> String {
        function_hash(&normalize_tokens(source))
    }

    #[test]
    fn formatting_and_comments_do_not_change_hash() {
        let one = "fn checked(input: u32) -> u32 { // comment\n input + 1 }";
        let two = "fn checked( input:u32)->u32{input+1}";
        assert_eq!(source_hash(one), source_hash(two));
    }

    #[test]
    fn token_changes_update_hash() {
        assert_ne!(source_hash("fn f() -> u32 { 1 }"), source_hash("fn f() -> u32 { 2 }"));
    }

    #[test]
    fn dependency_set_is_sorted_distinct_code_identifiers() {
        let source = "fn total(items: &[u32]) -> u32 {\n    // comment_word\n    let _first = items.first();\n    items.iter().fold(0_u32, |sum, item| sum.saturating_add(*item)) + Bias::of(_first)\n}";
        assert_eq!(
            dependency_set_hash(source),
            crate::determinism::full_hash(
                "Bias\n_first\nfirst\nfn\nfold\nitem\nitems\niter\nlet\nof\nsaturating_add\nsum\ntotal\nu32"
            ),
        );
    }

    #[test]
    fn unicode_identifiers_stay_whole_in_tokens_dependencies_and_macros() {
        let source = "fn 漢字() { let 値 = 1; 値 }";
        assert_eq!(normalize_tokens(source), "fn 漢字 ( ) { let 値 = 1 ; 値 }");
        assert_eq!(dependency_set_hash(source), crate::determinism::full_hash("fn\nlet\n値\n漢字"));
        assert!(contains_macro_call("{ 漢字!(1); }"));
        assert!(!contains_macro_call("{ let 漢字 = 1; if !(ready) {} }"));
    }

    #[test]
    fn token_hash_covers_literal_contents_and_code_after_quoted_slashes() {
        let fetch = |tail: &str| {
            format!("pub fn fetch() -> u32 {{\n    let _endpoint = \"https://example.invalid/api\"; {tail}\n}}\n")
        };
        let returns_one = fetch("1");
        let aborts = fetch("std::process::abort()");
        assert_ne!(source_hash(&returns_one), source_hash(&aborts));
        assert_ne!(dependency_set_hash(&returns_one), dependency_set_hash(&aborts));
        for (one, other) in [
            ("fn f() -> &'static str { \"a b\" }", "fn f() -> &'static str { \"a  b\" }"),
            ("fn f() -> &'static str { \"a \\\" // b\" }", "fn f() -> &'static str { \"a \\\" // c\" }"),
            ("fn f() -> &'static str { r#\"x // \" y\"# }", "fn f() -> &'static str { r#\"x // \" z\"# }"),
            ("fn f() -> &'static [u8] { br\"/\" }", "fn f() -> &'static [u8] { br\"*\" }"),
            ("fn f() -> u8 { b'/' }", "fn f() -> u8 { b'*' }"),
        ] {
            assert_ne!(source_hash(one), source_hash(other), "{one} vs {other}");
        }
    }

    #[test]
    fn token_hash_ignores_ordinary_comments_and_whitespace_around_literals() {
        let reference = source_hash("fn f() -> u32 { let _u = \"https://example.invalid\"; 1 }");
        for variant in [
            "fn f() -> u32 {\n    let _u = \"https://example.invalid\"; // trailing note\n    1\n}",
            "fn f() -> u32 { /* block /* nested */ note */ let _u = \"https://example.invalid\"; 1 }",
        ] {
            assert_eq!(source_hash(variant), reference, "{variant}");
        }
    }

    #[test]
    fn normalized_tokens_keep_literals_whole_and_drop_comments() {
        let normalized = normalize_tokens(
            "fn f() { /* gone */ let _a = b\"x  // y\"; // gone too\n let _b = '\\''; let _c = br#\"raw \" //\"#; 'outer: loop {} }",
        );
        for kept in ["b\"x  // y\"", "'\\''", "br#\"raw \" //\"#"] {
            assert!(normalized.contains(kept), "{kept} missing from {normalized}");
        }
        assert!(!normalized.contains("gone"), "comment text in {normalized}");
    }

    #[test]
    fn unicode_token_runs_preserve_byte_c_and_raw_literal_prefixes() {
        let normalized = normalize_tokens(
            "fn 漢字() { let _b = b\"x // y\"; let _c = c\"x /* y\"; let _br = br#\"漢字 //\"#; let _cr = cr#\"漢字 /*\"#; let _ch = b'\\''; }",
        );
        for literal in [
            "b\"x // y\"",
            "c\"x /* y\"",
            "br#\"漢字 //\"#",
            "cr#\"漢字 /*\"#",
            "b'\\''",
        ] {
            assert!(normalized.contains(literal), "{literal} missing from {normalized}");
        }
        assert!(normalized.contains("fn 漢字 ( )"), "Unicode name was split: {normalized}");
    }

    #[test]
    fn marker_is_not_part_of_function_span() {
        let source = "// octet-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("fn f"));
    }

    #[test]
    fn legacy_marker_is_not_part_of_function_span() {
        let source = "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("fn f"));
    }

    #[test]
    fn marker_before_attributes_attaches_to_function() {
        let source =
            "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\npub const fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(
            normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] pub const fn f")
        );
    }

    #[test]
    fn marker_looking_text_inside_comments_and_literals_never_attaches() {
        for source in [
            "/* outer /* nested */\n// octet-fingerprint: b3:abc; review=R; evidence=E\n*/ pub fn f() -> u32 { 1 }\n",
            "/*\n// slotcar-fingerprint: b3:abc; review=R; evidence=E\n*/ pub fn f() -> u32 { 1 }\n",
            "/*\n#[fingerprint(b3:abc, review=\"R\", evidence=\"E\")]\n*/ pub fn f() -> u32 { 1 }\n",
            "const TEXT: &str = r#\"\n// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n\"#; pub fn f() -> u32 { 1 }\n",
            "const TEXT: &str = r#\"\n#[fingerprint(b3:abc, review=\"R\", evidence=\"E\")]\n\"#; pub fn f() -> u32 { 1 }\n",
        ] {
            let functions = find_functions(source);
            assert_eq!(functions.len(), 1, "{source}");
            assert!(functions[0].marker.is_none(), "{source}");
        }
        let real =
            "    // octet-fingerprint: b3:abc; review=R; evidence=E\n    #[inline]\n\n    pub fn f() -> u32 { 1 }\n";
        assert!(find_functions(real)[0].marker.is_some());
        let real_attribute = "    #[fingerprint(b3:abc, review=\"R\", evidence=\"E\")]\n    fn f() -> u32 { 1 }\n";
        assert!(find_functions(real_attribute)[0].marker.is_some());
    }

    #[test]
    fn marker_lexical_state_stays_correct_across_multiple_functions() {
        let source = concat!(
            "// octet-fingerprint: b3:abc; review=R; evidence=E\n",
            "pub fn first() -> u32 { 1 }\n",
            "/* outer /* nested */\n",
            "// octet-fingerprint: b3:abc; review=R; evidence=E\n",
            "*/ pub fn forged() -> u32 { 2 }\n",
            "const TEXT: &str = r#\"\n",
            "#[fingerprint(b3:abc, review=\"R\", evidence=\"E\")]\n",
            "\"#; pub fn literal() -> u32 { 3 }\n",
            "// octet-fingerprint: b3:abc; review=R; evidence=E\n",
            "#[inline]\n",
            "pub fn last() -> u32 { 4 }\n",
        );
        let functions = find_functions(source);
        assert_eq!(functions.iter().map(|function| function.name.as_str()).collect::<Vec<_>>(), [
            "first", "forged", "literal", "last"
        ]);
        assert_eq!(functions.iter().map(|function| function.marker.is_some()).collect::<Vec<_>>(), [
            true, false, false, true
        ]);
        assert_eq!(functions.iter().map(|function| function.preceding_marker_in_code).collect::<Vec<_>>(), [
            true, false, false, true
        ]);
    }

    #[test]
    fn separated_outer_attributes_belong_to_function_but_markers_and_prior_items_do_not() {
        let source = "const OTHER: u32 = 7;\n// octet-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\n#[unsafe(export_name = \"alpha\")]\n/* explanation\n   of the export */\n\npub extern \"C\" fn exported() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        let span = &functions[0];
        let tokens = normalize_tokens(&source[span.start..span.end]);
        assert!(
            tokens.starts_with("# [ inline ] # [ unsafe ( export_name = \"alpha\" ) ] pub extern \"C\" fn exported")
        );
        assert!(!tokens.contains("OTHER"));
        assert!(!tokens.contains("octet-fingerprint"));
        assert!(span.marker.is_some(), "marker immediately above attached attribute must remain associated");
    }

    #[test]
    fn insertion_start_includes_visibility_before_fn_keyword() {
        let source = "#[inline]\npub fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_none());
        assert_eq!(functions[0].start, 0);
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] pub fn f"));
    }

    #[test]
    fn marker_before_multiline_attribute_attaches_to_function() {
        let source = "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\n#[allow(\n    ambient_clock,\n    reason = \"clock boundary\"\n)]\npub fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] # [ allow"));
    }

    #[test]
    fn multiline_export_attribute_with_literal_only_line_and_nested_brackets_stays_attached() {
        let source = "const PRIOR: [u8; 1] = [7];\n// octet-fingerprint: b3:abc; review=R; evidence=E\n#[cfg_attr(any(),\n    doc = [\"unused\"]\n)]\n#[doc = \"literal [brackets]\"]\n#[unsafe(export_name =\n    \"alpha\" /* ignored ] */\n)]\n// gap before function\npub extern \"C\" fn exported() -> u32 { 1 }\n";
        let spans = find_functions(source);
        assert_eq!(spans.len(), 1);
        let span = &spans[0];
        assert!(span.marker.is_some());
        let tokens = normalize_tokens(&source[span.start..span.end]);
        assert_eq!(span.start, source.find("#[cfg_attr").expect("first outer attribute"));
        assert!(!source[span.start..span.end].contains("octet-fingerprint"));
        assert!(
            tokens.starts_with("# [ cfg_attr ( any ( ) , doc = [ \"unused\" ] ) ] # [ doc = \"literal [brackets]\" ]")
        );
        assert!(tokens.contains("# [ unsafe ( export_name = \"alpha\" ) ]"));
        assert!(tokens.contains("pub extern \"C\" fn exported"));
        assert!(!tokens.contains("PRIOR"));
    }

    #[test]
    fn split_visibility_and_modifiers_are_part_of_the_marked_item() {
        let source = "const PRIOR: u32 = 7;\n// octet-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\npub(crate)\n// modifier gap\nunsafe\nextern \"C\"\nfn exported() -> u32 { 1 }\n";
        let spans = find_functions(source);
        assert_eq!(spans.len(), 1);
        let span = &spans[0];
        assert!(span.marker.is_some());
        let tokens = normalize_tokens(&source[span.start..span.end]);
        assert_eq!(span.start, source.find("#[inline]").expect("first outer attribute"));
        assert!(!source[span.start..span.end].contains("octet-fingerprint"));
        assert!(tokens.starts_with("# [ inline ] pub ( crate ) unsafe extern \"C\" fn exported"));
        assert!(!tokens.contains("PRIOR"));
    }

    #[test]
    fn macro_detection_ignores_unary_not_and_inequality() {
        assert!(contains_macro_call("{ assert!(value != 0); }"));
        assert!(contains_macro_call("{ path::macro_name! { value } }"));
        assert!(!contains_macro_call("{ if !is_ready && value != 0 { return true; } }"));
        assert!(!contains_macro_call("{ let message = \"not a macro!\"; }"));
    }

    #[test]
    fn macro_detection_needs_a_macro_name_outside_comments_and_literals() {
        for code in [
            "{ if !(ready) { return; } }",
            "{ while !(done) {} }",
            "{ return !(flag); }",
            "{ match !(flag) { _ => {} } }",
            "{ let _quoted = \"fake!(x)\"; let _bang = '!'; }",
            "{ let _raw = r#\"fake![x]\"#; }",
            "{ /* fake!(x) */ // fake!{x}\n }",
        ] {
            assert!(!contains_macro_call(code), "not a macro call: {code}");
        }
        for code in [
            "{ assert!(ready); }",
            "{ let _list = vec![1]; }",
            "{ let _text = format!(\"{ready}\"); }",
            "{ std::println!(\"ready\"); }",
            "{ if !matches!(ready, Some(_)) {} }",
            "{ let _url = \"https://example.invalid\"; assert!(ready); }",
        ] {
            assert!(contains_macro_call(code), "macro call: {code}");
        }
    }

    #[test]
    fn macro_generated_functions_are_not_source_fingerprinted() {
        let source = "macro_rules! define_checked {\n    ($name:ident) => {\n        pub fn $name(input: u32) -> u32 { input + 1 }\n    };\n}\n\ndefine_checked!(generated_checked);\n\n// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\npub fn explicit_wrapper(input: u32) -> u32 { generated_checked(input) }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "explicit_wrapper");
        assert!(functions[0].marker.is_some());
    }

    #[test]
    fn keyword_delimiters_and_unicode_raw_names_cover_executable_functions() {
        let source = r##"//! fn fake_doc() {}
/* fn fake_block() {} */
macro_rules! generated { () => { fn fake_generated() {} }; }
pub fn control() -> u8 {
    let 漢fn = 0;
    let _quoted = "fn fake_quoted() {}";
    let _raw = r#"fn fake_raw() {}"#;
    1
}
pub fn	foo() -> u8 { 2 }
pub fn
bar() -> u8 { 3 }
pub fn /* comment */ baz() -> u8 { 4 }
pub fn r#type() -> u8 { 5 }
pub fn 漢字() -> u8 { 6 }
trait Contract { fn declaration(); }
pub fn sum() -> u8 { control() + foo() + bar() + baz() + r#type() + 漢字() }
"##;
        let functions = find_functions(source);
        let names = functions.iter().map(|span| span.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["control", "foo", "bar", "baz", "r#type", "漢字", "sum"]);
        for (span, name) in functions.iter().zip(names) {
            let function = &source[span.start..span.end];
            assert!(
                function.contains(&format!("fn {name}"))
                    || function.contains(&format!("fn\t{name}"))
                    || function.contains(&format!("fn\n{name}"))
                    || function.contains(&format!("fn /* comment */ {name}"))
            );
            assert!(function.ends_with('}'), "incomplete body for {name}: {function}");
        }
    }

    #[test]
    fn marker_before_comment_separated_raw_name_attaches() {
        let source = "// octet-fingerprint: b3:abc; review=R; evidence=E\npub fn /* comment */ r#type() -> u8 { 5 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "r#type");
        assert!(functions[0].marker.is_some());
        assert!(source[functions[0].start..functions[0].end].starts_with("pub fn /* comment */ r#type"));
    }

    #[test]
    fn literal_functions_inside_macro_rules_are_not_source_fingerprinted() {
        let source = "macro_rules! define_literal {\n    () => {\n        pub fn generated_literal() -> u32 { 1 }\n    };\n}\n\npub fn explicit_function() -> u32 { 2 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "explicit_function");
    }

    #[test]
    fn trait_declarations_do_not_swallow_later_impl_methods() {
        let source = "trait T {\n    fn f(&self) -> u64;\n}\nimpl T for S {\n    // tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n    #[inline]\n    fn f(&self) -> u64 { 1 }\n}\nstruct S;\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] fn f"));
    }

    #[test]
    fn documentation_examples_are_not_functions() {
        let source = r#"//! fn inner_doc() {}
/// ```compile_fail
/// fn forge(claim: Claim) -> Witness { Witness { claim } }
/// ```
///
/// ```compile_fail
/// fn forge(table: &Table) {
///     table.receipt();
/// }
/// ```
/** fn block_doc() { */
#[must_use]
pub fn real(value: u32) -> u32 {
    value
}
"#;
        let functions = find_functions(source);
        let names = functions.iter().map(|span| span.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["real"]);
        let real = &source[functions[0].start..functions[0].end];
        assert!(real.starts_with("#[must_use]\npub fn real"));
        assert!(real.ends_with("value\n}"));
    }

    #[test]
    fn literals_and_nested_comments_neither_create_nor_end_functions() {
        let source = r##"/* outer /* nested fn fake_nested() { */ still fn fake_outer() { */
fn quoted() -> &'static str {
    let _url = "https://example.invalid/{";
    let _raw = r#"fn fake_raw() { " } "#;
    let _quote = '"';
    let _brace = '}';
    "// fn fake_line() {"
}
fn after<'a>(value: &'a str) -> &'a str {
    value // a closing } in a comment
}
"##;
        let functions = find_functions(source);
        let names = functions.iter().map(|span| span.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["quoted", "after"]);
        let quoted_start = source.find("fn quoted").expect("quoted function");
        let quoted_end = source.find("}\nfn after").expect("quoted closing brace") + 1;
        assert_eq!((functions[0].start, functions[0].end), (quoted_start, quoted_end));
        assert_eq!(functions[1].end, source.trim_end().len());
    }

    #[test]
    fn comments_inside_or_naming_macro_definitions_do_not_change_functions() {
        let definition = r#"macro_rules! define {
    // fn not_real() {
    ($name:ident) => { pub fn $name() -> u32 { 1 } };
}
pub fn explicit() -> u32 { 2 }
"#;
        let names = find_functions(definition).into_iter().map(|span| span.name).collect::<Vec<_>>();
        assert_eq!(names, ["explicit"]);

        let mention = r#"impl Widget {
    // a macro_rules! sketch {
    fn method(&self) -> u32 { 3 }
}
pub fn explicit() -> u32 { 4 }
"#;
        let names = find_functions(mention).into_iter().map(|span| span.name).collect::<Vec<_>>();
        assert_eq!(names, ["method", "explicit"]);
    }

    #[test]
    fn unicode_type_const_generic_signature_keeps_the_real_body() {
        let source = "struct 漢字<const N: usize>;\nfn create() -> 漢字<{ 2 }> { 漢字 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "create");
        assert_eq!(&source[functions[0].start..functions[0].end], "fn create() -> 漢字<{ 2 }> { 漢字 }");
    }

    #[test]
    fn array_and_const_generic_signatures_keep_their_bodies() {
        let source = r#"trait Encode {
    fn encode(&self, out: &mut [u8; 4]) -> [u8; 2];
    fn width(&self) -> usize { 2 }
}
fn pair(bytes: [u8; 2]) -> bool {
    bytes[0] == bytes[1]
}
impl Plan {
    fn lanes(&self) -> Result<&[Option<u32>; LANES], Error> {
        fn nested() {}
        Ok(&self.lanes)
    }
}
fn shaped<const N: usize>() -> Wrapper<{ N + 1 }>
where
    [(); N]: Sized,
{
    Wrapper
}
extern "C" {
    fn external(buffer: *mut [u8; 8]);
}
"#;
        let functions = find_functions(source);
        let names = functions.iter().map(|span| span.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["width", "pair", "lanes", "shaped"]);
        let text = |index: usize| &source[functions[index].start..functions[index].end];
        assert!(text(1).starts_with("fn pair(bytes: [u8; 2])") && text(1).ends_with("bytes[1]\n}"));
        assert!(text(2).trim_start().starts_with("fn lanes") && text(2).ends_with("Ok(&self.lanes)\n    }"));
        assert!(text(3).starts_with("fn shaped") && text(3).ends_with("Wrapper\n}"));

        // Comparisons in a macro's proof clauses are not generic arguments.
        let proof = "verus! {\npub proof fn ordered(value: nat)\n    ensures\n        value <= value, value < value + 1,\n{\n}\n}\nfn main() {}\n";
        let names = find_functions(proof).into_iter().map(|span| span.name).collect::<Vec<_>>();
        assert_eq!(names, ["ordered", "main"]);
    }
}
