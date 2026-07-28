use octet_lint_metadata::nominal_domain::DescriptorKind;
use octet_lint_metadata::nominal_domain::DomainDescriptor;
use octet_lint_metadata::nominal_domain::DomainDisposition;
use octet_lint_metadata::nominal_domain::NominalDomainCatalog;
use octet_lint_metadata::nominal_domain::RawRepresentation;
use octet_lint_metadata::nominal_domain::classify_descriptor;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::intravisit::FnKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "raw_domain_value";
const MAX_SYMBOL_SEGMENTS: usize = 4;

// Detects registered core fields and API values that still use primitive
// storage. Compatibility scopes stay outside this lint's reportable set.
declare_tool_lint! {
    pub tigerstyle::RAW_DOMAIN_VALUE,
    Allow,
    "registered core domain value uses a primitive representation"
}

pub struct RawDomainValue {
    catalog: NominalDomainCatalog,
}

impl_lint_pass!(RawDomainValue => [RAW_DOMAIN_VALUE]);

impl RawDomainValue {
    #[must_use]
    pub fn new(catalog: NominalDomainCatalog) -> Self {
        assert!(!catalog.domains.is_empty());
        Self { catalog }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RawOccurrence {
    representation: RawRepresentation,
    kind: DescriptorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerKind {
    Sequence,
    Optional,
    Result,
    Map,
    Other,
}

fn container_kind(path: &str) -> ContainerKind {
    match path {
        "alloc::vec::Vec" | "std::vec::Vec" => ContainerKind::Sequence,
        "core::option::Option" | "std::option::Option" => ContainerKind::Optional,
        "core::result::Result" | "std::result::Result" => ContainerKind::Result,
        "alloc::collections::btree::map::BTreeMap" | "std::collections::BTreeMap" => ContainerKind::Map,
        _ => ContainerKind::Other,
    }
}

fn direct_raw_representation(cx: &LateContext<'_>, resolved: ty::Ty<'_>) -> Option<RawRepresentation> {
    match resolved.kind() {
        ty::Uint(ty::UintTy::U64) => Some(RawRepresentation::U64),
        ty::Uint(ty::UintTy::Usize) => Some(RawRepresentation::Usize),
        ty::Adt(definition, _) => match cx.tcx.def_path_str(definition.did()).as_str() {
            "alloc::string::String" | "std::string::String" => Some(RawRepresentation::String),
            "std::path::PathBuf" => Some(RawRepresentation::Path),
            _ => None,
        },
        _ => None,
    }
}

fn raw_occurrences<'tcx>(
    cx: &LateContext<'tcx>,
    resolved: ty::Ty<'tcx>,
    default_kind: DescriptorKind,
) -> Vec<RawOccurrence> {
    if let Some(representation) = direct_raw_representation(cx, resolved) {
        return vec![RawOccurrence {
            representation,
            kind: default_kind,
        }];
    }

    let ty::Adt(definition, args) = resolved.kind() else {
        return Vec::new();
    };
    let path = cx.tcx.def_path_str(definition.did());
    match container_kind(&path) {
        ContainerKind::Sequence | ContainerKind::Optional => {
            args.types().next().map_or_else(Vec::new, |inner| raw_occurrences(cx, inner, default_kind))
        }
        ContainerKind::Result => {
            args.types().next().map_or_else(Vec::new, |inner| raw_occurrences(cx, inner, default_kind))
        }
        ContainerKind::Map => {
            let mut types = args.types();
            let key_occurrences =
                types.next().map_or_else(Vec::new, |key| raw_occurrences(cx, key, DescriptorKind::MapKey));
            let value_occurrences =
                types.next().map_or_else(Vec::new, |value| raw_occurrences(cx, value, default_kind));
            key_occurrences.into_iter().chain(value_occurrences).collect()
        }
        ContainerKind::Other => Vec::new(),
    }
}

fn source_path(cx: &LateContext<'_>, span: Span) -> String {
    cx.sess()
        .source_map()
        .lookup_char_pos(span.lo())
        .file
        .name
        .prefer_local_unconditionally()
        .to_string_lossy()
        .into_owned()
}

fn field_descriptor_kind(cx: &LateContext<'_>, field: &hir::FieldDef<'_>) -> DescriptorKind {
    let parent = cx.tcx.parent(field.def_id.to_def_id());
    if matches!(cx.tcx.def_kind(parent), DefKind::Variant) {
        DescriptorKind::EnumPayload
    } else {
        DescriptorKind::Field
    }
}

fn symbol_candidates(path: &str) -> Vec<String> {
    let segments = path.split("::").collect::<Vec<_>>();
    let upper_bound = segments.len().min(MAX_SYMBOL_SEGMENTS);
    (1..=upper_bound).rev().map(|count| segments[segments.len() - count..].join("::")).collect()
}

fn parameter_symbol_candidates(function_path: &str, parameter: &str) -> Vec<String> {
    symbol_candidates(&format!("{function_path}::{parameter}"))
}

fn exact_domain<'a>(
    catalog: &NominalDomainCatalog,
    source_path: &'a str,
    symbols: &'a [String],
    occurrence: RawOccurrence,
) -> Option<(&'a str, String)> {
    for symbol in symbols {
        let descriptor = DomainDescriptor {
            symbol,
            raw_representation: occurrence.representation,
            source_path,
            kind: occurrence.kind,
        };
        match classify_descriptor(catalog, descriptor) {
            DomainDisposition::ExactCore { domain_id } => return Some((symbol, domain_id)),
            DomainDisposition::Compatibility { .. } | DomainDisposition::ExactOutsideScope { .. } => return None,
            DomainDisposition::HeuristicCandidate { .. } | DomainDisposition::Unclassified => {}
        }
    }
    None
}

fn report_occurrences(
    cx: &LateContext<'_>,
    lint: &'static rustc_lint::Lint,
    catalog: &NominalDomainCatalog,
    span: Span,
    source: &str,
    symbols: &[String],
    occurrences: &[RawOccurrence],
) {
    for occurrence in occurrences {
        let Some((symbol, domain_id)) = exact_domain(catalog, source, symbols, *occurrence) else {
            continue;
        };
        emit_tiger_style_lint(
            cx,
            lint,
            span,
            format!(
                "registered domain value `{symbol}` uses raw `{}` storage in core scope `{domain_id}`",
                occurrence.representation.as_str()
            ),
            LINT_NAME,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for RawDomainValue {
    fn check_field_def(&mut self, cx: &LateContext<'tcx>, field: &'tcx hir::FieldDef<'tcx>) {
        if field.span.from_expansion() {
            return;
        }
        let resolved = cx.tcx.type_of(field.def_id).instantiate_identity();
        let occurrences = raw_occurrences(cx, resolved, field_descriptor_kind(cx, field));
        if occurrences.is_empty() {
            return;
        }

        let source = source_path(cx, field.span);
        let path = cx.tcx.def_path_str(field.def_id.to_def_id());
        let symbols = symbol_candidates(&path);
        report_occurrences(cx, RAW_DOMAIN_VALUE, &self.catalog, field.span, &source, &symbols, &occurrences);
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        if matches!(kind, FnKind::Closure) || span.from_expansion() {
            return;
        }

        let source = source_path(cx, span);
        let function_path = cx.tcx.def_path_str(def_id.to_def_id());
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        for (parameter, resolved) in body.params.iter().zip(signature.inputs()) {
            let hir::PatKind::Binding(_, _, ident, _) = parameter.pat.kind else {
                continue;
            };
            let occurrences = raw_occurrences(cx, *resolved, DescriptorKind::Parameter);
            if occurrences.is_empty() {
                continue;
            }
            let symbols = parameter_symbol_candidates(&function_path, ident.name.as_str());
            report_occurrences(cx, RAW_DOMAIN_VALUE, &self.catalog, ident.span, &source, &symbols, &occurrences);
        }

        let hir::FnRetTy::Return(output) = decl.output else {
            return;
        };
        let occurrences = raw_occurrences(cx, signature.output(), DescriptorKind::Result);
        if occurrences.is_empty() {
            return;
        }
        let symbols = parameter_symbol_candidates(&function_path, "return");
        report_occurrences(cx, RAW_DOMAIN_VALUE, &self.catalog, output.span, &source, &symbols, &occurrences);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_paths_have_explicit_classification() {
        assert_eq!(container_kind("alloc::vec::Vec"), ContainerKind::Sequence);
        assert_eq!(container_kind("core::option::Option"), ContainerKind::Optional);
        assert_eq!(container_kind("core::result::Result"), ContainerKind::Result);
        assert_eq!(container_kind("alloc::collections::btree::map::BTreeMap"), ContainerKind::Map);
        assert_eq!(container_kind("fixture::DomainId"), ContainerKind::Other);
    }

    #[test]
    fn symbol_candidates_prefer_qualified_domain_names() {
        let candidates = symbol_candidates("crate_name::module::DynamicPlaceholder::UnitOutput::unit");

        assert_eq!(candidates[0], "module::DynamicPlaceholder::UnitOutput::unit");
        assert_eq!(candidates[1], "DynamicPlaceholder::UnitOutput::unit");
        assert_eq!(candidates.last().map(String::as_str), Some("unit"));
    }

    #[test]
    fn exact_domain_rejects_compatibility_and_outside_scopes() {
        let catalog = octet_lint_metadata::nominal_domain::load_nominal_domain_catalog().expect("embedded catalog");
        let symbols = vec!["DynamicUnit::id".to_owned()];
        let occurrence = RawOccurrence {
            representation: RawRepresentation::String,
            kind: DescriptorKind::Field,
        };

        assert!(exact_domain(&catalog, "crates/crunch-build/src/dynamic_plan.rs", &symbols, occurrence).is_some());
        assert!(exact_domain(&catalog, "crates/other/src/lib.rs", &symbols, occurrence).is_none());
    }
}
