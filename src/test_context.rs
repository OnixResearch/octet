use rustc_hir as hir;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::find_attr;
use rustc_lint::LateContext;
use rustc_lint::LintContext;
use rustc_session::Session;
use rustc_span::Span;
use rustc_span::sym;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TruthDomain {
    can_be_true: bool,
    can_be_false: bool,
}

impl TruthDomain {
    const KNOWN_TRUE: Self = Self {
        can_be_true: true,
        can_be_false: false,
    };
    const KNOWN_FALSE: Self = Self {
        can_be_true: false,
        can_be_false: true,
    };
    const UNKNOWN: Self = Self {
        can_be_true: true,
        can_be_false: true,
    };

    fn all(parts: &[Self]) -> Self {
        Self {
            can_be_true: parts.iter().all(|part| part.can_be_true),
            can_be_false: parts.iter().any(|part| part.can_be_false),
        }
    }

    fn any(parts: &[Self]) -> Self {
        Self {
            can_be_true: parts.iter().any(|part| part.can_be_true),
            can_be_false: parts.iter().all(|part| part.can_be_false),
        }
    }

    fn not(self) -> Self {
        Self {
            can_be_true: self.can_be_false,
            can_be_false: self.can_be_true,
        }
    }
}

pub(crate) fn hir_attrs_indicate_test(sess: &Session, attrs: &[hir::Attribute]) -> bool {
    attrs.iter().any(|attr| hir_attr_indicates_test(sess, attr))
}

pub(crate) fn late_hir_id_is_in_test_context(cx: &LateContext<'_>, hir_id: hir::HirId, _span: Span) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        let in_test = match node {
            hir::Node::Item(item) => {
                hir_attrs_indicate_test(cx.sess(), cx.tcx.hir_attrs(item.hir_id()))
                    || item_span_indicates_test(cx.sess(), item.span)
                    || item_is_harness_test_fn(cx, item)
            }
            // `#[test]` cannot annotate associated functions, so only cfg gates apply here.
            hir::Node::ImplItem(impl_item) => {
                hir_attrs_indicate_test(cx.sess(), cx.tcx.hir_attrs(impl_item.hir_id()))
                    || item_span_indicates_test(cx.sess(), impl_item.span)
            }
            hir::Node::TraitItem(trait_item) => {
                hir_attrs_indicate_test(cx.sess(), cx.tcx.hir_attrs(trait_item.hir_id()))
                    || item_span_indicates_test(cx.sess(), trait_item.span)
            }
            _ => false,
        };
        if in_test {
            return true;
        }
    }

    false
}

/// Under `--test`, rustc's builtin `#[test]` macro consumes the attribute: the
/// function keeps no test attribute in HIR and is registered through a
/// same-named sibling `const` marked `#[rustc_test_marker]`. Without `--test`
/// the function is removed, so only harness crates can contain such markers.
fn item_is_harness_test_fn(cx: &LateContext<'_>, item: &hir::Item<'_>) -> bool {
    let hir::ItemKind::Fn { ident, .. } = item.kind else {
        return false;
    };
    if !cx.sess().opts.test {
        return false;
    }

    let def_id = item.owner_id.def_id;
    let marker = (cx.tcx.local_parent(def_id), ident.name);
    let module = cx.tcx.parent_module_from_def_id(def_id);
    let session = HarnessMarkerSession {
        crate_id: cx.tcx.stable_crate_id(rustc_span::def_id::LOCAL_CRATE),
        session_address: std::ptr::from_ref(cx.sess()).addr(),
    };
    HARNESS_MARKERS.with_borrow_mut(|cache| {
        if cache.session != Some(session) {
            *cache = HarnessMarkerCache {
                session: Some(session),
                modules: std::collections::HashMap::new(),
            };
        }
        cache.modules.entry(module).or_insert_with(|| module_harness_markers(cx, module)).contains(&marker)
    })
}

/// `(parent, name)` of every `#[rustc_test_marker]` const directly in one module.
type HarnessMarkers = std::collections::HashSet<(rustc_span::def_id::LocalDefId, rustc_span::Symbol)>;

#[derive(Clone, Copy, PartialEq, Eq)]
struct HarnessMarkerSession {
    crate_id: rustc_span::def_id::StableCrateId,
    session_address: usize,
}

#[derive(Default)]
struct HarnessMarkerCache {
    session: Option<HarnessMarkerSession>,
    modules: std::collections::HashMap<rustc_span::def_id::LocalModDefId, HarnessMarkers>,
}

// Lints run the test-context walk for most expressions. Rescanning every item of
// the enclosing module on each call made `--test` builds quadratic in module size,
// so each module's markers are collected once per compiler session.
std::thread_local! {
    static HARNESS_MARKERS: std::cell::RefCell<HarnessMarkerCache> =
        std::cell::RefCell::new(HarnessMarkerCache::default());
}

fn module_harness_markers(cx: &LateContext<'_>, module: rustc_span::def_id::LocalModDefId) -> HarnessMarkers {
    cx.tcx
        .hir_module_free_items(module)
        .filter_map(|id| {
            let candidate = cx.tcx.hir_item(id);
            let hir::ItemKind::Const(marker_ident, ..) = candidate.kind else {
                return None;
            };
            find_attr!(cx.tcx.hir_attrs(candidate.hir_id()), AttributeKind::RustcTestMarker(..))
                .then(|| (cx.tcx.local_parent(candidate.owner_id.def_id), marker_ident.name))
        })
        .collect()
}

pub(crate) fn item_span_indicates_test(sess: &Session, span: Span) -> bool {
    // Only the lines above the item matter. `span_to_lines` would materialize
    // every line of the item (a whole module for `mod tests`) on each call.
    let Ok(start) = sess.source_map().lookup_line(span.lo()) else {
        return false;
    };

    for idx in (0..start.line).rev() {
        let Some(line) = start.sf.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            if cfg_attr_source_requires_test(trimmed) {
                return true;
            }
            continue;
        }

        break;
    }

    false
}

fn hir_attr_indicates_test(sess: &Session, attr: &hir::Attribute) -> bool {
    if !attr.has_name(sym::cfg) {
        return false;
    }

    let Ok(source) = sess.source_map().span_to_snippet(attr.span()) else {
        return false;
    };

    cfg_attr_source_requires_test(&source)
}

fn cfg_attr_source_requires_test(source: &str) -> bool {
    let trimmed = source.trim();
    let inner = trimmed
        .strip_prefix("#[")
        .and_then(|rest| rest.strip_suffix(']'))
        .or_else(|| trimmed.strip_prefix("#![").and_then(|rest| rest.strip_suffix(']')))
        .unwrap_or(trimmed)
        .trim();

    let Some(cfg_expr) = inner.strip_prefix("cfg(").and_then(|rest| rest.strip_suffix(')')) else {
        return false;
    };

    !cfg_expr_truth(cfg_expr.trim(), false).can_be_true
}

fn cfg_expr_truth(expr: &str, test_enabled: bool) -> TruthDomain {
    let expr = expr.trim();

    if expr == "test" {
        return if test_enabled {
            TruthDomain::KNOWN_TRUE
        } else {
            TruthDomain::KNOWN_FALSE
        };
    }

    if let Some(args) = parse_cfg_call(expr, "all") {
        let parts = split_cfg_args(args).into_iter().map(|part| cfg_expr_truth(part, test_enabled)).collect::<Vec<_>>();
        return TruthDomain::all(&parts);
    }

    if let Some(args) = parse_cfg_call(expr, "any") {
        let parts = split_cfg_args(args).into_iter().map(|part| cfg_expr_truth(part, test_enabled)).collect::<Vec<_>>();
        return TruthDomain::any(&parts);
    }

    if let Some(args) = parse_cfg_call(expr, "not") {
        let parts = split_cfg_args(args);
        if let [part] = parts.as_slice() {
            return cfg_expr_truth(part, test_enabled).not();
        }
        return TruthDomain::UNKNOWN;
    }

    TruthDomain::UNKNOWN
}

fn parse_cfg_call<'a>(expr: &'a str, name: &str) -> Option<&'a str> {
    let rest = expr.strip_prefix(name)?.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    Some(inner)
}

fn split_cfg_args(source: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in source.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(source[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::cfg_attr_source_requires_test;
    use super::cfg_expr_truth;

    #[test]
    fn cfg_attr_detection_matches_test_only_forms() {
        assert!(cfg_attr_source_requires_test("#[cfg(test)]"));
        assert!(cfg_attr_source_requires_test("#![cfg(test)]"));
        assert!(cfg_attr_source_requires_test("#[cfg(all(test, feature = \"bench\"))]"));
        assert!(cfg_attr_source_requires_test("cfg(not(not(test)))"));
        assert!(!cfg_attr_source_requires_test("#[cfg(any(test, feature = \"bench\"))]"));
        assert!(!cfg_attr_source_requires_test("#[cfg(not(test))]"));
        assert!(!cfg_attr_source_requires_test("#![cfg(not(test))]"));
        assert!(!cfg_attr_source_requires_test("#[cfg(feature = \"testing\")]"));
        assert!(!cfg_attr_source_requires_test("#[allow(dead_code)]"));
    }

    #[test]
    fn cfg_not_test_can_still_be_true_without_test() {
        assert!(cfg_expr_truth("not(test)", false).can_be_true);
        assert!(!cfg_expr_truth("all(test, feature = \"bench\")", false).can_be_true);
    }
}
