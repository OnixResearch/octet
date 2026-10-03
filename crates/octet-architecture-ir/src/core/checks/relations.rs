pub(super) struct FactReferenceIndex<'a> {
    pub(super) spans: std::collections::BTreeSet<&'a str>,
    pub(super) items: std::collections::BTreeSet<&'a str>,
    pub(super) types: std::collections::BTreeSet<&'a str>,
    pub(super) traits: std::collections::BTreeSet<&'a str>,
    pub(super) signatures: std::collections::BTreeSet<&'a str>,
}

struct ReferenceContext<'a> {
    unit_id: Option<&'a str>,
    fact_id: Option<&'a str>,
    issues: &'a mut super::super::diagnostics::Issues,
}

impl ReferenceContext<'_> {
    fn require(&mut self, kind: &str, reference: &str, available: &std::collections::BTreeSet<&str>) {
        if !available.contains(reference) {
            self.issues.push(super::super::digest::issue(
                "invalid-reference",
                self.unit_id,
                self.fact_id,
                &format!("compiler architecture fact references missing {kind} `{reference}`"),
            ));
        }
    }
}

pub(super) fn validate_fact_references(
    fact: &crate::model::CompilerArchitectureFact,
    index: &FactReferenceIndex<'_>,
    unit_id: Option<&str>,
    issues: &mut super::super::diagnostics::Issues,
) {
    let mut context = ReferenceContext {
        unit_id,
        fact_id: Some(fact.fact_id()),
        issues,
    };
    match fact {
        crate::model::CompilerArchitectureFact::Module { .. }
        | crate::model::CompilerArchitectureFact::Trait { .. }
        | crate::model::CompilerArchitectureFact::Item { .. }
        | crate::model::CompilerArchitectureFact::Type { .. }
        | crate::model::CompilerArchitectureFact::Implementation { .. }
        | crate::model::CompilerArchitectureFact::Signature { .. } => {
            validate_structural_references(fact, index, &mut context);
        }
        crate::model::CompilerArchitectureFact::Call { .. }
        | crate::model::CompilerArchitectureFact::Effect { .. }
        | crate::model::CompilerArchitectureFact::Constructor { .. }
        | crate::model::CompilerArchitectureFact::Expansion { .. }
        | crate::model::CompilerArchitectureFact::Configuration { .. } => {
            validate_usage_references(fact, index, &mut context);
        }
        crate::model::CompilerArchitectureFact::TypeShape { .. }
        | crate::model::CompilerArchitectureFact::FunctionSafety { .. }
        | crate::model::CompilerArchitectureFact::CallArgumentProvenance { .. } => {
            validate_authority_references(fact, index, &mut context);
        }
    }
}

fn validate_structural_references(
    fact: &crate::model::CompilerArchitectureFact,
    index: &FactReferenceIndex<'_>,
    context: &mut ReferenceContext<'_>,
) {
    match fact {
        crate::model::CompilerArchitectureFact::Module { span_id, .. }
        | crate::model::CompilerArchitectureFact::Trait { span_id, .. } => {
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::Item {
            signature_id, span_id, ..
        } => {
            context.require("span", span_id, &index.spans);
            if let Some(signature_id) = signature_id {
                context.require("signature", signature_id, &index.signatures);
            }
        }
        crate::model::CompilerArchitectureFact::Implementation {
            trait_id,
            self_type_id,
            span_id,
            ..
        } => {
            context.require("span", span_id, &index.spans);
            context.require("type", self_type_id, &index.types);
            if let Some(trait_id) = trait_id {
                context.require("trait", trait_id, &index.traits);
            }
        }
        crate::model::CompilerArchitectureFact::Signature {
            owner_item_id,
            input_type_ids,
            output_type_id,
            ..
        } => {
            context.require("item", owner_item_id, &index.items);
            for type_id in input_type_ids {
                context.require("type", type_id, &index.types);
            }
            context.require("type", output_type_id, &index.types);
        }
        crate::model::CompilerArchitectureFact::Type { .. }
        | crate::model::CompilerArchitectureFact::Call { .. }
        | crate::model::CompilerArchitectureFact::Effect { .. }
        | crate::model::CompilerArchitectureFact::Constructor { .. }
        | crate::model::CompilerArchitectureFact::Expansion { .. }
        | crate::model::CompilerArchitectureFact::Configuration { .. }
        | crate::model::CompilerArchitectureFact::TypeShape { .. }
        | crate::model::CompilerArchitectureFact::FunctionSafety { .. }
        | crate::model::CompilerArchitectureFact::CallArgumentProvenance { .. } => {}
    }
}

fn validate_usage_references(
    fact: &crate::model::CompilerArchitectureFact,
    index: &FactReferenceIndex<'_>,
    context: &mut ReferenceContext<'_>,
) {
    match fact {
        crate::model::CompilerArchitectureFact::Call {
            caller_item_id,
            span_id,
            ..
        }
        | crate::model::CompilerArchitectureFact::Effect {
            caller_item_id,
            span_id,
            ..
        } => {
            context.require("item", caller_item_id, &index.items);
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::Constructor {
            caller_item_id,
            constructed_type_id,
            span_id,
            ..
        } => {
            context.require("item", caller_item_id, &index.items);
            context.require("type", constructed_type_id, &index.types);
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::Expansion {
            generated_span_id,
            callsite_span_id,
            ..
        } => {
            context.require("span", generated_span_id, &index.spans);
            context.require("span", callsite_span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::Configuration { span_id, .. } => {
            if let Some(span_id) = span_id {
                context.require("span", span_id, &index.spans);
            }
        }
        crate::model::CompilerArchitectureFact::Module { .. }
        | crate::model::CompilerArchitectureFact::Trait { .. }
        | crate::model::CompilerArchitectureFact::Item { .. }
        | crate::model::CompilerArchitectureFact::Type { .. }
        | crate::model::CompilerArchitectureFact::Implementation { .. }
        | crate::model::CompilerArchitectureFact::Signature { .. }
        | crate::model::CompilerArchitectureFact::TypeShape { .. }
        | crate::model::CompilerArchitectureFact::FunctionSafety { .. }
        | crate::model::CompilerArchitectureFact::CallArgumentProvenance { .. } => {}
    }
}

fn validate_authority_references(
    fact: &crate::model::CompilerArchitectureFact,
    index: &FactReferenceIndex<'_>,
    context: &mut ReferenceContext<'_>,
) {
    match fact {
        crate::model::CompilerArchitectureFact::TypeShape { type_id, span_id, .. } => {
            context.require("type", type_id, &index.types);
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::FunctionSafety {
            item_id,
            output_type_id,
            span_id,
            ..
        } => {
            context.require("item", item_id, &index.items);
            context.require("type", output_type_id, &index.types);
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::CallArgumentProvenance {
            caller_item_id,
            argument_type_id,
            span_id,
            ..
        } => {
            context.require("item", caller_item_id, &index.items);
            context.require("type", argument_type_id, &index.types);
            context.require("span", span_id, &index.spans);
        }
        crate::model::CompilerArchitectureFact::Module { .. }
        | crate::model::CompilerArchitectureFact::Trait { .. }
        | crate::model::CompilerArchitectureFact::Item { .. }
        | crate::model::CompilerArchitectureFact::Type { .. }
        | crate::model::CompilerArchitectureFact::Implementation { .. }
        | crate::model::CompilerArchitectureFact::Signature { .. }
        | crate::model::CompilerArchitectureFact::Call { .. }
        | crate::model::CompilerArchitectureFact::Effect { .. }
        | crate::model::CompilerArchitectureFact::Constructor { .. }
        | crate::model::CompilerArchitectureFact::Expansion { .. }
        | crate::model::CompilerArchitectureFact::Configuration { .. } => {}
    }
}

pub(super) fn validate_unknowns(
    unknowns: &[crate::model::UnknownCompilerFact],
    spans: &[crate::model::SourceSpan],
    facts: &[crate::model::CompilerArchitectureFact],
    unit_id: Option<&str>,
    issues: &mut super::super::diagnostics::Issues,
) {
    let spans = spans.iter().map(|span| span.span_id.as_str()).collect::<std::collections::BTreeSet<_>>();
    let items = facts
        .iter()
        .filter_map(|fact| match fact {
            crate::model::CompilerArchitectureFact::Item { item_id, .. } => Some(item_id.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut ids = std::collections::BTreeSet::new();
    for unknown in unknowns {
        if !ids.insert(unknown.unknown_id.as_str()) {
            issues.push(super::super::digest::issue(
                "duplicate-unknown",
                unit_id,
                None,
                "compiler architecture shard contains a duplicate unknown fact",
            ));
        }
        if issues.stale_identity(&unknown.unknown_id, || super::super::digest::unknown_fact_identity(unknown)) {
            issues.push(super::super::digest::issue(
                "unknown-identity",
                unit_id,
                None,
                "unknown compiler fact identity is stale or malformed",
            ));
        }
        if unknown.detail.trim().is_empty() {
            issues.push(super::super::digest::issue(
                "unknown-detail",
                unit_id,
                None,
                "unknown compiler fact has no detail",
            ));
        }
        let mut context = ReferenceContext {
            unit_id,
            fact_id: None,
            issues,
        };
        if let Some(owner_item_id) = &unknown.owner_item_id {
            context.require("item", owner_item_id, &items);
        }
        if let Some(span_id) = &unknown.span_id {
            context.require("span", span_id, &spans);
        }
    }
}
