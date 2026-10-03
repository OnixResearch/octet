use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Rejects broad or unjustified raw-boolean API lint suppressions.
    pub tigerstyle::RAW_BOOL_PARAMETER_SUPPRESSION,
    Warn,
    "raw boolean API suppressions must be local, owned compatibility expectations"
}

#[derive(Clone, Copy)]
pub enum SuppressionEnforcement {
    Advisory,
    Strict,
}

pub struct RawBoolParameterSuppression {
    enforcement: SuppressionEnforcement,
}

impl RawBoolParameterSuppression {
    pub fn new(enforcement: SuppressionEnforcement) -> Self {
        Self { enforcement }
    }
}

impl_lint_pass!(RawBoolParameterSuppression => [RAW_BOOL_PARAMETER_SUPPRESSION]);

fn justified_reason(reason: &str) -> bool {
    let Some(rest) = reason.strip_prefix("external compatibility contract: ") else {
        return false;
    };
    let Some((contract, owner)) = rest.split_once("; owner: ") else {
        return false;
    };
    contract.trim().len() >= 12
        && !contract.contains("TODO")
        && !owner.is_empty()
        && owner.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn lint_attrs(cx: &LateContext<'_>, hir_id: hir::HirId, local_callable: bool, enforcement: SuppressionEnforcement) {
    for attr in cx.tcx.hir_attrs(hir_id) {
        let is_allow = attr.has_name(sym::allow);
        let is_expect = attr.has_name(sym::expect);
        if !is_allow && !is_expect {
            continue;
        }
        if attr.span().from_expansion() {
            continue;
        }
        let Some(items) = attr.meta_item_list() else {
            continue;
        };
        let mut names_raw_bool = false;
        let mut names_broad_group = false;
        for item in &items {
            let Some(meta) = item.meta_item() else {
                continue;
            };
            let names = &meta.path.segments;
            names_raw_bool |= matches!(names.as_slice(), [lint] if lint.ident.name.as_str() == "raw_bool_parameter")
                || matches!(
                    names.as_slice(),
                    [tool, lint] if tool.ident.name.as_str() == "tigerstyle"
                        && lint.ident.name.as_str() == "raw_bool_parameter"
                );
            names_broad_group |= matches!(names.as_slice(), [lint] if lint.ident.name.as_str() == "warnings")
                || matches!(
                    names.as_slice(),
                    [tool, group] if tool.ident.name.as_str() == "tigerstyle"
                        && group.ident.name.as_str() == "structure"
                );
        }
        if !names_raw_bool && !names_broad_group {
            continue;
        }
        let reason = items
            .iter()
            .filter_map(|item| item.meta_item())
            .find(|meta| meta.has_name(sym::reason))
            .and_then(|meta| meta.value_str());
        let message = if names_broad_group {
            "broad warning or structure-group suppression can hide `raw_bool_parameter`; use a local owned compatibility expectation"
        } else if is_allow {
            "`raw_bool_parameter` does not permit `#[allow]`; use a local, owned `#[expect]` for an unavoidable compatibility API"
        } else if !local_callable {
            "`raw_bool_parameter` expectations must attach to one exact callable, not a crate, module, or enclosing item"
        } else if !reason.is_some_and(|reason| justified_reason(reason.as_str())) {
            "`raw_bool_parameter` expectation requires `reason = \"external compatibility contract: <specific contract>; owner: <named owner>\"`"
        } else {
            continue;
        };
        match enforcement {
            SuppressionEnforcement::Advisory => emit_tiger_style_lint(
                cx,
                RAW_BOOL_PARAMETER_SUPPRESSION,
                attr.span(),
                message,
                "raw_bool_parameter_suppression",
            ),
            SuppressionEnforcement::Strict => {
                // A source-level `allow` can suppress even a forbidden lint;
                // an unconditional compiler error cannot be bypassed.
                cx.tcx.dcx().span_err(attr.span(), format!("tigerstyle::raw_bool_parameter_suppression: {message}"));
            }
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for RawBoolParameterSuppression {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        lint_attrs(cx, hir::CRATE_HIR_ID, false, self.enforcement);
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, hir::ItemKind::Fn { .. }), self.enforcement);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::ImplItem<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, hir::ImplItemKind::Fn(..)), self.enforcement);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, hir::TraitItemKind::Fn(..)), self.enforcement);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        lint_attrs(cx, expr.hir_id, false, self.enforcement);
    }

    fn check_stmt(&mut self, cx: &LateContext<'tcx>, stmt: &'tcx hir::Stmt<'tcx>) {
        lint_attrs(cx, stmt.hir_id, false, self.enforcement);
    }

    fn check_arm(&mut self, cx: &LateContext<'tcx>, arm: &'tcx hir::Arm<'tcx>) {
        lint_attrs(cx, arm.hir_id, false, self.enforcement);
    }
}

#[cfg(test)]
mod tests {
    use super::justified_reason;

    #[test]
    fn requires_owned_specific_contract() {
        assert!(justified_reason("external compatibility contract: frozen wire protocol v1; owner: api-team"));
        assert!(!justified_reason("external compatibility contract: TODO; owner: someone"));
        assert!(!justified_reason("temporary"));
    }
}
