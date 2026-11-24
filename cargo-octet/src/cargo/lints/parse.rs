const MAX_EXCERPT_BYTES: usize = 4_096;
const LINT_NOTE_PREFIX: &str = "= note: `cargo::";
const SPAN_ARROW_PREFIX: &str = "--> ";
const WARNING_PREFIX: &str = "warning: ";
const ERROR_PREFIX: &str = "error: ";
const GENERATED_MARKER: &str = ") generated ";
const COMPILE_ERROR_PREFIX: &str = "error: could not compile ";
const FINALIZE_ERROR_PREFIX: &str = "error: could not finalize ";
const UNUSED_DEPENDENCIES_LINT: &str = "cargo::unused_dependencies";
const UNKNOWN_LINTS_LINT: &str = "cargo::unknown_lints";
const DEFAULT_LINE_COLUMN: u32 = 1;
const MIN_COLUMN_SPLIT_RADIX: u32 = 10;
#[rustfmt::skip]
const PROGRESS_PREFIXES: &[&str] = &[
    "    Checking ", "   Compiling ", "    Finished ", "       Fresh ", "       Dirty ",
    "    Blocking ", "     Locking ", "    Updating ", "  Downloaded ", " Downloading ",
    " Documenting ", "     Running ", "    Removing ", "    Building ",
];

struct PendingBlock {
    severity: &'static str,
    message: String,
    raw_path: Option<String>,
    line: u32,
    column: u32,
    snippet: Option<String>,
    lint_name: Option<String>,
}

#[derive(Default)]
struct UnitLintState {
    has_unused_dependencies: bool,
    has_unknown_lints: bool,
}

impl UnitLintState {
    fn reset(&mut self) {
        self.has_unused_dependencies = false;
        self.has_unknown_lints = false;
    }

    fn record_lint(&mut self, lint_name: &str) {
        if lint_name == UNUSED_DEPENDENCIES_LINT {
            self.has_unused_dependencies = true;
        } else if lint_name == UNKNOWN_LINTS_LINT {
            self.has_unknown_lints = true;
        }
    }

    fn infer_lint_for_message(&self, message: &str) -> Option<String> {
        if self.has_unused_dependencies && message.starts_with("unused dependency ") {
            return Some(UNUSED_DEPENDENCIES_LINT.to_owned());
        }
        if self.has_unknown_lints && message.starts_with("unknown lint: ") {
            return Some(UNKNOWN_LINTS_LINT.to_owned());
        }
        None
    }
}

#[cfg(test)]
pub(crate) fn extract_stderr_report(stderr_text: &str, is_truncated: bool) -> super::StderrReport {
    extract_stderr_report_with_flags(stderr_text, super::CaptureIntegrity {
        is_truncated,
        has_invalid_utf8: false,
    })
}

pub(crate) fn extract_stderr_report_with_flags(
    stderr_text: &str,
    capture: super::CaptureIntegrity,
) -> super::StderrReport {
    let mut result = super::StderrReport {
        diagnostics: Vec::new(),
        has_unused_manifest_key_lints_cargo: false,
        has_unknown_cargo_lint: false,
        errors: super::StderrErrorSignals::default(),
        capture,
        raw_excerpt: String::new(),
    };
    let mut pending: Option<PendingBlock> = None;
    let mut unit_state = UnitLintState::default();
    for raw_line in stderr_text.lines() {
        let stripped = super::ansi::strip_escape_sequences(raw_line);
        let line = stripped.as_ref();
        record_boundary_signals(line, &mut result);
        if is_diagnostic_header(line) {
            flush_pending_block(pending.take(), &unit_state, &mut result);
            pending = start_pending_block(line);
            append_excerpt_line(&mut result.raw_excerpt, line);
        } else if is_summary_line(line) {
            flush_pending_block(pending.take(), &unit_state, &mut result);
            unit_state.reset();
            if is_significant_non_block_line(line) {
                append_excerpt_line(&mut result.raw_excerpt, line);
            }
        } else if is_progress_line(line) {
            flush_pending_block(pending.take(), &unit_state, &mut result);
        } else if let Some(block) = pending.as_mut() {
            append_block_line(block, line, &mut unit_state, &mut result.raw_excerpt);
        } else if is_significant_non_block_line(line) {
            append_excerpt_line(&mut result.raw_excerpt, line);
        }
    }
    flush_pending_block(pending.take(), &unit_state, &mut result);
    if (capture.has_invalid_utf8 || capture.is_truncated) && result.raw_excerpt.is_empty() {
        fill_fallback_excerpt(stderr_text, &mut result.raw_excerpt);
    }
    result
}

fn fill_fallback_excerpt(stderr_text: &str, excerpt: &mut String) {
    for raw_line in stderr_text.lines() {
        let stripped = super::ansi::strip_escape_sequences(raw_line);
        let trimmed = stripped.trim();
        if !trimmed.is_empty() {
            append_excerpt_line(excerpt, trimmed);
        }
        if excerpt.len() >= MAX_EXCERPT_BYTES {
            break;
        }
    }
}

fn record_boundary_signals(line: &str, result: &mut super::StderrReport) {
    let trimmed = line.trim();
    if trimmed.contains("unused manifest key") && trimmed.contains("lints.cargo") {
        result.has_unused_manifest_key_lints_cargo = true;
    }
    if trimmed.contains("unknown lint:") && trimmed.contains("unused_dependencies") {
        result.has_unknown_cargo_lint = true;
    }
    if trimmed.starts_with(COMPILE_ERROR_PREFIX) {
        result.errors.has_compile_error = true;
    }
    if trimmed.starts_with(FINALIZE_ERROR_PREFIX) {
        result.errors.has_manifest_finalize_error = true;
    }
    if trimmed.starts_with("error[") {
        result.errors.has_unrecognized_error = true;
    }
}

fn is_summary_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with(COMPILE_ERROR_PREFIX) || trimmed.starts_with(FINALIZE_ERROR_PREFIX) {
        return true;
    }
    if !(trimmed.starts_with(WARNING_PREFIX) || trimmed.starts_with(ERROR_PREFIX)) {
        return false;
    }
    trimmed.contains(GENERATED_MARKER)
        || trimmed.ends_with(" warning emitted")
        || trimmed.ends_with(" warnings emitted")
}

fn is_progress_line(line: &str) -> bool {
    PROGRESS_PREFIXES.iter().any(|prefix| line.starts_with(prefix))
}

fn is_diagnostic_header(line: &str) -> bool {
    !is_summary_line(line) && (line.starts_with(WARNING_PREFIX) || line.starts_with(ERROR_PREFIX))
}

fn start_pending_block(line: &str) -> Option<PendingBlock> {
    let (severity, msg) = if let Some(msg) = line.strip_prefix(WARNING_PREFIX) {
        ("warning", msg)
    } else {
        ("error", line.strip_prefix(ERROR_PREFIX)?)
    };
    Some(PendingBlock {
        severity,
        message: msg.trim().to_owned(),
        raw_path: None,
        line: DEFAULT_LINE_COLUMN,
        column: DEFAULT_LINE_COLUMN,
        snippet: None,
        lint_name: None,
    })
}

fn append_block_line(block: &mut PendingBlock, line: &str, unit_state: &mut UnitLintState, excerpt: &mut String) {
    let trimmed = line.trim();
    if let Some(location) = trimmed.strip_prefix(SPAN_ARROW_PREFIX) {
        let (path, line_num, col_num) = extract_span_location(location.trim());
        block.raw_path = Some(path);
        block.line = line_num;
        block.column = col_num;
        append_excerpt_line(excerpt, line);
        return;
    }
    if let Some(lint_name) = extract_lint_note(trimmed) {
        unit_state.record_lint(&lint_name);
        block.lint_name = Some(lint_name);
        append_excerpt_line(excerpt, line);
        return;
    }
    if block.snippet.is_none()
        && let Some(snippet_text) = extract_numbered_snippet(line)
    {
        block.snippet = Some(snippet_text);
    }
    if !trimmed.is_empty() {
        append_excerpt_line(excerpt, line);
    }
}

fn flush_pending_block(pending: Option<PendingBlock>, unit_state: &UnitLintState, result: &mut super::StderrReport) {
    let Some(block) = pending else {
        return;
    };
    let Some(lint_name) = block.lint_name.or_else(|| unit_state.infer_lint_for_message(&block.message)) else {
        if block.severity == "error" {
            result.errors.has_unrecognized_error = true;
        }
        return;
    };
    let Ok(validated) = crate::domain_types::LintName::parse(&lint_name) else {
        result.errors.has_unrecognized_error = true;
        return;
    };
    let raw_path = block.raw_path.unwrap_or_else(|| "unknown".to_owned());
    result.diagnostics.push(super::StderrDiagnostic {
        lint_name: validated.as_str().to_owned(),
        severity: block.severity,
        message: block.message,
        raw_path,
        line: block.line,
        column: block.column,
        snippet: block.snippet,
    });
}

fn extract_span_location(location: &str) -> (String, u32, u32) {
    let Some((before_col, col_str)) = location.rsplit_once(':') else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let Some((path_str, line_str)) = before_col.rsplit_once(':') else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    let (Ok(line_num), Ok(col_num)) = (
        u32::from_str_radix(line_str, MIN_COLUMN_SPLIT_RADIX),
        u32::from_str_radix(col_str, MIN_COLUMN_SPLIT_RADIX),
    ) else {
        return (location.to_owned(), DEFAULT_LINE_COLUMN, DEFAULT_LINE_COLUMN);
    };
    (path_str.to_owned(), line_num.max(DEFAULT_LINE_COLUMN), col_num.max(DEFAULT_LINE_COLUMN))
}

fn extract_lint_note(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(LINT_NOTE_PREFIX)?;
    let (suffix, _) = rest.split_once('`')?;
    (!suffix.is_empty()).then(|| format!("cargo::{suffix}"))
}

fn extract_numbered_snippet(line: &str) -> Option<String> {
    let (prefix, content) = line.split_once(" | ")?;
    let trimmed_prefix = prefix.trim();
    if trimmed_prefix.is_empty() || !trimmed_prefix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let trimmed_content = content.trim();
    (!trimmed_content.is_empty()).then(|| trimmed_content.to_owned())
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
