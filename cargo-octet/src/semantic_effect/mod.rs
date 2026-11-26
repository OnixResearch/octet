//! Semantic MIR effect admission module (#17).
//!
//! Preserves the existing `selected-source-token-effect-closure` engine and provides an
//! explicitly scoped `semantic-mir-effect-closure` evaluator over compiler MIR observations.

pub mod admit;
pub mod classify;
mod correspondence;
pub mod evaluate;
pub mod model;

#[cfg(test)]
mod tests;

pub fn verify_receipt(receipt: &model::MirClosureReceipt) -> Vec<String> {
    let mut errors = Vec::new();
    if receipt.schema_version != model::RECEIPT_SCHEMA_VERSION {
        errors.push(format!("unsupported semantic receipt schema `{}`", receipt.schema_version));
    }
    if receipt.claim_tier != model::CLAIM_TIER {
        errors.push(format!(
            "semantic receipt claim_tier `{}` does not match `{}`",
            receipt.claim_tier,
            model::CLAIM_TIER
        ));
    }
    match model::compute_receipt_identity(receipt) {
        Ok(expected) if expected == receipt.receipt_identity => {}
        Ok(_) => errors.push("semantic receipt identity mismatch".to_owned()),
        Err(error) => errors.push(error),
    }
    errors
}
