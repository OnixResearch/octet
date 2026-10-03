const UNIT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.unit/v2";
const SPAN_ID_DOMAIN: &str = "octet.compiler-architecture-ir.span/v1";
const FACT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.fact/v1";
const UNKNOWN_ID_DOMAIN: &str = "octet.compiler-architecture-ir.unknown/v1";
const SHARD_ID_DOMAIN: &str = "octet.compiler-architecture-ir.shard/v2";
const IR_ID_DOMAIN: &str = "octet.compiler-architecture-ir.normalized/v2";
const COVERAGE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.coverage/v1";
const ISSUE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.issue/v1";
const RECEIPT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.receipt/v1";

/// Compute the canonical identity of a selected compilation unit.
///
/// # Errors
///
/// Returns a serialization error if the unit's identity fields cannot be encoded.
pub fn compilation_unit_identity(unit: &crate::model::CompilationUnit) -> Result<String, String> {
    crate::framing::canonical_identity(
        UNIT_ID_DOMAIN,
        &(
            &unit.package_name,
            &unit.package_version,
            &unit.target_name,
            &unit.target_kind,
            &unit.source_path,
            &unit.crate_types,
            &unit.target_triple,
            &unit.features,
            unit.default_features,
            unit.role,
            &unit.toolchain_blake3,
            &unit.collector_blake3,
            &unit.policy_blake3,
            &unit.source_blake3,
            &unit.invocation_blake3,
        ),
    )
}

/// Compute the canonical identity of a source span.
///
/// # Errors
///
/// Returns a serialization error if the span's identity fields cannot be encoded.
pub fn source_span_identity(span: &crate::model::SourceSpan) -> Result<String, String> {
    crate::framing::canonical_identity(
        SPAN_ID_DOMAIN,
        &(
            &span.origin,
            span.byte_start,
            span.byte_end,
            span.line_start,
            span.column_start,
            span.from_expansion,
        ),
    )
}

// Keep the variant-to-tuple mapping in one place. The expanded match remains exhaustive,
// so adding a new fact requires an explicit identity rather than a fallback branch.
macro_rules! define_fact_identity {
    ($( $variant:ident { $($field:ident),+ } => $kind:literal ),+ $(,)?) => {
        /// Compute the identity of one compiler fact, including its variant-specific fields.
        ///
        /// # Errors
        ///
        /// Returns a serialization error if the selected fact's fields cannot be encoded.
        pub fn fact_identity(fact: &crate::model::CompilerArchitectureFact) -> Result<String, String> {
            match fact {
                $(crate::model::CompilerArchitectureFact::$variant { $($field),+, .. } =>
                    crate::framing::canonical_identity(FACT_ID_DOMAIN, &($kind, $($field),+)),)+
            }
        }
    };
}

define_fact_identity! {
    Module { module_path, span_id } => "module",
    Item { item_id, qualified_path, item_kind, origin_crate, visibility, signature_id, span_id } => "item",
    Type { type_id, display, definition_path, origin_crate, type_kind } => "type",
    Trait { trait_id, qualified_path, origin_crate, span_id } => "trait",
    Implementation { implementation_id, trait_id, trait_definition, self_type_id, span_id } => "implementation",
    Signature { signature_id, owner_item_id, input_type_ids, output_type_id, is_public } => "signature",
    Call { caller_item_id, callee_item_id, resolved_definition, dispatch, span_id } => "call",
    Constructor { caller_item_id, constructed_type_id, constructor_definition, span_id } => "constructor",
    Effect { caller_item_id, capability_family, primitive_definition, span_id } => "effect",
    Expansion { generated_span_id, callsite_span_id, macro_definition } => "expansion",
    Configuration { predicate, active, span_id } => "configuration",
    TypeShape { type_id, definition_path, field_visibilities, field_type_definitions, implemented_traits, span_id } => "type-shape",
    FunctionSafety { item_id, qualified_path, output_type_id, is_unsafe, span_id } => "function-safety",
    CallArgumentProvenance {
        caller_item_id, callee_definition, argument_position, argument_type_id, origin_definitions, status, span_id
    } => "call-argument-provenance",
}

/// Compute the canonical identity of an unknown compiler fact.
///
/// # Errors
///
/// Returns a serialization error if the unknown's identity fields cannot be encoded.
pub fn unknown_fact_identity(unknown: &crate::model::UnknownCompilerFact) -> Result<String, String> {
    crate::framing::canonical_identity(
        UNKNOWN_ID_DOMAIN,
        &(unknown.kind, &unknown.owner_item_id, &unknown.target_identity, &unknown.span_id, &unknown.detail),
    )
}

/// Compute the canonical identity of a compiler architecture shard.
///
/// # Errors
///
/// Returns a serialization error if the shard's identity fields cannot be encoded.
pub fn shard_identity(shard: &crate::model::CompilerArchitectureShard) -> Result<String, String> {
    crate::framing::canonical_identity(
        SHARD_ID_DOMAIN,
        &(
            &shard.schema_version,
            &shard.unit,
            shard.collection_status,
            &shard.spans,
            &shard.facts,
            &shard.unknowns,
            &shard.non_claims,
        ),
    )
}

pub(super) fn issue(
    code: &str,
    unit_id: Option<&str>,
    fact_id: Option<&str>,
    message: &str,
) -> Result<crate::model::CompilerArchitectureIssue, String> {
    let issue_id = crate::framing::canonical_identity(ISSUE_ID_DOMAIN, &(code, unit_id, fact_id, message))?;
    Ok(crate::model::CompilerArchitectureIssue {
        schema_version: crate::model::COMPILER_ARCHITECTURE_ISSUE_SCHEMA_VERSION.to_owned(),
        issue_id,
        code: code.to_owned(),
        unit_id: unit_id.map(str::to_owned),
        fact_id: fact_id.map(str::to_owned),
        message: message.to_owned(),
    })
}

/// Compute the canonical identity of a compiler coverage artifact.
///
/// # Errors
///
/// Returns a serialization error if coverage fields cannot be encoded.
pub fn coverage_identity(coverage: &crate::model::CompilerArchitectureCoverage) -> Result<String, String> {
    crate::framing::canonical_identity(
        COVERAGE_ID_DOMAIN,
        &(
            &coverage.schema_version,
            coverage.status,
            &coverage.expected_unit_ids,
            &coverage.observed_unit_ids,
            &coverage.missing_unit_ids,
            &coverage.unexpected_unit_ids,
            &coverage.duplicate_unit_ids,
            &coverage.invalid_shard_ids,
        ),
    )
}

/// Compute the canonical identity of a normalized compiler Architecture IR artifact.
///
/// # Errors
///
/// Returns a serialization error if IR fields cannot be encoded.
pub fn ir_identity(ir: &crate::model::CompilerArchitectureIr) -> Result<String, String> {
    crate::framing::canonical_identity(
        IR_ID_DOMAIN,
        &(
            &ir.schema_version,
            &ir.policy_blake3,
            &ir.units,
            &ir.unit_memberships,
            &ir.spans,
            &ir.facts,
            &ir.unknowns,
            &ir.non_claims,
        ),
    )
}

/// Compute the canonical identity of a compiler architecture collection receipt.
///
/// # Errors
///
/// Returns a serialization error if receipt fields cannot be encoded.
pub fn receipt_identity(receipt: &crate::model::CompilerArchitectureReceipt) -> Result<String, String> {
    crate::framing::canonical_identity(
        RECEIPT_ID_DOMAIN,
        &(
            &receipt.schema_version,
            receipt.status,
            &receipt.policy_blake3,
            &receipt.coverage_id,
            &receipt.architecture_ir_id,
            &receipt.issue_ids,
            &receipt.non_claims,
        ),
    )
}
