//! Admission of custom policy attributes, never names found in attribute payloads.
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_span::Symbol;

pub(super) fn has_marker(cx: &LateContext<'_>, def_id: hir::def_id::LocalDefId, marker: &str) -> bool {
    let Some(path) = marker
        .split("::")
        .map(|segment| (!segment.is_empty()).then(|| Symbol::intern(segment)))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let hir_id = cx.tcx.local_def_id_to_hir_id(def_id);
    cx.tcx
        .hir_attrs(hir_id)
        .iter()
        .any(|attr| matches!(attr, hir::Attribute::Unparsed(_)) && attr.path_matches(&path))
}
