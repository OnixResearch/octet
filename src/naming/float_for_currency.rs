use rustc_errors::DiagDecorator;
use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint;
use rustc_session::declare_lint_pass;

declare_lint! {
    /// Detects struct fields using `f32` or `f64` whose name contains
    /// `cost`, `price`, `budget`, `balance`, `amount`, `fee`, or `total_cost`.
    /// IEEE 754 loses precision above 2^53 — accumulated rounding across
    /// thousands of operations adds up. Use integer types with explicit
    /// units (e.g., `cost_microdollars: u64`).
    pub FLOAT_FOR_CURRENCY,
    Allow,
    "floating point type used for currency"
}

declare_lint_pass!(FloatForCurrency => [FLOAT_FOR_CURRENCY]);

const CURRENCY_WORDS: &[&str] = &["cost", "price", "budget", "balance", "amount", "fee", "total_cost"];

const SKIP_WORDS: &[&str] = &["percentage", "ratio", "factor", "rate"];

fn is_float_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Float(_))
}

impl<'tcx> LateLintPass<'tcx> for FloatForCurrency {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let ItemKind::Struct(_, _, variant_data) = &item.kind else {
            return;
        };

        for field in variant_data.fields() {
            let name = field.ident.name.as_str();
            let lower = name.to_ascii_lowercase();

            // Skip non-currency names
            if !CURRENCY_WORDS.iter().any(|w| lower.contains(w)) {
                continue;
            }

            // Skip legitimately float names
            if SKIP_WORDS.iter().any(|w| lower.contains(w)) {
                continue;
            }

            // Check if the field type is f32 or f64
            let field_ty = cx.tcx.type_of(field.def_id).instantiate_identity();
            if is_float_ty(field_ty) {
                cx.opt_span_lint(
                    FLOAT_FOR_CURRENCY,
                    Some(field.span),
                    DiagDecorator(|diag| {
                        diag.primary_message(format!("floating point type for currency field `{name}`",));
                        diag.help(
                            "use integer types with explicit units \
                             (e.g., `cost_microdollars: u64`) to avoid precision loss",
                        );
                    }),
                );
            }
        }
    }
}
