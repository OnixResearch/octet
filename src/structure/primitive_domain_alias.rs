use octet_lint_metadata::nominal_domain::DescriptorKind;
use octet_lint_metadata::nominal_domain::DomainDescriptor;
use octet_lint_metadata::nominal_domain::DomainDisposition;
use octet_lint_metadata::nominal_domain::EnforcementMode;
use octet_lint_metadata::nominal_domain::NominalDomainCatalog;
use octet_lint_metadata::nominal_domain::RawRepresentation;
use octet_lint_metadata::nominal_domain::classify_descriptor;
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "primitive_domain_alias";

// Detects registered domain aliases that still resolve to primitive storage.
// Exact policy rows identify enforceable core domains. Name suffixes produce
// discovery output only while the catalog remains in inventory mode.
declare_tool_lint! {
    pub tigerstyle::PRIMITIVE_DOMAIN_ALIAS,
    Allow,
    "registered domain alias resolves to a primitive representation"
}

pub struct PrimitiveDomainAlias {
    catalog: NominalDomainCatalog,
    mode: EnforcementMode,
}

impl_lint_pass!(PrimitiveDomainAlias => [PRIMITIVE_DOMAIN_ALIAS]);

impl PrimitiveDomainAlias {
    #[must_use]
    pub fn new(catalog: NominalDomainCatalog, mode: EnforcementMode) -> Self {
        assert!(!catalog.domains.is_empty());
        assert!(!catalog.heuristic_suffixes.is_empty());
        Self { catalog, mode }
    }
}

fn raw_representation(cx: &LateContext<'_>, resolved: ty::Ty<'_>) -> Option<RawRepresentation> {
    match resolved.kind() {
        ty::Uint(ty::UintTy::U64) => Some(RawRepresentation::U64),
        ty::Uint(ty::UintTy::Usize) => Some(RawRepresentation::Usize),
        ty::Adt(definition, _) => raw_adt_representation(cx, definition.did()),
        _ => None,
    }
}

fn raw_adt_representation(cx: &LateContext<'_>, definition: hir::def_id::DefId) -> Option<RawRepresentation> {
    let path = cx.tcx.def_path_str(definition);
    match path.as_str() {
        "alloc::string::String" | "std::string::String" => Some(RawRepresentation::String),
        "std::path::PathBuf" => Some(RawRepresentation::Path),
        _ => None,
    }
}

fn source_path(cx: &LateContext<'_>, span: rustc_span::Span) -> String {
    cx.sess()
        .source_map()
        .lookup_char_pos(span.lo())
        .file
        .name
        .prefer_local_unconditionally()
        .to_string_lossy()
        .into_owned()
}

fn alias_message(symbol: &str, disposition: &DomainDisposition) -> Option<String> {
    match disposition {
        DomainDisposition::ExactCore { domain_id } => {
            Some(format!("domain alias `{symbol}` resolves to primitive storage declared by `{domain_id}`"))
        }
        DomainDisposition::HeuristicCandidate { suffix } => Some(format!(
            "alias `{symbol}` ends with reviewed domain suffix `{suffix}` and needs policy classification"
        )),
        DomainDisposition::Compatibility { .. }
        | DomainDisposition::ExactOutsideScope { .. }
        | DomainDisposition::Unclassified => None,
    }
}

fn disposition_is_reportable(mode: EnforcementMode, disposition: &DomainDisposition) -> bool {
    match disposition {
        DomainDisposition::ExactCore { .. } => true,
        DomainDisposition::HeuristicCandidate { .. } => mode == EnforcementMode::Inventory,
        DomainDisposition::Compatibility { .. }
        | DomainDisposition::ExactOutsideScope { .. }
        | DomainDisposition::Unclassified => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for PrimitiveDomainAlias {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::TyAlias(ident, ..) = item.kind else {
            return;
        };
        if item.span.from_expansion() {
            return;
        }

        let resolved = cx.tcx.type_of(item.owner_id.def_id).instantiate_identity();
        let Some(raw_representation) = raw_representation(cx, resolved) else {
            return;
        };
        let symbol = ident.name.as_str();
        let source_path = source_path(cx, item.span);
        let descriptor = DomainDescriptor {
            symbol,
            raw_representation,
            source_path: &source_path,
            kind: DescriptorKind::TypeAlias,
        };
        let disposition = classify_descriptor(&self.catalog, descriptor);
        if !disposition_is_reportable(self.mode, &disposition) {
            return;
        }
        let Some(message) = alias_message(symbol, &disposition) else {
            return;
        };

        emit_tiger_style_lint(cx, PRIMITIVE_DOMAIN_ALIAS, ident.span, message, LINT_NAME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_reports_exact_and_heuristic_candidates() {
        let exact = DomainDisposition::ExactCore {
            domain_id: "fixture.unit-id".to_owned(),
        };
        let heuristic = DomainDisposition::HeuristicCandidate {
            suffix: "Id".to_owned(),
        };

        assert!(disposition_is_reportable(EnforcementMode::Inventory, &exact));
        assert!(disposition_is_reportable(EnforcementMode::Inventory, &heuristic));
        assert!(alias_message("UnitId", &exact).is_some());
        assert!(alias_message("SessionId", &heuristic).is_some());
    }

    #[test]
    fn promoted_modes_never_report_heuristic_candidates() {
        let heuristic = DomainDisposition::HeuristicCandidate {
            suffix: "Id".to_owned(),
        };

        assert!(!disposition_is_reportable(EnforcementMode::Warn, &heuristic));
        assert!(!disposition_is_reportable(EnforcementMode::Deny, &heuristic));
        assert!(disposition_is_reportable(EnforcementMode::Deny, &DomainDisposition::ExactCore {
            domain_id: "fixture.unit-id".to_owned()
        }));
    }

    #[test]
    fn compatibility_and_outside_scopes_are_silent() {
        let compatibility = DomainDisposition::Compatibility {
            domain_id: "fixture.unit-id".to_owned(),
        };
        let outside = DomainDisposition::ExactOutsideScope {
            domain_id: "fixture.unit-id".to_owned(),
        };

        assert!(!disposition_is_reportable(EnforcementMode::Inventory, &compatibility));
        assert!(!disposition_is_reportable(EnforcementMode::Inventory, &outside));
        assert!(alias_message("UnitId", &compatibility).is_none());
        assert!(alias_message("UnitId", &outside).is_none());
    }
}
