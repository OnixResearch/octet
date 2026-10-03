#[derive(Clone, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CompilerArchitectureFact {
    Module {
        fact_id: String,
        module_path: String,
        span_id: String,
    },
    Item {
        fact_id: String,
        item_id: String,
        qualified_path: String,
        item_kind: super::ItemKind,
        origin_crate: String,
        visibility: String,
        signature_id: Option<String>,
        span_id: String,
    },
    Type {
        fact_id: String,
        type_id: String,
        display: String,
        definition_path: Option<String>,
        origin_crate: String,
        type_kind: super::TypeKind,
    },
    Trait {
        fact_id: String,
        trait_id: String,
        qualified_path: String,
        origin_crate: String,
        span_id: String,
    },
    Implementation {
        fact_id: String,
        implementation_id: String,
        trait_id: Option<String>,
        trait_definition: Option<String>,
        self_type_id: String,
        span_id: String,
    },
    Signature {
        fact_id: String,
        signature_id: String,
        owner_item_id: String,
        input_type_ids: Vec<String>,
        output_type_id: String,
        is_public: bool,
    },
    Call {
        fact_id: String,
        caller_item_id: String,
        callee_item_id: Option<String>,
        resolved_definition: Option<String>,
        dispatch: super::DispatchKind,
        span_id: String,
    },
    Constructor {
        fact_id: String,
        caller_item_id: String,
        constructed_type_id: String,
        constructor_definition: Option<String>,
        span_id: String,
    },
    Effect {
        fact_id: String,
        caller_item_id: String,
        capability_family: super::CapabilityFamily,
        primitive_definition: String,
        span_id: String,
    },
    Expansion {
        fact_id: String,
        generated_span_id: String,
        callsite_span_id: String,
        macro_definition: Option<String>,
    },
    Configuration {
        fact_id: String,
        predicate: String,
        active: bool,
        span_id: Option<String>,
    },
    TypeShape {
        fact_id: String,
        type_id: String,
        definition_path: String,
        field_visibilities: Vec<String>,
        #[serde(default = "super::empty_string_vec")]
        field_type_definitions: Vec<String>,
        implemented_traits: Vec<String>,
        span_id: String,
    },
    FunctionSafety {
        fact_id: String,
        item_id: String,
        qualified_path: String,
        output_type_id: String,
        is_unsafe: bool,
        span_id: String,
    },
    CallArgumentProvenance {
        fact_id: String,
        caller_item_id: String,
        callee_definition: String,
        argument_position: usize,
        argument_type_id: String,
        origin_definitions: Vec<String>,
        status: super::ValueProvenanceStatus,
        span_id: String,
    },
}

impl CompilerArchitectureFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        match self {
            Self::Module { fact_id, .. }
            | Self::Item { fact_id, .. }
            | Self::Type { fact_id, .. }
            | Self::Trait { fact_id, .. }
            | Self::Implementation { fact_id, .. }
            | Self::Signature { fact_id, .. }
            | Self::Call { fact_id, .. }
            | Self::Constructor { fact_id, .. }
            | Self::Effect { fact_id, .. }
            | Self::Expansion { fact_id, .. }
            | Self::Configuration { fact_id, .. }
            | Self::TypeShape { fact_id, .. }
            | Self::FunctionSafety { fact_id, .. }
            | Self::CallArgumentProvenance { fact_id, .. } => fact_id,
        }
    }
}
