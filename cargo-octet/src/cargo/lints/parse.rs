const MAX_EXCERPT_BYTES: usize = 4_096;
const CARGO_LINT_NOTE_PREFIX: &str = "= note: `cargo::";
const SPAN_ARROW_PREFIX: &str = "--> ";
const WARNING_PREFIX: &str = "warning: ";
const ERROR_PREFIX: &str = "error: ";
const GENERATED_SUFFIX: &str = " generated ";
const COMPILE_ERROR_PREFIX: &str = "error: could not compile ";
const FINALIZE_ERROR_PREFIX: &str = "error: could not finalize ";
const DEFAULT_LINE_COLUMN: u32 = 1;
const MIN_COLUMN_SPLIT_RADIX: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedCargoDiagnostic {
    pub(crate) lint_name: String,
    pub(crate) severity: &'static str,
    pub(crate) message: String,
    pub(crate) raw_path: String,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) snippet: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StderrErrorKind {
    None,
    CompileError,
    ManifestFinalizeError,
    UnrecognizedError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedCargoStderr {
    pub(crate) diagnostics: Vec<ParsedCargoDiagnostic>,
    pub(crate) has_unused_manifest_key_lints_cargo: bool,
    pub(crate) has_unknown_cargo_lint: bool,
    pub(crate) is_truncated: bool,
    pub(crate) error_kind: StderrErrorKind,
    pub(crate) raw_excerpt: String,
}

struct PendingBlock {
    severity: &'static str,
    message: String,
    raw_path: Option<String>,
    line: u32,
    column: u32,
    snippet: Option<String>,
    lint_name: Option<String>,
}

pub(crate) fn parse_cargo_stderr(stderr_text: &str, is_truncated: bool) -> ParsedCargoStderr {
    let mut result = ParsedCargoStderr {
        diagnostics: Vec::new(),
        has_unused_manifest_key_lints_cargo: false,
        has_unknown_cargo_lint: false,
        is_truncated,
        error_kind: StderrErrorKind::None,
        raw_excerpt: String::new(),
    };
    let mut pending: Option<PendingBlock> = None;
    for line in stderr_text.lines() {
        record_boundary_signals(line, &mut result);
        if is_diagnostic_header(line) {
            flush_pending_block(pending.take(), &mut result);
            pending = start_pending_block(line);
            append_excerpt_line(&mut result.raw_excerpt, line);
            continue;
        }
        if let Some(block) = pending.as_mut() {
            append_block_line(block, line, &mut result.raw_excerpt);
            if line.trim_start().starts_with("help: ") {
                flush_pending_block(pending.take(), &mut result);
            }
            continue;
        }
        if is_significant_non_block_line(line) {
            append_excerpt_line(&mut result.raw_excerpt, line);
        }
    }
    flush_pending_block(pending.take(), &mut result);
    result
}

fn record_boundary_signals(line: &str, result: &mut ParsedCargoStderr) {
    let trimmed = line.trim();
    if trimmed.contains("unused manifest key") && trimmed.contains("lints.cargo") {
        result.has_unused_manifest_key_lints_cargo = true;
    }
    if trimmed.contains("unknown lint:") && trimmed.contains("unused_dependencies") {
        result.has_unknown_cargo_lint = true;
    }
    if trimmed.starts_with(COMPILE_ERROR_PREFIX) {
        result.error_kind = StderrErrorKind::CompileError;
    } else if trimmed.starts_with(FINALIZE_ERROR_PREFIX) && result.error_kind == StderrErrorKind::None {
        result.error_kind = StderrErrorKind::ManifestFinalizeError;
    } else if trimmed.starts_with("error[") && result.error_kind != StderrErrorKind::CompileError {
        result.error_kind = StderrErrorKind::UnrecognizedError;
    }
}

fn is_diagnostic_header(line: &str) -> bool {
    let is_summary = line.contains(") generated ")
        || line.starts_with(COMPILE_ERROR_PREFIX)
        || line.starts_with(FINALIZE_ERROR_PREFIX);
    if is_summary {
        return false;
    }
    if line.starts_with(WARNING_PREFIX) {
        return !line.contains(GENERATED_SUFFIX);
    }
    line.starts_with(ERROR_PREFIX)
}

fn start_pending_block(line: &str) -> Option<PendingBlock> {
    if let Some(msg) = line.strip_prefix(WARNING_PREFIX) {
        return Some(new_pending_block("warning", msg));
    }
    line.strip_prefix(ERROR_PREFIX).map(|msg| new_pending_block("error", msg))
}

fn new_pending_block(severity: &'static str, message: &str) -> PendingBlock {
    PendingBlock {
        severity,
        message: message.trim().to_owned(),
        raw_path: None,
        line: DEFAULT_LINE_COLUMN,
        column: DEFAULT_LINE_COLUMN,
        snippet: None,
        lint_name: None,
    }
}

fn append_block_line(block: &mut PendingBlock, line: &str, excerpt: &mut String) {
    let trimmed = line.trim();
    if let Some(location) = trimmed.strip_prefix(SPAN_ARROW_PREFIX) {
        let (path, line_num, col_num) = parse_span_location(location.trim());
        block.raw_path = Some(path);
        block.line = line_num;
        block.column = col_num;
        append_excerpt_line(excerpt, line);
        return;
    }
    if let Some(lint_name) = parse_cargo_lint_note(trimmed) {
        block.lint_name = Some(lint_name);
        append_excerpt_line(excerpt, line);
        return;
    }
    if block.snippet.is_none()
        && let Some(snippet_text) = parse_numbered_snippet_line(line)
    {
        block.snippet = Some(snippet_text);
    }
    if !trimmed.is_empty() {
        append_excerpt_line(excerpt, line);
    }
}

fn flush_pending_block(pending: Option<PendingBlock>, result: &mut ParsedCargoStderr) {
    let Some(block) = pending else {
        return;
    };
    let Some(lint_name) = block.lint_name else {
        if block.severity == "error" && result.error_kind != StderrErrorKind::CompileError {
            result.error_kind = StderrErrorKind::UnrecognizedError;
        }
        return;
    };
    let Ok(validated) = crate::domain_types::LintName::parse(&lint_name) else {
        if result.error_kind != StderrErrorKind::CompileError {
            result.error_kind = StderrErrorKind::UnrecognizedError;
        }
        return;
    };
    let raw_path = block.raw_path.unwrap_or_else(|| "unknown".to_owned());
    result.diagnostics.push(ParsedCargoDiagnostic {
        lint_name: validated.as_str().to_owned(),
        severity: block.severity,
        message: block.message,
        raw_path,
        line: block.line,
        column: block.column,
        snippet: block.snippet,
    });
}

fn parse_span_location(location: &str) -> (String, u32, u32) {
    let Some((before_col, col_str)) = location.rsplit_once(':') else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let Some((path_str, line_str)) = before_col.rsplit_once(':') else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let Ok(line_num) = u32::from_str_radix(line_str, MIN_COLUMN_SPLIT_RADIX) else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let Ok(col_num) = u32::from_str_radix(col_str, MIN_COLUMN_SPLIT_RADIX) else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let normalized_line = if line_num == 0 { DEFAULT_LINE_COLUMN } else { line_num };
    let normalized_col = if col_num == 0 { DEFAULT_LINE_COLUMN } else { col_num };
    (path_str.to_owned(), normalized_line, normalized_col)
}

fn parse_cargo_lint_note(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(CARGO_LINT_NOTE_PREFIX)?;
    let (suffix, _) = rest.split_once('`')?;
    if suffix.is_empty() {
        return None;
    }
    Some(format!("cargo::{suffix}"))
}

fn parse_numbered_snippet_line(line: &str) -> Option<String> {
    let (prefix, content) = line.split_once(" | ")?;
    let trimmed_prefix = prefix.trim();
    if trimmed_prefix.is_empty() || !trimmed_prefix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let trimmed_content = content.trim();
    if trimmed_content.is_empty() {
        return None;
    }
    Some(trimmed_content.to_owned())
}

fn is_significant_non_block_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with(WARNING_PREFIX)
        || trimmed.starts_with(ERROR_PREFIX)
        || trimmed.starts_with("error[")
        || trimmed.starts_with("consider passing")
}

fn append_excerpt_line(excerpt: &mut String, line: &str) {
    if excerpt.len() >= MAX_EXCERPT_BYTES {
        return;
    }
    if !excerpt.is_empty() {
        excerpt.push('\n');
    }
    let remaining = MAX_EXCERPT_BYTES.saturating_sub(excerpt.len());
    if line.len() <= remaining {
        excerpt.push_str(line);
        return;
    }
    let mut end = remaining;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    excerpt.push_str(&line[..end]);
}
