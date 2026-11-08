use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::Variant;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects type names, enum variant names, and trait names containing
    /// multi-character uppercase acronyms like `RPCHandler` or `SQLQuery`.
    /// Tiger Style uses single-word capitalization: `RpcHandler`, `SqlQuery`.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// acronym_allowlist = ["IO", "ID"]
    /// ```
    pub tigerstyle::ACRONYM_STYLE,
    Warn,
    "acronym in type name should use single-word capitalization"
}

pub struct AcronymStyle {
    allowlist: Vec<String>,
}

impl AcronymStyle {
    pub fn new(allowlist: Vec<String>) -> Self {
        Self { allowlist }
    }
}

impl_lint_pass!(AcronymStyle => [ACRONYM_STYLE]);

/// Rewrite every acronym-style uppercase run (an acronym of 2+ uppercase
/// letters followed by a capitalized word, like `RPC` in `RPCHandler`) that
/// is not in `allowlist`. Returns `None` when nothing needs renaming.
fn suggest_acronym_fix(name: &str, allowlist: &[String]) -> Option<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut result = String::with_capacity(name.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_len = i - start;

            // The last uppercase char starts the next word, so the acronym
            // is `start..i - 1`; a one-letter word like `X` in `XAxis` is not
            // an acronym.
            let is_acronym = upper_len >= 3 && i < chars.len() && chars[i].is_ascii_lowercase();
            let acronym = &chars[start..(i - 1)];
            if is_acronym && !allowlist.iter().any(|a| a.chars().eq(acronym.iter().copied())) {
                // Keep first char uppercase, lowercase the acronym body,
                // keep the last uppercase char (it starts the next word)
                result.push(chars[start]);
                for c in &chars[(start + 1)..(i - 1)] {
                    result.push(c.to_ascii_lowercase());
                }
                result.push(chars[i - 1]);
            } else {
                for c in &chars[start..i] {
                    result.push(*c);
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    (result != name).then_some(result)
}

fn check_name(cx: &EarlyContext<'_>, name: &str, span: Span, allowlist: &[String]) {
    if let Some(suggested_name) = suggest_acronym_fix(name, allowlist) {
        emit_tiger_style_lint_with_suggestion(
            cx,
            ACRONYM_STYLE,
            span,
            format!("`{name}` has acronym-style capitalization"),
            "acronym_style",
            LintSuggestion::Single {
                span,
                message: format!("rename `{name}` to `{suggested_name}`"),
                replacement: suggested_name,
                applicability: Applicability::MaybeIncorrect,
            },
        );
    }
}

impl EarlyLintPass for AcronymStyle {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        match &item.kind {
            ItemKind::Struct(ident, ..) | ItemKind::Enum(ident, ..) => {
                check_name(cx, ident.name.as_str(), ident.span, &self.allowlist);
            }
            ItemKind::Trait(tr) => {
                check_name(cx, tr.ident.name.as_str(), tr.ident.span, &self.allowlist);
            }
            ItemKind::TyAlias(ta) => {
                check_name(cx, ta.ident.name.as_str(), ta.ident.span, &self.allowlist);
            }
            _ => {}
        }
    }

    fn check_variant(&mut self, cx: &EarlyContext<'_>, variant: &Variant) {
        check_name(cx, variant.ident.name.as_str(), variant.ident.span, &self.allowlist);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acronym_fix() {
        assert_eq!(suggest_acronym_fix("RPCHandler", &[]), Some("RpcHandler".into()));
        assert_eq!(suggest_acronym_fix("SQLQuery", &[]), Some("SqlQuery".into()));
        assert_eq!(suggest_acronym_fix("TLSConfig", &[]), Some("TlsConfig".into()));
        assert_eq!(suggest_acronym_fix("HTTPSServer", &[]), Some("HttpsServer".into()));
        assert_eq!(suggest_acronym_fix("IOError", &[]), Some("IoError".into()));
        assert_eq!(suggest_acronym_fix("HTTPRequestJSONBody", &[]), Some("HttpRequestJsonBody".into()));

        // Should NOT fire on these
        assert_eq!(suggest_acronym_fix("RpcHandler", &[]), None);
        assert_eq!(suggest_acronym_fix("Config", &[]), None);
        assert_eq!(suggest_acronym_fix("RPC", &[]), None);
        assert_eq!(suggest_acronym_fix("ID", &[]), None);
        assert_eq!(suggest_acronym_fix("Ok", &[]), None);
        assert_eq!(suggest_acronym_fix("Eq", &[]), None);
        // One-letter words are not acronyms
        assert_eq!(suggest_acronym_fix("XAxis", &[]), None);
        assert_eq!(suggest_acronym_fix("BTreeIndex", &[]), None);
    }

    #[test]
    fn test_acronym_allowlist_checks_every_run() {
        let allowlist = vec!["HTTP".to_string()];
        assert_eq!(suggest_acronym_fix("HTTPRequestJSONBody", &allowlist), Some("HTTPRequestJsonBody".into()));
        assert_eq!(suggest_acronym_fix("HTTPRequest", &allowlist), None);
        let allowlist = vec!["HTTP".to_string(), "JSON".to_string()];
        assert_eq!(suggest_acronym_fix("HTTPRequestJSONBody", &allowlist), None);
    }
}
