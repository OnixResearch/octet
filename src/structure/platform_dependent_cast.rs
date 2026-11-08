use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::DefKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;
use rustc_span::sym;
use rustc_trait_selection::infer::InferCtxtExt;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `as usize` and `as isize` casts from fixed-width integer
    /// types. On a 32-bit platform, `u64 as usize` silently truncates.
    /// Tiger Style mandates explicitly sized types for cross-platform
    /// consistency.
    pub tigerstyle::PLATFORM_DEPENDENT_CAST,
    Warn,
    "`as usize`/`as isize` from fixed-width integer is platform-dependent"
}

declare_lint_pass!(PlatformDependentCast => [PLATFORM_DEPENDENT_CAST]);

fn is_fixed_width_int(ty: ty::Ty<'_>) -> bool {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width().is_some(),
        ty::Uint(uint_ty) => uint_ty.bit_width().is_some(),
        _ => false,
    }
}

fn is_platform_sized(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(rustc_middle::ty::IntTy::Isize) | ty::Uint(rustc_middle::ty::UintTy::Usize))
}

fn ty_name(ty: ty::Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

/// How the enclosing body can propagate a failed `try_from` conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConversionFailurePath {
    /// Returns `Result<_, E>` with `E: From<TryFromIntError>`: `?` compiles.
    ResultQuestionMark,
    /// Returns `Option<_>`: `.ok()?` compiles.
    OptionQuestionMark,
    /// Returns `Result<_, E>` without a conversion from the cast error.
    ResultWithOtherError,
    /// `?` is not available in the enclosing body.
    Unavailable,
}

/// `From` impls that std provides on every platform, so the conversion is
/// lossless and needs no failure handling.
fn has_lossless_from(source_ty: ty::Ty<'_>, target_ty: ty::Ty<'_>) -> bool {
    match target_ty.kind() {
        ty::Uint(ty::UintTy::Usize) => matches!(source_ty.kind(), ty::Uint(ty::UintTy::U8 | ty::UintTy::U16)),
        ty::Int(ty::IntTy::Isize) => {
            matches!(source_ty.kind(), ty::Int(ty::IntTy::I8 | ty::IntTy::I16) | ty::Uint(ty::UintTy::U8))
        }
        _ => false,
    }
}

/// Return type of the innermost body (function, closure, or async block)
/// that a `?` inside `cx.enclosing_body` would return from.
fn enclosing_return_ty<'tcx>(cx: &LateContext<'tcx>) -> Option<ty::Ty<'tcx>> {
    let owner = cx.tcx.hir_body_owner_def_id(cx.enclosing_body?);
    match cx.tcx.def_kind(owner) {
        DefKind::Fn | DefKind::AssocFn => {
            Some(cx.tcx.instantiate_bound_regions_with_erased(cx.tcx.fn_sig(owner).instantiate_identity().output()))
        }
        DefKind::Closure => match cx.typeck_results().node_type(cx.tcx.local_def_id_to_hir_id(owner)).kind() {
            ty::Closure(_, args) => {
                Some(cx.tcx.instantiate_bound_regions_with_erased(args.as_closure().sig().output()))
            }
            ty::Coroutine(_, args) => Some(args.as_coroutine().return_ty()),
            _ => None,
        },
        _ => None,
    }
}

/// `<target as TryFrom<source>>::Error`, normalized.
fn try_from_error_ty<'tcx>(
    cx: &LateContext<'tcx>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> Option<ty::Ty<'tcx>> {
    let try_from = cx.tcx.get_diagnostic_item(sym::TryFrom)?;
    let error = cx
        .tcx
        .associated_items(try_from)
        .filter_by_name_unhygienic_and_kind(Symbol::intern("Error"), ty::AssocTag::Type)
        .next()?;
    let projection = ty::Ty::new_projection(cx.tcx, error.def_id, [target_ty, source_ty]);
    cx.tcx.try_normalize_erasing_regions(cx.typing_env(), projection).ok()
}

fn error_converts_from<'tcx>(cx: &LateContext<'tcx>, error_ty: ty::Ty<'tcx>, source_error_ty: ty::Ty<'tcx>) -> bool {
    let Some(from) = cx.tcx.get_diagnostic_item(sym::From) else {
        return false;
    };
    let error_ty = cx.tcx.erase_and_anonymize_regions(error_ty);
    let (infcx, param_env) = cx.tcx.infer_ctxt().build_with_typing_env(cx.typing_env());
    infcx
        .type_implements_trait(from, [error_ty, source_error_ty], param_env)
        .must_apply_modulo_regions()
}

fn conversion_failure_path<'tcx>(
    cx: &LateContext<'tcx>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> ConversionFailurePath {
    let Some(return_ty) = enclosing_return_ty(cx) else {
        return ConversionFailurePath::Unavailable;
    };
    let ty::Adt(adt, args) = return_ty.kind() else {
        return ConversionFailurePath::Unavailable;
    };
    if cx.tcx.is_diagnostic_item(sym::Option, adt.did()) {
        return ConversionFailurePath::OptionQuestionMark;
    }
    if !cx.tcx.is_diagnostic_item(sym::Result, adt.did()) {
        return ConversionFailurePath::Unavailable;
    }
    match try_from_error_ty(cx, source_ty, target_ty) {
        Some(source_error_ty) if error_converts_from(cx, args.type_at(1), source_error_ty) => {
            ConversionFailurePath::ResultQuestionMark
        }
        _ => ConversionFailurePath::ResultWithOtherError,
    }
}

fn platform_cast_suggestion<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &hir::Expr<'_>,
    inner: &hir::Expr<'_>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> Option<LintSuggestion> {
    // Neither `From::from` nor `TryFrom::try_from` is callable in const contexts.
    if cx
        .enclosing_body
        .is_some_and(|body| cx.tcx.hir_body_const_context(cx.tcx.hir_body_owner_def_id(body)).is_some())
    {
        return None;
    }
    let inner = expr_snippet(cx, inner)?;
    let target_name = ty_name(target_ty);
    if has_lossless_from(source_ty, target_ty) {
        return Some(LintSuggestion::Single {
            span: expr.span,
            message: format!("replace with lossless `{target_name}::from(...)`"),
            replacement: format!("{target_name}::from({inner})"),
            applicability: Applicability::MachineApplicable,
        });
    }
    let (replacement, applicability) = match conversion_failure_path(cx, source_ty, target_ty) {
        ConversionFailurePath::ResultQuestionMark => {
            (format!("{target_name}::try_from({inner})?"), Applicability::MaybeIncorrect)
        }
        ConversionFailurePath::OptionQuestionMark => {
            (format!("{target_name}::try_from({inner}).ok()?"), Applicability::MaybeIncorrect)
        }
        ConversionFailurePath::ResultWithOtherError => (
            format!("{target_name}::try_from({inner}).map_err(|_| /* error */)?"),
            Applicability::HasPlaceholders,
        ),
        ConversionFailurePath::Unavailable => (
            format!("{target_name}::try_from({inner}).unwrap_or_else(|_| /* handle out-of-range value */)"),
            Applicability::HasPlaceholders,
        ),
    };
    Some(LintSuggestion::Single {
        span: expr.span,
        message: format!("replace with `{target_name}::try_from(...)`"),
        replacement,
        applicability,
    })
}

impl<'tcx> LateLintPass<'tcx> for PlatformDependentCast {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        // Fire when casting from fixed-width to platform-sized
        if is_fixed_width_int(source_ty) && is_platform_sized(target_ty) {
            let source_name = ty_name(source_ty);
            let target_name = ty_name(target_ty);

            let message = format!("`as {target_name}` from `{source_name}` is platform-dependent");
            if let Some(suggestion) = platform_cast_suggestion(cx, expr, inner, source_ty, target_ty) {
                emit_tiger_style_lint_with_suggestion(
                    cx,
                    PLATFORM_DEPENDENT_CAST,
                    expr.span,
                    message,
                    "platform_dependent_cast",
                    suggestion,
                );
            } else {
                emit_tiger_style_lint(cx, PLATFORM_DEPENDENT_CAST, expr.span, message, "platform_dependent_cast");
            }
        }
    }
}
